use crate::pipeline::generator::TTSBackend;
use crate::tts::koko::TTSKokoParallel;

impl TTSBackend for TTSKokoParallel {
    fn generate(&self, text: &str, style: &str, lan: &str, speed: f32, instance_id: usize) -> Option<Vec<f32>> {
        let model_instance = self.get_model_instance(instance_id);
        self.tts_raw_audio_with_instance(
            text,
            lan,
            style,
            speed,
            None,
            None,
            None,
            None,
            model_instance,
        ).ok()
    }
}
