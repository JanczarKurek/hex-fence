"""AlphaZero generation loop: self-play -> train -> export -> parity -> eval -> promote.

Drives the Rust self-play/eval binaries (subprocess) and the Python training scripts. The Rust
binaries need ONNX Runtime: set `ORT_DYLIB_PATH` (shell.nix does this automatically).

Example:
    python pipeline/run.py --radius 3 --gens 3 --games 64 --sims 64 \
        --eval-games 40 --threads 8

Artifacts land in models/, data/run/, runs/ under --workdir (all gitignored).
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

PIPELINE = Path(__file__).resolve().parent


def fmt_secs(seconds: float) -> str:
    seconds = int(seconds)
    if seconds < 60:
        return f"{seconds}s"
    return f"{seconds // 60}m{seconds % 60:02d}s"


def step(label: str):
    print(f"  -> {label}", flush=True)


def run_py(script: str, *cargs):
    cmd = [sys.executable, str(PIPELINE / script), *map(str, cargs)]
    print("    +", " ".join(cmd), flush=True)
    subprocess.run(cmd, check=True, cwd=str(PIPELINE))


def run_bin(binary: str, *cargs, env=None):
    # stdout is captured (for the JSON result); stderr is inherited so the binary's live
    # progress bar streams straight to the terminal.
    cmd = [binary, *map(str, cargs)]
    print("    +", " ".join(cmd), flush=True)
    out = subprocess.run(cmd, check=True, stdout=subprocess.PIPE, text=True, env=env)
    if out.stdout:
        sys.stdout.write(out.stdout)
    lines = out.stdout.strip().splitlines() if out.stdout else []
    return lines[-1] if lines else ""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--radius", type=int, default=3)
    ap.add_argument("--gens", type=int, default=3)
    ap.add_argument("--games", type=int, default=64)
    ap.add_argument("--sims", type=int, default=64)
    ap.add_argument("--threads", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--eval-games", type=int, default=40)
    ap.add_argument("--eval-sims", type=int, default=48)
    ap.add_argument("--epochs", type=int, default=4)
    ap.add_argument("--bootstrap-games", type=int, default=0,
                    help="if >0, warm-start gen0 by behaviour-cloning the heuristic on this many "
                         "self-play games before the AlphaZero loop (closes the cold-start gap)")
    ap.add_argument("--bootstrap-epochs", type=int, default=20)
    ap.add_argument("--bootstrap-value-weight", type=float, default=0.3,
                    help="value-loss weight during behaviour-cloning. Heuristic-vs-heuristic outcomes "
                         "are near noise, so down-weight value and let the policy clone cleanly.")
    ap.add_argument("--value-weight", type=float, default=1.0,
                    help="value-loss weight during the per-generation self-play training")
    ap.add_argument("--lr", type=float, default=1e-3,
                    help="learning rate for per-generation training. Lower it (e.g. 5e-4) for gentle "
                         "refinement that doesn't overwrite the behaviour-cloned policy each gen.")
    ap.add_argument("--heuristic-games", type=int, default=0,
                    help="if >0, add this many fresh heuristic games to each generation's replay "
                         "buffer (anti-drift: keeps heuristic-quality racing in the training mix)")
    ap.add_argument("--blocks", type=int, default=5)
    ap.add_argument("--channels", type=int, default=64)
    ap.add_argument("--window", type=int, default=60000)
    ap.add_argument("--promote-threshold", type=float, default=0.55)
    ap.add_argument("--promote-ref-margin", type=float, default=0.0,
                    help="block promotion if the candidate's score vs the heuristic falls below "
                         "(best-so-far - this margin). Default 0.0 = promotion must not regress vs "
                         "the heuristic at all (the real fix for the intransitive drift seen earlier).")
    ap.add_argument("--value-shaping", action="store_true",
                    help="opt-in: speed-scale terminal value targets in neural self-play (fast wins "
                         "earn near full +/-1, slow grinds less). Experimental; default off.")
    ap.add_argument("--value-shaping-strength", type=float, default=0.2)
    ap.add_argument("--value-blend", type=float, default=1.0,
                    help="value target = blend*game_outcome + (1-blend)*distance_diff per position. "
                         "1.0 = pure outcome (classic). <1.0 mixes in the dense, learnable "
                         "distance-difference so the value head is useful for MCTS (the fix for "
                         "'more search makes the net worse').")
    ap.add_argument("--device", default="cpu",
                    help="torch device for training (e.g. 'cuda' for a ROCm/NVIDIA GPU). Self-play "
                         "and eval always run on the Rust/ONNX CPU path regardless.")
    ap.add_argument("--workdir", default=".")
    ap.add_argument("--selfplay-bin", default="target/release/selfplay")
    ap.add_argument("--eval-bin", default="target/release/eval")
    args = ap.parse_args()

    root = Path(args.workdir).resolve()
    models = root / "models"
    data_run = root / "data" / "run"
    runs = root / "runs"
    for d in (models, data_run, runs):
        d.mkdir(parents=True, exist_ok=True)

    selfplay_bin = str((root / args.selfplay_bin).resolve())
    eval_bin = str((root / args.eval_bin).resolve())
    env = dict(os.environ)
    if "ORT_DYLIB_PATH" not in env:
        print("WARNING: ORT_DYLIB_PATH not set; the Rust binaries will fail to load ONNX Runtime")

    def ckpt(g):
        return models / f"gen{g}.pt"

    def onnx(g):
        return models / f"gen{g}.onnx"

    # Generation 0: fresh network, optionally warm-started by behaviour-cloning the heuristic.
    # A cold-start net needs many self-play generations just to learn efficient racing, and
    # tends to settle into a weak self-play equilibrium it never escapes (measured: 0/40 vs the
    # heuristic across a 10-gen run). Cloning the heuristic first lands gen0 on a far stronger
    # manifold so the AlphaZero loop refines from there instead of rediscovering basics.
    if not ckpt(0).exists():
        run_py("init_model.py", "--radius", args.radius, "--out", ckpt(0),
               "--blocks", args.blocks, "--channels", args.channels)
        if args.bootstrap_games > 0:
            boot_dir = data_run / "bootstrap"
            step(f"bootstrap: heuristic self-play ({args.bootstrap_games} games)")
            run_bin(selfplay_bin, "--radius", args.radius, "--games", args.bootstrap_games,
                    "--out", boot_dir, "--policy", "heuristic", "--seed", 7,
                    "--threads", args.threads, "--value-blend", args.value_blend, env=env)
            step(f"bootstrap: behaviour-clone gen0 ({args.bootstrap_epochs} epochs)")
            run_py("train.py", "--data", boot_dir, "--radius", args.radius,
                   "--init", ckpt(0), "--out", ckpt(0), "--epochs", args.bootstrap_epochs,
                   "--blocks", args.blocks, "--channels", args.channels,
                   "--value-weight", args.bootstrap_value_weight, "--device", args.device,
                   "--window", max(args.window, args.bootstrap_games * 60))
    if not onnx(0).exists():
        run_py("export_onnx.py", "--ckpt", ckpt(0), "--out", onnx(0))

    print(
        f"pipeline: {args.gens} generations, {args.games} self-play games @ {args.sims} sims, "
        f"{args.eval_games} eval games, {args.threads} threads\n",
        flush=True,
    )

    best = 0
    # Heuristic score of the current best — the yardstick promotion is gated on. Seed it from
    # gen0 so the very first candidate is held to the warm-started baseline.
    step("baseline: eval gen0 vs heuristic")
    base = run_bin(eval_bin, "--radius", args.radius, "--a", onnx(0), "--b", "heuristic",
                   "--games", args.eval_games, "--sims", args.eval_sims,
                   "--threads", args.threads, env=env)
    best_ref = json.loads(base)["a_score"]
    print(f"    gen0 score_vs_heuristic={best_ref:.3f}", flush=True)

    # The published model is the strongest-vs-heuristic checkpoint across the whole run, tracked
    # independently of the head-to-head promotion ladder. That decoupling is the real fix for
    # intransitive drift: a gen can beat the prior best yet be worse vs the heuristic — we never
    # ship that one. Seeded from gen0 (its onnx already exists).
    best_ref_gen = 0
    best_ref_score = best_ref

    history = []
    gen_times = []
    start = time.time()
    for g in range(args.gens):
        gen_start = time.time()
        print(f"========== generation {g + 1}/{args.gens}  (current best = gen{best}) ==========",
              flush=True)
        gen_dir = data_run / f"gen{g}"

        # 1. Self-play with the current best model.
        step(f"self-play ({args.games} games)")
        shaping = ["--value-shaping", "--value-shaping-strength", args.value_shaping_strength] \
            if args.value_shaping else []
        run_bin(selfplay_bin, "--radius", args.radius, "--games", args.games,
                "--out", gen_dir, "--model", onnx(best), "--sims", args.sims,
                "--threads", args.threads, "--gen", g, "--value-blend", args.value_blend,
                *shaping, env=env)

        # 1b. Mix fresh heuristic games into this generation's buffer so self-play can't drift
        # away from heuristic-quality racing. They go in their own subdir (so the heuristic shard +
        # meta don't clobber the neural ones); load_shards globs recursively so they're still in the
        # training mix.
        if args.heuristic_games > 0:
            step(f"heuristic games ({args.heuristic_games})")
            run_bin(selfplay_bin, "--radius", args.radius, "--games", args.heuristic_games,
                    "--out", gen_dir / "heuristic", "--policy", "heuristic",
                    "--seed", 1000 + g, "--threads", args.threads,
                    "--value-blend", args.value_blend, env=env)

        # 2. Train a new candidate by refining the current CHAMPION (not a continuously-drifting
        # chain), over the replay window. Warm-starting from best means a generation that drifted
        # down vs the heuristic is discarded — the next one retries from the champion's weights.
        step(f"train ({args.epochs} epochs, from gen{best})")
        run_py("train.py", "--data", data_run, "--radius", args.radius,
               "--init", ckpt(best), "--out", ckpt(g + 1), "--epochs", args.epochs,
               "--blocks", args.blocks, "--channels", args.channels,
               "--value-weight", args.value_weight, "--lr", args.lr, "--device", args.device,
               "--window", args.window)

        # 3. Export + parity gate.
        step("export ONNX + parity gate")
        run_py("export_onnx.py", "--ckpt", ckpt(g + 1), "--out", onnx(g + 1))
        run_py("parity.py", "--ckpt", ckpt(g + 1), "--onnx", onnx(g + 1))

        # 4. Evaluate candidate vs current best (color-alternating).
        step(f"eval vs best ({args.eval_games} games)")
        summary = run_bin(eval_bin, "--radius", args.radius, "--a", onnx(g + 1),
                          "--b", onnx(best), "--games", args.eval_games,
                          "--sims", args.eval_sims, "--threads", args.threads, env=env)
        score = json.loads(summary)["a_score"]

        # Reference: candidate vs the hand-written heuristic.
        step(f"eval vs heuristic ({args.eval_games} games)")
        ref = run_bin(eval_bin, "--radius", args.radius, "--a", onnx(g + 1),
                     "--b", "heuristic", "--games", args.eval_games,
                     "--sims", args.eval_sims, "--threads", args.threads, env=env)
        ref_score = json.loads(ref)["a_score"]

        # Hill-climb directly on the objective: adopt the candidate as the new base (for both
        # self-play data generation AND the training warm-start) and champion iff it improves vs
        # the heuristic. A generation that drifts down vs the heuristic is discarded; the next one
        # retries from the same champion instead of compounding the drift into the self-play meta.
        beats_best = score >= args.promote_threshold  # head-to-head, kept for visibility
        promoted = ref_score > best_ref_score
        if promoted:
            best = g + 1
            best_ref_score = ref_score
            best_ref_gen = g + 1
        best_ref = best_ref_score

        gen_times.append(time.time() - gen_start)
        avg = sum(gen_times) / len(gen_times)
        remaining = avg * (args.gens - (g + 1))
        gate = "" if promoted else " [no vs-heuristic improvement]"
        print(
            f"=== gen {g + 1}: score_vs_best={score:.3f} score_vs_heuristic={ref_score:.3f} "
            f"promoted={promoted}{gate} best=gen{best} (ref={best_ref:.3f}) | "
            f"took {fmt_secs(gen_times[-1])}, "
            f"ETA {fmt_secs(remaining)} ({args.gens - (g + 1)} gens left) ===\n",
            flush=True,
        )
        history.append({
            "generation": g + 1,
            "score_vs_best": score,
            "score_vs_heuristic": ref_score,
            "promoted": promoted,
            "best": best,
        })

    print(f"total time: {fmt_secs(time.time() - start)}", flush=True)

    # Publish the strongest-vs-heuristic checkpoint as current.onnx (+ its radius sidecar). Its onnx
    # was exported before that generation's gate, so it always exists even if it wasn't promoted.
    current = models / "current.onnx"
    current.write_bytes(onnx(best_ref_gen).read_bytes())
    sidecar = Path(str(onnx(best_ref_gen)) + ".json")
    if sidecar.exists():
        (Path(str(current) + ".json")).write_bytes(sidecar.read_bytes())

    (runs / "history.json").write_text(json.dumps(history, indent=2))
    print(f"done. best(ladder)=gen{best} champion(vs-heuristic)=gen{best_ref_gen} "
          f"(score={best_ref_score:.3f}) -> {current}")


if __name__ == "__main__":
    main()
