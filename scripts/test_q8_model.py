#!/usr/bin/env python3
"""
Test Kokoro Q8 ONNX model with kokoro-onnx.

Usage:
    python test_q8_model.py --model ./models/onnx/kokoro-q8.onnx --voices ./data/voices.bin
    python test_q8_model.py --model ./models/onnx/kokoro-q8.onnx --voices ./data/voices.bin --config ./config.json
"""

import argparse
import os


def test_q8_model(model_path, voices_path, config_path):
    print("=" * 50)
    print("Testing Kokoro Q8 Model")
    print("=" * 50)

    if not os.path.exists(model_path):
        print(f"ERROR: Model not found: {model_path}")
        return

    print(f"Model: {model_path}")
    print(f"Model size: {os.path.getsize(model_path) / 1024 / 1024:.2f} MB")

    try:
        from kokoro_onnx import Kokoro
        print("kokoro_onnx imported successfully")
    except ImportError as e:
        print(f"ERROR: Failed to import kokoro_onnx: {e}")
        print("Please install: pip install kokoro-onnx")
        return

    print("\nLoading model...")
    try:
        kwargs = {}
        if config_path:
            kwargs['vocab_config'] = config_path
        kokoro = Kokoro(model_path, voices_path, **kwargs)
        print("Model loaded successfully!")
    except Exception as e:
        print(f"ERROR: Failed to load model: {e}")
        import traceback
        traceback.print_exc()
        return

    print("\nAvailable voices:")
    voices = kokoro.get_voices()
    for v in sorted(voices)[:10]:
        print(f"  - {v}")
    print(f"  ... and {len(voices) - 10} more")

    print("\nTesting synthesis...")
    test_cases = [
        ("Hello, this is a test.", "af_heart", "English female"),
        ("Life is like a box of chocolates.", "am_michael", "English male"),
        ("The quick brown fox jumps over the lazy dog.", "af_sarah", "English female"),
    ]

    import soundfile as sf
    import numpy as np

    for text, voice, desc in test_cases:
        print(f"\n  Testing: '{text}' ({desc})")
        try:
            samples, sample_rate = kokoro.create(text, voice=voice, speed=1.0, lang="en-us")

            nan_count = np.isnan(samples).sum()
            inf_count = np.isinf(samples).sum()

            if nan_count > 0 or inf_count > 0:
                print(f"    ERROR: Output contains {nan_count} NaN and {inf_count} Inf values!")
            else:
                min_val = samples.min()
                max_val = samples.max()
                duration = len(samples) / sample_rate

                print(f"    SUCCESS!")
                print(f"    Sample rate: {sample_rate} Hz")
                print(f"    Duration: {duration:.2f}s")
                print(f"    Range: [{min_val:.4f}, {max_val:.4f}]")

                output_file = f"/tmp/test_{voice}.wav"
                sf.write(output_file, samples, sample_rate)
                print(f"    Saved to: {output_file}")

        except Exception as e:
            print(f"    ERROR: {e}")
            import traceback
            traceback.print_exc()

    print("\n" + "=" * 50)
    print("Test completed!")
    print("=" * 50)


def main():
    parser = argparse.ArgumentParser(description='Test Kokoro Q8 ONNX model')
    parser.add_argument('--model', required=True,
                        help='Path to ONNX model file')
    parser.add_argument('--voices', required=True,
                        help='Path to voices .bin file')
    parser.add_argument('--config', default=None,
                        help='Path to vocab config.json (optional)')
    args = parser.parse_args()

    test_q8_model(
        model_path=args.model,
        voices_path=args.voices,
        config_path=args.config,
    )


if __name__ == '__main__':
    main()
