# VerdAnt Contracts — Project Proposal

**Soroban smart contracts implementing the on-chain responsibilities of the
VerdAnt ecosystem — farmer identity, verification, escrow, and financing.**

**Document status:** 2026-08-18 · Revision 2
**Owner:** Agent #2 (Contracts Engineer)
**Part of:** the VerdAnt three-repository system (this repo, `verdant-backend`,
`verdant-frontend`).

---

## 1. Background

VerdAnt is open agricultural technology & financial infrastructure built on
Stellar/Soroban. Its value rests on **verifiability**: any counterparty must be
able to check that a farmer is registered, that a production record was
verified, that escrowed funds are released only on the agreed condition, and
that financing milestones are met. Centralized databases cannot provide that
independent assurance; Soroban contracts can.

Per AD-004, only integrity-sensitive state lives on-chain. Documents and media
stay off-chain and are referenced by sha256 hashes, keeping storage costs
proportionate while preserving verifiability.

## 2. Objectives

1. Provide an on-chain **farmer identity** that anchors all five VerdAnt
   pillars (AD-005).
2. Provide **verification** records (AgroProof) issued and revocable by a
   designated verification authority.
3. Provide **escrow** (AgriLease / FarmFund) with programmable release
   conditions and SEP-41 token support.
4. Provide **milestone financing** (FarmFund) with programmable release,
   drawdown, repayment, and default handling.
5. Emit structured Soroban events consumed by the backend indexer, so on-chain
   state is reconstructable off-chain.

## 3. Scope

**In scope.** The `verdant-identity`, `verdant-verification`, `verdant-escrow`,
and `verdant-financing` contracts; in-env unit tests with ledger snapshots;
WASM-optimized release builds; deployment/invoke tooling.

**Out of scope.** Off-chain documents/media (AD-004); on-chain storage of
`va:`-prefixed identifiers (AD-009); the backend API and frontend surfaces
(handled by the other repositories).

## 4. Proposed solution & architecture

Four contracts, each with a typed, documented interface, enforced authorization,
and a published event set:

| Contract | Role | Status |
|----------|------|--------|
| `verdant-identity` | Farmer identity (AgriScout) | Implemented, 11 tests |
| `verdant-verification` | AgroProof verification | Implemented, 14 tests |
| `verdant-escrow` | AgriLease / FarmFund escrow | Implemented, 14 tests |
| `verdant-financing` | FarmFund milestone financing | Design accepted v1.0; impl pending |

**Stack.** Rust (edition 2024) · Soroban SDK 27.0.6 · Stellar RPC/integration
tooling · WASM-optimized release profile (`opt-level=z`, `lto=true`,
`panic=abort`) · `soroban-sdk` `testutils` for in-env tests with ledger
snapshots · Stellar CLI (`stellar contract build`) for builds/deploys.

**Design principles.**

- **Stellar must earn its place.** Only identity, ownership, verification
  state, escrow, financing state, settlement, transitions, and proofs go
  on-chain; documents/media stay off-chain referenced by hash (AD-004).
- **Authorization is enforced per entrypoint** with Stellar `require_auth`;
  both authorized and unauthorized paths are tested.
- **Shared VerdAnt primitives over per-module reinvention.**
- **Interface-first.** Entrypoints, state, and events are documented before
  implementation and verified against the code on acceptance.

### 4.1 `verdant-identity` — Farmer identity (AgriScout)

The farmer is the central identity across all five pillars (AD-005). A farmer is
a Stellar account; the contract records registration, metadata hashes, and
verification markers.

| Entrypoint | Signature | Authorization |
|-----------|-----------|---------------|
| `initialize` | `initialize(admin: Address)` | `admin.require_auth()` |
| `register_farmer` | `register_farmer(farmer: Address, metadata_hash: Bytes) -> Farmer` | `farmer.require_auth()` |
| `update_metadata` | `update_metadata(farmer: Address, metadata_hash: Bytes) -> Farmer` | `farmer.require_auth()` |
| `set_verification_marker` | `set_verification_marker(farmer, kind, issuer, issued_ledger)` | `farmer.require_auth()` |
| `get_farmer` | `get_farmer(farmer: Address) -> Farmer` | none (read) |
| `is_registered` | `is_registered(farmer: Address) -> bool` | none (read) |

Events: `Initialized`, `FarmerRegistered`, `FarmerMetadataUpdated`,
`VerificationMarkerSet`.

### 4.2 `verdant-verification` — AgroProof

Production/supply-chain verification records. The verification authority
(backend) issues and revokes verification records against a batch reference.

| Entrypoint | Signature | Authorization |
|-----------|-----------|---------------|
| `initialize` | `initialize(admin, verification_authority: Address)` | `admin.require_auth()` |
| `create_verification` | `create_verification(batch, subject, proof_hash, issuer) -> u64` | `verification_authority.require_auth()` |
| `revoke_verification` | `revoke_verification(verification_id: u64)` | `verification_authority.require_auth()` |
| `get_verification` | `get_verification(verification_id: u64) -> Verification` | none (read) |
| `get_batch_verifications` | `get_batch_verifications(batch: Bytes) -> Vec<u64>` | none (read) |

Events: `Initialized`, `VerificationCreated`, `VerificationRevoked`.

### 4.3 `verdant-escrow` — AgriLease / FarmFund escrow

Programmatic escrow with release conditions mirroring the booking/financing
flows. Funds are pulled from the depositor into the contract (SEP-41
`TokenClient`, XLM/USDC) and released/refunded on the condition.

| Entrypoint | Signature | Authorization |
|-----------|-----------|---------------|
| `initialize` | `initialize(admin, token: Address)` | `admin.require_auth()` |
| `create_escrow` | `create_escrow(depositor, beneficiary, amount, condition, booking_ref) -> u64` | `depositor.require_auth()` |
| `deposit` | `deposit(escrow_id, from, amount)` | `from.require_auth()` |
| `release` | `release(escrow_id, releaser, proof_hash)` | `releaser.require_auth()` |
| `refund` | `refund(escrow_id, refundee)` | `refundee.require_auth()` |
| `get_escrow` | `get_escrow(escrow_id: u64) -> Escrow` | none (read) |
| `get_escrows_for_booking` | `get_escrows_for_booking(booking_ref: Bytes) -> Vec<u64>` | none (read) |

`ReleaseCondition { kind: u32, releaser: Address, timeout_ledger: u32 }` where
`0 = Manual`, `1 = Milestone`, `2 = Timeout`. Events: `Initialized`,
`EscrowCreated`, `EscrowDeposited`, `EscrowReleased`, `EscrowRefunded`.

### 4.4 `verdant-financing` — FarmFund (Phase 7)

Milestone-based agricultural financing with programmable release. Contract
**design accepted v1.0**; implementation not yet started. Milestones carry
`deadline_ledger`, `proof_hash`, and `proof_amount` (positive = release to
beneficiary, negative = refund from beneficiary); the contract tracks drawdown,
repayment, and default.

## 5. Deliverables

### 5.1 Delivered

- `verdant-identity`, `verdant-verification`, `verdant-escrow` implemented with
  passing suites: **39 tests green** (identity 11, verification 14, escrow 14).
- Escrow tests use the generated client + a registered Stellar asset contract
  (`register_stellar_asset_contract_v2` + `StellarAssetClient::mint`); other
  contracts use `mock_all_auths()` client patterns.
- Ledger snapshots committed under each crate's `test_snapshots/`.
- Financing design accepted v1.0, addressing deposit semantics, the
  `proof_amount` field, and refund accounting.

### 5.2 Planned

- `verdant-financing` implementation (Phase 7).
- Deployment scripts + testnet integration (Phase 9+).
- Financing event spec for the indexer once implemented.

## 6. Design constraints & standards

- **Identifiers (AD-009).** On-chain keys are **typed** — never
  `va:`-prefixed. Contract-issued IDs (verification, escrow, financing) are
  counter-issued `u64` decimals, zero-padded to 12 digits at the presentation
  boundary (e.g. `va:verification:000000000042`). On-chain the counter is a
  typed `u64`; the `va:` form is rendered only by backend/frontend.
  Backend-issued reference keys (`va:batch:`, `va:booking:`, `va:asset:`) are
  UUIDv7, submitted to contracts as typed `Bytes`. `u64` comfortably exceeds
  10¹² (12 digits), so counter storage is safe.
- **Events are part of the contract.** Every contract publishes Soroban events
  consumed by the backend indexer; event payloads/topics are documented and part
  of each contract's acceptance criteria.

## 7. Timeline / roadmap

| Phase | Scope | Status |
|-------|-------|--------|
| 1 | Repository foundation, base tooling | Done |
| — | Shared identifier formats (AD-009, confirmed width in `u64`) | Done |
| 3 | `verdant-identity` (farmer identity) | Implemented + tested |
| 5 | `verdant-verification` (AgroProof) | Implemented + tested |
| 6–7 | `verdant-escrow` | Implemented + tested |
| 7 | Financing design accepted v1.0 | Done |
| 7 | `verdant-financing` implementation | Pending |
| 9+ | Deployment scripts + testnet integration | Pending |

## 8. Development & operations

Prerequisites: Rust (stable), the `wasm32v1-none` target, and the Stellar CLI.

```bash
rustup target add wasm32v1-none

# 1. Configure environment (testnet RPC, admin key)
cp .env.example .env

# 2. Run all tests
cargo test --workspace

# 3. Build optimized WASM for a contract crate (via Stellar CLI)
stellar contract build --package verdant-identity
# Output: target/wasm32v1-none/release/verdant_identity.wasm
```

### Lint / format

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## 9. Project layout

```
contracts/
├── identity/        # Farmer identity contract (AD-005) — implemented, 11 tests
├── verification/    # AgroProof verification — implemented, 14 tests
├── escrow/          # AgriLease/FarmFund escrow — implemented, 14 tests
├── financing/       # FarmFund financing — design accepted v1.0, impl pending
└── test_snapshots/  # per-contract ledger snapshots generated by tests
scripts/             # deploy / invoke tooling (Phase 9+)
```

Workspace members are declared in the root `Cargo.toml`; a release profile is
optimized for Soroban WASM deployment.

## 10. Ownership

Owned and maintained by **Agent #2 (Contracts Engineer)** as part of the VerdAnt
program. On-chain interfaces are coordinated through the program's integration
lead (Agent #4); the coordination root records contract designs and event specs.