/* ──────────────────────────────────────────────
   Kokoro TTS WASM — TypeScript 类型声明
   代码生成器: wasm-bindgen + 手动补充
   对应 Rust: crates/kokoros-core/src/wasm/api.rs
   ────────────────────────────────────────────── */

/** `new KokoroWASM(config)` 的配置项 */
interface KokoroConfig {
  /** 是否启用多音字消歧（默认 true） */
  usePolyphonic?: boolean;
}

/** 发音人信息 */
interface VoiceInfo {
  id: string;
  name: string;
  language: string;
  gender: 'male' | 'female';
}

/** `synthesize()` 的返回值 */
interface SynthesisResult {
  /** Bopomofo 音素序列（带声调数字） */
  phonemes: string;
  /** 可读注音显示 */
  phonemesDisplay: string;
  /** 原始输入文本 */
  text: string;
  /** PCM float32 音频数据 */
  audio: Float32Array;
  /** 采样率 (24000) */
  sampleRate: number;
}

/**
 * Kokoro TTS 主引擎（WASM 版）
 *
 * 所有方法均为**同步调用**——出错时抛出异常（非 Promise）。
 * constructor + pcm_samples_to_wav_data 除外（由 wasm-bindgen 处理）。
 */
declare class KokoroWASM {
  /**
   * @param config.usePolyphonic - 多音字消歧开关（默认 true）
   */
  constructor(config: KokoroConfig);

  // ════ G2P 方法（始终可用） ════

  /** 中文文本 → Bopomofo 音素 */
  phonemize(text: string, lang?: string): string;

  /** 中文文本 → IPA 音素 */
  phonemizeIPA(text: string, lang?: string): string;

  /** 中文文本 → 可读注音 */
  phonemizeDisplay(text: string, lang?: string): string;

  /** Bopomofo 音素 → Token ID 数组 */
  tokenize(phonemes: string, lang?: string): Uint32Array;

  /** 获取内置发音人列表 */
  getVoices(): VoiceInfo[];

  /** 返回音频采样率（固定 24000） */
  getSampleRate(): number;

  // ════ 推理方法（oxionnx 后端，需要 loadModel） ════

  /**
   * 加载 ONNX 模型（从二进制 bytes）
   * 必须在 synthesize/synthesizeWithPhonemes 之前调用。
   */
  loadModel(modelBytes: Uint8Array): void;

  /** 模型是否已加载 */
  isModelLoaded(): boolean;

  /**
   * 文本 → 语音（全链路，G2P + 推理一步完成）
   * @param text   输入文本（中文）
   * @param style  发音人风格嵌入 (Float32Array)
   * @param speed  语速倍率 (1.0 = 正常)
   */
  synthesize(text: string, style: Float32Array, speed: number): SynthesisResult;

  /**
   * 音素 → 语音（跳过 G2P，直接推理）
   * @param phonemes  Bopomofo 音素序列
   * @param style    发音人风格嵌入
   * @param speed    语速倍率
   */
  synthesizeWithPhonemes(phonemes: string, style: Float32Array, speed: number): Float32Array;
}

/**
 * Float32Array PCM → WAV 格式 (24000Hz, 16-bit mono)
 * @param samples - PCM float32 音频数据
 * @returns WAV 文件的 Uint8Array
 */
export function pcm_samples_to_wav_data(samples: Float32Array): Uint8Array;

export default KokoroWASM;
