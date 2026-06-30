#!/usr/bin/env bash
# Phase 1 deliverable: generate a real claim proof end-to-end and verify it.
#
#   cohort -> witness input -> witness -> groth16 proof -> local verify
#
# Reuses the same Poseidon255 + LeanIMT as the circuit and the contract, so the
# circuit's recomputed root matches the helper's published root.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh

CIRCUITS="circuits"
BUILD="$CIRCUITS/build"
PREP="scripts/quitrax-prepare/target/release/quitrax-prepare"
SNARKJS="npx snarkjs"

SIZE="${SIZE:-8}"
INDEX="${INDEX:-3}"
ROUND="${ROUND:-1}"
RECIPIENT="${RECIPIENT:-1234567890}"

echo "==> [1/5] Generate cohort ($SIZE beneficiaries)"
"$PREP" gen-cohort --size "$SIZE" --out "$BUILD/cohort.json"

echo "==> [2/5] Build witness input (index=$INDEX round=$ROUND)"
"$PREP" make-input --cohort "$BUILD/cohort.json" --index "$INDEX" \
  --round "$ROUND" --recipient "$RECIPIENT" --out "$BUILD/input.json"

echo "==> [3/5] Calculate witness"
( cd "$BUILD" && node claim_js/generate_witness.js claim_js/claim.wasm input.json witness.wtns )

echo "==> [4/5] Generate Groth16 proof"
$SNARKJS groth16 prove "$BUILD/claim_final.zkey" "$BUILD/witness.wtns" \
  "$BUILD/proof.json" "$BUILD/public.json"

echo "==> [5/5] Verify proof locally"
$SNARKJS groth16 verify "$BUILD/verification_key.json" "$BUILD/public.json" "$BUILD/proof.json"

echo "--- public signals (nullifierHash, stateRoot, roundId, recipient) ---"
cat "$BUILD/public.json"
echo "✅ Phase 1 deliverable — valid claim proof generated & verified locally"
