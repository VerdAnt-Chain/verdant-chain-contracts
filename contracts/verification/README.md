# verdant-verification — AgroProof

**Production / supply-chain verification records for the VerdAnt ecosystem.**

The verification authority (backend) issues and revokes verification records
against a batch reference, anchoring credible production attestations on-chain.

## Entrypoints

| Entrypoint | Signature | Authorization |
|-----------|-----------|---------------|
| `initialize` | `initialize(admin, verification_authority: Address)` | `admin.require_auth()` |
| `create_verification` | `create_verification(batch, subject, proof_hash, issuer) -> u64` | `verification_authority.require_auth()` |
| `revoke_verification` | `revoke_verification(verification_id: u64)` | `verification_authority.require_auth()` |
| `get_verification` | `get_verification(verification_id: u64) -> Verification` | none (read) |
| `get_batch_verifications` | `get_batch_verifications(batch: Bytes) -> Vec<u64>` | none (read) |

## Events

`Initialized`, `VerificationCreated`, `VerificationRevoked`.

## Design

- **Verification authority.** Only the designated authority (backend) can
  create or revoke records; this keeps issuance accountable.
- **Batch-scoped.** Records are created against a batch reference and can be
  enumerated per batch.
- **Proof by hash.** The `proof_hash` references off-chain evidence (AD-004);
  the evidence itself never lives on-chain.
- **Counter-issued IDs.** `create_verification` returns a `u64` counter, padded
  to 12 digits at the presentation boundary (AD-009, e.g.
  `va:verification:000000000042`).

## Tests

14 passing (in-env, `soroban-sdk` `testutils`, ledger snapshots under
`test_snapshots/`). Run from the workspace root:

```bash
cargo test -p verdant-verification
```

## Build

```bash
stellar contract build --package verdant-verification
# Output: target/wasm32v1-none/release/verdant_verification.wasm
```

## Related

- Design doc: coordination root `docs/contracts/verification.md`.
- Indexer spec: coordination root `docs/events/verification.md`.