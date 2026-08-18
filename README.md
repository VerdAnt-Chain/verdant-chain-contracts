# VerdAnt Contracts

Soroban smart contracts implementing the on-chain responsibilities of the
**VerdAnt** ecosystem: farmer identity, verification, escrow, and financing.
This repository is owned by **Agent #2 (Contracts Engineer)**.

VerdAnt is open agricultural technology & financial infrastructure built on
Stellar/Soroban. Per AD-004, only integrity-sensitive state lives on-chain;
documents/media stay off-chain and are referenced by sha256 hashes.

## Table of contents

- [Stack](#stack)
- [Repository role & interface contracts](#repository-role--interface-contracts)
- [Contracts](#contracts)
- [Design principles](#design-principles)
- [Identifiers (AD-009)](#identifiers-ad-009)
- [Project layout](#project-layout)
- [Local development](#local-development)
- [Tests](#tests)
- [Lint / format](#lint--format)
- [Events & indexing](#events--indexing)
- [Roadmap status](#roadmap-status)

## Stack

- **Rust** (edition 2024) · **Soroban SDK 27.0.6**
- **WASM-optimized release profile** (`opt-level=z`, `lto=true`,
  `panic=abort`) for deployment
- Stellar RPC / integration tooling for testnet and local sandbox
- `soroban-sdk` `testutils` feature for in-env unit tests with ledger
  snapshots (`test_snapshots/`)
- Built and deployed via the **Stellar CLI** (`stellar contract build`)

## Repository role & interface contracts

One of three VerdAnt repositories (`verdant-backend`, `verdant-frontend`,
`verdant-contracts`). See [`INSTRUCTIONS.md`](../INSTRUCTIONS.md) at the
coordination root for the master architecture and the Agent Responsibility
Table.

Contract entrypoints and on-chain state consumed by the backend/frontend are
documented in [`docs/contracts/`](../docs/contracts/) at the coordination root
— that is the **contract of record** for on-chain ↔ off-chain integration.
Indexer event subscription formats live in [`docs/events/`](../docs/events/).

## Contracts

### `verdant-identity` — Farmer identity (AgriScout) ✅

The farmer is the central identity across all five pillars (AD-005). A farmer
is a Stellar account; the contract records registration, metadata hashes, and
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
`VerificationMarkerSet`. Tests: **11 passing**.

### `verdant-verification` — AgroProof ✅

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
Tests: **14 passing**. Indexer spec: `docs/events/verification.md`.

### `verdant-escrow` — AgriLease / FarmFund escrow ✅

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
Tests: **14 passing**. Indexer spec: `docs/events/escrow.md`.

### `verdant-financing` — FarmFund (Phase 7) ⏳

Milestone-based agricultural financing with programmable release. Contract
**design accepted v1.0** (`docs/contracts/financing.md`); implementation not
yet started. Milestones carry `deadline_ledger`, `proof_hash`, and
`proof_amount` (positive = release to beneficiary, negative = refund from
beneficiary); the contract tracks drawdown, repayment, and default.

## Design principles

- **Stellar must earn its place** — only identity, ownership, verification
  state, escrow, financing state, settlement, transitions, and proofs go
  on-chain; documents/media stay off-chain referenced by hash (AD-004, §9).
- **Authorization is enforced per entrypoint** with Stellar `require_auth`;
  both authorized and unauthorized paths are tested (§12).
- **Shared VerdAnt primitives over per-module reinvention** (§3, §8).
- **Interface-first** — entrypoints, state, and events are documented in
  `docs/contracts/` before implementation and verified against the code on
  acceptance.

## Identifiers (AD-009)

Per [`docs/architecture/identifiers.md`](../docs/architecture/identifiers.md)
and AD-009:

- On-chain keys are **typed** — never `va:`-prefixed.
- Contract-issued IDs (verification, escrow, financing) are **counter-issued**
  `u64` decimals, zero-padded to 12 digits at the presentation boundary
  (e.g. `va:verification:000000000042`). On-chain the counter is a typed `u64`;
  the `va:` form is rendered only by backend/frontend.
- Backend-issued reference keys (`va:batch:`, `va:booking:`, `va:asset:`) are
  UUIDv7, submitted to contracts as typed `Bytes`.
- `u64` comfortably exceeds 10¹² (12 digits), so counter storage is safe.

## Project layout

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

## Local development

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

## Tests

```bash
cargo test --workspace
```

Current suite: **39 tests green** (identity 11, verification 14, escrow 14).
Escrow tests use the generated client + a registered Stellar asset contract
(`register_stellar_asset_contract_v2` + `StellarAssetClient::mint`); other
contracts use `mock_all_auths()` client patterns. Ledger snapshots are
committed under each crate's `test_snapshots/` (generated by the test
harness).

## Events & indexing

Every contract publishes Soroban events consumed by the backend indexer. The
off-chain subscription formats are the contract of record in
[`docs/events/`](../docs/events/) (verification, escrow; financing once
implemented). Event payloads and topics are documented per event and are part
of the acceptance criteria for each contract.

## Roadmap status

- [x] Phase 1: repository foundation, base tooling
- [x] Shared identifier formats (AD-009, confirmed width in `u64`)
- [x] Phase 3: `verdant-identity` (farmer identity) — implemented + tested
- [x] Phase 5: `verdant-verification` (AgroProof) — implemented + tested
- [x] Phases 6–7: `verdant-escrow` — implemented + tested
- [x] Financing design accepted v1.0 (`docs/contracts/financing.md`)
- [ ] `verdant-financing` implementation (Phase 7)
- [ ] Deployment scripts + testnet integration (Phase 9+)

The on-chain contract interfaces are documented in
[`docs/contracts/`](../docs/contracts/) at the coordination root.
