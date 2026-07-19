interface KokoroConfig {
  usePolyphonic?: boolean;
}

interface SynthesisOptions {
  lang?: 'zh' | 'zh-CN' | 'zh-TW' | 'en';
  speed?: number;
}

interface VoiceInfo {
  id: string;
  name: string;
  language: string;
  gender: 'male' | 'female';
}

interface SynthesisResult {
  phonemes: string;
  text: string;
}

declare class KokoroWASM {
  constructor(config: KokoroConfig);
  
  phonemize(text: string, lang?: string): Promise<string>;
  
  synthesize(text: string, options?: SynthesisOptions): Promise<SynthesisResult>;
  
  getVoices(): Promise<VoiceInfo[]>;
}

export default KokoroWASM;
