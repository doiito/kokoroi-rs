use std::sync::Arc;
use tract_onnx::prelude::*;

pub type OnnxModel = Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>;

pub struct OnnxInference {
    model: OnnxModel,
}

impl OnnxInference {
    pub fn from_bytes(model_bytes: &[u8]) -> Result<Self, String> {
        let model = tract_onnx::onnx()
            .model_for_read(&mut &model_bytes[..])
            .map_err(|e| format!("Failed to load ONNX model: {}", e))?
            .into_optimized()
            .map_err(|e| format!("Failed to optimize model: {}", e))?
            .into_runnable()
            .map_err(|e| format!("Failed to make model runnable: {}", e))?;

        Ok(Self { model: Arc::new(model) })
    }

    pub fn run(&self, input_ids: Vec<i64>, style: Vec<f32>, speed: f32) -> Result<Vec<f32>, String> {
        use ndarray::Array1;

        let input_ids_tensor: Tensor = tract_ndarray::Array2::from_shape_vec(
            (1, input_ids.len()),
            input_ids,
        ).map_err(|e| format!("Failed to create input_ids tensor: {}", e))?.into();

        let style_tensor: Tensor = tract_ndarray::Array2::from_shape_vec(
            (1, style.len()),
            style,
        ).map_err(|e| format!("Failed to create style tensor: {}", e))?.into();

        let speed_tensor: Tensor = Array1::from_vec(vec![speed]).into();

        let result = self.model.run(tvec!(
            input_ids_tensor.into(),
            style_tensor.into(),
            speed_tensor.into(),
        )).map_err(|e| format!("Inference failed: {}", e))?;

        let output = result[0].to_array_view::<f32>()
            .map_err(|e| format!("Failed to get output: {}", e))?;
        Ok(output.as_slice().unwrap().to_vec())
    }
}
