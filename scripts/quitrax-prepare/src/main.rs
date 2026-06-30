//! quitrax-prepare — off-chain credential / Merkle / witness helper.
//!
//! Hashing is byte-for-byte identical to the circom circuit and the Soroban
//! contract because it calls the SAME primitive — native Poseidon255 via
//! `soroban-poseidon` (the `poseidon_hash` / conversion helpers below are
//! copied from privacy-pools `coinutils`) — over the same `lean-imt` tree.
//!
//! Mapping to `claim.circom`:
//!   commitment    = Poseidon255(nullifier, secret)
//!   nullifierHash = Poseidon255(nullifier, roundId)
//!   stateRoot     = LeanIMT root over the cohort commitments
//!
//! Usage:
//!   quitrax-prepare gen-cohort --size N --out cohort.json
//!   quitrax-prepare make-input --cohort cohort.json --index I --round R \
//!                              --recipient DEC --out input.json
//!
//! Public-signal order emitted (matches snarkjs [outputs, public inputs]):
//!   [ nullifierHash, stateRoot, roundId, recipient ]

use lean_imt::{bls_scalar_to_bytes, bytes_to_bls_scalar, LeanIMT};
use num_bigint::BigUint;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use soroban_poseidon::poseidon_hash as poseidon_hash_native;
use soroban_sdk::{crypto::bls12_381::Fr as BlsScalar, BytesN, Env, Vec as SVec};

const TREE_DEPTH: u32 = 20; // matches Claim(20) in claim.circom

// ---- primitives (copied from coinutils so hashing stays identical) ----

/// Poseidon255 over field elements via the native SDK host function.
/// State size t = inputs.len() + 1 (rate = inputs.len()), matching the circuit.
fn poseidon_hash(env: &Env, inputs: &[BlsScalar]) -> BlsScalar {
    let mut u256_inputs = SVec::new(env);
    for input in inputs.iter() {
        u256_inputs.push_back(BlsScalar::to_u256(input));
    }
    let result_u256 = match inputs.len() {
        1 => poseidon_hash_native::<2, BlsScalar>(env, &u256_inputs),
        2 => poseidon_hash_native::<3, BlsScalar>(env, &u256_inputs),
        3 => poseidon_hash_native::<4, BlsScalar>(env, &u256_inputs),
        _ => panic!("poseidon_hash supports 1-3 inputs"),
    };
    BlsScalar::from_u256(result_u256)
}

fn bls_scalar_to_decimal_string(scalar: &BlsScalar) -> String {
    BigUint::from_bytes_be(&scalar.to_bytes().to_array()).to_str_radix(10)
}

fn decimal_string_to_bls_scalar(env: &Env, decimal: &str) -> BlsScalar {
    let value = BigUint::parse_bytes(decimal.as_bytes(), 10).expect("invalid decimal");
    let mut be = value.to_bytes_be();
    assert!(be.len() <= 32, "value exceeds 32 bytes / field size");
    let mut buf = [0u8; 32];
    buf[32 - be.len()..].copy_from_slice(&be);
    be.clear();
    BlsScalar::from_bytes(BytesN::from_array(env, &buf))
}

/// Strong in-field random scalar: 31 random bytes (< 2^248 < the BLS12-381
/// scalar field), so the value is always valid without rejection sampling.
fn rand_fr(env: &Env) -> BlsScalar {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes[1..]);
    BlsScalar::from_bytes(BytesN::from_array(env, &bytes))
}

/// Build a fixed-depth LeanIMT over the cohort commitments (same as on-chain).
fn build_tree(env: &Env, commitments: &[BlsScalar]) -> LeanIMT {
    let mut tree = LeanIMT::new(env, TREE_DEPTH);
    for c in commitments {
        tree.insert(bls_scalar_to_bytes(c.clone())).expect("insert leaf");
    }
    tree
}

// ---- file shapes ----

#[derive(Serialize, Deserialize)]
struct Beneficiary {
    index: usize,
    secret: String,
    nullifier: String,
    commitment: String,
}

#[derive(Serialize, Deserialize)]
struct Cohort {
    depth: u32,
    root: String,
    commitments: Vec<String>,
    beneficiaries: Vec<Beneficiary>,
}

#[derive(Serialize)]
struct CircomInput {
    #[serde(rename = "stateRoot")]
    state_root: String,
    #[serde(rename = "roundId")]
    round_id: String,
    recipient: String,
    secret: String,
    nullifier: String,
    #[serde(rename = "stateSiblings")]
    state_siblings: Vec<String>,
    #[serde(rename = "stateIndex")]
    state_index: String,
}

// ---- commands ----

fn gen_cohort(env: &Env, size: usize, out: &str) {
    let mut beneficiaries = Vec::with_capacity(size);
    let mut commitments_fr = Vec::with_capacity(size);
    let mut commitments_dec = Vec::with_capacity(size);

    for index in 0..size {
        let nullifier = rand_fr(env);
        let secret = rand_fr(env);
        let commitment = poseidon_hash(env, &[nullifier.clone(), secret.clone()]);

        commitments_dec.push(bls_scalar_to_decimal_string(&commitment));
        commitments_fr.push(commitment.clone());
        beneficiaries.push(Beneficiary {
            index,
            secret: bls_scalar_to_decimal_string(&secret),
            nullifier: bls_scalar_to_decimal_string(&nullifier),
            commitment: bls_scalar_to_decimal_string(&commitment),
        });
    }

    let tree = build_tree(env, &commitments_fr);
    let root = bytes_to_bls_scalar(&tree.get_root());

    let cohort = Cohort {
        depth: TREE_DEPTH,
        root: bls_scalar_to_decimal_string(&root),
        commitments: commitments_dec,
        beneficiaries,
    };
    std::fs::write(out, serde_json::to_string_pretty(&cohort).unwrap()).unwrap();
    println!("wrote {out}: {size} beneficiaries, root={}", cohort.root);
}

fn make_input(env: &Env, cohort_path: &str, index: usize, round: u64, recipient: &str, out: &str) {
    let cohort: Cohort =
        serde_json::from_str(&std::fs::read_to_string(cohort_path).unwrap()).unwrap();
    assert!(index < cohort.beneficiaries.len(), "index out of range");

    let commitments_fr: Vec<BlsScalar> = cohort
        .commitments
        .iter()
        .map(|c| decimal_string_to_bls_scalar(env, c))
        .collect();
    let tree = build_tree(env, &commitments_fr);

    let (siblings, _depth) = tree.generate_proof(index as u32).expect("merkle proof");
    let state_siblings: Vec<String> =
        siblings.iter().map(|s| bls_scalar_to_decimal_string(&s)).collect();
    assert_eq!(state_siblings.len(), TREE_DEPTH as usize, "sibling count != depth");

    let b = &cohort.beneficiaries[index];
    let nullifier = decimal_string_to_bls_scalar(env, &b.nullifier);
    let round_fr = decimal_string_to_bls_scalar(env, &round.to_string());
    let null_hash = poseidon_hash(env, &[nullifier, round_fr]);

    let input = CircomInput {
        state_root: cohort.root.clone(),
        round_id: round.to_string(),
        recipient: recipient.to_string(),
        secret: b.secret.clone(),
        nullifier: b.nullifier.clone(),
        state_siblings,
        state_index: index.to_string(),
    };
    std::fs::write(out, serde_json::to_string_pretty(&input).unwrap()).unwrap();

    println!("wrote {out} for index={index}, round={round}");
    println!("expected public signals:");
    println!("  nullifierHash = {}", bls_scalar_to_decimal_string(&null_hash));
    println!("  stateRoot     = {}", cohort.root);
    println!("  roundId       = {round}");
    println!("  recipient     = {recipient}");
}

// ---- tiny flag parser (avoids the clap -> windows-sys -> dlltool chain) ----

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let env = Env::default();
    // This is an off-chain helper: lift the on-chain CPU/mem budget so a
    // depth-20 tree (many Poseidon hashes per insert) doesn't trip ExceededLimit.
    env.cost_estimate().budget().reset_unlimited();
    match args.first().map(String::as_str) {
        Some("gen-cohort") => {
            let size = flag(&args, "--size").map(|s| s.parse().unwrap()).unwrap_or(8usize);
            let out = flag(&args, "--out").unwrap_or_else(|| "cohort.json".into());
            gen_cohort(&env, size, &out);
        }
        Some("make-input") => {
            let cohort = flag(&args, "--cohort").expect("--cohort required");
            let index = flag(&args, "--index").expect("--index required").parse().unwrap();
            let round = flag(&args, "--round").expect("--round required").parse().unwrap();
            let recipient = flag(&args, "--recipient").unwrap_or_else(|| "1".into());
            let out = flag(&args, "--out").unwrap_or_else(|| "input.json".into());
            make_input(&env, &cohort, index, round, &recipient, &out);
        }
        _ => {
            eprintln!("usage:");
            eprintln!("  quitrax-prepare gen-cohort --size N --out cohort.json");
            eprintln!("  quitrax-prepare make-input --cohort cohort.json --index I --round R --recipient DEC --out input.json");
            std::process::exit(2);
        }
    }
}
