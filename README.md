# Kokoro TTS — 高质量中文语音合成引擎

[![Build](https://github.com/doiito/kokoroi-rs/actions/workflows/build.yml/badge.svg)](https://github.com/doiito/kokoroi-rs/actions/workflows/build.yml)

**kokoroi-rs** 是 [Kokoro](https://github.com/hexgrad/kokoro) 的 Rust 实现，专注于高质量的中文语音合成（Text-to-Speech）。基于 Kokoro 模型，使用 ONNX Runtime 进行推理，提供 CLI 工具、HTTP API 服务和浏览器端 WASM 三种使用方式。

## 特性

- 🎯 **高质量中文语音合成** — 基于 Kokoro v1.1-zh 微调模型，使用 **Bopomofo（注音符号）音素** 替代传统 IPA，支持多音字消歧和变调处理
- 🚀 **高性能推理** — 多线程流水线架构，支持并行生成，实时率可达 5-10x
- 📦 **多平台静态编译** — Linux 下 musl 全静态链接 (x86_64 / ARM64)，零运行时依赖；Windows 下 MSVC 编译，开箱即用
- 🌐 **HTTP API 服务** — 内置 Axum Web 服务器，提供 REST API 和 SSE 流式接口，附带 Web 演示页面（支持直接播放和下载 WAV）
- 🌍 **浏览器 WASM 推理** — Rust WASM 做 G2P（中文分词、注音转换） + ONNX Runtime Web 做模型推理，纯浏览器端运行，无需服务器
- 🎤 **多种语音风格** — 支持 50+ 种发音人，覆盖中、日、韩、英、法等多语言
- ✂️ **智能文本分片** — 基于音素限制的智能分片策略，兼顾生成速度与自然度
- 📝 **流式输出** — 支持 SSE 实时流式音频输出，适用于 Web 端低延迟场景
- 🐍 **Python 工具链** — 包含模型导出（.onnx → Q8 量化）、G2P 权重提取、OpenAI 兼容客户端等脚本
- 🤖 **CI/CD 自动构建** — GitHub Actions 全自动编译，提交即出包

## 快

### 下载预编译二进制

从 [Releases](https://github.com/doiito/kokoroi-rs/releases) 页面下载对应平台的编译产物：

| 平台 | 文件 | 说明 |
|------|------|------|
| x86_64 Linux | `kokoro-x86_64-unknown-linux-musl.tar.gz` | 全静态 ELF，零依赖 |
| ARM64 Linux | `kokoro-aarch64-unknown-linux-musl.tar.gz` | 全静态 ELF，零依赖 |
| x86_64 Windows | `kokoro-x86_64-pc-windows-msvc.zip` | 含 `onnxruntime.dll`，解压即用 |

**Windows 用户注意**：压缩包内已包含 `onnxruntime.dll`，请确保 exe 与该 dll 在同一目录。

### 下载模型文件

需要准备以下模型文件：

- **Kokoro v1.1-zh ONNX 模型**：`models/kokoro-v1.1-zh-m.onnx`（约 79MB，int8 量化，推荐）
- **发音人数据**：`data/voices-v1.0.bin`（约 27MB）
- **配置文件**：`config.toml`（项目根目录已提供）

项目提供三种精度的 v1.1-zh 模型（位于 `models/` 目录）：

| 模型 | 大小 | 精度 | 说明 |
|------|------|------|------|
| `kokoro-v1.1-zh-s.onnx` | 47MB | int4 | 最小体积，适合快速下载和低内存环境 |
| `kokoro-v1.1-zh-m.onnx` | 79MB | int8 | **默认模型**，平衡体积与质量 |
| `kokoro-v1.1-zh-l.onnx` | 311MB | fp32 | 完整精度，质量最佳 |

模型文件可通过以下途径获取：

1. 从本项目的 [Releases](https://github.com/doiito/kokoroi-rs/releases) 页面下载
2. 运行 `./scripts/download_all.sh` 自动下载

> **G2P 架构变更**：v1.1-zh 模型使用 **Bopomofo（注音符号）音素** 替代传统 IPA 音素，配合 `ZH_VOCAB` 词表进行 tokenization。中文处理包括 jieba 分词、多音字消歧和变调处理，生成更自然的中文发音。

### 使用 CLI

完整 CLI 选项：

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `-t, --text` | 输入文本 | — |
| `-i, --input` | 输入文件 | — |
| `-m, --model` | ONNX 模型路径 | `models/kokoro-v1.1-zh-m.onnx` |
| `-d, --data` | 发音人数据路径 | `data/voices-v1.0.bin` |
| `-o, --output` | 输出文件 | `output.wav` |
| `-l, --lan` | 语言代码 | `zh` |
| `-s, --style` | 发音人 | `zm_yunyang` |
| `-p, --speed` | 语速倍率 | `0.7` |
| `-n, --threads` | 推理线程数 | `2` |
| `--max-chars` | 每块最大字符数 | `150` |
| `--max-phonemes` | 每块最大音素数 | `510` |
| `-P, --play` | 播放音频（需 aplay/ffplay） | — |

```bash
# 基本用法：输入文本生成语音
./koko --text "你好，欢迎使用 Kokoro 语音合成系统。" -o output.wav

# 从文件读取文本
./koko -i input.txt -o output.wav

# 指定发音人风格
./koko --text "今天天气真好" --style zf_xiaobei -o output.wav

# 指定模型（使用 fp32 全精度模型）
./koko --text "大家好" --model models/kokoro-v1.1-zh-l.onnx -o output.wav

# 调整语速
./koko --text "大家好" --speed 1.2 -o output.wav

# 调整推理线程数
./koko --text "长文本测试" --threads 4 -o output.wav

# 播放音频（需要 aplay 或 ffplay）
./koko --text "播放测试" --play
```

> **自动下载模型**：如果 `--model` 或 `--data` 指定的文件不存在，`koko` 会自动下载缺失的模型文件。
> 首次运行会自动下载 `models/kokoro-v1.1-zh-m.onnx`（约 79MB）和 `data/voices-v1.0.bin`（约 27MB）。

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
| `GET` | `/` | Web 演示页面（SSE 流式播放 + WAV 下载） |
| `GET` | `/health` | 健康检查 |
| `GET` | `/voices` | 获取所有可用发音人 |
| `POST` | `/tts` | 文本转语音（返回 JSON + Base64 WAV） |
| `POST` | `/tts/stream` | 流式文本转语音（SSE，逐块播放） |

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
model_path = "models/kokoro-v1.1-zh-m.onnx"   # v1.1-zh 模型路径
voices_path = "data/voices-v1.0.bin"          # 发音人数据路径
```

也可通过环境变量 `KOKOROS_CONFIG` 指定自定义配置文件路径。

## Feature Flags

kokoros-core 库提供以下 Cargo features：

| Feature | 说明 | 默认启用 |
|---------|------|----------|
| `ort` | ONNX Runtime 后端（pyke/ort） | ✅ |
| `chinese` | 中文支持（分词、归一化） | ✅ |
| `cuda` | CUDA GPU 推理 | ❌ |
| `wasm` | WASM 浏览器目标（G2P 音素化，不含 ONNX 推理） | ❌ |
| `onnx` | tract-onnx 纯 Rust ONNX 推理 | ❌ |
| `download` | 模型自动下载 | ❌ |
| `audio-encode` | MP3 / Opus 编码 | ❌ |
| `native` | 全功能原生构建（ort + download + audio-encode） | ❌ |

## CI/CD 自动构建

本项目使用 GitHub Actions 全自动构建多平台二进制。每次推送或创建标签时，流水线会自动：

| 架构 | 运行环境 | 构建方式 |
|------|---------|---------|
| x86_64 Linux musl | ubuntu-latest | ONNX Runtime 源码编译，musl 全静态链接 |
| ARM64 Linux musl | ubuntu-latest | ORT 源码交叉编译，aarch64 musl 静态链接 |
| x86_64 Windows MSVC | windows-latest | 下载预编译 ORT，MSVC 动态链接 |
| WASM32 browser | ubuntu-latest | wasm-pack, WASM G2P + ONNX Runtime Web（运行时加载） |

Linux musl 的 ORT 第一次构建约需 20–30 分钟（cmake 编译），之后缓存命中仅需 2–5 分钟。

工作流文件：[`.github/workflows/build.yml`](.github/workflows/build.yml)

## 从源码构建

### Linux (musl 全静态)

#### 前置条件

- Rust 1.85+（推荐使用 [rustup](https://rustup.rs/) 安装）
- cmake, ninja, protobuf-compiler, wget

#### 一键构建（推荐）

项目提供 `scripts/build_musl.sh` 和 `scripts/build_aarch64_musl.sh`，使用本地预编译 ORT 库快速构建：

```bash
# x86_64 musl
./scripts/build_musl.sh

# ARM64 musl
./scripts/build_aarch64_musl.sh
```

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

# 4. 构建 koko / kokoros-server
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
Invoke-WebRequest https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-win-x64-1.23.2.zip -OutFile ort.zip
Expand-Archive ort.zip -DestinationPath C:\onnxruntime-win-x64-1.23.2
```

#### 构建

```powershell
# 设置环境变量
$env:ORT_LIB_LOCATION = "C:\onnxruntime-x64-1.23.2\lib"
$env:ORT_PREFER_DYNAMIC_LINK = "1"

# 构建
cargo build --release -p koko --target x86_64-pc-windows-msvc
cargo build --release -p kokoros-server --target x86_64-pc-windows-msvc

# 复制 ONNX Runtime DLL 到输出目录
Copy-Item "C:\onnxruntime-x64-1.23.2\bin\onnxruntime.dll" "target\x86_64-pc-windows-msvc\release\"
```

> Windows 构建默认使用动态链接，`onnxruntime.dll` 需要与 exe 在同一目录。
> 如需静态链接，可自行编译 ORT 静态库并设置 `-StaticLink` 参数（参考 `scripts/build_windows.ps1`）。

### WASM (浏览器)

#### 前置条件

- Rust 1.85+（[rustup](https://rustup.rs/) 安装）
- wasm32-unknown-unknown target：`rustup target add wasm32-unknown-unknown`
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

#### 构建

```bash
# 一键构建
./scripts/build_wasm.sh

# 或手动执行
wasm-pack build crates/kokoros-core \
  --target web \
  --out-dir ../../static/wasm-pkg \
  -- \
  --features wasm \
  --no-default-features
```

产物输出到 `static/wasm-pkg/`：
| 文件 | 说明 |
|------|------|
| `kokoros_bg.wasm` | WASM 二进制 (约 1.6MB，仅 G2P 音素化，不含 ONNX 推理) |
| `kokoros.js` | wasm-bindgen 胶水代码 |
| `kokoros.d.ts` | TypeScript 类型声明 |

构建后可通过 `static/` 目录下的 `rust_wasm_demo.html` 在浏览器中测试。

> **架构说明**：WASM 二进制仅处理 G2P（中文分词、拼音转换、音素化）。ONNX 模型推理使用 [ONNX Runtime Web](https://github.com/microsoft/onnxruntime-web)（CDN: `onnxruntime-web@1.21.0`），在浏览器中通过 JavaScript 加载和执行。模型文件（`*.onnx`、`voices.json`、`*.bin`）需要单独下载，参考 FAQ。

### Python 脚本

Python 脚本目录（`scripts/`）提供模型导出和辅助工具：

| 脚本 | 用途 |
|------|------|
| `export_q8.py` | Kokoro ONNX 模型 → Q8 量化导出 |
| `test_q8_model.py` | 量化模型验证 |
| `extract_g2pm_weights.py` | G2P 权重提取（→ Rust 常量） |
| `run_openai.py` | OpenAI 兼容 API 测试客户端 |

```bash
pip install -r scripts/requirements.txt

# 导出 Q8 量化模型
python scripts/export_q8.py \
  --kokoro-src /path/to/kokoro \
  --output-dir ./models

# 验证模型
python scripts/test_q8_model.py --model ./models/model_fp16.onnx
```

另外 `static/server.py` 提供了一个基于 Python ONNX Runtime 的轻量级后端服务器，不依赖 Rust 编译：

```bash
cd static
pip install onnxruntime numpy
python server.py --port 8080
```

## 项目结构

```
kokoroi-rs/
├── .github/workflows/
│   └── build.yml              # CI/CD 多平台自动构建
├── crates/
│   ├── koko-cli/              # CLI 工具入口
│   ├── kokoros-core/          # 核心 TTS 引擎（含 WASM 入口）
│   │   └── src/
│   │       ├── tts/
│   │       │   ├── chinese/   # 中文 G2P（分词、拼音、注音、变调）
│   │       │   ├── koko.rs    # TTS 引擎（tokenize + ONNX 推理）
│   │       │   ├── phonemizer.rs  # 音素化（Bopomofo 模式）
│   │       │   ├── tokenize.rs    # 词表 tokenization（ZH_VOCAB）
│   │       │   └── vocab.rs       # 词表定义（ZH_VOCAB + MODEL_VOCAB）
│   │       ├── onn/           # ONNX Runtime 推理后端
│   │       └── wasm/          # WASM 浏览器入口
│   ├── kokoros-server/        # HTTP API 服务器
│   └── misaki/                # G2P（字素转音素）引擎
├── models/                    # v1.1-zh ONNX 模型文件
│   ├── kokoro-v1.1-zh-s.onnx  # int4 量化（47MB）
│   ├── kokoro-v1.1-zh-m.onnx  # int8 量化（79MB，默认）
│   └── kokoro-v1.1-zh-l.onnx  # fp32 全精度（311MB）
├── data/                      # 模型数据
│   └── voices-v1.0.bin        # 发音人嵌入数据（.npz 格式）
├── scripts/
│   ├── build_musl.sh          # x86_64 musl 构建
│   ├── build_aarch64_musl.sh  # ARM64 musl 构建
│   ├── build_ort_musl.sh      # 通用 ORT 源码编译
│   ├── build_native.sh        # 原生 glibc 构建（开发调试）
│   ├── build_wasm.sh          # WASM 浏览器目标构建
│   ├── build_windows.ps1      # Windows MSVC 构建
│   ├── build_static.sh
│   ├── package.sh             # 打包发布
│   ├── download_all.sh        # 一键下载模型
│   ├── download_models.sh
│   ├── download_voices.sh
│   ├── x86_64-musl-linker.sh
│   ├── aarch64-musl-linker.sh
│   ├── musl_stubs.c           # musl ORT glibc 符号桩
│   ├── export_q8.py           # ONNX → Q8 导出
│   ├── test_q8_model.py       # Q8 模型验证
│   ├── extract_g2pm_weights.py # G2P 权重提取
│   ├── run_openai.py          # OpenAI API 测试客户端
│   └── requirements.txt       # Python 依赖
├── static/
│   ├── index.html             # API 服务器演示页（SSE + WAV 下载）
│   ├── wasm_demo.html         # ONNX Runtime Web + WASM G2P 演示
│   ├── browser_demo.html      # 流式语音合成演示
│   ├── rust_wasm_demo.html    # WASM G2P + ONNX Runtime Web 混合演示（v1.1-zh S/M/L 模型）
│   ├── kokoros.d.ts           # WASM TypeScript 类型定义
│   ├── models/
│   │   ├── onnx/              # 浏览器用 ONNX 模型
│   │   └── voices/            # 发音人配置 + 风格嵌入 *.bin
│   ├── server.py              # Python 后端服务器
│   └── wasm-pkg/              # wasm-pack 构建产物
│       ├── kokoros_bg.wasm
│       ├── kokoros.js
│       └── kokoros.d.ts
├── config.toml                # 服务器配置
└── data/                      # 模型数据（需自行下载）
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

### Q: WASM demo 页面无法加载

WASM demo 页面需要以下额外数据文件放置在 `static/` 目录下（不在代码仓库中，需从 [Releases](https://github.com/doiito/kokoroi-rs/releases) 下载）：

- `models/onnx/kokoro-v1.1-zh-*.onnx` — v1.1-zh ONNX 模型文件（S/M/L 三选一）
- `models/voices/voices.json` — 发音人配置
- `models/voices/*.bin` — 语音风格嵌入数据

### Q: WASM demo 用的是什么推理引擎？

Rust WASM 二进制（`kokoros_bg.wasm`）仅处理 G2P（中文分词、注音转换），模型推理使用 [ONNX Runtime Web](https://github.com/microsoft/onnxruntime-web) v1.21.0，通过 CDN 加载（`https://cdn.jsdelivr.net/npm/onnxruntime-web@1.21.0/dist/ort.min.js`）。所有计算在浏览器本地完成，无需服务器。

### Q: CLI/Server 使用的是哪种 G2P 模式？

v1.1-zh 模型使用 **Bopomofo（注音符号）音素** 模式：
- 中文文本 → jieba 分词 → 拼音（含多音字消歧）→ 变调处理 → Bopomofo 注音 → ZH_VOCAB tokenization → ONNX 推理
- 相比传统的 IPA 音素方案，Bopomofo 在中文字素映射上更精确，发音更自然

### Q: 如何切换模型精度？

CLI 使用 `--model` 参数：
```bash
./koko --text "你好" --model models/kokoro-v1.1-zh-l.onnx
```

Server 修改 `config.toml` 中的 `model_path` 后重启。

### Q: 什么音频后处理？

CLI 和 Server 使用简单的振幅阈值静音裁切（无 DC 偏移消除、无淡入淡出），与 Kokoros-main 参考实现行为一致。模型原始输出中的前导静音（~730ms）和尾部低噪（~1.1s）被裁切，保留约 50ms 的自然前置静音。

### Q: Windows 构建需要安装什么？

需要 Rust + MSVC 工具链。安装 `rustup-init.exe` 时选择 "default toolchain" 即可自动配置 MSVC。
如需最小安装，也可使用 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（仅安装 "C++ 生成工具" 工作负载）。

### Q: 如何添加新的发音人？

Kokoro TTS 的发音人数据包含在 `voices-v1.0.bin` 文件中。如需自定义发音人，请参考 Kokoro 官方文档。

## 许可证

本项目基于 [MIT](LICENSE) 许可证开源。

## 致谢

- [Kokoro](https://github.com/hexgrad/kokoro) — 原始模型和训练代码
- [pyke/ort](https://github.com/pykeio/ort) — Rust ONNX Runtime 绑定
- [ONNX Runtime Web](https://github.com/microsoft/onnxruntime-web) — 浏览器端 ONNX 推理引擎
- [OnnxruntimeBuilder](https://github.com/csukuangfj/OnnxruntimeBuilder) — ONNX Runtime musl 构建脚本
