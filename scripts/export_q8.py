#!/usr/bin/env python3
"""
Export Kokoro model to ONNX with Q8 quantization.

Usage:
    # Export with kokoro source in same directory as config/model
    python export_q8.py --config ./config.json --model ./kokoro.pth --output-dir ./models/onnx

    # With custom kokoro package path
    python export_q8.py --kokoro-src /path/to/kokoro --config ./config.json --model ./kokoro.pth
"""

import argparse
import json
import os
import sys

import torch
import torch.onnx
import onnx
from onnxruntime.quantization import quantize_dynamic, QuantType


def export_to_onnx(config_path, model_path, output_dir, kokoro_src_dir):
    if kokoro_src_dir:
        sys.path.insert(0, kokoro_src_dir)

    print("Loading PyTorch model...")

    with open(config_path, 'r') as f:
        config = json.load(f)

    print(f"Config loaded: n_token={config.get('n_token')}, hidden_dim={config.get('hidden_dim')}")

    from kokoro import KModel

    model = KModel(model=model_path, config=config_path)
    model.eval()

    print("Model loaded successfully")

    # Create dummy inputs
    batch_size = 1
    seq_len = 50
    style_dim = 256

    input_ids = torch.zeros((batch_size, seq_len), dtype=torch.long)
    input_ids[0, 0] = 0  # BOS
    input_ids[0, 1:10] = torch.randint(1, 100, (9,))
    input_ids[0, 10:] = 0

    style = torch.randn((batch_size, style_dim), dtype=torch.float32)
    speed = torch.tensor([1.0], dtype=torch.float32)

    print(f"Input shapes: input_ids={input_ids.shape}, style={style.shape}, speed={speed.shape}")

    os.makedirs(output_dir, exist_ok=True)

    fp32_path = os.path.join(output_dir, 'model_fp32.onnx')
    q8_path = os.path.join(output_dir, 'model_q8.onnx')

    print(f"Exporting to ONNX (opset 20)...")

    torch.onnx.export(
        model,
        (input_ids, style, speed),
        fp32_path,
        input_names=['input_ids', 'style', 'speed'],
        output_names=['waveform'],
        dynamic_axes={
            'input_ids': {0: 'batch', 1: 'sequence'},
            'style': {0: 'batch'},
            'speed': {0: 'batch'},
            'waveform': {0: 'batch', 1: 'audio_length'}
        },
        opset_version=20,
        do_constant_folding=True,
    )

    print(f"FP32 model saved to: {fp32_path}")

    print("Verifying ONNX model...")
    onnx_model = onnx.load(fp32_path)
    onnx.checker.check_model(onnx_model)
    print("ONNX model is valid")

    print("Quantizing to Q8...")
    quantize_dynamic(
        fp32_path,
        q8_path,
        weight_type=QuantType.QInt8,
        op_types_to_quantize=['MatMul', 'Gemm', 'Conv'],
    )

    print(f"Q8 model saved to: {q8_path}")

    fp32_size = os.path.getsize(fp32_path) / 1024 / 1024
    q8_size = os.path.getsize(q8_path) / 1024 / 1024

    print(f"\nFile sizes:")
    print(f"  FP32: {fp32_size:.2f} MB")
    print(f"  Q8:   {q8_size:.2f} MB")

    return fp32_path, q8_path


def main():
    parser = argparse.ArgumentParser(description='Export Kokoro model to ONNX with Q8 quantization')
    parser.add_argument('--kokoro-src', default=None,
                        help='Path to kokoro source directory (added to sys.path)')
    parser.add_argument('--config', required=True,
                        help='Path to model config.json')
    parser.add_argument('--model', required=True,
                        help='Path to model .pth file')
    parser.add_argument('--output-dir', default='./models/onnx',
                        help='Output directory for ONNX files (default: ./models/onnx)')
    args = parser.parse_args()

    export_to_onnx(
        config_path=args.config,
        model_path=args.model,
        output_dir=args.output_dir,
        kokoro_src_dir=args.kokoro_src,
    )


if __name__ == '__main__':
    main()
