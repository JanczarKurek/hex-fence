"""Gate: ONNX Runtime output must match PyTorch within tolerance.

Run before any exported model is used for self-play / in-game inference.

    python pipeline/parity.py --ckpt models/gen1.pt --onnx models/gen1.onnx
"""

import argparse

import numpy as np
import onnxruntime as ort
import torch

from model import load_checkpoint


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ckpt", required=True)
    ap.add_argument("--onnx", required=True)
    ap.add_argument("--tol", type=float, default=1e-4)
    ap.add_argument("--batch", type=int, default=4)
    args = ap.parse_args()

    model = load_checkpoint(args.ckpt)
    model.eval()
    dim = model.dim
    planes = model.config()["planes"]

    rng = np.random.default_rng(0)
    x = rng.standard_normal((args.batch, planes, dim, dim), dtype=np.float32)

    with torch.no_grad():
        torch_policy, torch_value = model(torch.from_numpy(x))
    torch_policy = torch_policy.numpy()
    torch_value = torch_value.numpy()

    sess = ort.InferenceSession(args.onnx, providers=["CPUExecutionProvider"])
    onnx_policy, onnx_value = sess.run(None, {"planes": x})

    policy_diff = np.abs(torch_policy - onnx_policy).max()
    value_diff = np.abs(torch_value - onnx_value.reshape(torch_value.shape)).max()
    print(f"policy max|diff|={policy_diff:.2e}  value max|diff|={value_diff:.2e}")

    assert policy_diff < args.tol, f"policy parity {policy_diff} >= {args.tol}"
    assert value_diff < args.tol, f"value parity {value_diff} >= {args.tol}"
    print("parity OK")


if __name__ == "__main__":
    main()
