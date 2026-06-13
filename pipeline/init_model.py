"""Create a freshly-initialized HexNet checkpoint (generation 0 bootstrap).

Run: python pipeline/init_model.py --radius 3 --out models/gen0.pt
"""

import argparse

import torch

from model import HexNet, save_checkpoint


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--radius", type=int, required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--channels", type=int, default=64)
    ap.add_argument("--blocks", type=int, default=5)
    ap.add_argument("--seed", type=int, default=0)
    args = ap.parse_args()

    torch.manual_seed(args.seed)
    model = HexNet(args.radius, channels=args.channels, blocks=args.blocks)
    # Put BatchNorm into a usable eval state (running stats default to mean 0 / var 1).
    model.eval()
    save_checkpoint(model, args.out)
    print(f"initialized {args.out} (radius={args.radius}, policy_len={model.policy_len})")


if __name__ == "__main__":
    main()
