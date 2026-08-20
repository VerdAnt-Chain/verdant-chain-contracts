# verdant-financing — FarmFund

**Milestone-based agricultural financing with programmable release for the
VerdAnt ecosystem.**

Farmers draw down financing in stages; funds are released (or refunded) as
milestones are met, with explicit deadline, proof, and amount semantics. The
contract tracks drawdown, repayment, and default.

## Design

Milestones carry:

- `deadline_ledger` — the ledger by which the milestone must be proven.
- `proof_hash` — off-chain reference to the milestone evidence (AD-004).
- `proof_amount` — signed value: **positive** releases funds to the
  beneficiary, **negative** refunds funds from the beneficiary.

The contract tracks drawdown, repayment, and default across the financing
lifecycle. `FinancingDeposited` increments the deposited balance;
`FinancingRefunded` records refunds against the financing.

## Events

Contract events include `FinancingCreated`, `FinancingDeposited`,
`FinancingReleased`, and `FinancingRefunded` (payloads documented in the
coordination root's `docs/contracts/financing.md` and indexed by the backend).

## Tests

15 passing (in-env, `soroban-sdk` `testutils`, ledger snapshots under
`test_snapshots/`). Run from the workspace root:

```bash
cargo test -p verdant-financing
```

## Build

```bash
stellar contract build --package verdant-financing
# Output: target/wasm32v1-none/release/verdant_financing.wasm
```

## Related

- Design doc (accepted v1.0): coordination root `docs/contracts/financing.md`.
- Indexer projection: backend migration `0006_financing_projection.sql`.