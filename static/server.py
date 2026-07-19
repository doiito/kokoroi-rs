#!/usr/bin/env python3
"""Kokoro TTS 后端服务器 - 使用 ONNX Runtime 提供可靠的语音合成"""

import argparse
import json
import os
import struct
import wave
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

import numpy as np
import onnxruntime as ort

MODEL_PATH = os.path.join(os.path.dirname(__file__), 'models/onnx/model_fp16.onnx')
VOICES_DIR = os.path.join(os.path.dirname(__file__), 'models/voices')
VOICES_JSON = os.path.join(os.path.dirname(__file__), 'models/voices/voices.json')
STATIC_DIR = os.path.join(os.path.dirname(__file__))
SAMPLE_RATE = 24000
STYLE_DIM = 256
MAX_TOKENS = 510


def load_voices():
    with open(VOICES_JSON, 'r') as f:
        return json.load(f)


def load_voice_style(voice_id):
    voices = load_voices()
    if voice_id not in voices:
        return None
    voice_info = voices[voice_id]
    voice_path = os.path.join(VOICES_DIR, voice_info['file'])
    if not os.path.exists(voice_path):
        return None
    data = np.fromfile(voice_path, dtype=np.float32)
    return data


class KokoroServer:
    def __init__(self, model_path):
        self.session = ort.InferenceSession(
            model_path,
            providers=['CPUExecutionProvider'],
            sess_options=ort.SessionOptions()
        )
        self.voices = load_voices()
        print(f'Model loaded: {model_path}')
        print(f'Model inputs: {[(i.name, i.type, i.shape) for i in self.session.get_inputs()]}')
        print(f'Model outputs: {[(o.name, o.type, o.shape) for o in self.session.get_outputs()]}')
        print(f'Voices loaded: {len(self.voices)}')

    def synthesize(self, tokens, voice_id, speed=1.0):
        token_list = json.loads(tokens) if isinstance(tokens, str) else tokens
        padded = [0] + token_list + [0]
        input_ids = np.array([padded], dtype=np.int64)

        style_data = load_voice_style(voice_id)
        if style_data is None:
            style = np.random.randn(1, STYLE_DIM).astype(np.float32) * 0.1
        else:
            token_len = len(token_list)
            if style_data.shape[0] >= token_len + 1:
                style = style_data[token_len:token_len + STYLE_DIM].reshape(1, STYLE_DIM)
            elif style_data.shape[0] >= STYLE_DIM:
                style = style_data[:STYLE_DIM].reshape(1, STYLE_DIM)
            else:
                style = np.random.randn(1, STYLE_DIM).astype(np.float32) * 0.1

        speed_tensor = np.array([speed], dtype=np.float32)

        outputs = self.session.run(['waveform'], {
            'input_ids': input_ids,
            'style': style.astype(np.float32),
            'speed': speed_tensor,
        })

        audio = outputs[0].flatten()
        audio = np.nan_to_num(audio, nan=0.0, posinf=0.0, neginf=0.0)
        audio = np.clip(audio, -1.0, 1.0)

        return audio


class RequestHandler(BaseHTTPRequestHandler):
    server_instance = None

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_cors_headers()
        self.end_headers()

    def do_GET(self):
        parsed = urlparse(self.path)

        if parsed.path == '/api/voices':
            self.handle_voices()
        elif parsed.path == '/api/synthesize':
            self.handle_synthesize(parsed)
        elif parsed.path == '/api/status':
            self.handle_status()
        else:
            self.serve_static(parsed.path)

    def do_POST(self):
        parsed = urlparse(self.path)

        if parsed.path == '/api/synthesize':
            content_length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(content_length) if content_length > 0 else b'{}'
            self.handle_synthesize_post(body)
        else:
            self.send_error(404)

    def send_cors_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')

    def handle_voices(self):
        self.send_response(200)
        self.send_cors_headers()
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(self.server_instance.voices).encode())

    def handle_status(self):
        self.send_response(200)
        self.send_cors_headers()
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps({'status': 'ok', 'sample_rate': SAMPLE_RATE}).encode())

    def handle_synthesize(self, parsed):
        params = parse_qs(parsed.query)
        text = params.get('text', [''])[0]
        voice = params.get('voice', ['zf_001'])[0]
        speed = float(params.get('speed', ['1.0'])[0])

        if not text:
            self.send_error(400, 'Missing text parameter')
            return

        try:
            tokens_param = params.get('tokens', [None])[0]
            if tokens_param:
                tokens = json.loads(tokens_param)
            else:
                self.send_error(400, 'Missing tokens parameter')
                return

            audio = self.server_instance.synthesize(tokens, voice, speed)
            self.send_wav(audio)
        except Exception as e:
            self.send_error(500, str(e))

    def handle_synthesize_post(self, body):
        try:
            data = json.loads(body)
            tokens = data.get('tokens', [])
            voice = data.get('voice', 'zf_001')
            speed = float(data.get('speed', 1.0))

            if not tokens:
                self.send_error(400, 'Missing tokens')
                return

            audio = self.server_instance.synthesize(tokens, voice, speed)
            self.send_wav(audio)
        except Exception as e:
            self.send_error(500, str(e))

    def send_wav(self, audio):
        self.send_response(200)
        self.send_cors_headers()
        self.send_header('Content-Type', 'audio/wav')
        self.send_header('Content-Disposition', 'inline; filename="speech.wav"')
        self.end_headers()

        buf = io.BytesIO()
        with wave.open(buf, 'wb') as wav:
            wav.setnchannels(1)
            wav.setsampwidth(2)
            wav.setframerate(SAMPLE_RATE)
            audio_int16 = (audio * 32767).astype(np.int16)
            wav.writeframes(audio_int16.tobytes())
        self.wfile.write(buf.getvalue())

    def serve_static(self, path):
        if not path or path == '/':
            path = '/index.html'
        file_path = STATIC_DIR + path

        if not os.path.exists(file_path) or os.path.isdir(file_path):
            self.send_error(404)
            return

        ext = os.path.splitext(file_path)[1].lower()
        content_types = {
            '.html': 'text/html; charset=utf-8',
            '.js': 'application/javascript; charset=utf-8',
            '.css': 'text/css; charset=utf-8',
            '.wasm': 'application/wasm',
            '.onnx': 'application/octet-stream',
            '.bin': 'application/octet-stream',
            '.json': 'application/json; charset=utf-8',
            '.wav': 'audio/wav',
            '.png': 'image/png',
            '.jpg': 'image/jpeg',
            '.svg': 'image/svg+xml',
        }

        self.send_response(200)
        self.send_cors_headers()
        self.send_header('Content-Type', content_types.get(ext, 'application/octet-stream'))
        self.send_header('Content-Length', str(os.path.getsize(file_path)))
        self.end_headers()

        with open(file_path, 'rb') as f:
            self.wfile.write(f.read())

    def log_message(self, format, *args):
        print(f'[Request] {args[0]} {args[1]}')


def main():
    parser = argparse.ArgumentParser(description='Kokoro TTS Server')
    parser.add_argument('--port', type=int, default=8000, help='Server port')
    parser.add_argument('--host', type=str, default='0.0.0.0', help='Server host')
    parser.add_argument('--model', type=str, default=MODEL_PATH, help='ONNX model path')
    args = parser.parse_args()

    print(f'Starting Kokoro TTS server on http://{args.host}:{args.port}')
    print(f'Static files: {STATIC_DIR}')
    print(f'Model: {args.model}')
    print()

    global io
    import io

    server = HTTPServer((args.host, args.port), RequestHandler)
    RequestHandler.server_instance = KokoroServer(args.model)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print('\nServer stopped.')
        server.server_close()


if __name__ == '__main__':
    main()