#!/usr/bin/env bash
# Phase 0 de-risk: build, test, and deploy the official BLS12-381 groth16_verifier
# to Stellar testnet, proving the on-chain ZK verification pipeline end-to-end.
#
# Prereqs: stellar CLI v27, rustup target wasm32v1-none, funded `quitrax-admin` identity.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh
export PATH="$(pwd)/.localbin:$PATH"

VERIFIER_DIR="contracts/reference-soroban-examples/groth16_verifier"
NETWORK=testnet
SOURCE=quitrax-admin

echo "==> [1/4] Building groth16_verifier (wasm32v1-none)"
( cd "$VERIFIER_DIR" && stellar contract build )

echo "==> [2/4] Running verifier unit tests (verifies bundled sample proof off-chain + in env)"
( cd "$VERIFIER_DIR" && cargo test 2>&1 | tail -15 )

echo "==> [3/4] Optimizing WASM"
WASM="$VERIFIER_DIR/target/wasm32v1-none/release/soroban_groth16_verifier_contract.wasm"
stellar contract optimize --wasm "$WASM" --wasm-out "${WASM%.wasm}.optimized.wasm"

echo "==> [4/4] Deploying to $NETWORK"
VERIFIER_ID=$(stellar contract deploy \
  --wasm "${WASM%.wasm}.optimized.wasm" \
  --source "$SOURCE" --network "$NETWORK" 2>&1 | grep -oE 'C[A-Z0-9]{55}' | tail -1)

echo "VERIFIER_ID=$VERIFIER_ID"
mkdir -p .stellar-deploy
echo "$VERIFIER_ID" > .stellar-deploy/groth16_verifier.id

echo "==> [5/5] Encoding sample proof + verifying on-chain (end-to-end pipeline check)"
DATA="$VERIFIER_DIR/data"
ARGS="$DATA/soroban-args"
( cd scripts/groth16-encoder && cargo build --release )
scripts/groth16-encoder/target/release/groth16-encoder "$DATA" "$ARGS"

RES_OK=$(stellar contract invoke --id "$VERIFIER_ID" --source "$SOURCE" --network "$NETWORK" \
  -- verify_proof --vk "$(cat "$ARGS/vk.json")" --proof "$(cat "$ARGS/proof.json")" \
  --pub_signals "$(cat "$ARGS/pub_signals.json")" 2>/dev/null | tail -1)
RES_BAD=$(stellar contract invoke --id "$VERIFIER_ID" --source "$SOURCE" --network "$NETWORK" \
  -- verify_proof --vk "$(cat "$ARGS/vk.json")" --proof "$(cat "$ARGS/proof.json")" \
  --pub_signals '["22"]' 2>/dev/null | tail -1)

echo "   verify_proof(33) = $RES_OK   (expect true)"
echo "   verify_proof(22) = $RES_BAD  (expect false)"
[ "$RES_OK" = "true" ] && [ "$RES_BAD" = "false" ] || { echo "❌ on-chain verification mismatch"; exit 1; }
echo "✅ Phase 0 pipeline green — verifier deployed & proof verified on-chain: $VERIFIER_ID"
