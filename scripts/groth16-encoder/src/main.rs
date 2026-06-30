//! groth16-encoder — snarkjs (BLS12-381) → Soroban `groth16_verifier` CLI args.
//!
//! The deployed contract's `verify_proof(vk, proof, pub_signals)` takes
//! `BytesN<96>` (G1), `BytesN<192>` (G2) and `Vec<U256>`. Each curve point is
//! the arkworks *uncompressed* serialization, exactly as the verifier's
//! `src/test.rs` builds it via `G1Affine::from_array` / `G2Affine::from_array`.
//! We reproduce that serialization here and emit hex so the values fed on-chain
//! are byte-for-byte identical to the ones the in-env unit test already proved.
//!
//! Usage:
//!   groth16-encoder <data_dir> <out_dir>
//! Reads <data_dir>/{verification_key.json,proof.json,public.json};
//! writes <out_dir>/{vk.json,proof.json,pub_signals.json} ready for
//! `stellar contract invoke ... -- verify_proof --vk file://... ` style args.

use ark_serialize::CanonicalSerialize;
use core::str::FromStr;
use serde::Deserialize;
use std::{env, fs, path::Path};

use ark_bls12_381::{Fq, Fq2};

#[derive(Deserialize)]
struct VkJson {
    vk_alpha_1: [String; 3],
    vk_beta_2: [[String; 2]; 3],
    vk_gamma_2: [[String; 2]; 3],
    vk_delta_2: [[String; 2]; 3],
    #[serde(rename = "IC")]
    ic: Vec<[String; 3]>,
}

#[derive(Deserialize)]
struct ProofJson {
    pi_a: [String; 3],
    pi_b: [[String; 2]; 3],
    pi_c: [String; 3],
}

fn g1_hex(x: &str, y: &str) -> String {
    let p = ark_bls12_381::G1Affine::new(Fq::from_str(x).unwrap(), Fq::from_str(y).unwrap());
    let mut buf = [0u8; 96];
    p.serialize_uncompressed(&mut buf[..]).unwrap();
    hex::encode(buf)
}

fn g2_hex(x1: &str, x2: &str, y1: &str, y2: &str) -> String {
    let x = Fq2::new(Fq::from_str(x1).unwrap(), Fq::from_str(x2).unwrap());
    let y = Fq2::new(Fq::from_str(y1).unwrap(), Fq::from_str(y2).unwrap());
    let p = ark_bls12_381::G2Affine::new(x, y);
    let mut buf = [0u8; 192];
    p.serialize_uncompressed(&mut buf[..]).unwrap();
    hex::encode(buf)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let data_dir = args.get(1).map(String::as_str).unwrap_or("data");
    let out_dir = args.get(2).map(String::as_str).unwrap_or("data/soroban-args");
    fs::create_dir_all(out_dir).unwrap();

    let d = Path::new(data_dir);
    let o = Path::new(out_dir);

    // --- verification key ---
    let vk: VkJson =
        serde_json::from_str(&fs::read_to_string(d.join("verification_key.json")).unwrap()).unwrap();
    let ic: Vec<String> = vk
        .ic
        .iter()
        .map(|p| g1_hex(&p[0], &p[1]))
        .collect();
    let vk_out = serde_json::json!({
        "alpha": g1_hex(&vk.vk_alpha_1[0], &vk.vk_alpha_1[1]),
        "beta":  g2_hex(&vk.vk_beta_2[0][0], &vk.vk_beta_2[0][1], &vk.vk_beta_2[1][0], &vk.vk_beta_2[1][1]),
        "gamma": g2_hex(&vk.vk_gamma_2[0][0], &vk.vk_gamma_2[0][1], &vk.vk_gamma_2[1][0], &vk.vk_gamma_2[1][1]),
        "delta": g2_hex(&vk.vk_delta_2[0][0], &vk.vk_delta_2[0][1], &vk.vk_delta_2[1][0], &vk.vk_delta_2[1][1]),
        "ic": ic,
    });
    fs::write(o.join("vk.json"), serde_json::to_string_pretty(&vk_out).unwrap()).unwrap();

    // --- proof ---
    let pf: ProofJson =
        serde_json::from_str(&fs::read_to_string(d.join("proof.json")).unwrap()).unwrap();
    let pf_out = serde_json::json!({
        "a": g1_hex(&pf.pi_a[0], &pf.pi_a[1]),
        "b": g2_hex(&pf.pi_b[0][0], &pf.pi_b[0][1], &pf.pi_b[1][0], &pf.pi_b[1][1]),
        "c": g1_hex(&pf.pi_c[0], &pf.pi_c[1]),
    });
    fs::write(o.join("proof.json"), serde_json::to_string_pretty(&pf_out).unwrap()).unwrap();

    // --- public signals (Vec<U256>, decimal strings) ---
    let pubs: Vec<String> =
        serde_json::from_str(&fs::read_to_string(d.join("public.json")).unwrap()).unwrap();
    fs::write(o.join("pub_signals.json"), serde_json::to_string(&pubs).unwrap()).unwrap();

    println!("wrote {}/vk.json, proof.json, pub_signals.json", out_dir);
}
