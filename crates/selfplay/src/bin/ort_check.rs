//! Run the ONNX evaluator on a deterministic input and dump its output as JSON.
//!
//! Paired with `pipeline/ort_check.py`, which feeds the identical input through the PyTorch
//! model and asserts the outputs match — the Rust<->Python (ort vs torch) parity gate.
//!
//! Requires `ORT_DYLIB_PATH` to point at `libonnxruntime.so`.
//!
//! Usage: `ort_check <model.onnx> <dim>` (prints JSON to stdout).

use giereczka_core::encoding::PLANES;
use giereczka_core::mcts::Evaluator;
use giereczka_core::onnx::OnnxEvaluator;

fn main() {
    let mut args = std::env::args().skip(1);
    let model = args.next().expect("model path");
    let dim: usize = args.next().expect("dim").parse().expect("dim is a number");

    let count = PLANES * dim * dim;
    let input: Vec<f32> = (0..count).map(|i| (i % 13) as f32 / 13.0).collect();

    let evaluator = OnnxEvaluator::from_file(&model, dim).expect("load onnx model");
    let (policy, value) = evaluator.evaluate(&input);

    let policy_sum: f32 = policy.iter().sum();
    let head: Vec<f32> = policy.iter().take(8).copied().collect();
    let json = serde_json::json!({
        "value": value,
        "policy_sum": policy_sum,
        "policy_head": head,
        "policy_len": policy.len(),
    });
    println!("{}", serde_json::to_string(&json).unwrap());
}
