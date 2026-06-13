"""Export a HexNet checkpoint to ONNX for Rust-side inference (the `ort` crate).

Exports RAW policy logits (no masking/softmax in the graph) plus the tanh value, with a
dynamic batch axis. A `<onnx>.json` sidecar records the radius so the game can refuse a model
that doesn't match the board.

Run: python pipeline/export_onnx.py --ckpt models/gen1.pt --out models/gen1.onnx
"""

import argparse
import json

import torch

from model import load_checkpoint


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ckpt", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--opset", type=int, default=17)
    args = ap.parse_args()

    model = load_checkpoint(args.ckpt)
    model.eval()

    dummy = torch.zeros(1, model.config()["planes"], model.dim, model.dim)
    torch.onnx.export(
        model,
        dummy,
        args.out,
        input_names=["planes"],
        output_names=["policy_logits", "value"],
        dynamic_axes={
            "planes": {0: "batch"},
            "policy_logits": {0: "batch"},
            "value": {0: "batch"},
        },
        opset_version=args.opset,
        # Use the legacy TorchScript exporter (the dynamo path needs onnxscript and is
        # overkill for this small static CNN).
        dynamo=False,
    )

    with open(args.out + ".json", "w") as f:
        json.dump(model.config(), f)
    print(f"exported {args.out} (radius={model.radius}, policy_len={model.policy_len})")


if __name__ == "__main__":
    main()
