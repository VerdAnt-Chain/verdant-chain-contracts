# verdant-escrow — AgriLease / FarmFund Escrow

**Programmatic escrow for the VerdAnt ecosystem, mirroring the booking and
financing flows.**

Funds are pulled from the depositor into the contract (SEP-41 `TokenClient`,
XLM/USDC) and released or refunded when the agreed condition is met.

## Entrypoints

| Entrypoint | Signature | Authorization |
|-----------|-----------|---------------|
| `initialize` | `initialize(admin, token: Address)` | `admin.require_auth()` |
| `create_escrow` | `create_escrow(depositor, beneficiary, amount, condition, booking_ref) -> u64` | `depositor.require_auth()` |
| `deposit` | `deposit(escrow_id, from, amount)` | `from.require_auth()` |
| `release` | `release(escrow_id, releaser, proof_hash)` | `releaser.require_auth()` |
| `refund` | `refund(escrow_id, refundee)` | `refundee.require_auth()` |
| `get_escrow` | `get_escrow(escrow_id: u64) -> Escrow` | none (read) |
| `get_escrows_for_booking` | `get_escrows_for_booking(booking_ref: Bytes) -> Vec<u64>` | none (read) |

`ReleaseCondition { kind: u32, releaser: Address, timeout_ledger: u32 }`:

| kind | Meaning |
|------|---------|
| `0` | Manual — `releaser` authorizes release |
| `1` | Milestone — release on milestone proof |
| `2` | Timeout — automatic release/refund at `timeout_ledger` |

## Events

`Initialized`, `EscrowCreated`, `EscrowDeposited`, `EscrowReleased`,
`EscrowRefunded`.

## Design

- **SEP-41 tokens.** The contract is initialized with a token address and
  pulls funds via the token client (XLM/USDC).
- **Booking-scoped.** Escrows can be enumerated by `booking_ref`, mirroring the
  AgriLease booking flow.
- **Conditioned release.** `release` requires the configured releaser and a
  `proof_hash` reference; `refund` returns funds to the refundee.
- **Counter-issued IDs.** `create_escrow` returns a `u64` counter (AD-009).

## Tests

14 passing (in-env, using the generated client + a registered Stellar asset
contract via `register_stellar_asset_contract_v2` + `StellarAssetClient::mint`;
ledger snapshots under `test_snapshots/`). Run from the workspace root:

```bash
cargo test -p verdant-escrow
```

## Build

```bash
stellar contract build --package verdant-escrow
# Output: target/wasm32v1-none/release/verdant_escrow.wasm
```

## Related

- Design doc: coordination root `docs/contracts/escrow.md`.
- Indexer spec: coordination root `docs/events/escrow.md`.