# VerdAnt Contracts

Soroban smart contracts implementing on-chain responsibilities for the VerdAnt
ecosystem: identity, ownership, verification, escrow, financing, settlement,
and proofs.

## Stack

- **Rust** · **Soroban SDK** · **Stellar RPC** / integration tooling
- Contract tests against testnet and/or local sandbox
- WASM-optimized release profile for Soroban deployment

## Repository role

One of three VerdAnt repositories (`verdant-backend`, `verdant-frontend`,
`verdant-contracts`). See [`INSTRUCTIONS.md`](../INSTRUCTIONS.md) at the
coordination root for the master architecture and the Agent Responsibility
Table (Agent #2 owns this repository).

Contract entrypoints consumed by the backend/frontend are documented in
[`docs/contracts/`](../docs/contracts/) at the coordination root — that is the
contract of record for on-chain ↔ off-chain integration.

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
cargo clippy --workspace -- -D warnings
```

## Project layout

```
contracts/
├── identity/       # Farmer identity contract (AD-005: farmer is central identity)
└── ...             # further on-chain modules land here (verification, escrow, ...)
scripts/            # deploy / invoke tooling (Phase 9+)
```

## Design principles

- **Stellar must earn its place** — only identity, ownership, verification
  state, escrow, financing state, settlement, transitions, and proofs go
  on-chain; documents/media stay off-chain and are referenced by hash (§9).
- **Authorization is enforced per entrypoint** with Stellar `require_auth`;
  both authorized and unauthorized paths are tested (§12).
- **Shared VerdAnt primitives over per-module reinvention** (§3, §8).

## Roadmap status

- [x] Phase 1: repository foundation, base tooling
- [x] Phase 3: Farmer identity contract (`verdant-identity`)
- [ ] Shared identifier formats (coordination with Agent #4)
- [ ] Verification / escrow contracts (AgroProof, AgriLease, FarmFund)

The on-chain contract interface is documented in
[`docs/contracts/farmer-identity.md`](../docs/contracts/farmer-identity.md) at
the coordination root.
