# Kokoro TTS Rust — Architecture Design Document

> **Version:** 0.1  
> **Last updated:** 2026-07-14  
> **Project:** [kokoroi-rs](https://github.com/doiito/kokoroi-rs) — Rust 实现的高质量中文语音合成引擎

---

## 1. Overview

kokoroi-rs 是 [Kokoro TTS](https://github.com/kokoro-tts/kokoro) 的 Rust 实现。它将 Kokoro ONNX 模型、中文 G2P（Grapheme-to-Phoneme）引擎、多线程流水线推理和 HTTP 服务层组合成一个高性能、可独立部署的文本转语音系统。

### 1.1 核心设计目标

- **高质量中文语音合成** — 基于 Kokoro 模型，支持 50+ 发音人，覆盖中、日、韩、英、法等多语言
- **高吞吐量** — 多线程流水线架构，文本分片 + 并行推理 + 有序输出，实时率 5–10x
- **零依赖部署** — Linux musl 全静态编译，单 ELF 二进制运行
- **多接口** — CLI / REST API / SSE 流式 / OpenAI 兼容 API
- **跨平台** — x86_64 Linux / ARM64 Linux / x86_64 Windows / WASM

### 1.2 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interfaces                       │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ koko-cli │  │ kokoros-svr │  │ kokoros-openai       │   │
│  │ (CLI)    │  │ (REST+SSE)  │  │ (OpenAI-compat API)  │   │
│  └────┬─────┘  └──────┬───────┘  └──────────┬───────────┘   │
│       │               │                      │              │
├───────┴───────────────┴──────────────────────┴──────────────┤
│                    kokoros-core (核心引擎)                    │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                   Pipeline                               │ │
│  │  TextSplitter → Preprocessor → Generator → AudioQueue  │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌──────────┐  ┌──────────┐  ┌─────────────────────────┐   │
│  │ Chinese  │  │ misaki   │  │ ONNX Inference           │   │
│  │ G2P      │  │ (EN G2P) │  │ (ort::Session)           │   │
│  └──────────┘  └──────────┘  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Crate Architecture (Monorepo)

工作区使用 Cargo workspace resolver "3"，包含 5 个 crate：

| Crate | Type | Purpose |
|-------|------|---------|
| `kokoros-core` | 库 (`cdylib` + `rlib`) | 核心 TTS 引擎、流水线、G2P、ONNX 推理 |
| `koko-cli` | 二进制 | CLI 工具 |
| `kokoros-server` | 二进制 | HTTP REST + SSE API 服务器 |
| `kokoros-openai` | 库 | OpenAI 兼容 API 服务器 |
| `misaki` | 库 | 英语 POS-aware G2P 引擎 |

### 2.1 依赖关系

```
koko-cli ──────────┐
                    ├──► kokoros-core ──► ort (ONNX Runtime)
kokoros-server ─────┘         │
                    │         ├──► misaki
kokoros-openai ─────┘         │
                    └──► jieba-rs (中文分词)
                         ├──► pinyin, chinese-number
                         ├──► ndarray, ndarray-npy
                         ├──► crossbeam (流水线通道)
                         └──► hound (WAV 编码)
```

### 2.2 kokoros-core 特性矩阵

`kokoros-core` 通过 Cargo features 控制编译内容：

| Feature | 用途 | 默认 |
|---------|------|------|
| `ort` | ONNX Runtime 后端 | ✅ |
| `chinese` | 中文 G2P 支持 | ✅ |
| `cuda` | CUDA 执行提供程序 | ❌ |
| `download` | 自动下载模型文件 | ❌ |
| `audio-encode` | MP3/Opus 编码 | ❌ |
| `native` | 本地开发全功能集 | ❌ |
| `wasm` | WASM 编译目标 | ❌ |
| `tract-onnx` | tract-onnx 替代后端 | ❌ |
| `oxionnx` | oxiONNX 替代后端 | ❌ |

---

## 3. Core Pipeline Architecture

流水线是系统的核心。它将 TTS 过程分为三个阶段，通过有界通道连接，支持并行处理。

### 3.1 流水线总图

```
┌──────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  Input Stage │     │  Preprocess      │     │  Generate        │
│              │     │  (1 thread)      │     │  (N workers)     │
│  TextChunks  │────►│                  │────►│                  │────► Audio
│  (crossbeam  │     │  clean_text()    │     │  ONNX inference  │      Output
│   bounded)   │     │  split_sentences │     │  per chunk       │
│              │     │  phonemize()     │     │                  │
│              │     │  tokenize()      │     │  SortedAudioQueue│
│              │     │  filter/validate │     │  (BTreeMap)      │
│              │     │                  │     │                  │
└──────────────┘     └──────────────────┘     └──────────────────┘
     TextChunkQueue        TokenQueue              SortedAudioQueue
```

### 3.2 数据结构

#### 3.2.1 通道类型

```
TextChunkQueue — crossbeam::bounded 通道, 承载 PipelineMessage<TextChunk>
TokenQueue    — crossbeam::bounded 通道, 承载 PipelineMessage<ChunkTask>
SortedAudioQueue — Mutex<BTreeMap<usize, Vec<f32>>> + Condvar
```

所有跨线程通信使用 `crossbeam::channel::bounded`，避免无界队列导致内存膨胀。

#### 3.2.2 关键类型

```rust
TextChunk   { index: usize, text: Arc<String> }
ChunkTask   { index: usize, text: Arc<String> }
AudioChunk  { index: usize, samples: Vec<f32> }
PipelineMessage<T> = Data(T) | End   // 哨兵模式
```

在 `Preprocessor` 和 `Generator` 之间通过 `TokenQueue` 传递 `ChunkTask`。预处理器通过 `send_end()` 发送与 worker 数量匹配的 End 信号，每个 worker 消费一个后退出。

#### 3.2.3 SortedAudioQueue

`SortedAudioQueue` 是一个有序的音频块累积器。它接收来自多个 Generator worker 的音频块（可能乱序到达），并按 `index` 顺序输出：

```rust
SortedAudioQueue {
    data:       Mutex<BTreeMap<usize, Vec<f32>>>,  // 按 index 排序
    condvar:    Condvar,                             // 唤醒等待的消费者
    next_index: Mutex<usize>,                        // 下一个应输出的序号
    ended:      Mutex<bool>,                         // 是否所有块已提交
}
```

核心机制：
- `push(index, samples)` — 插入 BTreeMap，notify 消费者
- `pop_next(timeout_ms)` — 若 `next_index` 对应的块就绪则返回，否则 condvar wait
- `mark_ended()` — 标记完成，`is_complete()` 返回 true 后消费者可终止
- `mark_failed(index)` — 插入空 Vec 避免死锁

### 3.3 阶段一：Text Splitter（文本分片）

**模块:** `pipeline/text_splitter.rs`

输入原始文本，输出按语义边界 + 音素限制分割的句子列表。

#### 3.3.1 句子分割 (split_into_sentences)

基于规则的分句器，处理：

- **终止符识别:** `. ! ? … 。！？` 以及换行符
- **括号匹配:** 阻止在括号/引号内部分句（维护 stack）
- **缩写排除:** `Mr. Dr. U.S.A.` 等不触发分句
- **URL/Email 排除:** 包含 `://` 或 `@` 的 token 不触发分句
- **数字列表排除:** 行首数字 + 空格不触发分句
- **中文标点映射:** 全角 → 半角

#### 3.3.2 音素限制分片 (split_by_phoneme_limit)

在句子分割基础上，将长文本切成音素数不超过 `max_phonemes`（默认 510）的块：

1. 逐句计算音素数，累积到当前块
2. 若超限则开始新块
3. 若单句已超限，使用 `split_long_sentence`：
   - 优先在 `! . ? …` 处分
   - 其次在 `: ;` 处分
   - 再者在 `, ，、` 处分
   - 最后在空白处分（waterfall 策略）
   - 如果仍然超限，使用 `split_recursive` 二分法

### 3.4 阶段二：Preprocessor（预处理）

**模块:** `pipeline/preprocessor.rs`

在独立线程中运行，消费 `TextChunkQueue`，产出 `TokenQueue`。

处理流程：

```
TextChunk
  │
  ├─ clean_text() ── 过滤控制字符，trim，合并空白
  │
  ├─ split_by_phoneme_limit(text, |t| phonemize(t).chars().count())
  │     └─ 内部调用 TextSplitter，phonemize 获得音素长度
  │
  ├─ 对每个子块：phonemize → 检查 phoneme_len ∈ [min, max]
  │     ├─ 过短（< min_phonemes=3）→ 跳过并 warn
  │     ├─ 过长（> max_phonemes=510）→ 跳过并 warn
  │     └─ 合法 → 封装为 ChunkTask { index, text } → send()
  │
  └─ 收到 End → 向每个 worker 发送 End → 终止
```

**关键设计:** 预处理器使用 `Arc<AtomicUsize>` 报告有效的输出块数，供 Generator 端确认总工作量。

Stats 收集（通过 `PreprocessStats`）:
- `total_input_chunks` — 输入块数
- `valid_output_chunks` — 有效输出块数
- `skipped_empty / skipped_short / skipped_long` — 跳过的原因统计
- `split_count` — 额外分片数

### 3.5 阶段三：Generator（生成）

**模块:** `pipeline/generator.rs`

泛型结构 `Generator<TTS: TTSBackend>`，`spawn_generator_workers()` 生成 `N` 个工作线程，每个 worker 循环：

```
loop {
    recv(TokenQueue) → match:
        Data(task) → tts.generate(task.text, style, lan, speed, worker_id)
                      ├─ Some(samples) → audio_queue.push(task.index, samples)
                      └─ None          → audio_queue.mark_failed(task.index)
        End         → break
        Err         → break
}
```

**TTSBackend trait:**
```rust
pub trait TTSBackend: Clone + Send + Sync + 'static {
    fn generate(&self, text: &str, style: &str, lan: &str, speed: f32, instance_id: usize) -> Option<Vec<f32>>;
}
```

通过 trait 实现解耦流水线与具体的 TTS 引擎。`TTSKokoParallel` 实现了 `TTSBackend`。

### 3.6 流水线启动（CLI 示例）

```
1. 创建 TextChunkQueue, TokenQueue, SortedAudioQueue
2. 启动 Preprocessor 线程
3. 启动 N 个 Generator worker 线程
4. 将输入文本逐块发送到 TextChunkQueue
5. 发送 End 信号
6. 主线程消费 SortedAudioQueue 直到 is_complete()
7. Join 所有线程
```

---

## 4. TTS Engine

### 4.1 TTSKoko（单实例）

**模块:** `tts/koko.rs`

`TTSKoko` 封装单个 ONNX Session + 发音人嵌入。

```rust
TTSKoko {
    model_path: String,
    model: Arc<Mutex<OrtKoko>>,    // 单个 ONNX Session（互斥访问）
    styles: HashMap<String, Vec<[[f32; 256]; 1]>>,  // 发音人嵌入
    init_config: InitConfig,
    phonemizer: Phonemizer,
}
```

**核心方法 `process_internal`:**

```
1. split_text_into_chunks(text, max_tokens=500, lan)
     └─ 逐句 phonemize → tokenize → 累积直到超限 → 分块
     └─ 超长句按词再次切分
     └─ 支持两种分块策略：max_tokens 和 max_words

2. 对每个 chunk:
   a. tokenize_with_alignment(text, lan)  或 tokenize_full_no_alignment(text, lan)
        └─ 音素化 → tokenize() → Vec<i64>
        └─ alignment 模式额外返回 word_map: Vec<(word, start, end)>

   b. mix_styles(style_name, tokens_len)
        └─ 单发音人：直接从 styles HashMap 查找对应 index 的嵌入
        └─ 混合发音人("voice1.3+voice2.7")：按比例加权求和（0.1 倍数）

   c. padding: [BOS=0] + tokens + [EOS=0]

   d. model.infer(tokens_batch, styles, speed)
        └─ ONNX Session::run() → (audio_array, durations_opt)
        └─ Standard 模型: (audio, None)
        └─ Timestamped 模型: (audio, Some(durations))

   e. 若 durations 存在，计算 WordAlignment:
        每个词的 start/end_sec = 累积 durations / 40fps → 按 chunk_audio_sec 线性缩放
```

**执行模式:**
- `ExecutionMode::Batch` — 全部块处理完后合并音频 + 全局时间戳偏移
- `ExecutionMode::Stream(callback)` — 每块完成后立即回调，低延迟

### 4.2 TTSKokoParallel（多实例并行）

```rust
TTSKokoParallel {
    models: Vec<Arc<Mutex<OrtKoko>>>,  // 多个独立 ONNX Session
    styles: HashMap<...>,
    phonemizer: Phonemizer,
    ...
}
```

**设计动机:** `OrtKoko` 内部的 `ort::Session` 不是 `Sync`，需要 `Mutex` 保护。单实例 + 单 Mutex 在多 worker 场景下成为瓶颈。`TTSKokoParallel` 创建 `num_instances` 个独立的 ONNX Session，每个 worker 通过 `get_model_instance(worker_id)` 获取专用的模型副本，消除锁竞争。

```rust
pub fn get_model_instance(&self, worker_id: usize) -> Arc<Mutex<OrtKoko>> {
    let index = worker_id % self.models.len();
    Arc::clone(&self.models[index])
}
```

**实例创建:** 通过 `new_with_instances(model_path, voices_path, num_instances)` 异步加载，每个实例读取同一个模型文件创建独立的 `ort::Session`。`styles` HashMap 在所有实例间共享。

### 4.3 ONNX 推理层

**模块:** `onn/`

#### 4.3.1 OrtKoko

`OrtKoko` 封装 `ort::Session`，在 `set_sess()` 时自动检测模型类型：

```rust
pub enum ModelStrategy {
    Standard(Session),       // input: tokens, style, speed → output: audio
    Timestamped(Session),    // input: input_ids, style, speed → output: waveform, durations
}
```

检测逻辑：若输入中包含 `input_ids` 或输出数量 > 1，则为 Timestamped 模型。

**推理方法 `infer`:**

```rust
fn infer(tokens, styles, speed, request_id, instance_id, chunk_number)
  → Result<(ArrayBase<f32>, Option<Vec<f32>>)>
```

- 输入：token IDs (i64)、style embedding (f32, 256-dim)、speed (f32)
- 输出：audio waveform (f32, 变长) + 可选 durations (f32, 每个 token 的帧数)
- 输入通过 `SessionInputs` 传入，输出通过 tensor name 提取
- 支持 `audio` / `waveforms` 两个可能的 output name 做 fallback

#### 4.3.2 OrtBase trait

```rust
pub trait OrtBase {
    fn load_model(&mut self, model_path: String) -> Result<(), String>;
    fn set_sess(&mut self, sess: Session);
    fn sess(&self) -> Option<&Session>;
}
```

`load_model` 实现：
1. 读取模型文件到内存
2. 创建 SessionBuilder
3. 根据 feature `cuda` 选择 CUDA 或 CPU 执行提供程序
4. 从内存提交模型 (`commit_from_memory`)

模型从内存加载而非文件路径，支持 WASM 和嵌入式场景。

### 4.4 发音人混合

`mix_styles` 支持两种模式：

**单发音人:** `"zm_yunyang"` → 直接查找 styles 表，取对应 token_len 的嵌入。
**混合发音人:** `"zf_xiaobei.3+zm_yunyang.7"` → 按比例加权平均，各部分权重 = portion × 0.1。

发音人数据存储在 `.npz` 文件中，每个发音人是 `(511, 1, 256)` 的 3D 张量，通过 `ndarray-npy` 读取。

---

## 5. Chinese G2P Pipeline

**模块:** `tts/chinese/`

中文 G2P 将汉字序列转换为音素（Bopomofo 注音或 IPA）。

### 5.1 处理流程

```
输入文本
  │
  ├─ 数字转换 (convert_numbers)
  │     └─ chinese-number crate: "123" → "一百二十三"
  │
  ├─ 标点映射 (map_punctuation)
  │     └─ 全角 → 上半角: "，"→", " "。"→". " "！"→"! "
  │
  ├─ 根据 output 模式分支：
  │
  ├─ Bopomofo 模式 (use_bopomofo=true):
  │   ├─ ZH_SEGMENT_PATTERN 分割中文/非中文段
  │   ├─ jieba-rs 分词 (segment_with_pos) → 含 POS tag
  │   ├─ tone_sandhi::pre_merge_for_modify → 合并需要变调的词组
  │   ├─ word_to_pinyin_with_disambiguation → 多音字消歧
  │   │     └─ PolyphonicDisambiguator: 基于规则的上下文消歧
  │   ├─ tone_sandhi::apply_tone_sandhi → 三声变调等
  │   └─ transcription::pinyin_to_bopomofo → 拼音→注音
  │
  └─ IPA 模式 (use_bopomofo=false):
      ├─ jieba-rs 分词
      ├─ word_to_pinyin → 拼音（带声调数字）
      └─ transcription::pinyin_to_ipa → 拼音→IPA
```

### 5.2 多音字消歧

`PolyphonicDisambiguator`（`chinese/polyphonic.rs`）维护一个多音字 → 发音的映射表，基于上下文规则消歧。支持 WASM 环境的 fallback。

### 5.3 变调处理

`tone_sandhi`（`chinese/tone_sandhi.rs`）实现：
- **三声变调:** 两个三声相连，前一个变为二声
- **"一"、"不"变调:** 根据后字声调变化
- **轻声识别:** 部分词尾自动轻声

### 5.4 音素映射

`transcription`（`chinese/transcription.rs`）维护：
- 拼音 → Bopomofo 映射表 (`ZH_MAP`)
- 拼音 → IPA 映射表

输出示例:
- Bopomofo: `"你好" → "ㄋㄧˇㄏㄠˇ"` (带声调)
- IPA: `"你好" → "ni2 xɑʊ2"`

### 5.5 中文分词的平台适配

```rust
#[cfg(not(target_arch = "wasm32"))]
static ref JIEBA: Mutex<Jieba> = ...;  // jieba-rs

#[cfg(target_arch = "wasm32")]
fn segment(text) -> Vec<String> { simple_segment(text) }  // 基于规则的简单分词
```

WASM 编译时跳过 jieba-rs（无法在 WASM 环境加载字典文件），使用字符级简单分割。

---

## 6. misaki G2P Engine (English)

**Crate:** `misaki` (v0.3.0)

misaki 是一个 POS-aware 的英语 G2P 引擎，完全独立于 kokoros-core，可作为独立库使用。

### 6.1 架构

```
G2P {
    lexicon: Lexicon,           // 发音词典 + 形态学规则
    tagger: PerceptronTagger,   // 感知机 POS tagger
    rules: Box<dyn LanguageRules>,  // 语言特定规则 (English)
    fallback: Option<Box<dyn Fallback>>,  // 字符级 fallback
    subtoken_regex: Regex,      // 子 token 分割
}
```

### 6.2 处理流程

```
1. preprocess(text) → 清理文本，提取 token
2. tokenize(text) → 基于正则的子 token 分割
3. tagger.tag(words) → 感知机 POS 标注
4. 对每个 token 依次查询:
   a. Lexicon::get_word(word, tag, stress, context) → 发音词典
   b. 若含连字符: 递归 g2p 各部件
   c. 数字转换: num2words → 递归 g2p
   d. LanguageRules::apply_rules → 词干规则 (-s, -ed, -ing)
   e. Fallback::phonemize → 字符级发音预测
   f. 单字符: 规范化 + 递归 g2p
5. 拼接所有 token 的音素输出
```

### 6.3 语言规则（English）

`English` 实现了 `LanguageRules`:

```rust
impl LanguageRules for English {
    fn apply_rules(&self, word, tag, lexicon) -> Option<String> {
        lexicon.stem_s(word, tag)     // 处理 -s 后缀
            .or_else(|| lexicon.stem_ed(word, tag))  // 处理 -ed 后缀
            .or_else(|| lexicon.stem_ing(word, tag))  // 处理 -ing 后缀
    }
}
```

### 6.4 Lexicon

基于 CMU Pronouncing Dictionary 数据，包含：
- 基础发音映射
- 大写词的重音处理
- 上下文依赖（future vowel, "to" 后的 a/an 变体）
- 形态学规则（词干提取 + 后缀发音规则）

---

## 7. CLI Tool (koko-cli)

### 7.1 命令行接口

```
Usage: koko [OPTIONS]

Options:
  -i, --input <FILE>     输入文件
  -t, --text <TEXT>      直接输入文本（与 -i 二选一）
  -o, --output <FILE>    输出 WAV 路径 [default: output.wav]
  -l, --lan <LANG>       语言代码 [default: zh]
  -m, --model <PATH>     ONNX 模型路径 [default: models/kokoro-v1.0.onnx]
  -d, --data <PATH>      发音人数据路径 [default: data/voices-v1.0.bin]
  -s, --style <STYLE>    发音人 [default: zm_yunyang]
  -p, --speed <FLOAT>    语速 [default: 0.7]
  -n, --threads <INT>    推理线程数 [default: 2]
      --max-chars <INT>  每块最大字符数 [default: 150]
      --max-phonemes <INT> 每块最大音素数 [default: 510]
  -P, --play             播放音频
      --buffer-chunks <INT> 缓冲块数 [default: 2]
```

### 7.2 实时播放架构

`--play` 模式下支持两种播放路径：

- **Windows:** `rodio` crate（通过 `OutputStream` + `Sink`）
- **Linux/macOS:** 管道输出到外部进程 `aplay` / `ffplay`

播放线程使用 `--buffer-chunks` 控制初始缓冲量，实现低延迟启动 + 平滑播放。对每个音频块应用淡入/淡出（720 samples ≈ 30ms at 24kHz）消除拼接爆音。

### 7.3 流式文件读取

`StreamingFileReader` 按行读取输入文件，每行作为一个 `TextChunk` 送入流水线。适用于大文件流式处理。

---

## 8. HTTP API Servers

### 8.1 kokoros-server (原生 API)

基于 Axum web 框架，提供 REST + SSE 接口。

**路由:**
```
GET  /              → Web 演示页面 (static/index.html)
GET  /health        → 健康检查
GET  /voices        → 发音人列表
POST /tts           → 文本转语音 (WAV Base64)
POST /tts/stream    → 流式 TTS (SSE)
```

**`POST /tts` 流程:**

```
1. 解析请求 JSON { text, voice, speed }
2. 启动 Preprocessor + Generator pipeline
3. TextSplitter 预分片 → 发送到 TextChunkQueue
4. 消费 SortedAudioQueue 直到所有块完成
5. float_samples_to_wav() → Base64 编码 → 响应 JSON
```

**`POST /tts/stream` 流程 (SSE):**

```
1. 创建 mpsc::channel
2. 启动 pipeline（与 /tts 相同）
3. 在独立线程中消费 SortedAudioQueue
4. 每块完成 → 转换为 PCM i16 → Base64 → SSE Event
5. 发送 Complete event → 关闭流
```

SSE 事件类型:
```json
{"chunk_index": 0, "audio_base64": "..."}  // 音频块
{"total_duration": 2.35}                    // 完成信号
```

### 8.2 kokoros-openai (OpenAI 兼容 API)

提供与 OpenAI TTS API 兼容的 HTTP 接口，方便集成到现有 OpenAI 生态工具中。

**路由:**
```
GET  /                  → 健康检查
POST /v1/audio/speech   → TTS 生成
GET  /v1/audio/voices   → 发音人列表
GET  /v1/models         → 模型列表
GET  /v1/models/{id}    → 单个模型信息
```

**OpenAI 兼容细节:**

- 发音人映射: `"alloy" → "af_alloy"`, `"echo" → "am_echo"` 等
- `input` / `voice` / `speed` / `response_format` / `stream` 全部支持
- 音频格式: WAV / MP3 / PCM / Opus
- 模型列表: `tts-1`, `tts-1-hd`, `kokoro`, `gpt-4o-mini-tts`
- 流式实现使用 windowed parallel processing

**流式并行处理:**

```
chunks → split_text_into_speech_chunks()
  │
  └─ 按 punctuation + break words 分片
  │
  └─ window = num_instances → 并行提交 TTS 任务
  │
  └─ BTreeMap 按 chunk_id 排序输出
  │
  └─ 流式响应 PCM 数据
```

与 `kokoros-server` 的 SSE 不同，OpenAI 兼容 API 使用 `Body::from_stream` 直接输出 PCM 字节流。

---

## 9. Tokenizer & Vocabulary

**模块:** `tts/tokenize.rs`, `tts/vocab.rs`

### 9.1 词汇表

两个静态词汇表:
- `VOCAB` — 主词汇表（IPA 音素符号）
- `ZH_VOCAB` — 中文词汇表（Bopomofo 符号 + 扩展）

每个音素字符映射到唯一的 token ID。

### 9.2 Tokenize

```rust
pub fn tokenize(phonemes: &str) -> Vec<i64>
```

逐字符查询 `ZH_VOCAB` 和 `VOCAB`，返回 token ID 数组。未知字符被过滤。

### 9.3 Round-trip

```rust
pub fn tokens_to_phonemes(tokens: &[i64]) -> String
```

通过 `REVERSE_VOCAB` 反向映射，用于调试验证。

---

## 10. Audio Processing

### 10.1 音频规格

| 参数 | 值 |
|------|-----|
| Sample Rate | 24000 Hz |
| Channels | 1 (mono) |
| Bit Depth | 32-bit float (内部) / 16-bit PCM (输出) |
| Format | WAV / PCM / MP3 / Opus |

### 10.2 后处理

**`kokoros-server/src/audio.rs`:**

- `normalize_audio`: 如果最大振幅 > 1.0，线性缩放到 0.95
- `apply_fade`: 淡入/淡出处理，消除块拼接爆音
- `float_samples_to_wav`: f32 samples → WAV (i16) 编码
- `float_samples_to_pcm`: f32 samples → raw PCM i16 bytes
- `merge_audio_chunks`: 时间顺序拼接音频块

### 10.3 特征门控

- WAV 编码使用 `hound` crate（基础依赖，始终可用）
- MP3 和 Opus 编码由 `audio-encode` feature 控制（musl 构建时禁用）

---

## 11. Build System & CI/CD

### 11.1 构建目标

| 目标 | 运行时 | 链接方式 | CI 环境 |
|------|--------|----------|---------|
| x86_64-unknown-linux-musl | Linux | 全静态 (ELF) | ubuntu-latest + musl-cross |
| aarch64-unknown-linux-musl | ARM64 Linux | 全静态 (ELF) | ubuntu-latest + aarch64-musl-cross |
| x86_64-pc-windows-msvc | Windows | 动态 ORT (DLL) | windows-latest |

### 11.2 ONNX Runtime 构建策略

**Linux musl:**
- `scripts/build_ort_musl.sh`: 下载 ONNX Runtime 源码 → cmake 编译 → 创建组合 `libonnxruntime.a`
  - 解决 abseil 循环依赖：x86_64 使用组合档案（所有 .o 合并为单个 .a），aarch64 使用 `--start-group`/`--end-group`
- `scripts/build_musl.sh`: 使用本地预编译 ORT 库进行 Rust 构建
- 工具链：`x86_64-linux-musl-cross` / `aarch64-linux-musl-cross`

**Windows:**
- 下载预编译 `onnxruntime-win-x64-*.zip`
- 动态链接 `onnxruntime.dll`
- 使用 `ORT_PREFER_DYNAMIC_LINK=1` 环境变量

### 11.3 Feature 管理

```toml
# musl 静态构建
cargo build --release -p koko --target x86_64-unknown-linux-musl --no-default-features

# 原生开发
cargo build --release -p koko
```

`--no-default-features` 跳过 `download`、`audio-encode` 等 musl 不兼容的 feature（`reqwest` TLS、`mp3lame-encoder` 等）。

### 11.4 GitHub Actions CI

`.github/workflows/build.yml`:

| Job | 触发条件 | 产物 |
|-----|----------|------|
| build-x86_64-linux | push, tag | `kokoro-x86_64-unknown-linux-musl.tar.gz` |
| build-aarch64-linux | push, tag | `kokoro-aarch64-unknown-linux-musl.tar.gz` |
| build-windows | push, tag | `kokoro-x86_64-pc-windows-msvc.zip` |
| release | tag | GitHub Release |

每个 job 构建 `koko` 和 `kokoros-server` 两个二进制。

---

## 12. Configuration

### 12.1 服务器配置 (config.toml)

```toml
host = "0.0.0.0"           # 监听地址
port = 3000                # 监听端口
threads = 4                # 推理线程数
max_chars = 400            # 单次最大字符数
model_path = "models/kokoro-v1.0.onnx"
voices_path = "data/voices-v1.0.bin"
```

通过 `KOKOROS_CONFIG` 环境变量指定自定义路径。

### 12.2 CLI 参数优先级

CLI 参数 > 配置文件 (仅 server) > 代码默认值

---

## 13. Data Flow: End-to-End Example

用户输入 `"你好世界，欢迎使用 Kokoro。"`

```
TextSplitter
  ├─ split_into_sentences: ["你好世界，欢迎使用 Kokoro。"]
  └─ split_by_phoneme_limit (max_phonemes=510):
       → ["你好世界，欢迎使用 Kokoro。"]  (单句未超限)

Preprocessor
  ├─ clean_text: "你好世界，欢迎使用 Kokoro。"
  ├─ phonemize("zh"): "ㄋㄧˇㄏㄠˇㄕˋㄐㄧㄝˋ，ㄏㄨㄢㄧㄥˊㄕˇㄩㄥˋKokoro。"
  ├─ phoneme count: 32 (未超限)
  ├─ tokenize: [13, 27, 83, 15, ..., 0]  (Vec<i64>)
  └─ ChunkTask { index: 0, text: "你好世界，欢迎使用 Kokoro。" }

Generator (Worker 0)
  ├─ TTSKokoParallel::tts_raw_audio_with_instance
  │   ├─ phonemize + tokenize (同上)
  │   ├─ mix_styles("zm_yunyang", 32) → [256-dim embedding]
  │   ├─ pad: [0] + tokens + [0]
  │   ├─ ONNX infer → audio (N samples @ 24kHz)
  │   └─ audio_queue.push(0, samples)
  └─ SortedAudioQueue.pop_next(0) → samples

Output
  ├─ write_wav("output.wav", samples, 24000)
  └─ Audio duration ≈ text_len × avg_syllable_duration
```

---

## 14. WASM Support

`kokoros-core` 支持编译为 WASM 目标：

- `oxionnx` 替代 `ort` 作为 ONNX 推理后端
- `jieba-rs` 替换为规则分词（`simple_segment`）
- `web-sys` 提供浏览器 AudioContext 支持
- `serde-wasm-bindgen` 提供 JS 序列化

**Feature:** `wasm = ["chinese", "oxionnx-backend", "wasm-bindgen", ...]`

当前 WASM 支持为实验性功能，核心引擎已验证可编译。

---

## 15. Security Considerations

1. **模型文件完整性:** 支持 SHA256 验证（通过 `download` feature）
2. **输入验证:** 控制字符过滤，文本长度限制 (`max_chars`)
3. **内存安全:** Rust 保证无内存安全问题；ONNX Runtime 使用 ort safe bindings
4. **拒绝服务防护:** 有界通道防止流水线积压；音素限制防止过长的推理
5. **CORS:** 默认允许所有 origin（可在生产环境收紧）

---

## 16. Performance Characteristics

| 操作 | 耗时参考 | 说明 |
|------|----------|------|
| ONNX 模型加载 | ~2-5s | 80MB ONNX 文件加载 + Session 创建 |
| 发音人数据加载 | ~1-2s | 150MB .npz 文件解析 |
| 首 Token 延迟 | ~100-500ms | 取决于文本长度和 chunk 策略 |
| 推理速度 | 5-10x RTF | 24kHz 音频比实时快 5-10 倍 |
| 内存占用 (推理) | ~300-500MB | 单个 ONNX Session |
| 多实例扩展 | 近线性 | N 个 Session = N 倍吞吐（无锁争用） |

**性能优化要点:**
- `TTSKokoParallel` 消除 Mutex 竞争，使得多 worker 可线性扩展
- TextSplitter 的 waterfall 分片策略在打断自然度和满足限制间取得平衡
- SortedAudioQueue 使用 BTreeMap 实现 O(log n) 的无序块重排序

---

## 17. Code Map

```
kokoroi-rs/
├── .github/workflows/build.yml       # CI/CD 流水线
├── Cargo.toml                        # 工作区根配置
├── config.toml                       # 服务器配置
├── scripts/
│   ├── build_musl.sh                 # x86_64 musl 构建脚本
│   ├── build_aarch64_musl.sh         # ARM64 musl 构建脚本
│   ├── build_ort_musl.sh             # ORT 源码编译 + 组合 lib
│   └── x86_64-musl-linker.sh         # musl 链接器包装
│
├── crates/kokoros-core/src/
│   ├── lib.rs                        # crate 根，feature-gated 导出
│   ├── pipeline/
│   │   ├── mod.rs                    # 公共导出
│   │   ├── types.rs                  # TextChunk, ChunkTask, PipelineMessage
│   │   ├── queue.rs                  # TextChunkQueue, TokenQueue, SortedAudioQueue
│   │   ├── text_splitter.rs          # 句子分割 + 音素限制分片
│   │   ├── preprocessor.rs           # 文本→token 预处理
│   │   ├── generator.rs              # 并行推理 worker + TTSBackend trait
│   │   └── tts_backend.rs            # TTSBackend for TTSKokoParallel
│   ├── tts/
│   │   ├── mod.rs                    # 特性导出
│   │   ├── koko.rs                   # TTSKoko, TTSKokoParallel 核心
│   │   ├── phonemizer.rs             # 跨语言音素化路由
│   │   ├── tokenize.rs               # 音素→token ID
│   │   ├── vocab.rs                  # 双词汇表 (IPA + Bopomofo)
│   │   ├── normalize.rs              # 文本规范化
│   │   └── chinese/
│   │       ├── mod.rs                # ChineseG2P 主引擎
│   │       ├── polyphonic.rs         # 多音字消歧
│   │       ├── tone_sandhi.rs        # 变调处理
│   │       └── transcription.rs      # 拼音→注音/IPA 映射
│   ├── onn/
│   │   ├── mod.rs                    # 多后端导出
│   │   ├── ort_base.rs               # OrtBase trait + 默认 impl
│   │   └── ort_koko.rs               # OrtKoko + ModelStrategy
│   └── utils/                        # 工具函数
│
├── crates/koko-cli/src/
│   ├── main.rs                       # CLI 入口 + 流水线编排
│   └── streaming_reader.rs           # 流式文件读取
│
├── crates/kokoros-server/src/
│   ├── main.rs                       # Axum 服务器 + 路由
│   ├── config.rs                     # TOML 配置加载
│   └── audio.rs                      # 音频后处理 + 编码
│
├── crates/kokoros-openai/src/
│   └── lib.rs                        # OpenAI 兼容 API
│
└── crates/misaki/src/
    ├── lib.rs                        # 公共导出
    ├── g2p.rs                        # G2P 主引擎
    ├── lexicon.rs                    # 发音词典 + 形态学
    ├── tagger.rs                     # 感知机 POS tagger
    ├── token.rs                      # MToken 类型
    ├── data.rs                       # 词典数据加载
    ├── language.rs                   # Language enum
    ├── languages/
    │   ├── mod.rs                    # LanguageRules trait
    │   └── english.rs                # 英语规则 (stem_s/ed/ing)
    └── fallback.rs                   # 字符级 fallback 发音器
```

---

## 18. Key Design Decisions

| 决策 | 选择 | 理由 |
|------|------|------|
| 通道库 | crossbeam | 比 std::sync::mpsc 更灵活，支持 bounded + clone |
| 音频排序 | BTreeMap + Condvar | 比 channels 更自然的有序收集模式 |
| ONNX 绑定 | ort (pyke/ort) | 最活跃的 Rust ONNX Runtime 绑定 |
| 模型加载 | 从内存 | 支持 WASM 和嵌入式场景 |
| 多实例并行 | Vec<Arc<Mutex<Session>>> | 避免单 Mutex 争用 |
| G2P 输出 | Bopomofo 优先 | Kokoro 模型 native 训练的注音输入 |
| HTTP 框架 | Axum | 异步 + 类型安全 + SSE 原生支持 |
| 中文分词 | jieba-rs | 工业级中文分词，WASM 有规则 fallback |
| 构建 | 多平台分离 | musl 静态 vs MSVC 动态，各取最优 |

---

## 19. Future Considerations

- **多语言 G2P 扩展:** 当前 `Phonemizer` 对非中文语言直接返回原文，可集成更多语言的 G2P
- **GPU 推理:** CUDA feature 已预留，生产部署可启用
- **流式 SSE 改进:** 当前 `kokoros-server` SSE 使用 Base64 编码，可考虑二进制 frame
- **发音人缓存:** 当前 `styles` HashMap 在 `TTSKokoParallel` 中被每个实例克隆，可封装为 `Arc`
- **模型热加载:** 支持运行时切换 TTS 模型
- **健康检查增强:** 深度健康检查（模型响应验证）
