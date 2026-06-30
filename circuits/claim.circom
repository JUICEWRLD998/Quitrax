pragma circom 2.2.0;

// Quitrax — anonymous aid claim circuit (BLS12-381 / Poseidon255)
//
// Built on the official Stellar `privacy-pools` ZK substrate so that the
// in-circuit hashing is byte-for-byte consistent with the on-chain LeanIMT +
// poseidon255 used by the Soroban contract.
//
// Proves in zero knowledge that the prover:
//   1. knows (nullifier, secret) whose commitment = Poseidon255(nullifier, secret)
//      is a leaf in the round's beneficiary Merkle tree (stateRoot), AND
//   2. correctly derived the PER-ROUND nullifierHash = Poseidon255(nullifier, roundId)
// WITHOUT revealing which beneficiary they are.
//
// `recipient` (payout address, as a field element) is bound into the proof so a
// leaked proof cannot be redirected to another address (anti front-running).
//
// Per-round nullifier semantics:
//   - One claim per beneficiary per round (contract rejects a repeated nullifierHash).
//   - roundId is mixed in, so a beneficiary's nullifierHash in round N is
//     unlinkable to round N+1 — they may claim every round, never twice.
//
// Compile:
//   circom claim.circom --r1cs --wasm --sym --prime bls12381 \
//     -l lib -l <circomlib/circuits> -o build
//
// snarkjs public signal order = [outputs, public inputs] in declaration order:
//   [ nullifierHash, stateRoot, roundId, recipient ]

include "poseidon255.circom";
include "merkleProof.circom";

template Claim(treeDepth) {
    // ---- public inputs ----
    signal input stateRoot;   // round's published beneficiary Merkle root
    signal input roundId;     // current aid round
    signal input recipient;   // payout address (field element), bound into proof

    // ---- public output ----
    signal output nullifierHash;   // per-round spent-marker

    // ---- private inputs ----
    signal input secret;                   // beneficiary secret
    signal input nullifier;                // beneficiary nullifier
    signal input stateSiblings[treeDepth]; // Merkle authentication path
    signal input stateIndex;               // leaf index in the tree

    // 1. commitment = Poseidon255(nullifier, secret)
    component commitmentHasher = Poseidon255(2);
    commitmentHasher.in[0] <== nullifier;
    commitmentHasher.in[1] <== secret;
    signal commitment <== commitmentHasher.out;

    // 2. per-round nullifierHash = Poseidon255(nullifier, roundId)
    component nullifierHasher = Poseidon255(2);
    nullifierHasher.in[0] <== nullifier;
    nullifierHasher.in[1] <== roundId;
    nullifierHash <== nullifierHasher.out;

    // 3. prove commitment ∈ round's Merkle tree (LeanIMT-consistent)
    component stateRootChecker = MerkleProof(treeDepth);
    stateRootChecker.leaf <== commitment;
    stateRootChecker.leafIndex <== stateIndex;
    stateRootChecker.siblings <== stateSiblings;
    stateRoot === stateRootChecker.out;

    // 4. bind recipient address into the constraint system.
    //    No-op square => `recipient` is a real public signal of THIS proof;
    //    changing the payout address invalidates the proof.
    signal recipientSquared <== recipient * recipient;
}

// Tree depth 20 => up to 2^20 (~1,048,576) beneficiaries per round.
component main { public [ stateRoot, roundId, recipient ] } = Claim(20);
