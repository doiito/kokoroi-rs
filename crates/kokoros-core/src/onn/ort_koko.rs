use std::borrow::Cow;

use model_schema::v1_0_timestamped::DURATIONS;
use ndarray::{ArrayBase, IxDyn, OwnedRepr};
use ort::{
    session::{Session, SessionInputValue, SessionInputs},
    value::{Tensor, Value},
};
use super::ort_base::OrtBase;

mod model_schema {
    pub const STYLE: &str = "style";
    pub const SPEED: &str = "speed";

    pub mod v1_0 {
        pub const TOKENS: &str = "tokens";
        pub const AUDIO: &str = "audio";
    }

    pub mod v1_0_timestamped {
        pub const TOKENS: &str = "input_ids";
        pub const AUDIO: &str = "waveform";
        pub const DURATIONS: &str = "durations";
    }
}

pub enum ModelStrategy {
    Standard(Session),
    Timestamped(Session),
}

impl ModelStrategy {
    fn audio_key(&self) -> &'static str {
        match self {
            ModelStrategy::Standard(_) => model_schema::v1_0::AUDIO,
            ModelStrategy::Timestamped(_) => model_schema::v1_0_timestamped::AUDIO,
        }
    }

    fn tokens_key(&self) -> &'static str {
        match self {
            ModelStrategy::Standard(_) => model_schema::v1_0::TOKENS,
            ModelStrategy::Timestamped(_) => model_schema::v1_0_timestamped::TOKENS,
        }
    }
}

pub struct OrtKoko {
    inner: Option<ModelStrategy>,
}

impl OrtBase for OrtKoko {
    fn set_sess(&mut self, sess: Session) {
        let input_names: Vec<String> = sess.inputs().iter().map(|i| i.name().to_string()).collect();
        let output_count = sess.outputs().len();

        let has_input_ids = input_names.iter().any(|name| name == "input_ids");

        let strategy = if has_input_ids || output_count > 1 {
            ModelStrategy::Timestamped(sess)
        } else {
            ModelStrategy::Standard(sess)
        };

        self.inner = Some(strategy);
    }

    fn sess(&self) -> Option<&Session> {
        self.inner.as_ref().map(|strategy| match strategy {
            ModelStrategy::Standard(sess) => sess,
            ModelStrategy::Timestamped(sess) => sess,
        })
    }
}

impl OrtKoko {
    pub fn new(model_path: String) -> Result<Self, String> {
        let mut instance = OrtKoko { inner: None };
        instance.load_model(model_path)?;
        Ok(instance)
    }

    pub fn strategy(&self) -> Option<&ModelStrategy> {
        self.inner.as_ref()
    }

    fn prepare_inputs(
        tokens_key: &'static str,
        tokens: Vec<Vec<i64>>,
        styles: Vec<Vec<f32>>,
        speed: f32,
    ) -> Result<Vec<(Cow<'static, str>, SessionInputValue<'static>)>, Box<dyn std::error::Error>>
    {
        let shape = [tokens.len(), tokens[0].len()];
        let flat_tokens: Vec<i64> = tokens.into_iter().flatten().collect();

        let shape_style = [styles.len(), styles[0].len()];
        let style_data: Vec<f32> = styles.into_iter().flatten().collect();
        let style_tensor = Tensor::from_array((shape_style, style_data))?;

        let speed_tensor = Tensor::from_array(([1], vec![speed]))?;

        let tokens_tensor = Tensor::from_array((shape, flat_tokens))?;

        Ok(vec![
            (Cow::Borrowed(tokens_key), SessionInputValue::Owned(Value::from(tokens_tensor))),
            (Cow::Borrowed(model_schema::STYLE), SessionInputValue::Owned(Value::from(style_tensor))),
            (Cow::Borrowed(model_schema::SPEED), SessionInputValue::Owned(Value::from(speed_tensor))),
        ])
    }

    pub fn infer(
        &mut self,
        tokens: Vec<Vec<i64>>,
        styles: Vec<Vec<f32>>,
        speed: f32,
        _request_id: Option<&str>,
        _instance_id: Option<&str>,
        _chunk_number: Option<usize>,
    ) -> Result<(ArrayBase<OwnedRepr<f32>, IxDyn>, Option<Vec<f32>>), Box<dyn std::error::Error>>
    {
        let strategy = self.inner.as_mut().ok_or("Session is not initialized.")?;
        let audio_key = strategy.audio_key();
        let tokens_key = strategy.tokens_key();
        let inputs = Self::prepare_inputs(tokens_key, tokens, styles, speed)?;

        match strategy {
            ModelStrategy::Standard(sess) => {
                let outputs = sess.run(SessionInputs::from(inputs))?;

                let (shape, data) = outputs[audio_key]
                    .try_extract_tensor::<f32>()
                    .or_else(|_| outputs["waveforms"].try_extract_tensor::<f32>())
                    .map_err(|_| "Standard Model: Could not find 'audio' output")?;

                let shape_vec: Vec<usize> = shape.iter().map(|&i| i as usize).collect();
                let audio_array = ArrayBase::from_shape_vec(shape_vec, data.to_vec())?;

                Ok((audio_array, None))
            }
            ModelStrategy::Timestamped(sess) => {
                let outputs = sess.run(SessionInputs::from(inputs))?;

                let (shape, data) = outputs[audio_key]
                    .try_extract_tensor::<f32>()
                    .or_else(|_| outputs["audio"].try_extract_tensor::<f32>())
                    .map_err(|_| "Timestamped Model: Could not find 'waveforms' or 'audio'")?;

                let shape_vec: Vec<usize> = shape.iter().map(|&i| i as usize).collect();
                let audio_array = ArrayBase::from_shape_vec(shape_vec, data.to_vec())?;

                let durations_vec = outputs
                    .get(DURATIONS)
                    .and_then(|v| v.try_extract_tensor::<f32>().ok())
                    .map(|(_, d)| d.to_vec());

                Ok((audio_array, durations_vec))
            }
        }
    }
}
