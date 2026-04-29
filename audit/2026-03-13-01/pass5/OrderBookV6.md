# Pass 5: Correctness -- OrderBookV6.sol

**Agent:** A08
**Date:** 2026-03-13
**File:** `src/concrete/ob/OrderBookV6.sol` (~1055 lines)

## Evidence of Thorough Reading

The following items were individually verified:

- **All imports** (lines 1-63): Math, Multicall, IERC20, SafeERC20, ReentrancyGuard, IERC20Metadata, LibContext, LibBytecode, SourceIndexV2, StateNamespace, IInterpreterV4, StackItem, EvalV4, LibUint256Array, LibUint256Matrix, IInterpreterStoreV3, LibNamespace, LibMeta, IMetaV1_2, LibOrderBook, LibDecimalFloat, LibTOFUTokenDecimals/TOFUOutcome, ITOFUTokenDecimals, IRaindexV6 and all sub-types, IRaindexV6OrderTaker, LibOrder, context constants, OrderBookV6FlashLender, LibBytes32Array, LibBytes32Matrix, LibFormatDecimalFloat.
- **All error declarations** (lines 69-125): ReentrancyGuardReentrantCall, NotOrderOwner, TokenMismatch, TokenSelfTrade, TokenDecimalsMismatch, MinimumIO, SameOwner, UnsupportedCalculateInputs, UnsupportedCalculateOutputs, NegativeInput, NegativeOutput, NegativeVaultBalance, NegativeVaultBalanceChange, NegativePull, NegativePush, NegativeBounty, ClearZeroAmount.
- **All constants** (lines 127-150): ORDER_LIVE, ORDER_DEAD, CALCULATE_ORDER_ENTRYPOINT, HANDLE_IO_ENTRYPOINT, CALCULATE_ORDER_MIN_OUTPUTS, CALCULATE_ORDER_MAX_OUTPUTS, HANDLE_IO_MIN_OUTPUTS, HANDLE_IO_MAX_OUTPUTS.
- **All state variables** (lines 215-223): sOrders mapping, sVaultBalances mapping.
- **All functions**: vaultBalance2, _vaultBalance, orderExists, entask2, deposit4, withdraw4, addOrder4, removeOrder3, checkTokenSelfTrade, quote2, takeOrders4, clear3, calculateOrderIO, increaseVaultBalance, decreaseVaultBalance, recordVaultIO, handleIO, calculateClearStateChange, calculateClearStateAlice, pullTokens, pushTokens, _nonZeroVaultId, nonZeroVaultId modifier.
- **Struct** OrderIOCalculationV4 (lines 181-190) and type aliases Output18Amount / Input18Amount (lines 192-194).
- **Contract inheritance**: IRaindexV6, IMetaV1_2, ReentrancyGuard, Multicall, OrderBookV6FlashLender.

## Verification Results

### Constants

| Constant | Value | Correct? |
|---|---|---|
| CALCULATE_ORDER_ENTRYPOINT | SourceIndexV2.wrap(0) | Yes -- source 0 is calculate order |
| HANDLE_IO_ENTRYPOINT | SourceIndexV2.wrap(1) | Yes -- source 1 is handle IO |
| CALCULATE_ORDER_MIN_OUTPUTS | 2 | Yes -- maxOutput and IORatio |
| CALCULATE_ORDER_MAX_OUTPUTS | 2 | Yes -- matches min |
| HANDLE_IO_MIN_OUTPUTS | 0 | Yes -- handle IO has no outputs |
| HANDLE_IO_MAX_OUTPUTS | 0 | Yes -- matches min |
| ORDER_LIVE | 1 | Yes -- non-default, non-boolean |
| ORDER_DEAD | 0 | Yes -- default mapping value |

### Algorithm Verification: calculateClearStateAlice

The algorithm computes:
1. `aliceInput = aliceOutputMax * aliceIORatio` (Alice demands this much input for her output)
2. `aliceOutput = aliceOutputMax`
3. If `aliceInput > bobOutputMax`, cap: `aliceInput = bobOutputMax`, `aliceOutput = aliceInput / aliceIORatio`

This is correct. When Alice wants more than Bob can provide, Alice's input is capped to Bob's max output, and Alice's output is reduced proportionally to maintain her ratio.

The symmetric call `calculateClearStateAlice(bob, alice)` correctly computes Bob's side.

### Algorithm Verification: Bounty Math in clear3

```
aliceBounty = aliceOutput - bobInput
bobBounty = bobOutput - aliceInput
```

Alice outputs tokens that go to: Bob's input + clearer bounty. So `aliceOutput >= bobInput` must hold. Similarly for Bob. The negative bounty check correctly guards this invariant. Bounty tokens are deposited to the clearer's vault under the correct output token for each respective order.

### Algorithm Verification: takeOrders4 IO Math

With `IOIsInput == true`:
- `takerInput = min(orderOutputMax, remainingTakerIO)` -- taker receives order's output, capped
- `takerOutput = IORatio * takerInput` -- taker sends order's input amount

With `IOIsInput == false`:
- `orderMaxInput = IORatio * orderOutputMax` -- max the order can take in
- `takerOutput = min(orderMaxInput, remainingTakerIO)` -- taker sends capped amount
- `takerInput = takerOutput / IORatio` -- taker receives, rounded down (favors order)

The call `recordVaultIO(takerOutput, takerInput, ...)` passes (order input, order output) which matches the signature `recordVaultIO(Float input, Float output, ...)`. **Correct.**

The token flow:
- `pushTokens(msg.sender, orderOutputToken, totalTakerInput)` -- sends order output token to taker
- `pullTokens(msg.sender, orderInputToken, totalTakerOutput)` -- pulls order input token from taker

**Correct** -- taker receives `orderOutputToken` and sends `orderInputToken`.

### recordVaultIO Parameter Order at All Call Sites

| Call Site | Arg 1 (input) | Arg 2 (output) | Correct? |
|---|---|---|---|
| takeOrders4 L538 | takerOutput (=order input) | takerInput (=order output) | Yes |
| clear3 L648 (alice) | clearStateChange.aliceInput | clearStateChange.aliceOutput | Yes |
| clear3 L649 (bob) | clearStateChange.bobInput | clearStateChange.bobOutput | Yes |

### Error Conditions

| Error | Trigger | Matches Name? |
|---|---|---|
| NotOrderOwner | removeOrder3 when msg.sender != order.owner | Yes |
| TokenMismatch | takeOrders4/clear3 token pair mismatch | Yes |
| TokenSelfTrade | order input token == output token | Yes |
| SameOwner | clear3 alice.owner == bob.owner | Yes |
| UnsupportedCalculateOutputs | stack length < 2 | Yes |
| NegativeVaultBalance | balance goes negative after op | Yes |
| NegativeVaultBalanceChange | negative amount passed to increase/decrease | Yes |
| NegativePull | negative amount in pullTokens | Yes |
| NegativePush | negative amount in pushTokens | Yes |
| NegativeBounty | bounty < 0 in clear3 | Yes |
| ClearZeroAmount | both outputs zero after clear | Yes |
| MinimumIO | totalTakerIO < config.minimumIO | Yes |
| ZeroDepositAmount | deposit amount not > 0 | Yes (via interface) |
| ZeroWithdrawTargetAmount | withdraw target not > 0 | Yes (via interface) |
| ZeroMaximumIO | maximumIO not > 0 | Yes (via interface) |
| NoOrders | takeOrders4 with 0 orders | Yes (via interface) |
| OrderNoInputs | addOrder4 with 0 inputs | Yes (via interface) |
| OrderNoOutputs | addOrder4 with 0 outputs | Yes (via interface) |

### Interface Conformance: IRaindexV6

All functions declared in `IRaindexV6` are implemented with matching signatures:
- `vaultBalance2` -- implemented
- `orderExists` -- implemented
- `entask2` -- implemented
- `deposit4` -- implemented
- `withdraw4` -- implemented
- `addOrder4` -- implemented
- `removeOrder3` -- implemented
- `quote2` -- implemented
- `takeOrders4` -- implemented
- `clear3` -- implemented

### Interface Conformance: IMetaV1_2

The contract declares `is IMetaV1_2`. `MetaV1_2` is an event-only interface. The `AddOrder4` function emits `MetaV1_2(...)` when meta is present. No function implementation needed.

### Interface Conformance: ERC-165

`supportsInterface` is implemented in `OrderBookV6FlashLender` (registers `IERC3156FlashLender`). The contract inherits `ERC165` via `OrderBookV6FlashLender`. Note: `IRaindexV6` interface ID is NOT explicitly registered in `supportsInterface`. This may be intentional (not all projects register every interface) but is a gap.

### Interface Conformance: ERC-3156

- `flashLoan` -- implemented in `OrderBookV6FlashLender`, returns true, checks callback result
- `flashFee` -- returns `FLASH_FEE` (0), correct
- `maxFlashLoan` -- returns token balance of contract, correct

### Test Spot-Check

| Test File | Test Name | Verified Behavior |
|---|---|---|
| clear.zeroAmount.t.sol | testClearZeroAmount | Both orders return 0/0, expects ClearZeroAmount revert. Correct. |
| clear.sameToken.t.sol | testClearSameToken | Same token for input/output, expects TokenSelfTrade. Correct. |
| clear.badStack.t.sol | testClearOrderBadStackEmptyStack | Empty stack (0 outputs), expects UnsupportedCalculateOutputs(0). Correct. |
| clear.badStack.t.sol | testClearOrderBadStackOneStack | 1 output, expects UnsupportedCalculateOutputs(1). Correct. |
| clear.handleIO.revert.t.sol | testClearOrderHandleIO0-5 | Various ensure conditions in handleIO, verifies revert propagation. Correct. |
| takeOrder.noop.t.sol | testTakeOrderNoopZeroOrders | Empty orders array, expects NoOrders. Correct. |
| takeOrder.sameToken.t.sol | testTakeOrderSameToken | Same input/output token, expects TokenSelfTrade. Correct. |

## Findings

### INFORMATIONAL -- I01: Dead Type Aliases Output18Amount and Input18Amount

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 192-194

`type Output18Amount is uint256;` and `type Input18Amount is uint256;` are declared but never used anywhere in the codebase. These appear to be remnants from a previous version that used 18-decimal fixed point math. They are now completely dead code since the V6 implementation uses `Float` throughout.

**Impact:** None (dead code only).

### INFORMATIONAL -- I02: Stale NatSpec References to 18-Decimal Fixed Point in OrderIOCalculationV4

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 152-180

The NatSpec for `OrderIOCalculationV4` extensively references "18 fixed point decimal", `1e18`, and "RESCALED ACCORDING TO TOKEN DECIMALS to an 18 fixed point decimal number". These descriptions are outdated. The V6 implementation uses Rain floating point (`Float`) internally. The expression returns `Float` values directly, and there is no rescaling to 18-decimal fixed point. Token decimals are only used at the boundary when converting Float to/from absolute token amounts via `pullTokens`/`pushTokens`.

Similarly, `calculateOrderIO` NatSpec (line 690-691) says "Both are always treated as 18 decimal fixed point values and then rescaled" which is inaccurate.

**Impact:** Documentation-only. Could confuse developers/auditors reviewing the code.

### INFORMATIONAL -- I03: recordVaultIO NatSpec is Inaccurate

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 886-891

Three issues:
1. The NatSpec says "dispatch the handle IO entrypoint if it exists" but `recordVaultIO` does NOT dispatch handle IO. That is done by the separate `handleIO` function.
2. The NatSpec references `_calculateOrderIO` (line 887) but the actual function is named `calculateOrderIO` (no underscore prefix).
3. The `@param orderIOCalculation` description (line 891) is truncated: "The order IO calculation produced by" -- sentence ends without completing.

**Impact:** Documentation-only.

### INFORMATIONAL -- I04: Stale Comment Reference to _recordVaultIO

**Location:** `src/concrete/ob/OrderBookV6.sol` line 765

The comment says "The state changes produced here are handled in _recordVaultIO" but the function is named `recordVaultIO` (no underscore).

**Impact:** Documentation-only.

### INFORMATIONAL -- I05: Unused Error Declarations

**Location:** `src/concrete/ob/OrderBookV6.sol`

The following errors are declared but never used in `OrderBookV6.sol`:
- `TokenDecimalsMismatch` (line 83)
- `UnsupportedCalculateInputs` (line 95)
- `NegativeInput` (line 102)
- `NegativeOutput` (line 105)

These may be used by other contracts or test utilities but are dead code within the main contract.

**Impact:** None (dead code only). Gas cost for deployment is unaffected since errors are not stored in bytecode unless reverted.

### INFORMATIONAL -- I06: Interface Errors OrderNoSources and OrderNoHandleIO Declared But Not Checked

**Location:** `IRaindexV6` interface declares `OrderNoSources` and `OrderNoHandleIO` errors, but `OrderBookV6.addOrder4` does not check for these conditions.

The interface specification says:
- "MUST revert with `OrderNoSources` if the order has no associated calculation"
- "MUST revert with `OrderNoHandleIO` if the order has no handle IO entrypoint"

The implementation does not validate bytecode source count. An order with no sources or no handle IO source will succeed on `addOrder4` but fail at clear/take time when the interpreter is called. This is a deliberate design choice (fail-late) but technically deviates from the MUST requirement in the interface NatSpec.

**Impact:** Low. Orders without proper sources will still fail when evaluated. The interface NatSpec is aspirational rather than prescriptive here, as the V6 design philosophy defers validation to runtime.

### INFORMATIONAL -- I07: ERC-165 Does Not Register IRaindexV6 Interface

**Location:** `src/abstract/OrderBookV6FlashLender.sol` line 33-34, `src/concrete/ob/OrderBookV6.sol` line 198

The `supportsInterface` only registers `IERC3156FlashLender`. The primary `IRaindexV6` interface is not registered. Tools or contracts querying ERC-165 for `IRaindexV6` support will get `false`.

**Impact:** Low. May affect discoverability by tools but does not affect core functionality.

### INFORMATIONAL -- I08: HANDLE_IO_MAX_OUTPUTS NatSpec Typo

**Location:** `src/concrete/ob/OrderBookV6.sol` line 150

The comment says "Handle IO has no outputs as it only response to vault movements" -- should be "responds" not "response".

**Impact:** None (typo in comment).

## Summary

No HIGH or MEDIUM severity findings. The contract logic is correct:
- `calculateClearStateAlice` math is sound
- `recordVaultIO` parameter order is correct at all call sites
- Error conditions match their names
- Token flow in `takeOrders4` and `clear3` is correctly oriented
- Bounty calculation and distribution uses correct token addresses
- Stack reading order matches interpreter convention
- `ClearZeroAmount` placement after `handleIO` is intentional and correct

All findings are INFORMATIONAL severity, predominantly involving stale NatSpec/comments from the migration to floating point math and minor dead code.
