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
    ap.add_argument("--blocks", type=int, default=5)
    ap.add_argument("--channels", type=int, default=64)
    ap.add_argument("--window", type=int, default=60000)
    ap.add_argument("--promote-threshold", type=float, default=0.55)
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

    # Generation 0: fresh network.
    if not ckpt(0).exists():
        run_py("init_model.py", "--radius", args.radius, "--out", ckpt(0),
               "--blocks", args.blocks, "--channels", args.channels)
    if not onnx(0).exists():
        run_py("export_onnx.py", "--ckpt", ckpt(0), "--out", onnx(0))

    print(
        f"pipeline: {args.gens} generations, {args.games} self-play games @ {args.sims} sims, "
        f"{args.eval_games} eval games, {args.threads} threads\n",
        flush=True,
    )

    best = 0
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
        run_bin(selfplay_bin, "--radius", args.radius, "--games", args.games,
                "--out", gen_dir, "--model", onnx(best), "--sims", args.sims,
                "--threads", args.threads, "--gen", g, env=env)

        # 2. Train a new candidate from the previous candidate, over the replay window.
        step(f"train ({args.epochs} epochs)")
        run_py("train.py", "--data", data_run, "--radius", args.radius,
               "--init", ckpt(g), "--out", ckpt(g + 1), "--epochs", args.epochs,
               "--blocks", args.blocks, "--channels", args.channels, "--window", args.window)

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

        promoted = score >= args.promote_threshold
        if promoted:
            best = g + 1

        gen_times.append(time.time() - gen_start)
        avg = sum(gen_times) / len(gen_times)
        remaining = avg * (args.gens - (g + 1))
        print(
            f"=== gen {g + 1}: score_vs_best={score:.3f} score_vs_heuristic={ref_score:.3f} "
            f"promoted={promoted} best=gen{best} | took {fmt_secs(gen_times[-1])}, "
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

    # Publish the best model as current.onnx (+ its radius sidecar).
    current = models / "current.onnx"
    current.write_bytes(onnx(best).read_bytes())
    sidecar = Path(str(onnx(best)) + ".json")
    if sidecar.exists():
        (Path(str(current) + ".json")).write_bytes(sidecar.read_bytes())

    (runs / "history.json").write_text(json.dumps(history, indent=2))
    print(f"done. best=gen{best} -> {current}")


if __name__ == "__main__":
    main()
