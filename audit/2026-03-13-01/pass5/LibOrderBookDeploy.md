# Pass 5: Correctness — LibOrderBookDeploy.sol

**Agent:** A14
**File:** `src/lib/deploy/LibOrderBookDeploy.sol` (53 lines)

## Evidence of Thorough Reading

- Read all 53 lines in full
- Verified import of `Vm` from `forge-std/Vm.sol` (line 5)
- Verified imports from generated pointer files (lines 7-16): `BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE` for both `OrderBookV6` and `OrderBookV6SubParser`
- Verified 4 library constants (lines 27-39)
- Verified `etchOrderBook(Vm vm)` function (lines 45-52)

## Correctness Verification

### Library Constants

| Constant | Source | NatSpec Claim | Verified |
|---|---|---|---|
| `ORDERBOOK_DEPLOYED_ADDRESS` | `ORDERBOOK_ADDR` from `OrderBookV6.pointers.sol` | "The address of the `OrderBookV6` contract when deployed with the rain standard zoltu deployer" | Consistent with import alias |
| `ORDERBOOK_DEPLOYED_CODEHASH` | `ORDERBOOK_HASH` from `OrderBookV6.pointers.sol` | "The code hash of the `OrderBookV6` contract when deployed with the rain standard zoltu deployer" | Consistent with import alias |
| `SUB_PARSER_DEPLOYED_ADDRESS` | `SUB_PARSER_ADDR` from `OrderBookV6SubParser.pointers.sol` | "The address of the `OrderBookV6SubParser` contract when deployed with the rain standard zoltu deployer" | Consistent with import alias |
| `SUB_PARSER_DEPLOYED_CODEHASH` | `SUB_PARSER_HASH` from `OrderBookV6SubParser.pointers.sol` | "The code hash of the `OrderBookV6SubParser` contract when deployed with the rain standard zoltu deployer" | Consistent with import alias |

### `etchOrderBook` Function (lines 45-52)

**NatSpec claim (lines 41-43):** "Etches the runtime bytecode of the orderbook and sub parser at their expected deterministic addresses. Skips any contract whose codehash already matches."

**Implementation verification:**

1. **Orderbook etch (lines 46-48):**
   - Condition: `ORDERBOOK_DEPLOYED_CODEHASH != ORDERBOOK_DEPLOYED_ADDRESS.codehash`
   - Action: `vm.etch(ORDERBOOK_DEPLOYED_ADDRESS, ORDERBOOK_RUNTIME_CODE)`
   - Skips if codehash already matches -- NatSpec accurate
   - Uses `.codehash` which returns `keccak256(code)` for deployed contracts, or `0` for empty accounts. This means if the address has no code (is an EOA or doesn't exist), the condition will be `hash != 0` which is true (assuming the hash is non-zero), so it will etch. Correct behavior for test setup.

2. **SubParser etch (lines 49-51):**
   - Same pattern. Correct.

**Context:** This library uses `Vm` from forge-std, so it's only usable in Foundry test/script contexts. The `@title` NatSpec (lines 18-23) accurately describes the library's purpose: idempotent deployments against precommitted addresses.

### Import Aliasing

The imports use clear aliasing:
- `BYTECODE_HASH as ORDERBOOK_HASH` / `SUB_PARSER_HASH`
- `DEPLOYED_ADDRESS as ORDERBOOK_ADDR` / `SUB_PARSER_ADDR`
- `RUNTIME_CODE as ORDERBOOK_RUNTIME_CODE` / `SUB_PARSER_RUNTIME_CODE`

Since both generated files export the same symbol names (`BYTECODE_HASH`, `DEPLOYED_ADDRESS`, `RUNTIME_CODE`), the aliasing is necessary and correctly disambiguates between the two contracts.

## Findings

No findings. All NatSpec matches implementation. The idempotent etch pattern is correct and the codehash comparison is the right way to skip already-etched contracts.

## Summary

| ID | Severity | Description |
|---|---|---|
| (none) | -- | -- |

The library correctly provides idempotent test-time deployment helpers for OrderBookV6 and its sub-parser. Constants are directly sourced from generated pointer files, and the etch function's skip-if-matching behavior is accurately documented.
