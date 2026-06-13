//! Headless self-play data generator.
//!
//! Two modes:
//!   * `--policy heuristic|random`: baseline playouts with a one-hot policy target (bootstrap).
//!   * `--model <onnx>`: real AlphaZero self-play — MCTS guided by the network, recording the
//!     visit-count distribution as the policy target. Parallelised across `--threads` workers.
//!
//! Every position is encoded with `giereczka_core::encoding` and the
//! `(planes, policy, value, legal_mask)` samples are written as a safetensors shard.
//!
//! Usage:
//!   selfplay --radius 3 --games 200 --out data/gen1 --model models/gen1.onnx --sims 128 --threads 8

use std::collections::HashSet;
use std::path::Path;

use giereczka_core::encoding::{Encoder, PLANES};
use giereczka_core::heuristic::{AiRng, choose_heuristic_action};
use giereczka_core::mcts::{MctsConfig, run_mcts};
use giereczka_core::onnx::OnnxEvaluator;
use giereczka_core::progress::Progress;
use giereczka_core::state::{GameAction, TurnState};

use safetensors::tensor::{Dtype, TensorView};

const MAX_PLY: usize = 400;
const PLAYER_COUNT: usize = 2;
const DIRICHLET_EPSILON: f32 = 0.25;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Policy {
    Heuristic,
    Random,
    Neural,
}

struct Args {
    radius: i32,
    games: usize,
    out: String,
    seed: u64,
    policy: Policy,
    model: Option<String>,
    sims: usize,
    temp_moves: usize,
    threads: usize,
    generation: i64,
}

fn parse_args() -> Args {
    let mut radius = 3;
    let mut games = 10;
    let mut out = String::from("data/smoke");
    let mut seed = 42u64;
    let mut policy = Policy::Heuristic;
    let mut model = None;
    let mut sims = 128;
    let mut temp_moves = 12;
    let mut threads = 1;
    let mut generation = -1;

    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().expect("missing value for flag");
        match flag.as_str() {
            "--radius" => radius = value().parse().expect("radius"),
            "--games" => games = value().parse().expect("games"),
            "--out" => out = value(),
            "--seed" => seed = value().parse().expect("seed"),
            "--policy" => {
                policy = match value().as_str() {
                    "heuristic" => Policy::Heuristic,
                    "random" => Policy::Random,
                    "neural" => Policy::Neural,
                    other => panic!("unknown policy: {other}"),
                }
            }
            "--model" => model = Some(value()),
            "--sims" => sims = value().parse().expect("sims"),
            "--temp-moves" => temp_moves = value().parse().expect("temp-moves"),
            "--threads" => threads = value().parse::<usize>().expect("threads").max(1),
            "--gen" => generation = value().parse().expect("gen"),
            other => panic!("unknown flag: {other}"),
        }
    }

    if model.is_some() {
        policy = Policy::Neural;
    }

    Args {
        radius,
        games,
        out,
        seed,
        policy,
        model,
        sims,
        temp_moves,
        threads,
        generation,
    }
}

/// One recorded training position (canonical frame).
struct Sample {
    planes: Vec<f32>,
    policy: Vec<f32>,
    legal_mask: Vec<u8>,
    player: usize,
    value: f32,
}

fn actions_equal(a: &GameAction, b: &GameAction) -> bool {
    match (a, b) {
        (GameAction::Move { target: t1 }, GameAction::Move { target: t2 }) => t1 == t2,
        (GameAction::PlaceFence { edges: e1 }, GameAction::PlaceFence { edges: e2 }) => {
            let s1: HashSet<_> = e1.iter().collect();
            let s2: HashSet<_> = e2.iter().collect();
            s1 == s2
        }
        _ => false,
    }
}

fn legal_mask(encoder: &Encoder, state: &TurnState, k: usize, policy_len: usize) -> Vec<u8> {
    let mut mask = vec![0u8; policy_len];
    for (idx, _) in encoder.enumerate_legal_fast(state, k) {
        mask[idx] = 1;
    }
    mask
}

/// Assign the value target to a game's samples from each recorded state's perspective.
fn backfill_values(samples: &mut [Sample], start: usize, winner: Option<usize>) {
    for sample in &mut samples[start..] {
        sample.value = match winner {
            Some(w) if w == sample.player => 1.0,
            Some(_) => -1.0,
            None => 0.0,
        };
    }
}

fn play_game_baseline(
    encoder: &Encoder,
    radius: i32,
    random: bool,
    rng: &mut AiRng,
    samples: &mut Vec<Sample>,
) -> Option<usize> {
    let mut state = TurnState::new(PLAYER_COUNT, radius);
    let policy_len = encoder.policy_len();
    let start = samples.len();

    let mut ply = 0;
    while state.winner.is_none() && ply < MAX_PLY {
        let (planes, k) = encoder.encode(&state);
        let legal = encoder.enumerate_legal_fast(&state, k);
        if legal.is_empty() {
            break;
        }

        let action = if random {
            None
        } else {
            choose_heuristic_action(&state, rng)
        }
        .filter(|a| legal.iter().any(|(_, la)| actions_equal(la, a)))
        .unwrap_or_else(|| legal[rng.choose_index(legal.len())].1);

        let chosen_idx = legal
            .iter()
            .find_map(|(idx, la)| actions_equal(la, &action).then_some(*idx))
            .expect("chosen action must be legal");

        let mut policy_vec = vec![0.0f32; policy_len];
        let mut mask = vec![0u8; policy_len];
        for (idx, _) in &legal {
            mask[*idx] = 1;
        }
        policy_vec[chosen_idx] = 1.0;

        samples.push(Sample {
            planes,
            policy: policy_vec,
            legal_mask: mask,
            player: state.current_player,
            value: 0.0,
        });

        state
            .try_apply_action(action)
            .expect("policy produced an illegal action");
        ply += 1;
    }

    backfill_values(samples, start, state.winner);
    state.winner
}

fn play_game_neural(
    encoder: &Encoder,
    radius: i32,
    eval: &OnnxEvaluator,
    sims: usize,
    temp_moves: usize,
    rng: &mut AiRng,
    samples: &mut Vec<Sample>,
) -> Option<usize> {
    let mut state = TurnState::new(PLAYER_COUNT, radius);
    let policy_len = encoder.policy_len();
    let start = samples.len();

    let mut ply = 0;
    while state.winner.is_none() && ply < MAX_PLY {
        let (planes, k) = encoder.encode(&state);
        let player = state.current_player;
        let config = MctsConfig {
            simulations: sims,
            temperature: if ply < temp_moves { 1.0 } else { 0.0 },
            dirichlet_epsilon: DIRICHLET_EPSILON,
            ..MctsConfig::default()
        };

        let Some(result) = run_mcts(&state, encoder, eval, config, rng) else {
            break;
        };

        samples.push(Sample {
            planes,
            policy: result.policy_target,
            legal_mask: legal_mask(encoder, &state, k, policy_len),
            player,
            value: 0.0,
        });

        state
            .try_apply_action(result.action)
            .expect("mcts produced an illegal action");
        ply += 1;
    }

    backfill_values(samples, start, state.winner);
    state.winner
}

fn f32_to_bytes(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// Generate self-play samples, parallelised across threads for neural mode.
fn generate(args: &Args, encoder: &Encoder) -> (Vec<Sample>, [usize; PLAYER_COUNT], usize) {
    let threads = if args.policy == Policy::Neural {
        args.threads
    } else {
        1
    };

    let progress = Progress::new("self-play", args.games);
    let progress = &progress;
    let results: Vec<(Vec<Sample>, [usize; PLAYER_COUNT], usize)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let count = args.games / threads + usize::from(t < args.games % threads);
                scope.spawn(move || {
                    let mut eval = None;
                    if args.policy == Policy::Neural {
                        let model = args
                            .model
                            .as_ref()
                            .expect("--model required for neural mode");
                        eval = Some(
                            OnnxEvaluator::from_file(model, encoder.dim())
                                .expect("load onnx model"),
                        );
                    }

                    let mut samples = Vec::new();
                    let mut wins = [0usize; PLAYER_COUNT];
                    let mut draws = 0usize;
                    for g in 0..count {
                        let seed = args
                            .seed
                            .wrapping_add((t as u64).wrapping_mul(0x9E3779B97F4A7C15))
                            .wrapping_add(g as u64)
                            .wrapping_mul(0xD1B54A32D192ED03);
                        let mut rng = AiRng::seeded(seed);
                        let winner = match args.policy {
                            Policy::Neural => play_game_neural(
                                encoder,
                                args.radius,
                                eval.as_ref().unwrap(),
                                args.sims,
                                args.temp_moves,
                                &mut rng,
                                &mut samples,
                            ),
                            Policy::Heuristic => play_game_baseline(
                                encoder,
                                args.radius,
                                false,
                                &mut rng,
                                &mut samples,
                            ),
                            Policy::Random => play_game_baseline(
                                encoder,
                                args.radius,
                                true,
                                &mut rng,
                                &mut samples,
                            ),
                        };
                        match winner {
                            Some(w) => wins[w] += 1,
                            None => draws += 1,
                        }
                        progress.finish_one();
                    }
                    (samples, wins, draws)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut samples = Vec::new();
    let mut wins = [0usize; PLAYER_COUNT];
    let mut draws = 0;
    for (mut local, local_wins, local_draws) in results {
        samples.append(&mut local);
        for i in 0..PLAYER_COUNT {
            wins[i] += local_wins[i];
        }
        draws += local_draws;
    }
    (samples, wins, draws)
}

fn main() {
    let args = parse_args();
    let encoder = Encoder::new(args.radius);
    let dim = encoder.dim();
    let policy_len = encoder.policy_len();

    let (samples, wins, draws) = generate(&args, &encoder);
    let batch = samples.len();
    assert!(batch > 0, "no samples were produced");

    let mut planes_flat = Vec::with_capacity(batch * PLANES * dim * dim);
    let mut policy_flat = Vec::with_capacity(batch * policy_len);
    let mut value_flat = Vec::with_capacity(batch);
    let mut mask_flat = Vec::with_capacity(batch * policy_len);
    for sample in &samples {
        planes_flat.extend_from_slice(&sample.planes);
        policy_flat.extend_from_slice(&sample.policy);
        value_flat.push(sample.value);
        mask_flat.extend_from_slice(&sample.legal_mask);
    }

    let planes_bytes = f32_to_bytes(&planes_flat);
    let policy_bytes = f32_to_bytes(&policy_flat);
    let value_bytes = f32_to_bytes(&value_flat);

    let tensors = [
        (
            "planes".to_string(),
            TensorView::new(Dtype::F32, vec![batch, PLANES, dim, dim], &planes_bytes).unwrap(),
        ),
        (
            "policy".to_string(),
            TensorView::new(Dtype::F32, vec![batch, policy_len], &policy_bytes).unwrap(),
        ),
        (
            "value".to_string(),
            TensorView::new(Dtype::F32, vec![batch], &value_bytes).unwrap(),
        ),
        (
            "legal_mask".to_string(),
            TensorView::new(Dtype::U8, vec![batch, policy_len], &mask_flat).unwrap(),
        ),
    ];

    std::fs::create_dir_all(&args.out).expect("create output dir");
    let shard_path = format!(
        "{}/shard_r{}_seed{}.safetensors",
        args.out, args.radius, args.seed
    );
    safetensors::serialize_to_file(tensors, &None, Path::new(&shard_path))
        .expect("write safetensors shard");

    let meta = serde_json::json!({
        "radius": args.radius,
        "player_count": PLAYER_COUNT,
        "planes": PLANES,
        "dim": dim,
        "policy_len": policy_len,
        "n_cells": encoder.n_cells(),
        "games": args.games,
        "samples": batch,
        "wins": wins,
        "draws": draws,
        "generation": args.generation,
        "shard": shard_path,
    });
    let meta_path = format!("{}/meta.json", args.out);
    std::fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).expect("write meta");

    println!(
        "games={} samples={} wins={:?} draws={} -> {}",
        args.games, batch, wins, draws, shard_path
    );
}
