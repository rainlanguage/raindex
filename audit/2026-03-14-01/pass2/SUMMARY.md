# Pass 2 Summary -- Test Coverage

**Audit namespace:** `audit/2026-03-14-01`
**Date:** 2026-03-14

## Scope

| Agent | Source file | Lines |
|-------|-----------|-------|
| A01 | `src/abstract/OrderBookV6ArbCommon.sol` | 60 |
| A02 | `src/abstract/OrderBookV6ArbOrderTaker.sol` | 71 |
| A03 | `src/abstract/OrderBookV6FlashBorrower.sol` | 167 |
| A04 | `src/abstract/OrderBookV6FlashLender.sol` | 80 |
| A05 | `src/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sol` | 47 |
| A11 | `src/lib/LibOrderBook.sol` | 139 |
| A12 | `src/lib/LibOrderBookArb.sol` | 77 |
| A13 | `src/lib/LibOrderBookSubParser.sol` | 633 |
| A14 | `src/lib/deploy/LibOrderBookDeploy.sol` | 69 |
| A15 | `script/Deploy.sol` | 143 |

## Findings

| ID | Severity | File | Description |
|----|----------|------|-------------|
| GAP-A03-1 | Low | OrderBookV6FlashBorrower.sol | No reentrancy test for `arb4` (flash borrower path) |
| GAP-A05-1 | Low | GenericPoolOrderBookV6ArbOrderTaker.sol | No test asserting approval revocation in `onTakeOrders2` |
| GAP-A11-1 | Low | LibOrderBook.sol | No direct unit test for `doPost` with empty post array |
| GAP-A11-2 | Low | LibOrderBook.sol | No test for `doPost` skipping tasks with empty bytecode |
| GAP-A12-1 | Low | LibOrderBookArb.sol | No test for zero-balance token transfers path |
| GAP-A12-2 | Medium | LibOrderBookArb.sol | No test for output token profit sweep |
| GAP-A12-3 | Medium | LibOrderBookArb.sol | No test verifying post-arb task context values |
| GAP-A12-4 | Low | LibOrderBookArb.sol | No fuzz testing on finalizeArb |
| GAP-A13-1 | Low | LibOrderBookSubParser.sol | No dedicated sub-parser unit tests for deposit/withdraw context words |
| GAP-A14-1 | Medium | LibOrderBookDeploy.sol | RouteProcessor constants not tested in deploy tests |
| GAP-A14-2 | Medium | LibOrderBookDeploy.sol | `etchOrderBook` RouteProcessor branch not verified |
| GAP-A14-3 | Low | LibOrderBookDeploy.sol | Prod fork tests do not verify RouteProcessor deployment |
| GAP-A15-1 | Low | Deploy.sol | Deploy script has zero unit test coverage |

## Details (A01-A05)

### GAP-A03-1: No reentrancy test for arb4

`arb5` on `OrderBookV6ArbOrderTaker` has a dedicated reentrancy test that verifies `nonReentrant` blocks re-entry via `takeOrders4`. No equivalent test exists for `arb4` on `OrderBookV6FlashBorrower`, which has a different reentrancy vector (flash loan callback -> _exchange -> re-enter arb4).

**Fix:** `.fixes/GAP-A03-1-FlashBorrowerReentrancy.t.sol`

### GAP-A05-1: No approval revocation assertion for onTakeOrders2

The flash borrower path has `GenericPoolOrderBookV6FlashBorrower.approvalRevoked.t.sol` asserting spender allowance is zero after `_exchange`. The equivalent approve-call-revoke pattern in `GenericPoolOrderBookV6ArbOrderTaker.onTakeOrders2` lacks a dedicated assertion verifying allowance returns to zero.

**Fix:** `.fixes/GAP-A05-1-OrderTakerApprovalRevoked.t.sol`

## Details (A11-A15)

### GAP-A11-1: No direct unit test for doPost with empty post array
`doPost` is only tested indirectly through deposit4, withdraw4, and finalizeArb. No test explicitly calls it with an empty `TaskV2[]` to verify the no-op path.

### GAP-A11-2: No test for doPost skipping tasks with empty bytecode
Line 119 checks `task.evaluable.bytecode.length > 0` before evaluating. No test passes a task with zero-length bytecode to verify silent skipping.

### GAP-A12-1: No test for zero-balance token transfers
No test exercises finalizeArb where both token balances are zero, verifying the `if (balance > 0)` guards prevent unnecessary transfer calls.

### GAP-A12-2: No test for output token profit sweep
Only input token profit is tested. The output token transfer at line 45 of `LibOrderBookArb.sol` is not tested with a nonzero balance.

**Fix:** `.fixes/GAP-A12-2-OutputTokenProfit.sol`

### GAP-A12-3: No test verifying post-arb task context values
Both existing arb tests pass an empty task (zero-address interpreter), so the context column (Float-encoded input/output/gas balances) is never actually read or verified.

**Fix:** `.fixes/GAP-A12-3-ArbContextVerification.sol`

### GAP-A12-4: No fuzz testing on finalizeArb
Both tests use fixed values. No fuzz testing covers different decimals, amounts, or Float encoding edge cases.

### GAP-A13-1: No dedicated sub-parser unit tests for deposit/withdraw words
All 17 order-clearing context words have dedicated `OrderBookV6SubParser.context*.t.sol` test files. The 11 deposit/withdraw words (depositor, deposit-token, deposit-vault-id, deposit-vault-before, deposit-vault-after, withdrawer, withdraw-token, withdraw-vault-id, withdraw-vault-before, withdraw-vault-after, withdraw-target-amount) have no equivalent sub-parser unit tests. They are covered indirectly by integration tests in `OrderBookV6.deposit.entask.t.sol` and `OrderBookV6.withdraw.entask.t.sol`.

### GAP-A14-1: RouteProcessor constants not tested
`LibOrderBookDeploy.t.sol` tests address/codehash constants for OrderBook and SubParser but not for RouteProcessor. No test verifies `ROUTE_PROCESSOR_DEPLOYED_ADDRESS` or `ROUTE_PROCESSOR_DEPLOYED_CODEHASH`.

**Fix:** `.fixes/GAP-A14-1-RouteProcessorDeployTest.sol`

### GAP-A14-2: etchOrderBook RouteProcessor branch not verified
`testEtchOrderBook` and `testEtchOrderBookIdempotent` assert codehashes for OrderBook and SubParser after etching, but not for RouteProcessor. The RouteProcessor etch at lines 65-67 is unverified.

**Fix:** `.fixes/GAP-A14-1-RouteProcessorDeployTest.sol` (same file, includes etch test)

### GAP-A14-3: Prod fork tests skip RouteProcessor
`LibOrderBookDeployProd.t.sol` forks 5 networks and checks OrderBook + SubParser deployments. RouteProcessor is not checked on any network.

**Fix:** `.fixes/GAP-A14-3-ProdRouteProcessor.sol`

### GAP-A15-1: Deploy script has zero test coverage
`script/Deploy.sol` has no unit tests. This is common for Foundry deploy scripts and is mitigated by CI dry-run workflows. No fix proposed -- CI integration testing is the appropriate coverage mechanism.

## Overall Assessment

### A11 (LibOrderBook.sol)
`doPost` has solid indirect coverage through deposit, withdraw, and arb entask tests. The gaps are low severity -- edge-case paths (empty array, empty bytecode) are not directly tested but are exercised by the broader test suite's use of empty task arrays.

### A12 (LibOrderBookArb.sol)
`finalizeArb` has two targeted tests but they are narrow. The medium-severity gaps (output token profit and context verification) represent genuinely untested code paths. The output token `safeTransfer` path with nonzero balance is never exercised, and the Float context construction is never verified by a reading task.

### A13 (LibOrderBookSubParser.sol)
Excellent coverage for order-clearing context words (17/17 have dedicated tests). The deposit/withdraw words are covered by integration tests but lack the fine-grained sub-parser unit testing pattern used for all other words. The `authoringMetaV2()` function is tested via the pointers test.

### A14 (LibOrderBookDeploy.sol)
Strong coverage for OrderBook and SubParser constants/etch. The RouteProcessor is a consistent blind spot -- its constants, etch behavior, and prod deployment are all untested. This is the most actionable finding in this batch.

### A15 (Deploy.sol)
No unit tests, but this is expected for a Foundry deploy script. CI dry-runs provide the coverage.
