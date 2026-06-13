# Dev commands for the AlphaZero self-play pipeline.
# Run inside `nix-shell` (provides python deps + ORT_DYLIB_PATH for the ort crate).

radius := "3"

# Type-check everything.
check:
    cargo check --workspace

# Run the rules-engine + MCTS + encoding unit tests (no ONNX Runtime needed).
test:
    cargo test -p giereczka-core

# Build the self-play and eval binaries (release; needed by the pipeline).
build-pipeline:
    cargo build --release -p giereczka-selfplay -p giereczka-eval

# Verify the Rust<->Python encoding index contract.
contract-test:
    cargo run -q -p giereczka-selfplay --bin contract -- {{radius}} > data/contract_r{{radius}}.json
    cd pipeline && python3 test_contract.py ../data/contract_r{{radius}}.json

# Generate a baseline (heuristic) self-play shard.
selfplay-heuristic games="50":
    cargo run --release -p giereczka-selfplay -- --radius {{radius}} --games {{games}} --out data/smoke

# Neural self-play with a model (MCTS visit-count targets).
selfplay model games="100" sims="128":
    ./target/release/selfplay --radius {{radius}} --games {{games}} --out data/run/manual \
        --model {{model}} --sims {{sims}} --threads `nproc`

# Evaluate model A vs B (B may be an .onnx path, "heuristic", or "alphabeta").
eval a b games="40" sims="64":
    ./target/release/eval --radius {{radius}} --a {{a}} --b {{b}} --games {{games}} \
        --sims {{sims}} --threads `nproc`

# Run the full generation loop.
loop gens="5" games="128" sims="128":
    python3 pipeline/run.py --radius {{radius}} --gens {{gens}} --games {{games}} \
        --sims {{sims}} --threads `nproc`
