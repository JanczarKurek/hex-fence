"""Rust(ort) vs PyTorch parity on an identical deterministic input.

The Rust `ort_check` binary writes its evaluation as JSON; this script feeds the same input
through the PyTorch checkpoint and asserts agreement.

    python pipeline/ort_check.py <ckpt.pt> <rust_eval.json>
"""

import json
import sys

import numpy as np
import torch

import contract as C
from model import load_checkpoint


def main():
    ckpt_path, rust_json_path = sys.argv[1], sys.argv[2]
    model = load_checkpoint(ckpt_path)
    model.eval()
    dim = model.dim
    count = C.PLANES * dim * dim

    flat = np.array([(i % 13) / 13.0 for i in range(count)], dtype=np.float32)
    x = flat.reshape(1, C.PLANES, dim, dim)

    with torch.no_grad():
        policy, value = model(torch.from_numpy(x))
    policy = policy.numpy().reshape(-1)
    value = float(value.reshape(-1)[0])

    rust = json.load(open(rust_json_path))

    d_value = abs(value - rust["value"])
    d_sum = abs(float(policy.sum()) - rust["policy_sum"])
    d_head = max(abs(float(policy[i]) - rust["policy_head"][i]) for i in range(8))
    print(
        f"value torch={value:.5f} rust={rust['value']:.5f} d={d_value:.2e}; "
        f"policy_sum d={d_sum:.2e}; head d={d_head:.2e}; policy_len={rust['policy_len']}"
    )

    assert rust["policy_len"] == C.policy_len(model.radius), "policy_len mismatch"
    assert d_value < 1e-3, f"value parity {d_value}"
    assert d_head < 1e-3, f"policy head parity {d_head}"
    print("rust-ort vs torch parity OK")


if __name__ == "__main__":
    main()
