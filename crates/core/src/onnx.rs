//! ONNX Runtime-backed neural [`Evaluator`] (feature `ort`).
//!
//! Loads a model exported by `pipeline/export_onnx.py` and runs CPU inference. The model
//! outputs raw policy logits + a tanh value; masking/softmax happen in the MCTS layer.
//! Uses the `load-dynamic` strategy: ONNX Runtime is dlopened at runtime from `ORT_DYLIB_PATH`.

use std::path::Path;
use std::sync::Mutex;

use ort::session::Session;
use ort::value::Tensor;

use crate::encoding::PLANES;
use crate::mcts::Evaluator;

/// `Session::run` takes `&mut self`, but the [`Evaluator`] trait is `&self`. A `Mutex` bridges
/// this and keeps the evaluator `Send + Sync`, so it can be both a per-worker self-play
/// evaluator and a shared Bevy resource in the game. The lock is uncontended in the per-worker
/// case (one evaluator per thread).
pub struct OnnxEvaluator {
    session: Mutex<Session>,
    dim: usize,
}

impl OnnxEvaluator {
    pub fn from_file(path: impl AsRef<Path>, dim: usize) -> ort::Result<Self> {
        // The net is tiny; ONNX Runtime's default intra-op thread pool just oversubscribes.
        // Parallelism is at the game level (one evaluator per worker), so pin to 1 thread.
        let session = Session::builder()?
            .with_intra_threads(1)?
            .commit_from_file(path)?;
        Ok(Self {
            session: Mutex::new(session),
            dim,
        })
    }
}

impl Evaluator for OnnxEvaluator {
    fn evaluate(&self, planes: &[f32]) -> (Vec<f32>, f32) {
        let dim = self.dim;
        let input = Tensor::from_array(([1usize, PLANES, dim, dim], planes.to_vec()))
            .expect("build input tensor");

        let mut session = self.session.lock().expect("onnx session mutex");
        let outputs = session
            .run(ort::inputs!["planes" => input])
            .expect("run onnx session");

        let (_shape, policy) = outputs["policy_logits"]
            .try_extract_tensor::<f32>()
            .expect("extract policy");
        let (_shape, value) = outputs["value"]
            .try_extract_tensor::<f32>()
            .expect("extract value");

        (policy.to_vec(), value[0])
    }
}
