# Pass 2: Test Coverage — A06 GenericPoolOrderBookV6FlashBorrower

**File:** `src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol`

## Evidence of Reading

### Source: `GenericPoolOrderBookV6FlashBorrower.sol` (53 lines)

**Contract:** `GenericPoolOrderBookV6FlashBorrower is OrderBookV6FlashBorrower`

| Item | Kind | Line |
|---|---|---|
| `GenericPoolOrderBookV6FlashBorrower` | contract | 26 |
| `constructor(OrderBookV6ArbConfig)` | constructor | 30 |
| `_exchange(TakeOrdersConfigV5, bytes)` | internal virtual override | 33 |
| `receive()` | external payable | 51 |
| `fallback()` | external payable | 52 |

**Inherited (from `OrderBookV6FlashBorrower`, relevant):**

| Item | Kind | Line (parent) |
|---|---|---|
| `supportsInterface(bytes4)` | public view override | 66 |
| `onFlashLoan(address, address, uint256, uint256, bytes)` | external | 82 |
| `arb4(IRaindexV6, TakeOrdersConfigV5, bytes, TaskV2)` | external payable nonReentrant | 129 |
| `BadInitiator` | error | 20 |
| `BadLender` | error | 25 |
| `FlashLoanFailed` | error | 28 |

**`_exchange` logic (lines 33-47):**
1. Decodes `exchangeData` into `(address spender, address pool, bytes encodedFunctionCall)`.
2. Reads `borrowedToken` from `takeOrders.orders[0].order.validOutputs[...].token`.
3. `forceApprove(spender, type(uint256).max)` on the borrowed token.
4. `pool.functionCallWithValue(encodedFunctionCall, address(this).balance)` -- forwards all ETH.
5. `forceApprove(spender, 0)` to revoke approval.

### Test Files Read in Full

| Test File | Tests |
|---|---|
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sender.t.sol` | `testGenericPoolOrderBookV6FlashBorrowerTakeOrdersSender` (fuzz, 10 runs) |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.approvalRevoked.t.sol` | `testApprovalRevokedAfterExchange` |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.ethForwarded.t.sol` | `testEthForwardedToExchangeDuringExchange` |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.exchangeRevert.t.sol` | `testExchangeRevertPropagates` |
| `test/abstract/OrderBookV6ArbCommon.fallback.t.sol` | `testFallbackAcceptsCalldata`, `testFallbackAcceptsEmptyCalldata`, `testReceiveAcceptsETH`, `testFallbackAcceptsETHWithCalldata` (FlashBorrower variant) |
| `test/abstract/OrderBookV6FlashBorrower.badLenderApproval.t.sol` | `testBadLenderRevertsWithApproval` |
| `test/abstract/OrderBookV6FlashBorrower.noOrders.t.sol` | `testArb4NoOrders` |
| `test/abstract/OrderBookV6FlashBorrower.wrongTask.t.sol` | `testArb4WrongTask` (fuzz, 10 runs) |
| `test/abstract/OrderBookV6FlashBorrower.realTokenTransfers.t.sol` | `testArb4RealTokenTransfers` |
| `test/abstract/OrderBookV6FlashBorrower.lenderValidation.t.sol` | `testMaliciousLenderCannotExploitOnFlashLoan` |
| `test/abstract/OrderBookV6FlashBorrower.flashLoanFailed.t.sol` | `testFlashLoanFailed` |
| `test/abstract/OrderBookV6FlashBorrower.mixedDecimals.t.sol` | `testArb4MixedDecimals` |
| `test/abstract/OrderBookV6FlashBorrower.badInitiator.t.sol` | `testOnFlashLoanBadInitiator` (fuzz) |

## Coverage Summary

**What IS tested:**
- Happy-path arb cycle with real ERC20 transfers (18-decimal tokens)
- Happy-path arb with mixed decimals (6/18)
- Approve-call-revoke pattern (max approval during call, zero after)
- ETH forwarding to pool via `functionCallWithValue`
- Exchange revert propagation
- `receive()` and `fallback()` accept ETH and arbitrary calldata
- `BadLender` error when malicious lender calls `onFlashLoan` directly
- `BadLender` error when malicious orderbook is passed to `arb4`
- `BadInitiator` error when initiator is not the arb contract (fuzz)
- `NoOrders` error on empty orders array
- `WrongTask` error on mismatched task hash (fuzz)
- `FlashLoanFailed` error when `flashLoan` returns false
- Construction emits `Construct` event (via `ArbTest` base)

## Findings

### A06-1: No test for `spender != pool` in `_exchange` (LOW)

**Location:** `_exchange`, lines 33-47 of `GenericPoolOrderBookV6FlashBorrower.sol`

**Description:** The `_exchange` function decodes three values from `exchangeData`: `spender`, `pool`, and `encodedFunctionCall`. The `spender` receives the ERC20 approval while the `pool` receives the function call. The NatSpec explicitly documents that these can differ ("If you are unsure, simply set it to the pool address"). However, every test sets `spender == pool`. There is no test exercising the case where the `spender` and `pool` are different addresses, which is a documented use case (e.g., router patterns where a router contract is called but a separate vault/spender pulls tokens).

**Impact:** If a code change accidentally conflated the two addresses (e.g., approving `pool` instead of `spender`), no test would catch it.

### A06-2: No fuzz test over `exchangeData` decoding in `_exchange` (LOW)

**Location:** `_exchange`, line 34-35 of `GenericPoolOrderBookV6FlashBorrower.sol`

**Description:** The `exchangeData` parameter is `abi.decode`d into `(address, address, bytes)`. There are no fuzz tests that exercise malformed or adversarial `exchangeData` payloads. While Solidity's `abi.decode` will revert on malformed data, a fuzz test would provide confidence that no edge case in the decoding or downstream usage (e.g., zero-address spender, zero-address pool, empty `encodedFunctionCall`) causes unexpected behavior rather than a clean revert.

**Impact:** Low -- `abi.decode` provides built-in safety, but explicit coverage of edge cases (address(0) spender, address(0) pool, empty bytes) would strengthen the test suite.

### A06-3: `supportsInterface` not tested for `GenericPoolOrderBookV6FlashBorrower` (INFO)

**Location:** Inherited `supportsInterface` from `OrderBookV6FlashBorrower`, line 66

**Description:** There is no test verifying that `GenericPoolOrderBookV6FlashBorrower.supportsInterface` correctly returns `true` for `IERC3156FlashBorrower.interfaceId` and `IERC165.interfaceId`, and `false` for unsupported interface IDs. While this is standard OpenZeppelin behavior, explicit tests for ERC-165 compliance are a best practice for contracts that declare interface support.

**Impact:** Informational -- unlikely to break, but good hygiene.
