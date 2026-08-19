# verdant-identity — Farmer Identity (AgriScout)

**On-chain farmer identity for the VerdAnt ecosystem. The farmer is the central
identity across all five product pillars (AD-005).**

A farmer is a Stellar account. This contract records registration, metadata
hashes, and verification markers. Documents and media stay off-chain and are
referenced by sha256 hashes (AD-004).

## Entrypoints

| Entrypoint | Signature | Authorization |
|-----------|-----------|---------------|
| `initialize` | `initialize(admin: Address)` | `admin.require_auth()` |
| `register_farmer` | `register_farmer(farmer: Address, metadata_hash: Bytes) -> Farmer` | `farmer.require_auth()` |
| `update_metadata` | `update_metadata(farmer: Address, metadata_hash: Bytes) -> Farmer` | `farmer.require_auth()` |
| `set_verification_marker` | `set_verification_marker(farmer, kind, issuer, issued_ledger)` | `farmer.require_auth()` |
| `get_farmer` | `get_farmer(farmer: Address) -> Farmer` | none (read) |
| `is_registered` | `is_registered(farmer: Address) -> bool` | none (read) |

## Events

`Initialized`, `FarmerRegistered`, `FarmerMetadataUpdated`,
`VerificationMarkerSet`.

## Design

- **Metadata hashes, not content.** Registration and metadata updates persist
  only a `Bytes` hash; the referenced document/media stays off-chain (AD-004).
- **Self-service registration.** Farmers authorize their own registration and
  metadata updates via `require_auth`.
- **Verification markers** are set by the farmer (via their own auth) and carry
  `kind`, `issuer`, and `issued_ledger`.

## Tests

11 passing (in-env, `soroban-sdk` `testutils`, ledger snapshots under
`test_snapshots/`). Run from the workspace root:

```bash
cargo test -p verdant-identity
```

## Build

```bash
stellar contract build --package verdant-identity
# Output: target/wasm32v1-none/release/verdant_identity.wasm
```

## Related

- Design doc: coordination root `docs/contracts/farmer-identity.md`.
- Identifiers: AD-009 (typed on-chain keys; `va:` prefixes never stored
  on-chain).