"""Golden test: Python's re-derived encoding contract must match the Rust dump.

Run:
    cargo run -q -p giereczka-selfplay --bin contract -- 3 > data/contract_r3.json
    python3 pipeline/test_contract.py data/contract_r3.json

Pure-Python (no numpy / safetensors needed) so it can gate the pipeline cheaply.
"""

import json
import sys

import contract as C


def check(golden_path: str) -> None:
    with open(golden_path) as f:
        g = json.load(f)

    radius = g["radius"]

    assert g["n_cells"] == C.cell_count(radius), (g["n_cells"], C.cell_count(radius))
    assert g["dim"] == C.dim(radius), (g["dim"], C.dim(radius))
    assert g["planes"] == C.PLANES, (g["planes"], C.PLANES)
    assert g["policy_len"] == C.policy_len(radius), (g["policy_len"], C.policy_len(radius))
    assert g["fence_slots_per_cell"] == C.FENCE_SLOTS_PER_CELL
    assert g["fence_shapes"] == C.FENCE_SHAPES, (g["fence_shapes"], C.FENCE_SHAPES)

    # The cell ordering is the single most important shared convention.
    golden_cells = [tuple(c) for c in g["cells"]]
    derived_cells = C.board_cells(radius)
    assert golden_cells == derived_cells, "cell ordering mismatch between Rust and Python"

    # Re-derive the full fence index space and confirm it is a bijection into the fence block.
    n = C.cell_count(radius)
    seen = set()
    for ci in range(n):
        for sh in range(len(C.FENCE_SHAPES)):
            for orient in range(6):
                idx = C.fence_index(radius, ci, sh, orient)
                assert n <= idx < C.policy_len(radius), idx
                assert idx not in seen, f"duplicate fence index {idx}"
                seen.add(idx)
    assert len(seen) == n * 24

    print(
        f"contract OK: radius={radius} n_cells={n} "
        f"policy_len={g['policy_len']} cells+indices match"
    )


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: test_contract.py <golden.json>", file=sys.stderr)
        sys.exit(2)
    check(sys.argv[1])
