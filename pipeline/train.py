"""Train HexNet on self-play shards (one optimization run over a replay buffer).

Loss = cross-entropy(policy_target, masked log-softmax(logits))
     + MSE(value_target, value)
     + L2 (via optimizer weight decay).

Run:
    python pipeline/train.py --data data/gen0 --radius 3 --epochs 4 \
        --init models/gen0.pt --out models/gen1.pt
"""

import argparse

import numpy as np
import torch
import torch.nn.functional as F

from data import load_shards
from model import HexNet, load_checkpoint, save_checkpoint


def masked_policy_loss(logits, target, mask):
    # Set illegal logits to -inf so they get zero softmax probability.
    neg_inf = torch.finfo(logits.dtype).min
    masked = torch.where(mask > 0, logits, torch.full_like(logits, neg_inf))
    logp = F.log_softmax(masked, dim=1)
    # Cross-entropy against the (already legal-only) soft target distribution.
    return -(target * logp).sum(dim=1).mean()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", required=True)
    ap.add_argument("--radius", type=int, required=True)
    ap.add_argument("--init", default=None, help="checkpoint to warm-start from")
    ap.add_argument("--out", required=True)
    ap.add_argument("--epochs", type=int, default=4)
    ap.add_argument("--batch", type=int, default=256)
    ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--weight-decay", type=float, default=1e-4)
    ap.add_argument("--value-weight", type=float, default=1.0,
                    help="weight on the value MSE term: loss = policy_ce + value_weight*value_mse. "
                         "Lower it during behaviour-cloning (heuristic-vs-heuristic outcomes are near "
                         "noise); keep ~1.0 for self-play where the value signal is informative.")
    ap.add_argument("--channels", type=int, default=64)
    ap.add_argument("--blocks", type=int, default=5)
    ap.add_argument("--window", type=int, default=None)
    ap.add_argument("--device", default="cpu")
    args = ap.parse_args()

    data = load_shards(args.data, window=args.window)
    planes = torch.from_numpy(data["planes"]).float()
    policy = torch.from_numpy(data["policy"]).float()
    value = torch.from_numpy(data["value"]).float()
    mask = torch.from_numpy(data["legal_mask"].astype(np.float32))
    n = planes.shape[0]
    print(f"loaded {n} samples, policy_len={policy.shape[1]}")

    device = torch.device(args.device)
    if args.init:
        model = load_checkpoint(args.init, map_location=device)
    else:
        model = HexNet(args.radius, channels=args.channels, blocks=args.blocks)
    model = model.to(device)

    opt = torch.optim.Adam(model.parameters(), lr=args.lr, weight_decay=args.weight_decay)

    model.train()
    total_batches = (n + args.batch - 1) // args.batch
    for epoch in range(args.epochs):
        perm = torch.randperm(n)
        total_p, total_v, batches = 0.0, 0.0, 0
        for start in range(0, n, args.batch):
            if batches % max(1, total_batches // 20) == 0:
                pct = 100 * batches // max(1, total_batches)
                print(
                    f"\r  epoch {epoch + 1}/{args.epochs}  batch {batches}/{total_batches} ({pct}%)",
                    end="",
                    flush=True,
                )
            idx = perm[start : start + args.batch]
            bp = planes[idx].to(device)
            btarget = policy[idx].to(device)
            bvalue = value[idx].to(device)
            bmask = mask[idx].to(device)

            logits, pred_value = model(bp)
            loss_p = masked_policy_loss(logits, btarget, bmask)
            loss_v = F.mse_loss(pred_value, bvalue)
            loss = loss_p + args.value_weight * loss_v

            opt.zero_grad()
            loss.backward()
            opt.step()

            total_p += loss_p.item()
            total_v += loss_v.item()
            batches += 1
        print(
            f"\r  epoch {epoch + 1}/{args.epochs} "
            f"policy_loss={total_p / batches:.4f} value_loss={total_v / batches:.4f}"
            "                    "
        )

    save_checkpoint(model, args.out)
    print(f"saved {args.out}")


if __name__ == "__main__":
    main()
