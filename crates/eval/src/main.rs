//! Evaluation harness: play candidate model A against an opponent B and report A's win rate.
//!
//! Games alternate colors (A is player 0 in even games, player 1 in odd) to cancel any
//! first-move advantage. MCTS runs with temperature 0 and no Dirichlet noise (deterministic-ish
//! strong play). Used by the orchestration loop to gate promotion.
//!
//! Usage:
//!   eval --radius 3 --a models/gen1.onnx --b models/gen0.onnx --games 40 --sims 64 --threads 8
//!   eval --radius 3 --a models/gen1.onnx --b heuristic --games 40
//!
//! Prints a JSON summary on the last line.

use giereczka_core::encoding::Encoder;
use giereczka_core::heuristic::{AiRng, choose_alpha_beta_action, choose_heuristic_action};
use giereczka_core::mcts::{MctsConfig, run_mcts};
use giereczka_core::onnx::OnnxEvaluator;
use giereczka_core::progress::Progress;
use giereczka_core::state::{GameAction, TurnState};

const MAX_PLY: usize = 400;
const PLAYER_COUNT: usize = 2;

#[derive(Clone)]
enum Opponent {
    Model(String),
    Heuristic,
    AlphaBeta,
}

struct Args {
    radius: i32,
    a: String,
    b: Opponent,
    games: usize,
    sims: usize,
    threads: usize,
    seed: u64,
}

fn parse_args() -> Args {
    let mut radius = 3;
    let mut a = None;
    let mut b = None;
    let mut games = 40;
    let mut sims = 64;
    let mut threads = 1;
    let mut seed = 1;

    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().expect("missing value for flag");
        match flag.as_str() {
            "--radius" => radius = value().parse().expect("radius"),
            "--a" => a = Some(value()),
            "--b" => {
                b = Some(match value().as_str() {
                    "heuristic" => Opponent::Heuristic,
                    "alphabeta" => Opponent::AlphaBeta,
                    path => Opponent::Model(path.to_string()),
                })
            }
            "--games" => games = value().parse().expect("games"),
            "--sims" => sims = value().parse().expect("sims"),
            "--threads" => threads = value().parse::<usize>().expect("threads").max(1),
            "--seed" => seed = value().parse().expect("seed"),
            other => panic!("unknown flag: {other}"),
        }
    }

    Args {
        radius,
        a: a.expect("--a <onnx> required"),
        b: b.expect("--b <onnx|heuristic|alphabeta> required"),
        games,
        sims,
        threads,
        seed,
    }
}

enum Agent {
    Model(OnnxEvaluator),
    Heuristic,
    AlphaBeta,
}

impl Agent {
    fn choose(
        &self,
        state: &TurnState,
        encoder: &Encoder,
        sims: usize,
        rng: &mut AiRng,
    ) -> Option<GameAction> {
        match self {
            Agent::Heuristic => choose_heuristic_action(state, rng),
            Agent::AlphaBeta => choose_alpha_beta_action(state, rng, 3)
                .or_else(|| choose_heuristic_action(state, rng)),
            Agent::Model(eval) => {
                let config = MctsConfig {
                    simulations: sims,
                    temperature: 0.0,
                    dirichlet_epsilon: 0.0,
                    ..MctsConfig::default()
                };
                run_mcts(state, encoder, eval, config, rng).map(|result| result.action)
            }
        }
    }
}

/// Play one game. `agents[i]` controls player `i`. Returns the winning player index.
fn play_game(
    encoder: &Encoder,
    radius: i32,
    agents: [&Agent; PLAYER_COUNT],
    sims: usize,
    rng: &mut AiRng,
) -> Option<usize> {
    let mut state = TurnState::new(PLAYER_COUNT, radius);
    let mut ply = 0;
    while state.winner.is_none() && ply < MAX_PLY {
        let current = state.current_player;
        let Some(action) = agents[current].choose(&state, encoder, sims, rng) else {
            break;
        };
        if state.try_apply_action(action).is_err() {
            break;
        }
        ply += 1;
    }
    state.winner
}

fn build_agent(opponent: &Opponent, model_a: &str, which: char, dim: usize) -> Agent {
    match (which, opponent) {
        ('a', _) => Agent::Model(OnnxEvaluator::from_file(model_a, dim).expect("load model A")),
        ('b', Opponent::Model(path)) => {
            Agent::Model(OnnxEvaluator::from_file(path, dim).expect("load model B"))
        }
        ('b', Opponent::Heuristic) => Agent::Heuristic,
        ('b', Opponent::AlphaBeta) => Agent::AlphaBeta,
        _ => unreachable!(),
    }
}

fn main() {
    let args = parse_args();
    let encoder = Encoder::new(args.radius);
    let dim = encoder.dim();

    let progress = Progress::new("eval", args.games);
    let progress = &progress;
    // (a_wins, b_wins, draws) accumulated across worker threads.
    let totals: Vec<(usize, usize, usize)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..args.threads)
            .map(|t| {
                let count = args.games / args.threads + usize::from(t < args.games % args.threads);
                let encoder = &encoder;
                let args = &args;
                scope.spawn(move || {
                    let agent_a = build_agent(&args.b, &args.a, 'a', dim);
                    let agent_b = build_agent(&args.b, &args.a, 'b', dim);

                    let mut a_wins = 0;
                    let mut b_wins = 0;
                    let mut draws = 0;
                    for g in 0..count {
                        // Global game index, so color alternation is stable across threads.
                        let game_index = g * args.threads + t;
                        let a_is_player0 = game_index % 2 == 0;
                        let agents: [&Agent; PLAYER_COUNT] = if a_is_player0 {
                            [&agent_a, &agent_b]
                        } else {
                            [&agent_b, &agent_a]
                        };
                        let mut rng = AiRng::seeded(
                            args.seed
                                .wrapping_add(game_index as u64)
                                .wrapping_mul(0x9E3779B97F4A7C15)
                                | 1,
                        );
                        let winner = play_game(encoder, args.radius, agents, args.sims, &mut rng);
                        let a_player = if a_is_player0 { 0 } else { 1 };
                        match winner {
                            Some(w) if w == a_player => a_wins += 1,
                            Some(_) => b_wins += 1,
                            None => draws += 1,
                        }
                        progress.finish_one();
                    }
                    (a_wins, b_wins, draws)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let (a_wins, b_wins, draws) = totals
        .into_iter()
        .fold((0, 0, 0), |acc, x| (acc.0 + x.0, acc.1 + x.1, acc.2 + x.2));

    let games = a_wins + b_wins + draws;
    let score = (a_wins as f64 + 0.5 * draws as f64) / games as f64;

    let opponent = match &args.b {
        Opponent::Model(p) => p.clone(),
        Opponent::Heuristic => "heuristic".to_string(),
        Opponent::AlphaBeta => "alphabeta".to_string(),
    };
    let summary = serde_json::json!({
        "a": args.a,
        "b": opponent,
        "games": games,
        "a_wins": a_wins,
        "b_wins": b_wins,
        "draws": draws,
        "a_score": score,
    });
    println!("{}", serde_json::to_string(&summary).unwrap());
}
