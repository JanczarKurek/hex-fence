"""AlphaZero policy+value network for the hex Quoridor variant.

Small ResNet over the 12-plane canonical board. Two heads:
  * policy: raw logits over the full action space (masking happens at inference / loss time,
    NOT in the graph — keeps the ONNX export static, see export_onnx.py).
  * value: tanh in [-1, 1], the side-to-move's expected outcome.
"""

import torch
import torch.nn as nn
import torch.nn.functional as F

import contract as C


class ResidualBlock(nn.Module):
    def __init__(self, channels: int):
        super().__init__()
        self.conv1 = nn.Conv2d(channels, channels, 3, padding=1, bias=False)
        self.bn1 = nn.BatchNorm2d(channels)
        self.conv2 = nn.Conv2d(channels, channels, 3, padding=1, bias=False)
        self.bn2 = nn.BatchNorm2d(channels)

    def forward(self, x):
        y = F.relu(self.bn1(self.conv1(x)))
        y = self.bn2(self.conv2(y))
        return F.relu(x + y)


class HexNet(nn.Module):
    def __init__(self, radius: int, channels: int = 64, blocks: int = 5):
        super().__init__()
        self.radius = radius
        self.dim = C.dim(radius)
        self.policy_len = C.policy_len(radius)
        self.channels = channels
        self.blocks = blocks

        self.stem_conv = nn.Conv2d(C.PLANES, channels, 3, padding=1, bias=False)
        self.stem_bn = nn.BatchNorm2d(channels)
        self.tower = nn.Sequential(*[ResidualBlock(channels) for _ in range(blocks)])

        self.policy_conv = nn.Conv2d(channels, 2, 1, bias=False)
        self.policy_bn = nn.BatchNorm2d(2)
        self.policy_fc = nn.Linear(2 * self.dim * self.dim, self.policy_len)

        self.value_conv = nn.Conv2d(channels, 1, 1, bias=False)
        self.value_bn = nn.BatchNorm2d(1)
        self.value_fc1 = nn.Linear(self.dim * self.dim, 64)
        self.value_fc2 = nn.Linear(64, 1)

    def forward(self, planes):
        x = F.relu(self.stem_bn(self.stem_conv(planes)))
        x = self.tower(x)

        p = F.relu(self.policy_bn(self.policy_conv(x)))
        policy_logits = self.policy_fc(p.flatten(1))

        v = F.relu(self.value_bn(self.value_conv(x)))
        v = F.relu(self.value_fc1(v.flatten(1)))
        value = torch.tanh(self.value_fc2(v)).squeeze(-1)

        return policy_logits, value

    def config(self) -> dict:
        return {
            "radius": self.radius,
            "channels": self.channels,
            "blocks": self.blocks,
            "dim": self.dim,
            "policy_len": self.policy_len,
            "planes": C.PLANES,
        }


def save_checkpoint(model: HexNet, path: str) -> None:
    torch.save({"state_dict": model.state_dict(), "config": model.config()}, path)


def load_checkpoint(path: str, map_location="cpu") -> HexNet:
    ckpt = torch.load(path, map_location=map_location, weights_only=False)
    cfg = ckpt["config"]
    model = HexNet(cfg["radius"], channels=cfg["channels"], blocks=cfg["blocks"])
    model.load_state_dict(ckpt["state_dict"])
    return model
