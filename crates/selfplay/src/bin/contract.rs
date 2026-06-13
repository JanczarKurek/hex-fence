//! Dump the encoding contract (canonical cell ordering + index constants) as JSON.
//!
//! Consumed by `pipeline/test_contract.py`, which independently re-derives the same map
//! and asserts equality — locking the Rust<->Python action-index convention.
//!
//! Usage: `contract [radius]` (prints JSON to stdout).

use giereczka_core::encoding::{Encoder, PLANES};
use giereczka_core::fence_rules::FenceShape;
use giereczka_core::hex::board_cells;

fn main() {
    let radius: i32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    let encoder = Encoder::new(radius);
    let cells: Vec<[i32; 2]> = board_cells(radius).iter().map(|c| [c.q, c.r]).collect();
    let shape_names: Vec<&str> = FenceShape::ALL
        .iter()
        .map(|s| match s {
            FenceShape::S => "S",
            FenceShape::SMirrored => "SMirrored",
            FenceShape::C => "C",
            FenceShape::Y => "Y",
        })
        .collect();

    let contract = serde_json::json!({
        "radius": radius,
        "n_cells": encoder.n_cells(),
        "dim": encoder.dim(),
        "planes": PLANES,
        "policy_len": encoder.policy_len(),
        "fence_slots_per_cell": 24,
        "fence_shapes": shape_names,
        "cells": cells,
    });

    println!("{}", serde_json::to_string(&contract).unwrap());
}
