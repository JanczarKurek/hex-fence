"""Load a self-play safetensors shard and validate the data invariants.

Run: python3 pipeline/check_shard.py data/smoke/shard_r3_seed7.safetensors
Requires numpy + safetensors.
"""

import sys

import numpy as np
from safetensors.numpy import load_file


def check(path: str) -> None:
    data = load_file(path)
    planes = data["planes"]
    policy = data["policy"]
    value = data["value"]
    mask = data["legal_mask"]

    for name, arr in data.items():
        print(name, arr.dtype, arr.shape)

    batch = planes.shape[0]
    assert policy.shape[0] == batch and value.shape[0] == batch and mask.shape[0] == batch
    assert policy.shape[1] == mask.shape[1]

    # Phase-1 policy targets are one-hot on the action taken.
    assert np.allclose(policy.sum(axis=1), 1.0), "policy rows must sum to 1"
    chosen = policy.argmax(axis=1)
    assert mask[np.arange(batch), chosen].all(), "chosen action must be inside the legal mask"

    # Every legal mask must contain at least one legal action.
    assert (mask.sum(axis=1) > 0).all()

    # Values are game outcomes from the side-to-move's perspective.
    assert set(np.unique(value)).issubset({-1.0, 0.0, 1.0}), set(np.unique(value))

    # Planes are normalized features.
    assert planes.min() >= 0.0 and planes.max() <= 1.0 + 1e-6, (planes.min(), planes.max())

    print(f"shard OK: {batch} samples, policy_len={policy.shape[1]}")


if __name__ == "__main__":
    check(sys.argv[1])
