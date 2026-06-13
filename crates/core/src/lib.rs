//! `giereczka-core` — the pure, Bevy-free rules engine for the hex-board Quoridor variant.
//!
//! This crate holds the deterministic game logic (`TurnState`), hex geometry, fence shapes,
//! and the heuristic/alpha-beta baseline AI. It has no rendering or ECS dependencies, so the
//! self-play and training binaries can compile and iterate without pulling in Bevy.

pub mod encoding;
pub mod fence_rules;
pub mod heuristic;
pub mod hex;
pub mod mcts;
#[cfg(feature = "ort")]
pub mod onnx;
pub mod player;
pub mod progress;
pub mod state;

#[cfg(test)]
mod rules_tests;
