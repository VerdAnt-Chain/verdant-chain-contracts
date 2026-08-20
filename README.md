# VerdAnt Contracts

**Soroban smart contracts implementing the on-chain responsibilities of the
VerdAnt ecosystem — farmer identity, verification, escrow, and financing.**

VerdAnt is open agricultural technology & financial infrastructure built on
Stellar/Soroban. Per AD-004, only integrity-sensitive state lives on-chain;
documents/media stay off-chain and are referenced by sha256 hashes.

## Prerequisites

- Rust (stable)
- The `wasm32v1-none` target: `rustup target add wasm32v1-none`
- The Stellar CLI (`stellar`) for WASM builds and deploys

## Setup

```bash
# 1. Configure environment (testnet RPC, admin key)
cp .env.example .env

# 2. Run all tests
cargo test --workspace

# 3. Build optimized WASM for a contract crate (via Stellar CLI)
stellar contract build --package verdant-identity
# Output: target/wasm32v1-none/release/verdant_identity.wasm
```

## Scripts

| Command | Purpose |
|---------|---------|
| `cargo test --workspace` | Run all contract test suites |
| `stellar contract build --package <name>` | Build WASM for a contract crate |
| `cargo fmt --all -- --check` | Check formatting |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lint with warnings as errors |

## Architecture

Four contracts, each with a typed, documented interface, enforced
authorization, and a published event set:

| Contract | Role | Status |
|----------|------|--------|
| `verdant-identity` | Farmer identity (AgriScout) | Implemented, 11 tests |
| `verdant-verification` | AgroProof verification | Implemented, 14 tests |
| `verdant-escrow` | AgriLease / FarmFund escrow | Implemented, 14 tests |
| `verdant-financing` | FarmFund milestone financing | Implemented, 15 tests |

**Stack.** Rust (edition 2024) · Soroban SDK 27.0.6 · Stellar RPC/integration
tooling · WASM-optimized release profile (`opt-level=z`, `lto=true`,
`panic=abort`) · `soroban-sdk` `testutils` for in-env tests with ledger
snapshots · Stellar CLI for builds/deploys.

**Design principles.**

- **Stellar must earn its place.** Only identity, ownership, verification
  state, escrow, financing state, settlement, transitions, and proofs go
  on-chain; documents/media stay off-chain referenced by hash (AD-004).
- **Authorization is enforced per entrypoint** with Stellar `require_auth`;
  both authorized and unauthorized paths are tested.
- **Shared VerdAnt primitives over per-module reinvention.**
- **Interface-first.** Entrypoints, state, and events are documented before
  implementation and verified against the code.

## Contracts

Each contract has a dedicated README in its crate:

| Crate | README |
|-------|--------|
| `contracts/identity/` | [verdant-identity](contracts/identity/README.md) |
| `contracts/verification/` | [verdant-verification](contracts/verification/README.md) |
| `contracts/escrow/` | [verdant-escrow](contracts/escrow/README.md) |
| `contracts/financing/` | [verdant-financing](contracts/financing/README.md) |

### `verdant-identity` — Farmer identity (AgriScout)

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
`VerificationMarkerSet`. Tests: **11 passing**.

### `verdant-verification` — AgroProof

Production/supply-chain verification records. The verification authority
(backend) issues and revokes verification records against a batch reference.

| Entrypoint | Signature | Authorization |
|-----------|-----------|---------------|
| `initialize` | `initialize(admin, verification_authority: Address)` | `admin.require_auth()` |
| `create_verification` | `create_verification(batch, subject, proof_hash, issuer) -> u64` | `verification_authority.require_auth()` |
| `revoke_verification` | `revoke_verification(verification_id: u64)` | `verification_authority.require_auth()` |
| `get_verification` | `get_verification(verification_id: u64) -> Verification` | none (read) |
| `get_batch_verifications` | `get_batch_verifications(batch: Bytes) -> Vec<u64>` | none (read) |

Events: `Initialized`, `VerificationCreated`, `VerificationRevoked`. Tests:
**14 passing**.

### `verdant-escrow` — AgriLease / FarmFund escrow

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
`EscrowCreated`, `EscrowDeposited`, `EscrowReleased`, `EscrowRefunded`. Tests:
**14 passing**.

### `verdant-financing` — FarmFund

Milestone-based agricultural financing with programmable release. Milestones
carry `deadline_ledger`, `proof_hash`, and `proof_amount` (positive = release to
beneficiary, negative = refund from beneficiary); the contract tracks drawdown,
repayment, and default. Tests: **15 passing**.

## Identifiers (AD-009)

- On-chain keys are **typed** — never `va:`-prefixed.
- Contract-issued IDs (verification, escrow, financing) are **counter-issued**
  `u64` decimals, zero-padded to 12 digits at the presentation boundary (e.g.
  `va:verification:000000000042`). On-chain the counter is a typed `u64`; the
  `va:` form is rendered only by backend/frontend.
- Backend-issued reference keys (`va:batch:`, `va:booking:`, `va:asset:`) are
  UUIDv7, submitted to contracts as typed `Bytes`.
- `u64` comfortably exceeds 10¹² (12 digits), so counter storage is safe.

## Events & indexing

Every contract publishes Soroban events consumed by the backend indexer. The
off-chain subscription formats are the contract of record in the coordination
root's `docs/events/` (verification, escrow; financing event specs land there
as well). Event payloads and topics are documented per event and are part of
each contract's acceptance criteria.

## Tests

```bash
cargo test --workspace
```

Current suite: **54 tests green** (identity 11, verification 14, escrow 14,
financing 15). Escrow tests use the generated client + a registered Stellar
asset contract (`register_stellar_asset_contract_v2` + `StellarAssetClient::mint`);
other contracts use `mock_all_auths()` client patterns. Ledger snapshots are
committed under each crate's `test_snapshots/` (generated by the test harness).

## Lint / format

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Project layout

```
contracts/
├── identity/        # Farmer identity contract (AD-005) — implemented, 11 tests
├── verification/    # AgroProof verification — implemented, 14 tests
├── escrow/          # AgriLease/FarmFund escrow — implemented, 14 tests
├── financing/       # FarmFund financing — implemented, 15 tests
└── test_snapshots/  # per-contract ledger snapshots generated by tests
scripts/             # deploy / invoke tooling (Phase 9+)
```

Workspace members are declared in the root `Cargo.toml`; a release profile is
optimized for Soroban WASM deployment.

## Contributing

1. Fork the repo and create a branch from `main`.
2. Run the workspace suite and verify `cargo fmt --check` and `cargo clippy
   --workspace --all-targets -- -D warnings`.
3. Open a pull request. Entrypoints, state, and events must match the
   documented interface in the coordination root.

## License

Apache License 2.0. See the `LICENSE` file.