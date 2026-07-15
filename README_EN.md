# Kokoro TTS — High-Quality Text-to-Speech Engine

[![Build](https://github.com/doiito/kokoroi-rs/actions/workflows/build.yml/badge.svg)](https://github.com/doiito/kokoroi-rs/actions/workflows/build.yml)

**kokoroi-rs** is a Rust implementation of [Kokoro](https://github.com/hexgrad/kokoro), focusing on high-quality Chinese text-to-speech synthesis. Powered by the Kokoro model with ONNX Runtime inference, it offers both a CLI tool and an HTTP API server.

## Features

- 🎯 **High-Quality Chinese TTS** — Based on the Kokoro model with multiple Chinese voice styles
- 🚀 **High-Performance Inference** — Multi-threaded pipeline architecture with parallel generation, real-time factor of 5-10x
- 📦 **Cross-Platform Static Builds** — Fully static musl ELF for Linux (x86_64 / ARM64), MSVC binaries for Windows — zero or minimal runtime dependencies
- 🌐 **HTTP API Server** — Built-in Axum web server with REST API and SSE streaming, includes a web demo page
- 🎤 **50+ Voice Styles** — Supports Chinese, Japanese, Korean, English, French, and more
- ✂️ **Smart Text Chunking** — Phoneme-limit-based chunking strategy balancing speed and naturalness
- 📝 **Streaming Output** — SSE real-time streaming audio for low-latency web scenarios
- 🤖 **CI/CD Automation** — GitHub Actions builds every platform on every push

## Quick Start

### Download Prebuilt Binaries

Download the binary for your platform from the [Releases](https://github.com/doiito/kokoroi-rs/releases) page:

| Platform | File | Notes |
|----------|------|-------|
| x86_64 Linux | `kokoro-x86_64-unknown-linux-musl.tar.gz` | Fully static ELF, zero deps |
| ARM64 Linux | `kokoro-aarch64-unknown-linux-musl.tar.gz` | Fully static ELF, zero deps |
| x86_64 Windows | `kokoro-x86_64-pc-windows-msvc.zip` | Includes `onnxruntime.dll`, extract & run |

**Windows users**: The archive includes `onnxruntime.dll` — keep it in the same directory as the .exe files.

### Download Model Files

You need the following model files:

- **Kokoro ONNX Model**: `models/kokoro-v1.0.onnx` (~80MB)
- **Voice Data**: `data/voices-v1.0.bin` (~150MB)
- **Configuration**: `config.toml` (provided in project root)

Model files can be obtained from:

1. The [Kokoro official project](https://github.com/hexgrad/kokoro)
2. Run `./scripts/download_models.sh`

### Using the CLI

```bash
# Basic: generate speech from text
./koko --text "Hello, welcome to Kokoro TTS system." -o output.wav

# Read from file
./koko -i input.txt -o output.wav

# Specify voice style (default: zm_yunyang)
./koko --text "Nice weather today" --style af_bella -o output.wav

# Adjust speech speed
./koko --text "Hello everyone" --speed 1.2 -o output.wav

# Set inference threads
./koko --text "Long text test" --threads 4 -o output.wav

# Play audio (requires aplay or ffplay)
./koko --text "Playback test" --play
```

On Windows, replace `./koko` with `koko.exe`.

### Starting the API Server

```bash
# Start with default config
./kokoros-server

# Use custom config
KOKOROS_CONFIG=my_config.toml ./kokoros-server
```

The server listens on `0.0.0.0:3000` by default — open your browser to access the Web demo.

#### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | Web demo page |
| `GET` | `/health` | Health check |
| `GET` | `/voices` | List all available voices |
| `POST` | `/tts` | Text-to-speech (returns WAV Base64) |
| `POST` | `/tts/stream` | Streaming TTS (SSE) |

**POST `/tts` Request:**

```json
{
  "text": "Hello world",
  "voice": "af_bella",
  "speed": 1.0
}
```

**Successful Response:**

```json
{
  "success": true,
  "audio_base64": "UklGRiQ...base64-encoded WAV data",
  "duration": 2.35
}
```

## Supported Voices

50+ voices across multiple languages:

| Prefix | Language | Examples |
|--------|----------|----------|
| `zf_`, `zm_` | Chinese | `zf_xiaobei`, `zm_yunyang` |
| `af_`, `am_` | English | `af_bella`, `am_adam` |
| `jf_`, `jm_` | Japanese | `jf_alpha`, `jm_kumo` |
| `bf_`, `bm_` | British English | `bf_alice`, `bm_daniel` |
| `ef_`, `em_` | Spanish | `ef_dora`, `em_alex` |
| `ff_` | French | `ff_siwis` |
| `hf_`, `hm_` | Hindi | `hf_alpha`, `hm_omega` |
| `if_`, `im_` | Italian | `if_sara`, `im_nicola` |
| `pf_`, `pm_` | Portuguese | `pf_dora`, `pm_alex` |

## Configuration

The server reads from `config.toml`:

```toml
# Kokoro TTS Server Configuration
host = "0.0.0.0"           # Listen address
port = 3000                # Listen port
threads = 4                # Inference threads
max_chars = 400            # Max characters per request
model_path = "models/kokoro-v1.0.onnx"   # Model path
voices_path = "data/voices-v1.0.bin"     # Voice data path
```

Set the `KOKOROS_CONFIG` environment variable to use a custom config file.

## CI/CD Pipeline

This project uses GitHub Actions to automatically build binaries for all supported platforms on every push and tag:

| Architecture | Runner | Build Method |
|-------------|--------|-------------|
| x86_64 Linux musl | ubuntu-latest | ONNX Runtime from source, fully static musl |
| ARM64 Linux musl | ubuntu-latest | ORT cross-compiled, aarch64 musl static |
| x86_64 Windows MSVC | windows-latest | Pre-built ORT download, MSVC dynamic link |

For Linux musl targets, the first ORT build takes ~20–30 min (cmake from source); subsequent runs hit the cache and complete in ~2–5 min.

Workflow: [`.github/workflows/build.yml`](.github/workflows/build.yml)

## Building from Source

### Linux (Fully Static musl)

#### Prerequisites

- Rust 1.85+ (install via [rustup](https://rustup.rs/))
- cmake, ninja, protobuf-compiler, wget

#### One-Click Build (Quick Start)

Use the provided scripts that work with the pre-built local ORT:

```bash
# x86_64 musl
./scripts/build_musl.sh

# ARM64 musl
./scripts/build_aarch64_musl.sh
```

**Building ORT yourself with `build_ort_musl.sh`:**

```bash
# 1. Download musl cross-compiler
wget https://musl.cc/x86_64-linux-musl-cross.tgz
tar -xzf x86_64-linux-musl-cross.tgz

# 2. Download ONNX Runtime source
wget https://github.com/microsoft/onnxruntime/archive/refs/tags/v1.23.2.tar.gz
tar -xzf v1.23.2.tar.gz

# 3. Build ORT + produce combined libonnxruntime.a
bash scripts/build_ort_musl.sh \
  x86_64-unknown-linux-musl \
  /path/to/x86_64-linux-musl-cross \
  /path/to/onnxruntime-1.23.2 \
  /path/to/output

# 4. Build koko / kokoros-server
ORT_LIB_LOCATION=/path/to/output \
RUSTFLAGS="-C target-feature=+crt-static" \
cargo build --release -p koko --target x86_64-unknown-linux-musl --no-default-features
cargo build --release -p kokoros-server --target x86_64-unknown-linux-musl --no-default-features
```

> For ARM64 (aarch64), replace `x86_64` with `aarch64`.
>
> `--no-default-features` is required for musl builds because audio-encoding crates
> (mp3lame, audiopus) cannot be cross-compiled under the musl toolchain.

### Windows (MSVC)

#### Prerequisites

- Rust 1.85+ (install via [rustup](https://rustup.rs/))
- Visual Studio Build Tools or Visual Studio 2022 (with MSVC workload)
- If using `rustup-init.exe`, add the MSVC target:
  ```
  rustup target add x86_64-pc-windows-msvc
  ```

#### Get ONNX Runtime

Download the pre-built Windows ORT package:

```powershell
Invoke-WebRequest https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-win-x64-1.23.2.zip -OutFile ort.zip
Expand-Archive ort.zip -DestinationPath C:\onnxruntime-win-x64-1.23.2
```

#### Build

```powershell
# Set environment variables
$env:ORT_LIB_LOCATION = "C:\onnxruntime-win-x64-1.23.2\lib"
$env:ORT_PREFER_DYNAMIC_LINK = "1"

# Build
cargo build --release -p koko --target x86_64-pc-windows-msvc
cargo build --release -p kokoros-server --target x86_64-pc-windows-msvc

# Copy DLL alongside the exes
Copy-Item "C:\onnxruntime-win-x64-1.23.2\bin\onnxruntime.dll" "target\x86_64-pc-windows-msvc\release\"
```

> Windows uses dynamic linking by default — `onnxruntime.dll` must be in the same directory as the exe.
> For static linking, build ORT from source and pass `-StaticLink` (see `build_windows.ps1`).

## Project Structure

```
kokoroi-rs/
├── .github/workflows/
│   └── build.yml           # CI/CD multi-platform auto-build
├── crates/
│   ├── koko-cli/           # CLI binary entrypoint
│   ├── kokoros-core/       # Core TTS engine
│   ├── kokoros-server/     # HTTP API server
│   ├── kokoros-openai/     # OpenAI-compatible API
│   └── misaki/             # G2P engine (grapheme-to-phoneme)
├── scripts/
│   ├── build_musl.sh       # x86_64 musl build (local prebuilt ORT)
│   ├── build_aarch64_musl.sh # ARM64 musl build
│   ├── build_ort_musl.sh   # Generic ORT from-source build + combined lib
│   ├── x86_64-musl-linker.sh
│   ├── aarch64-musl-linker.sh
│   ├── musl_stubs.c        # glibc stub for x86_64 musl ORT
│   ├── build_native.sh     # Native glibc build (dev/debug)
│   ├── build_static.sh
│   ├── build_wasm.sh
│   ├── package.sh
│   ├── download_models.sh
│   └── download_voices.sh
├── config.toml             # Server configuration
├── static/                 # Web page static assets
└── data/                   # Model data (download separately)
```

## FAQ

### Q: "Cannot load model" error

Ensure the ONNX model file path is correct and the file is not corrupted. Model files must be downloaded separately.

### Q: Linux musl static build linker errors

For aarch64 (when using component ORT libs), abseil static libraries have circular dependencies. The `scripts/aarch64-musl-linker.sh` wrapper resolves this with `--start-group`/`--end-group`.

With a combined `libonnxruntime.a` (produced by `scripts/build_ort_musl.sh`), no wrapper is needed since all symbols are merged into one archive. The CI pipeline uses this approach for both architectures.

### Q: Windows error "onnxruntime.dll not found"

Make sure `onnxruntime.dll` is in the same directory as `koko.exe` / `kokoros-server.exe`. Release downloads include this DLL.

### Q: What do I need to build on Windows?

Rust + MSVC toolchain. Install `rustup-init.exe` with the "default toolchain" option to get MSVC automatically, or install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (select the "C++ build tools" workload).

### Q: How to add custom voices?

Voice data is embedded in `voices-v1.0.bin`. For custom voices, refer to the Kokoro official documentation.

## License

This project is open source under the [MIT](LICENSE) license.

## Acknowledgements

- [Kokoro TTS](https://github.com/hexgrad/kokoro) — Original model and training code
- [pyke/ort](https://github.com/pykeio/ort) — Rust ONNX Runtime bindings
- [OnnxruntimeBuilder](https://github.com/csukuangfj/OnnxruntimeBuilder) — ONNX Runtime musl build scripts
