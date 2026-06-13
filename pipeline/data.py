"""Load self-play safetensors shards into a training replay buffer."""

import glob
import os

import numpy as np
from safetensors.numpy import load_file


def load_shards(data_dir: str, window: int | None = None):
    """Load and concatenate every `*.safetensors` shard under `data_dir`.

    If `window` is set, keep only the most recently modified shards summing to ~`window`
    samples (a sliding replay-buffer window across generations).
    """
    paths = sorted(
        glob.glob(os.path.join(data_dir, "**", "*.safetensors"), recursive=True),
        key=os.path.getmtime,
    )
    if not paths:
        raise FileNotFoundError(f"no shards under {data_dir}")

    planes, policy, value, mask = [], [], [], []
    total = 0
    for path in reversed(paths):  # newest first
        shard = load_file(path)
        planes.append(shard["planes"])
        policy.append(shard["policy"])
        value.append(shard["value"])
        mask.append(shard["legal_mask"])
        total += shard["value"].shape[0]
        if window is not None and total >= window:
            break

    return {
        "planes": np.concatenate(planes, axis=0),
        "policy": np.concatenate(policy, axis=0),
        "value": np.concatenate(value, axis=0),
        "legal_mask": np.concatenate(mask, axis=0),
    }
