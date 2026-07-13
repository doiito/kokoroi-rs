# Kokoro TTS — 高质量中文语音合成引擎

[![Build](https://github.com/your-org/kokoroi-rs/actions/workflows/build.yml/badge.svg)](https://github.com/your-org/kokoroi-rs/actions/workflows/build.yml)

**kokoroi-rs** 是 [Kokoro TTS](https://github.com/kokoro-tts) 的 Rust 实现，专注于高质量的中文语音合成（Text-to-Speech）。基于 Kokoro 模型，使用 ONNX Runtime 进行推理，提供 CLI 工具和 HTTP API 服务两种使用方式。

## 特性

- 🎯 **高质量中文语音合成** — 基于 Kokoro 模型，支持多种中文发音人
- 🚀 **高性能推理** — 多线程流水线架构，支持并行生成，实时率可达 5-10x
- 📦 **多平台静态编译** — Linux 下 musl 全静态链接 (x86_64 / ARM64)，零运行时依赖；Windows 下 MSVC 编译，开箱即用
- 🌐 **HTTP API 服务** — 内置 Axum Web 服务器，提供 REST API 和 SSE 流式接口，附带 Web 演示页面
- 🎤 **多种语音风格** — 支持 50+ 种发音人，覆盖中、日、韩、英、法等多语言
- ✂️ **智能文本分片** — 基于音素限制的智能分片策略，兼顾生成速度与自然度
- 📝 **流式输出** — 支持 SSE 实时流式音频输出，适用于 Web 端低延迟场景
- 🤖 **CI/CD 自动构建** — GitHub Actions 全自动编译，提交即出包

## 快速开始

### 下载预编译二进制

从 [Releases](https://github.com/your-org/kokoroi-rs/releases) 页面下载对应平台的编译产物：

| 平台 | 文件 | 说明 |
|------|------|------|
| x86_64 Linux | `kokoro-x86_64-unknown-linux-musl.tar.gz` | 全静态 ELF，零依赖 |
| ARM64 Linux | `kokoro-aarch64-unknown-linux-musl.tar.gz` | 全静态 ELF，零依赖 |
| x86_64 Windows | `kokoro-x86_64-pc-windows-msvc.zip` | 含 `onnxruntime.dll`，解压即用 |

**Windows 用户注意**：压缩包内已包含 `onnxruntime.dll`，请确保 exe 与该 dll 在同一目录。

### 下载模型文件

需要准备以下模型文件：

- **Kokoro ONNX 模型**：`models/kokoro-v1.0.onnx`（约 80MB）
- **发音人数据**：`data/voices-v1.0.bin`（约 150MB）
- **配置文件**：`config.toml`（项目根目录已提供）

模型文件可从以下途径获取：

1. 从 [Kokoro 官方项目](https://github.com/kokoro-tts/kokoro) 下载
2. 运行 `./scripts/download_models.sh` 自动下载

### 使用 CLI

```bash
# 基本用法：输入文本生成语音
./koko --text "你好，欢迎使用 Kokoro 语音合成系统。" -o output.wav

# 从文件读取文本
./koko -i input.txt -o output.wav

# 指定发音人风格（默认 zm_yunyang）
./koko --text "今天天气真好" --style zf_xiaobei -o output.wav

# 调整语速
./koko --text "大家好" --speed 1.2 -o output.wav

# 调整推理线程数
./koko --text "长文本测试" --threads 4 -o output.wav

# 播放音频（需要 aplay 或 ffplay）
./koko --text "播放测试" --play
```

Windows 下使用方式相同，将 `./koko` 替换为 `koko.exe` 即可。

### 启动 API 服务器

```bash
# 使用默认配置启动
./kokoros-server

# 指定自定义配置
KOKOROS_CONFIG=my_config.toml ./kokoros-server
```

服务器默认监听 `0.0.0.0:3000`，打开浏览器访问即可看到 Web 演示界面。

#### API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/` | Web 演示页面 |
| `GET` | `/health` | 健康检查 |
| `GET` | `/voices` | 获取所有可用发音人 |
| `POST` | `/tts` | 文本转语音（返回 WAV Base64） |
| `POST` | `/tts/stream` | 流式文本转语音（SSE） |

**POST `/tts` 请求示例：**

```json
{
  "text": "你好世界",
  "voice": "zf_xiaobei",
  "speed": 1.0
}
```

**成功响应：**

```json
{
  "success": true,
  "audio_base64": "UklGRiQ...base64编码的WAV数据",
  "duration": 2.35
}
```

## 支持的发音人

总共支持 50+ 种发音人，涵盖多种语言和风格：

| 前缀 | 语言 | 示例 |
|------|------|------|
| `zf_`, `zm_` | 中文 | `zf_xiaobei`, `zm_yunyang` |
| `af_`, `am_` | 英文 | `af_bella`, `am_adam` |
| `jf_`, `jm_` | 日文 | `jf_alpha`, `jm_kumo` |
| `bf_`, `bm_` | 英式英文 | `bf_alice`, `bm_daniel` |
| `ef_`, `em_` | 西班牙文 | `ef_dora`, `em_alex` |
| `ff_` | 法文 | `ff_siwis` |
| `hf_`, `hm_` | 印地文 | `hf_alpha`, `hm_omega` |
| `if_`, `im_` | 意大利文 | `if_sara`, `im_nicola` |
| `pf_`, `pm_` | 葡萄牙文 | `pf_dora`, `pm_alex` |

## 配置

服务器通过 `config.toml` 文件配置：

```toml
# Kokoro TTS Server Configuration
host = "0.0.0.0"        # 监听地址
port = 3000             # 监听端口
threads = 4             # 推理线程数
max_chars = 400         # 单次最大字符数
model_path = "models/kokoro-v1.0.onnx"   # 模型路径
voices_path = "data/voices-v1.0.bin"     # 发音人数据路径
```

也可通过环境变量 `KOKOROS_CONFIG` 指定自定义配置文件路径。

## CI/CD 自动构建

本项目使用 GitHub Actions 全自动构建多平台二进制。每次推送或创建标签时，流水线会自动：

| 架构 | 运行环境 | 构建方式 |
|------|---------|---------|
| x86_64 Linux musl | ubuntu-latest | ONNX Runtime 源码编译，musl 全静态链接 |
| ARM64 Linux musl | ubuntu-latest | ORT 源码交叉编译，aarch64 musl 静态链接 |
| x86_64 Windows MSVC | windows-latest | 下载预编译 ORT，MSVC 动态链接 |

Linux musl 的 ORT 第一次构建约需 20–30 分钟（cmake 编译），之后缓存命中仅需 2–5 分钟。

工作流文件：[`.github/workflows/build.yml`](.github/workflows/build.yml)

## 从源码构建

### Linux (musl 全静态)

#### 前置条件

- Rust 1.85+（推荐使用 [rustup](https://rustup.rs/) 安装）
- cmake, ninja, protobuf-compiler, wget

#### 一键构建（推荐）

项目提供 `scripts/build_ort_musl.sh` 自动编译 ONNX Runtime 并创建组合 `libonnxruntime.a`：

```bash
# x86_64 musl
./scripts/build_musl.sh

# ARM64 musl
./scripts/build_aarch64_musl.sh
```

这两个脚本使用项目本地的预编译 ORT 库，适合已有开发环境的场景。

**使用 build_ort_musl.sh 自己编译 ORT：**

```bash
# 1. 下载 musl 交叉编译工具链
wget https://musl.cc/x86_64-linux-musl-cross.tgz
tar -xzf x86_64-linux-musl-cross.tgz

# 2. 下载 ONNX Runtime 源码
wget https://github.com/microsoft/onnxruntime/archive/refs/tags/v1.23.2.tar.gz
tar -xzf v1.23.2.tar.gz

# 3. 一键编译 ORT + 应用
export PATH="/path/to/x86_64-linux-musl-cross/bin:$PATH"
bash scripts/build_ort_musl.sh \
  x86_64-unknown-linux-musl \
  /path/to/x86_64-linux-musl-cross \
  /path/to/onnxruntime-1.23.2 \
  /path/to/output

# 4. 构建 koko/kokoros-server
ORT_LIB_LOCATION=/path/to/output \
RUSTFLAGS="-C target-feature=+crt-static" \
cargo build --release -p koko --target x86_64-unknown-linux-musl --no-default-features
cargo build --release -p kokoros-server --target x86_64-unknown-linux-musl --no-default-features
```

> ARM64 (aarch64) 同理，将 `x86_64` 替换为 `aarch64` 即可。
>
> musl 构建启用 `--no-default-features` 以跳过音频编码依赖（mp3lame, audiopus），
> 这些 crate 在 musl 交叉编译环境下不可用。

### Windows (MSVC)

#### 前置条件

- Rust 1.85+（[rustup](https://rustup.rs/) 安装）
- Visual Studio Build Tools 或 Visual Studio 2022（含 MSVC 工具链）
- 若使用 `rustup-init.exe` 安装，添加 MSVC target：
  ```
  rustup target add x86_64-pc-windows-msvc
  ```

#### 获取 ONNX Runtime

下载预编译的 Windows 版 ONNX Runtime：

```powershell
# 下载
Invoke-WebRequest https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-win-x64-1.23.2.zip -OutFile ort.zip
Expand-Archive ort.zip -DestinationPath C:\onnxruntime-win-x64-1.23.2
```

#### 构建

```powershell
# 设置环境变量
$env:ORT_LIB_LOCATION = "C:\onnxruntime-win-x64-1.23.2\lib"
$env:ORT_PREFER_DYNAMIC_LINK = "1"

# 构建
cargo build --release -p koko --target x86_64-pc-windows-msvc
cargo build --release -p kokoros-server --target x86_64-pc-windows-msvc

# 复制 ONNX Runtime DLL 到输出目录
Copy-Item "C:\onnxruntime-win-x64-1.23.2\bin\onnxruntime.dll" "target\x86_64-pc-windows-msvc\release\"
```

> Windows 构建默认使用动态链接，`onnxruntime.dll` 需要与 exe 在同一目录。
> 如需静态链接，可自行编译 ORT 静态库并设置 `-StaticLink` 参数（参考 `build_windows.ps1`）。

## 项目结构

```
kokoroi-rs/
├── .github/workflows/
│   └── build.yml           # CI/CD 多平台自动构建
├── crates/
│   ├── koko-cli/           # CLI 工具入口
│   ├── kokoros-core/       # 核心 TTS 引擎
│   ├── kokoros-server/     # HTTP API 服务器
│   ├── kokoros-openai/     # OpenAI 兼容 API
│   └── misaki/             # G2P（字素转音素）引擎
├── scripts/
│   ├── build_musl.sh       # x86_64 musl 构建（本地预编译 ORT）
│   ├── build_aarch64_musl.sh # ARM64 musl 构建
│   ├── build_ort_musl.sh   # 通用 ORT 源码编译 + 组合 lib
│   ├── x86_64-musl-linker.sh
│   ├── aarch64-musl-linker.sh
│   ├── musl_stubs.c        # x86_64 musl ORT 所需的 glibc 符号桩
│   ├── build_native.sh     # 原生 glibc 构建（开发调试）
│   ├── build_static.sh
│   ├── build_wasm.sh
│   ├── package.sh
│   ├── download_models.sh
│   └── download_voices.sh
├── config.toml             # 服务器配置
├── static/                 # Web 页面静态资源
└── data/                   # 模型数据（需自行下载）
```

## 常见问题

### Q: 运行时报错 "Cannot load model"

确保 ONNX 模型文件路径正确，且文件未被损坏。模型文件需单独下载。

### Q: Linux musl 静态构建报链接错误

aarch64 目标（使用组件 ORT 库时）需要 `--start-group`/`--end-group` 解决 abseil 循环依赖。`scripts/aarch64-musl-linker.sh` 自动处理此问题。

x86_64 目标若使用组合 `libonnxruntime.a` 则无需此处理，因为所有符号已合并到一个档案中。

CI 流水线使用 `scripts/build_ort_musl.sh` 自动生成组合 `libonnxruntime.a`，两种架构均无需 `--start-group`。

### Q: Windows 下报错 "找不到 onnxruntime.dll"

请确保 `onnxruntime.dll` 与 `koko.exe` / `kokoros-server.exe` 在同一目录。从 Releases 下载的 Windows 压缩包已包含此 DLL。

### Q: Windows 构建需要安装什么？

需要 Rust + MSVC 工具链。安装 `rustup-init.exe` 时选择 "default toolchain" 即可自动配置 MSVC。
如需最小安装，也可使用 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（仅安装 "C++ 生成工具" 工作负载）。

### Q: 如何添加新的发音人？

Kokoro TTS 的发音人数据包含在 `voices-v1.0.bin` 文件中。如需自定义发音人，请参考 Kokoro 官方文档。

## 许可证

本项目基于 [MIT](LICENSE) 许可证开源。

## 致谢

- [Kokoro TTS](https://github.com/kokoro-tts/kokoro) — 原始模型和训练代码
- [pyke/ort](https://github.com/pykeio/ort) — Rust ONNX Runtime 绑定
- [OnnxruntimeBuilder](https://github.com/csukuangfj/OnnxruntimeBuilder) — ONNX Runtime musl 构建脚本
