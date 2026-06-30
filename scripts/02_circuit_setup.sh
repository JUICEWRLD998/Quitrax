#!/usr/bin/env bash
# Phase 1: compile claim.circom and run the Groth16 trusted setup on BLS12-381.
# Produces build/claim.{r1cs,wasm,sym}, build/claim_final.zkey, build/verification_key.json.
#
# The circuit has ~13.8k constraints, so a 2^14 Powers-of-Tau is sufficient.
# This is a *demo* ceremony (single deterministic contribution) — the README
# documents that a real deployment needs a multi-party ceremony.
set -euo pipefail
cd "$(dirname "$0")/../circuits"
source ../scripts/env.sh

CIRCOM="$(cd .. && pwd)/.localbin/circom.exe"
SNARKJS="npx snarkjs"
POT=14
mkdir -p build
cd build

echo "==> [1/6] Compile circuit (bls12381)"
"$CIRCOM" ../claim.circom --r1cs --wasm --sym --prime bls12381 \
  -l ../lib -l ../node_modules/circomlib/circuits -o .

echo "==> [2/6] Powers of Tau (new, bls12381, 2^$POT)"
$SNARKJS powersoftau new bls12381 $POT pot_0.ptau -v

echo "==> [3/6] Contribute to Powers of Tau"
$SNARKJS powersoftau contribute pot_0.ptau pot_1.ptau \
  --name="quitrax-demo-1" -v -e="quitrax phase1 entropy $POT"

echo "==> [4/6] Prepare phase 2"
$SNARKJS powersoftau prepare phase2 pot_1.ptau pot_final.ptau -v

echo "==> [5/6] Groth16 setup + zkey contribution"
$SNARKJS groth16 setup claim.r1cs pot_final.ptau claim_0.zkey
$SNARKJS zkey contribute claim_0.zkey claim_final.zkey \
  --name="quitrax-demo-zkey-1" -v -e="quitrax zkey entropy"

echo "==> [6/6] Export verification key"
$SNARKJS zkey export verificationkey claim_final.zkey verification_key.json

echo "✅ Phase 1 setup complete — build/claim_final.zkey + verification_key.json"
