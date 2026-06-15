"""Canonical encoding contract, re-derived independently in Python.

This MUST match `giereczka_core::encoding` exactly. `test_contract.py` cross-checks this
against a golden JSON dumped by the Rust `contract` binary.

The pieces that must agree across languages:
  * `board_cells(radius)` — the deterministic cell ordering (move block + fence anchors).
  * `FENCE_SHAPES` order — the `shape` axis of the fence policy block.
  * `fence_index` — the policy-head index formula.
"""

# Fence shape order — load-bearing (matches FenceShape::ALL discriminant order in Rust).
FENCE_SHAPES = ["S", "SMirrored", "C", "Y"]
FENCE_SLOTS_PER_CELL = len(FENCE_SHAPES) * 6  # 24
PLANES = 14  # 12 board/edge/count planes + 2 BFS distance-to-goal fields (self, opponent)


def cell_count(radius: int) -> int:
    return 3 * radius * radius + 3 * radius + 1


def board_cells(radius: int):
    """On-board cells sorted by (r, q) — outer loop r, inner loop q."""
    cells = []
    for r in range(-radius, radius + 1):
        for q in range(-radius, radius + 1):
            s = -q - r
            if abs(q) <= radius and abs(r) <= radius and abs(s) <= radius:
                cells.append((q, r))
    return cells


def dim(radius: int) -> int:
    return 2 * radius + 1


def policy_len(radius: int) -> int:
    n = cell_count(radius)
    return n + n * FENCE_SLOTS_PER_CELL


def move_index(cell_idx: int) -> int:
    return cell_idx


def fence_index(radius: int, anchor_cell_idx: int, shape_idx: int, orientation: int) -> int:
    n = cell_count(radius)
    return n + anchor_cell_idx * FENCE_SLOTS_PER_CELL + shape_idx * 6 + (orientation % 6)
