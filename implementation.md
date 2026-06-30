# Quitrax — Implementation Plan

> **One-liner:** Quitrax is privacy-preserving humanitarian cash assistance on Stellar. Refugees and at-risk people prove they are *eligible and unique* — and claim USDC aid — **without ever revealing who they are**, while donors and auditors get **cryptographic proof that every cent reached a distinct, eligible human.**

> **Tagline:** *Aid in the open. Identity in the dark.*

> **Hackathon:** [Stellar Hacks: Real-World ZK](https://dorahacks.io/hackathon/stellar-hacks-zk) · $10,000 XLM pool · Submission deadline **July 3, 2026** (extended). Today: **June 29, 2026.**

> **Repo/codename:** `Quitrax` (brand name is flexible — alternates: *Umbra*, *Aegis*, *Veil*).

---

## 0. Architecture locked in Phase 0 (supersedes earlier BN254 notes)

After standing up the toolchain and studying the official `stellar/soroban-examples`, we locked these decisions. **Where this section conflicts with later BN254 wording, this section wins** (the later text is the original pre-research draft, kept for narrative).

### Curve & hash: **BLS12-381 + `poseidon255`** (not BN254)
The *proven, on-chain-verified* ZK path on Stellar today is **BLS12-381**: both the official `groth16_verifier` example and the `privacy-pools` example use `env.crypto().bls12_381()` (`g1_add`, `g1_mul`, `pairing_check`) and ship working testnet demos. Crucially, our circuit hashes with **Poseidon**, and the *same* Poseidon must run in-circuit (Circom) **and** on-chain (Rust, inside LeanIMT). The privacy-pools repo provides exactly this: **`poseidon255`** — a Poseidon over the BLS12-381 scalar field that is byte-for-byte consistent between Circom and Rust. Choosing BLS12-381 inherits that consistency for free; choosing BN254 would force us to re-derive a matching Poseidon under a 4-day clock. **Decision: BLS12-381 end-to-end.**

### Substrate: build on the official **`privacy-pools`** example
`privacy-pools` is Stellar's own reference for our exact primitive (commitments → Merkle tree → nullifier-gated withdrawal + ZK proof). We **reuse** its proven crypto substrate and **build** the novel application + product on top:

| Reuse (proven, from `soroban-examples`) | Build (our novel contribution) |
|---|---|
| `groth16_verifier` contract (BLS12-381) | **Aid-claim circuit** (`claim.circom`) — simplified `Withdraw` + **per-round nullifier** + **recipient binding** |
| `poseidon255.circom` / `poseidon255_constants.circom` | **Quitrax contract** — `register_cohort` (admin issues eligibility) + `open_round`/`claim` (USDC disbursement, per-round nullifier set) |
| `merkleProof.circom` (LeanIMT-consistent) | **3-app product**: Admin console · Recipient PWA · Donor/transparency dashboard + the *Reveal/Conceal* viz |
| `lean-imt` Rust lib (on-chain Merkle tree) | **Off-chain orchestration**: cohort credential issuance, round funding, indexer serving Merkle paths |
| `circom2soroban` CLI (VK/proof → contract hex) | **Per-round nullifier semantics** (`Poseidon255(nullifier, roundId)`) — claim once per round, unlinkable across rounds |
| `coinutils` CLI (coin/commitment helpers) | The emotional narrative, demo, threat model, award-grade UI |

**Model mapping vs. privacy-pools:** their *deposit* (user pays in) → our **`register_cohort`** (NGO issues eligibility, no payment); their *withdraw* → our **`claim`** (fixed per-round USDC to a fresh recipient). Their *association set* concept ≈ our "NGO-approved eligible cohort."

### Hard consistency requirement
On-chain LeanIMT (Rust `poseidon255`) and the circuit's `MerkleProof` (Circom `poseidon255`) **must** hash identically — guaranteed by reusing their files verbatim. We never hand-roll Poseidon.

### Toolchain reality (see `docs/SETUP.md`)
Stellar CLI **v27.0.0** + circom **v2.2.3** installed as **prebuilt Windows binaries** in `.localbin/` (the link was too slow/flaky to compile from source). Windows TLS **revocation checks** were disabled for git/cargo/curl (`CRYPT_E_REVOCATION_OFFLINE`). Soroban build target is **`wasm32v1-none`**. ZK deps (snarkjs, circomlib) installed locally under `circuits/`.

---

## 1. Why this wins

Judges (top engineers, founders, VCs) reward four things. Quitrax maxes all four:

### 1.1 Emotional weight (the story judges remember)
In humanitarian cash assistance there is a brutal, unsolved tradeoff:

- **Donors & auditors need accountability** — proof aid reached *eligible, unique, real* people. No ghost recipients, no double-dipping, no diversion. Without it, funding dries up.
- **Recipients are the most vulnerable people on earth** — refugees, abuse survivors, dissidents, the undocumented. A *public ledger* linking their identity to "received aid from NGO X in region Y" is not a privacy nuisance; it is a **targeting list**. It gets people persecuted, extorted, or killed.

Today orgs are forced to choose: **surveil everyone** (KYC that endangers and excludes the undocumented) **or fly blind** (fraud, donor distrust). This is real: the UNHCR ran live USDC aid disbursements *on Stellar* to displaced Ukrainians — Stellar is **already** the rails for this. Quitrax adds the missing privacy layer.

**Zero-knowledge dissolves the tradeoff entirely.** That is the "we've never seen this" moment.

### 1.2 Perfect Stellar fit
The hackathon explicitly says real-world finance (stablecoins, cross-border payments, financial inclusion) is *especially welcome*. Quitrax is the most on-mission application possible for Stellar: moving real money to real people in the real world — privately and provably.

### 1.3 ZK is load-bearing, not decorative
Remove the ZK and the product collapses. The entire value proposition *is* the proof. We use:
- **Poseidon Merkle membership** — "I'm in the approved beneficiary set" without revealing which leaf.
- **Nullifiers** — Sybil/double-claim resistance: "I haven't already claimed this round," unlinkable across rounds.
- **Groth16 over BN254** verified *on-chain in Soroban* using the **brand-new Protocol 25/26 host functions** (`bn254` pairing + native `poseidon`). We are showcasing exactly the primitives this hackathon exists to promote.

### 1.4 Technical depth + de-risked build
We build on the **official** [`stellar/soroban-examples/groth16_verifier`](https://github.com/stellar/soroban-examples/tree/main/groth16_verifier) (Circom 2.2.1 → snarkjs → Soroban BN254). Native `poseidon`/`poseidon2` host functions ([CAP-75](https://developers.stellar.org/docs/build/apps/zk)) make the Merkle tree cheap both in-circuit and on-chain. The hard cryptography is already proven to work on Stellar — our job is the *application*: circuit design, round/nullifier accounting, disbursement, and a beautiful end-to-end product.

---

## 2. What we are building (system overview)

Three surfaces over one ZK protocol and one Soroban contract:

```
                         ┌─────────────────────────────────────────┐
                         │            QUITRAX PROTOCOL              │
                         └─────────────────────────────────────────┘
   (1) ADMIN / NGO            (2) RECIPIENT (PWA)        (3) DONOR / AUDITOR
   ─────────────────         ──────────────────────     ────────────────────
   • Register cohort          • Open claim credential     • Live transparency
     → secret credentials       (QR / link)                 dashboard
   • Build Poseidon Merkle    • Fetch Merkle path         • Total funded / spent
     tree, publish root       • Generate Groth16 proof    • # unique claims
   • Fund round w/ USDC          in-browser (WASM)         • Proof: every claim
   • Set per-claim amount     • Submit to Soroban            distinct & eligible
                              • Receive USDC to a          • ZERO identities
                                fresh address                 leaked
            │                          │                          │
            └──────────────┬───────────┴───────────┬──────────────┘
                           ▼                        ▼
                 ┌───────────────────┐    ┌────────────────────────┐
                 │  Quitrax Soroban  │    │  Off-chain ZK toolkit  │
                 │     contract      │◄───│ Circom + snarkjs (WASM)│
                 │ • verify(proof)   │    │ Poseidon Merkle helper │
                 │ • nullifier set   │    └────────────────────────┘
                 │ • USDC disburse   │
                 │   (BN254+Poseidon │
                 │    host fns)      │
                 └───────────────────┘
                           │
                    Stellar Testnet
```

### The flow in plain English
1. **NGO** loads a beneficiary cohort (vetted offline, as today). For each beneficiary it generates a secret credential `(nullifier, secret)` and a public **commitment** `leaf = Poseidon(nullifier, secret)`. It builds a Poseidon Merkle tree of all commitments, **publishes only the root on-chain**, and funds the round with USDC.
2. The **beneficiary** receives their credential privately (QR code / claim link — same channel NGOs already use for vouchers). They open the Quitrax PWA, which fetches their Merkle path, and **generates a zero-knowledge proof in the browser** that: their commitment is in the tree **AND** their per-round nullifier is correctly derived — without revealing the commitment or any identity.
3. They submit the proof to the **Soroban contract** along with a **fresh payout address**. The contract verifies the Groth16 proof (BN254 pairing), checks the nullifier hasn't been spent this round, marks it spent, and **transfers USDC** to the fresh address. On-chain there is **no link** between the person and the payment.
4. The **donor dashboard** reads the chain: budget, number of commitments, number of unique nullifiers (= unique claims), funds disbursed, and a cryptographic guarantee that no one claimed twice — **with no personal data anywhere.**

---

## 3. ZK design (the heart of the project)

### 3.1 Credential & commitment
- `secret` — random field element, private to beneficiary.
- `nullifier` — random field element, private to beneficiary.
- `commitment (leaf)` = `Poseidon(nullifier, secret)` — public, placed in the Merkle tree.

### 3.2 Per-round nullifier (Sybil resistance + unlinkability)
- `nullifierHash` = `Poseidon(nullifier, roundId)`
- **One claim per beneficiary per round** (contract rejects a repeated `nullifierHash`).
- Because `roundId` is mixed in, the same beneficiary's nullifier in Round 5 is **unlinkable** to their nullifier in Round 6. They can claim every round, but never twice, and nobody can correlate their claims over time.

### 3.3 The circuit — `circuits/claim.circom`
**Public signals:**
- `merkleRoot` — round's published Poseidon Merkle root.
- `roundId` — current round.
- `nullifierHash` — to be marked spent.
- `recipientAddress` — payout address, bound into the proof (anti-front-running, see 3.4).

**Private signals:**
- `secret`, `nullifier`
- `pathElements[DEPTH]`, `pathIndices[DEPTH]` — Merkle authentication path.

**Constraints:**
1. `leaf <== Poseidon(nullifier, secret)`
2. Verify Merkle path: fold `leaf` up `DEPTH` levels using `Poseidon(left,right)` per `pathIndices`, assert final hash `=== merkleRoot`.
3. `computedNullifierHash <== Poseidon(nullifier, roundId)`; assert `=== nullifierHash`.
4. **Bind recipient:** `addrSq <== recipientAddress * recipientAddress` (a no-op constraint that forces `recipientAddress` into the proof; changing it invalidates the proof).

`DEPTH = 20` (supports ~1M beneficiaries; tune down to 16 for faster demo proving if needed).

### 3.4 Why bind the recipient address?
A submitted proof is public in the mempool. Without binding, an attacker could copy a victim's valid proof and redirect the USDC to themselves before the victim's tx lands. By committing `recipientAddress` as a public signal, the proof is **only valid for that exact payout address** — proof theft is worthless.

### 3.5 Proving system
- **Groth16 over BLS12-381** (see §0) — matches the official Soroban examples; verified on-chain via `env.crypto().bls12_381().pairing_check`. Tiny constant-size proofs. Circom compiled with `--prime bls12381`.
- **Trusted setup:** for the hackathon we run a local Powers-of-Tau (bls12381) + circuit-specific phase-2 (`snarkjs groth16 setup`). We **document this honestly** as a demo ceremony and note that production would use a multi-party ceremony.

### 3.6 Toolchain
`Circom 2.2.3 --prime bls12381` → `snarkjs` (Groth16, bls12381) → witness/proof generation runs **in-browser via the snarkjs WASM** so the secret never leaves the device. Proof + public signals + verification key are encoded for Soroban using the reused **`circom2soroban`** CLI from `privacy-pools` (handles the canonical BLS12-381 byte layout).

---

## 4. Soroban contract — `contracts/quitrax`

Built on top of `stellar/soroban-examples/groth16_verifier`, extended with application logic.

### 4.1 Storage
- `vk` — Groth16 verification key (set once at init).
- `usdc` — address of the USDC Stellar Asset Contract (testnet).
- `admin` — NGO admin address.
- `rounds: Map<u32, Round>` where `Round { merkle_root, amount, budget_remaining, active }`.
- `spent: Map<(round_id, nullifier_hash), bool>` — the nullifier set.

### 4.2 Functions
| Fn | Caller | Effect |
|---|---|---|
| `init(admin, usdc, vk)` | deployer | one-time setup |
| `open_round(round_id, merkle_root, amount, budget)` | admin | publish root, pull `budget` USDC into contract, mark active |
| `claim(round_id, recipient, proof, pub_signals)` | anyone | **core** (below) |
| `close_round(round_id)` | admin | stop claims, refund remainder |
| `round_stats(round_id)` | view | budget, spent, #claims — powers the dashboard |

### 4.3 `claim` logic (the critical path)
```
1. require round.active && round.budget_remaining >= round.amount
2. require pub_signals.merkle_root == round.merkle_root
3. require pub_signals.round_id   == round_id
4. require pub_signals.recipient  == recipient          // address binding
5. require !spent[(round_id, pub_signals.nullifier_hash)]   // anti double-claim
6. require groth16_verify(vk, proof, pub_signals) == true    // BLS12-381 pairing_check (via groth16_verifier contract)
7. spent[(round_id, nullifier_hash)] = true
8. usdc.transfer(contract, recipient, round.amount)
9. round.budget_remaining -= round.amount
10. emit ClaimEvent { round_id, nullifier_hash, amount }     // NO identity
```
`groth16_verify` is the reused `groth16_verifier` contract using `bls12_381::{g1_add, g1_mul, pairing_check}` host functions; Merkle correctness is enforced *inside* the circuit, so the contract only trusts the stored root — keeping the contract lightweight. Public-signal order passed to the verifier: `[nullifierHash, stateRoot, roundId, recipient]` (snarkjs emits outputs first, then public inputs).

### 4.4 Tests
- Unit (Rust): valid proof passes; tampered proof fails; replayed nullifier fails; wrong-round root fails; budget exhaustion fails; recipient-binding mismatch fails.
- Integration: deploy to **testnet**, run a full cohort → claim → disburse cycle.

---

## 5. Tech stack

| Layer | Choice | Notes |
|---|---|---|
| Circuits | Circom 2.2.3 (`--prime bls12381`) + `poseidon255` + circomlib gadgets | LeanIMT-consistent with on-chain |
| Proving | snarkjs (Groth16/bls12381), WASM in-browser | secret never leaves device |
| Contract | Rust + `soroban-sdk` v25 (`crypto::bls12_381`) + `lean-imt` | adapts `privacy-pools` + `groth16_verifier` |
| Chain | Stellar **Testnet** (Protocol 25+) | USDC via SAC / test asset |
| Stellar client | `@stellar/stellar-sdk`, Stellar Wallets Kit (Freighter for admin) | recipients use ephemeral keypairs |
| Frontend | Next.js 14 (App Router) + TypeScript + Tailwind | PWA for recipient app |
| UI motion | Framer Motion (`motion-framer`), subtle Three.js starfield | see §6 |
| Components | shadcn/ui + Magic UI (animated components plugin) | clean, professional |
| Merkle/indexer | Lightweight Node/Next API route serving tree + paths (paths are non-secret) | JSON store is fine for demo |

---

## 6. UI / UX — clean, professional, award-grade

Design direction synthesized from Awwwards fintech/dashboard winners and 2025–26 fintech UX trends (dark mode, bold typography, progressive disclosure, glassmorphism used sparingly).

### 6.1 Design system
- **Theme:** deep-space dark. Background `#0A0B1A` → `#0D1026`. Surfaces use subtle glass (`backdrop-blur`, 1px hairline borders at 8–12% white).
- **Palette:** Stellar-adjacent. Primary accent **electric violet `#7C5CFC`**; success/trust **aqua `#21D4B4`**; warning amber for "exposed" states. High-contrast off-white text `#E8EAF6`.
- **Typography:** display = a confident geometric sans (e.g. `Clash Display` / `Satoshi`); body = `Inter`. Big, calm headline scale. Numbers in tabular figures.
- **Motion:** restrained and physical. Spring transitions on cards; numbers count up; proof-generation has a tasteful "computing proof" shimmer. No gratuitous animation — judges read it as polish, not noise.
- **Subtle 3D:** a slow, low-density Three.js **starfield** behind the hero/dashboard only (Stellar = stars). Performance-budgeted, disabled on low-power.
- **Progressive disclosure:** recipient sees "You're eligible. Tap to claim $50 USDC." The cryptography (nullifier, Merkle path, proof bytes) lives behind an "Advanced / Verify" disclosure.

### 6.2 The killer judge moment — the **Reveal/Conceal split-screen**
A single hero visualization, two panels side by side over the *same* disbursement:
- **LEFT — "A normal ledger"**: a public transaction list with names, locations, biometrics, amounts highlighted in alarming amber — "this is a targeting list."
- **RIGHT — "Quitrax"**: the identical aid delivered; on-chain you see only `nullifierHash`, a fresh address, an amount, and a green ✓ "proven eligible & unique." Same accountability, zero exposure.

Animate the left panel's identity fields *dissolving into proofs* as it morphs into the right. This 8-second moment is the thumbnail of the demo video and the slide judges screenshot.

### 6.3 The three apps
1. **Admin / NGO console** — cohort upload, Merkle root publish, round funding, live spend gauge. Authoritative, data-dense, calm.
2. **Recipient PWA** — mobile-first, 3 taps: *Open credential → Generating proof… → Received $50 USDC.* Reassuring, human, multilingual-ready. This is where the emotion lives.
3. **Donor / Transparency dashboard** — the public proof-of-integrity view + the split-screen viz. Built to make a funder trust the system in 10 seconds.

### 6.4 Build approach for UI
Use the `meta-skills:modern-web-design` skill for the design system, `core-3d-animation:motion-framer` for transitions, the `animated-component-libraries` (Magic UI) plugin for polished primitives, and a minimal R3F/Three.js starfield. Keep it shippable — fidelity over feature count.

---

## 7. Phased implementation plan (June 29 → July 3)

Each phase has a **demoable deliverable** so we always have something to submit even if later phases slip.

### Phase 0 — Setup & de-risk (Day 1 morning)
- [ ] Init monorepo: `circuits/`, `contracts/`, `web/`, `scripts/`, `docs/`.
- [ ] Install Circom 2.2.1, snarkjs, Rust + `soroban-cli`, Node.
- [ ] Clone & build `stellar/soroban-examples/groth16_verifier`; deploy it to testnet and verify the sample `a*b=c` proof end-to-end. **This proves the whole pipeline before we invest in app logic.**
- [ ] Create testnet accounts (admin, contract); fund via friendbot; set up a USDC test asset / SAC.
- **Deliverable:** a verified Groth16 proof on testnet. Pipeline green.

### Phase 1 — ZK circuit (Day 1 afternoon → Day 2 morning)
- [ ] Write `claim.circom` (Poseidon leaf, Merkle verify, per-round nullifier, address binding).
- [ ] Compile, run Powers-of-Tau + phase-2 setup, export vkey.
- [ ] Build a TS helper: generate `(secret, nullifier)`, commitment, build Poseidon Merkle tree, produce paths.
- [ ] Generate & locally verify a real claim proof; write the BN254→Soroban encoder.
- **Deliverable:** `node prove.js` produces a valid claim proof + public signals.

### Phase 2 — Soroban contract (Day 2)
- [ ] Fork the verifier; add `Round`, nullifier `spent` set, `open_round`, `claim`, `close_round`, `round_stats`.
- [ ] Wire USDC transfer on successful claim.
- [ ] Rust tests: pass / tamper-fail / replay-fail / budget-fail / binding-fail.
- [ ] Deploy to testnet; verify a real claim proof on-chain disburses USDC.
- **Deliverable:** on-chain anonymous claim moves real testnet USDC. **This is the core win — secured by mid-event.**

### Phase 3 — End-to-end glue (Day 2 evening → Day 3 morning)
- [ ] CLI/script: admin registers a 100-person cohort → publishes root → funds round.
- [ ] Script: a recipient claims and receives USDC to a fresh address.
- [ ] Minimal indexer API: serve round root + Merkle paths to the web app.
- **Deliverable:** full loop runnable from terminal — the spine of the demo.

### Phase 4 — Frontend, three apps (Day 3 → Day 4)
- [ ] Design system + shared components (theme, glass cards, starfield, motion).
- [ ] **Recipient PWA**: credential open → in-browser proof (snarkjs WASM) → submit → success. *(highest priority — the emotional core)*
- [ ] **Donor dashboard** + the **Reveal/Conceal split-screen** viz. *(highest wow)*
- [ ] **Admin console**: cohort upload, root publish, fund, live spend gauge.
- **Deliverable:** clickable, beautiful, end-to-end product on testnet.

### Phase 5 — Polish, story, submit (Day 4 → Day 5 / July 3)
- [ ] Seed a compelling demo cohort + funded round; rehearse the happy path.
- [ ] Record **2–3 min demo video**: open on the human story → live anonymous claim → split-screen reveal → "every claim provably eligible, unique, and untraceable, settling real USDC on Stellar."
- [ ] README: problem, architecture diagram, ZK explainer, run instructions, honest notes on trusted setup & threat model.
- [ ] Ensure repo is open-source + clean. Submit on DoraHacks before the deadline.
- **Deliverable:** submission complete.

### Stretch (only if ahead of schedule)
- **Selective auditor disclosure:** an auditor view-key proves aggregate stats (total disbursed = Σ claims) without identities — directly hits the SDF's "compliance + privacy" theme.
- **Conditional aid (zk-CCT):** funds unlock only when a recipient proves (in ZK) a condition was met (e.g. school attendance attestation) — nods to real conditional-cash-transfer programs.
- **Proof-of-conservation:** a contract-level invariant proof that Σ disbursements ≤ funded budget.

---

## 8. Demo script (for the video — judges decide here)

1. **0:00 Hook (story):** "A refugee needs aid. The ledger that delivers it could also be the list that gets her killed." Show the alarming LEFT panel.
2. **0:25 The idea:** Quitrax — provable aid, protected people. On Stellar.
3. **0:40 Admin:** NGO funds a $10,000 USDC round for a 100-person cohort; publishes the Merkle root. (10s.)
4. **1:00 Recipient (the heart):** On a phone, she opens her claim. "Generating zero-knowledge proof…" → "Received $50 USDC." No name, no location, nothing. (30s.)
5. **1:35 The reveal:** Split-screen morph — identity fields dissolve into proofs. Same aid, zero exposure.
6. **2:00 Donor dashboard:** 100 unique claims, $5,000 disbursed, **0 double-claims — cryptographically guaranteed**, **0 identities on-chain.**
7. **2:25 Tech depth (fast):** Circom + Poseidon Merkle + Groth16 verified on-chain via Stellar's new BN254 host functions. ZK is load-bearing.
8. **2:40 Close:** "Accountability for donors. Safety for the vulnerable. Finally, both."

---

## 9. Risks & mitigations

| Risk | Mitigation |
|---|---|
| BN254 Soroban encoding finicky (endianness/coords) | Reuse the official example's proven encoder; validate against its sample first in Phase 0. |
| In-browser proving too slow | Drop Merkle `DEPTH` to 16; keep cohort small for demo; show "computing proof" UX so latency reads as feature. |
| Trusted-setup credibility question | Disclose honestly as a demo ceremony; document MPC ceremony path for production. Pre-empt the judge question. |
| Scope creep across 3 apps | Phase ordering guarantees the on-chain core (Phase 2) ships first; apps are layered on a working spine. Recipient PWA + dashboard are the only must-haves; admin can be a script if time-short. |
| USDC/SAC setup friction on testnet | Fall back to a self-issued test stablecoin asset; the disbursement logic is identical. |

---

## 10. Submission checklist
- [ ] Public open-source repo (MIT), clean history, README with architecture + run steps.
- [ ] ZK is *demonstrably* load-bearing (circuit + on-chain verifier, documented).
- [ ] Touches Stellar: proofs verified in a Soroban contract on testnet; real USDC moves.
- [ ] 2–3 min demo video with the human story + live claim + reveal.
- [ ] Honest threat-model & trusted-setup notes.
- [ ] Submitted on DoraHacks before **July 3, 2026**.

---

## 11. Naming note
`Quitrax` is the working name (matches the folder). If a more legible brand is preferred, strong alternates that signal "protection + light/shadow" (on-theme for ZK + Stellar): **Umbra** ("aid in the open, identity in the dark"), **Aegis** (the shield), **Veil**. Final call is the team's — the concept and architecture above are unchanged either way.

---

### Sources
- [Stellar Hacks: Real-World ZK — DoraHacks](https://dorahacks.io/hackathon/stellar-hacks-zk)
- [ZK Proofs on Stellar — Stellar Docs](https://developers.stellar.org/docs/build/apps/zk)
- [stellar/soroban-examples — groth16_verifier](https://github.com/stellar/soroban-examples/tree/main/groth16_verifier)
- [5 Real-World Zero-Knowledge Use Cases — Stellar](https://stellar.org/blog/developers/5-real-world-zero-knowledge-use-cases)
- [Noir Groth16 backend + Stellar BN254 encoding — JamesBachini](https://jamesbachini.com/noir-groth16/)
