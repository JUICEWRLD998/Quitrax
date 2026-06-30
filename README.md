# Quitrax

> **Aid in the open. Identity in the dark.**
>
> Privacy-preserving humanitarian cash assistance on **Stellar**. Vulnerable people prove they are *eligible and unique* and claim USDC aid **without revealing who they are** — while donors and auditors get **cryptographic proof that every cent reached a distinct, eligible human.**

Built for **[Stellar Hacks: Real-World ZK](https://dorahacks.io/hackathon/stellar-hacks-zk)**.

---

## The problem

In humanitarian cash assistance there is a brutal, unsolved tradeoff:

- **Donors need accountability** — proof aid reached eligible, unique, real people. No ghosts, no double-dipping.
- **Recipients are the most vulnerable people on earth** — refugees, abuse survivors, dissidents. A *public ledger* linking their identity to "received aid from NGO X" is a **targeting list**.

Today, orgs must choose: surveil everyone (endangering them) or fly blind (fraud). **Zero-knowledge dissolves the tradeoff.**

## How it works

| Actor | Action |
|---|---|
| **NGO / Admin** | Registers a vetted cohort → issues secret credentials → publishes a Poseidon **Merkle root** on-chain → funds a round in USDC. |
| **Recipient** | Opens their credential → generates a **zero-knowledge proof** in-browser (Merkle membership + per-round nullifier) → claims USDC to a **fresh address**. No identity touches the chain. |
| **Donor / Auditor** | Reads the chain: total funded, unique claims, **zero double-claims (cryptographically guaranteed)**, **zero identities exposed**. |

ZK is **load-bearing**: a Circom circuit proves eligibility + uniqueness; a **Soroban contract verifies the Groth16 proof on-chain** using Stellar's native **BN254** pairing + **Poseidon** host functions (Protocol 25/26), then disburses USDC.

## Monorepo layout

```
quitrax/
├─ circuits/      # Circom circuit (claim.circom), trusted setup, proof scripts
├─ contracts/     # Soroban verifier + round/nullifier/disbursement logic (Rust)
├─ web/           # Next.js apps: Admin console · Recipient PWA · Donor dashboard
├─ scripts/       # End-to-end orchestration (cohort → round → claim) + env.sh
├─ docs/          # Architecture, threat model, demo script
└─ implementation.md   # Full plan & phase breakdown
```

## Status

Phase 0 (toolchain + de-risk) in progress. See [`implementation.md`](./implementation.md) for the full plan and [`docs/SETUP.md`](./docs/SETUP.md) for environment setup.

## Tech

Circom 2.2.3 · snarkjs (Groth16/BN254) · Soroban (`soroban-sdk` v25 BN254/Poseidon) · Stellar CLI v27 · Stellar Testnet · Next.js · TypeScript · Tailwind.

## License

MIT
