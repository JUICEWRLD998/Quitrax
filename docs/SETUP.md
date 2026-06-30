# Environment Setup

This documents the exact toolchain Quitrax uses and the platform gotchas we hit, so the build is reproducible.

## Required tools

| Tool | Version | Purpose |
|---|---|---|
| Node.js | ≥ 20 (tested 24.14) | snarkjs, web apps, scripts |
| Rust + Cargo | ≥ 1.85 (tested 1.96) | Soroban contracts |
| `wasm32-unknown-unknown` target | — | compile contracts to WASM |
| Stellar CLI (`stellar`) | v27.0.0 | build/deploy/invoke on testnet |
| circom | v2.2.3 | compile ZK circuits |
| snarkjs | ^0.7.5 | trusted setup, proving, verification |
| circomlib | ^2.0.5 | Poseidon + Merkle gadgets |

## Quick start

```bash
# From repo root, every shell:
source scripts/env.sh        # sets PATH (.localbin, cargo, npm) + cargo revocation fix

# ZK deps (local, reproducible):
cd circuits && npm install && cd ..

# Verify tools:
stellar --version
circom --version
node circuits/node_modules/snarkjs/build/cli.cjs --version
```

## Windows gotcha — TLS revocation (`CRYPT_E_REVOCATION_OFFLINE`)

On this Windows host, schannel could not reach the certificate-revocation server, which made
`git`, `cargo`, and `curl` fail or hang on HTTPS (GitHub, crates.io). Fixes applied:

```bash
# git
git config --global http.schannelCheckRevoke false
# cargo  (persisted in ~/.cargo/config.toml)
[http]
check-revoke = false
# curl (per-invocation)
curl --ssl-no-revoke ...
```

`scripts/env.sh` exports `CARGO_HTTP_CHECK_REVOKE=false` as a belt-and-suspenders measure.

## Prebuilt binaries (network was slow)

Rather than compile the Stellar CLI and circom from source over a slow link, we use the official
prebuilt Windows binaries, kept in `.localbin/` (git-ignored):

- Stellar CLI: `stellar-cli-27.0.0-x86_64-pc-windows-msvc.tar.gz` → `stellar.exe`
- circom: `circom-windows-amd64.exe` → `circom.exe`

`scripts/env.sh` puts `.localbin/` on `PATH`. To re-provision on a fresh machine, see
`scripts/provision.sh`.

## Testnet

- Network: Stellar **Testnet** (Protocol 25+, for BN254 + Poseidon host functions).
- Accounts funded via Friendbot.
- USDC: testnet USDC issuer or a self-issued test stablecoin (disbursement logic is identical).
