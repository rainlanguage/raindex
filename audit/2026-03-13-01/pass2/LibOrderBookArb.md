# Pass 2: Test Coverage — LibOrderBookArb.sol
**Agent:** A12
**File:** src/lib/LibOrderBookArb.sol

## Evidence of Thorough Reading

**Library:** `LibOrderBookArb` (line 20)

### Errors / Types / Constants
| Item | Kind | Line |
|------|------|------|
| `NonZeroBeforeArbStack()` | error | 14 |
| `BadLender(address)` | error | 18 |

### Functions
| Function | Visibility | Line |
|----------|-----------|------|
| `finalizeArb(TaskV2, address, uint8, address, uint8)` | internal | 23 |

### Imports Used
- `TaskV2` from `rain.raindex.interface` (line 5)
- `IERC20` from OpenZeppelin (line 6)
- `LibOrderBook` (line 7)
- `Address` from OpenZeppelin (line 8)
- `SafeERC20` from OpenZeppelin (line 9)
- `IERC20Metadata` from OpenZeppelin (line 10) -- **imported but unused**
- `LibDecimalFloat`, `Float` from `rain.math.float` (line 11)

## Function Analysis: `finalizeArb`

`finalizeArb` performs three sequential operations:

1. **Input token sweep** (lines 33-42): Reads `balanceOf(address(this))` for `ordersInputToken`, transfers any non-zero balance to `msg.sender` via `safeTransfer`, converts balance to `Float` via `fromFixedDecimalLossyPacked` using `inputDecimals`, stores in `col[0]`.

2. **Output token sweep** (lines 44-54): Same pattern for `ordersOutputToken` with `outputDecimals`, stores in `col[1]`.

3. **Native gas sweep** (lines 56-69): Reads `address(this).balance`, sends entire balance to `msg.sender` via `Address.sendValue`, converts to `Float` via `packLossless` with hardcoded `-18` exponent, stores in `col[2]`.

4. **Post-task dispatch** (lines 71-75): Wraps the 3-element column into a context matrix and calls `LibOrderBook.doPost(context, post)` with the task.

## Test Inventory

### Direct Tests
**None.** There is no test file for `LibOrderBookArb` itself. A grep for `LibOrderBookArb`, `finalizeArb`, `NonZeroBeforeArbStack`, and `BadLender` across the entire `test/` directory returns zero results.

### Indirect Coverage via Consumer Contracts

| Test File | What it exercises |
|-----------|-------------------|
| `test/abstract/OrderBookV6ArbOrderTaker.context.t.sol` | Calls `arb5` which calls `finalizeArb`. Mocks `balanceOf` to return `3e12` (input) and `4e12` (output), sets `vm.deal` to `5e18`. Validates context column values via a Rain expression in the task. |
| `test/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sender.t.sol` | Fuzz test of `arb5` happy path (mock tokens, zero balances). Exercises `finalizeArb` with default-zero token balances and no gas balance. |
| `test/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sender.t.sol` | Fuzz test of `arb4` happy path. Exercises `finalizeArb` after flash loan with zero balances. |
| `test/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sender.t.sol` | Same as GenericPool but for RouteProcessor variant. |
| `test/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol` | Tests `arb5` with expression eval, exercises `finalizeArb` indirectly. |
| `test/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.expression.t.sol` | Same as above for RouteProcessor variant. |

### Coverage Assessment for Each Code Path

| Code Path | Covered? | Notes |
|-----------|----------|-------|
| Input token balance > 0, `safeTransfer` to `msg.sender` | Partial | Context test mocks `balanceOf` to non-zero and mocks `transfer` to true. Transfer destination/amount are NOT asserted. |
| Input token balance == 0, skip transfer | Yes (implicit) | Most fuzz tests use mock tokens with zero balance. |
| Output token balance > 0, `safeTransfer` to `msg.sender` | Partial | Same as input: mocked but transfer args unverified. |
| Output token balance == 0, skip transfer | Yes (implicit) | Same as above. |
| Native gas balance > 0, `Address.sendValue` | Partial | Context test sets `vm.deal(arb, 5e18)`. But no assertion on the actual ETH transfer to `msg.sender`. |
| Native gas balance == 0, `Address.sendValue(0)` | Yes (implicit) | Fuzz tests with no `vm.deal`. |
| `fromFixedDecimalLossyPacked` for various decimals | Minimal | Context test uses 12-decimal tokens. No test with 0, 6, 8, or 18 decimals. |
| `packLossless` for gas balance | Minimal | Only tested with `5e18`. No edge-case values. |
| `doPost` called with correct context shape | Partial | Context test verifies values via expression, but does not verify the context matrix dimensions. |

## Findings

### A12-P2-1 [LOW] `NonZeroBeforeArbStack` and `BadLender` errors are dead code with zero test coverage

**Location:** Lines 14, 18

These two errors are defined in `LibOrderBookArb.sol` but are never used anywhere in the library's code. They are imported by `OrderBookV6ArbOrderTaker.sol` (line 20) and `OrderBookV6FlashBorrower.sol` (line 18), but are never referenced in those contracts' code either -- they appear only in `import` statements. No test references either error.

This is dead code. The errors either belonged to an earlier version of the arb logic that has since been refactored, or they were intended for use that was never implemented.

**Impact:** Dead error declarations create confusion for auditors and developers who may expect corresponding revert conditions to exist. If these errors were meant to guard against real attack vectors (e.g., an unauthorized flash loan lender calling `onFlashLoan`), the missing guards represent a security gap. However, `onFlashLoan` in `OrderBookV6FlashBorrower` already uses `BadInitiator` (a separate error declared in that file), so `BadLender` appears to be a leftover.

### A12-P2-2 [LOW] No test verifies that token transfers in `finalizeArb` go to the correct recipient with correct amounts

**Location:** Lines 36-37, 47-48

The context test (`OrderBookV6ArbOrderTaker.context.t.sol`) mocks `IERC20.balanceOf` and `IERC20.transfer` but does not use `vm.expectCall` to verify that `safeTransfer` is called with `msg.sender` as the recipient and the full balance as the amount. The mock accepts any `transfer` call and returns `true`.

A bug that transferred tokens to `address(0)`, to `address(this)`, or transferred a partial amount would pass all existing tests.

**Impact:** The core economic purpose of `finalizeArb` -- sweeping profit to the arb caller -- has no assertion on the actual transfer parameters.

### A12-P2-3 [LOW] No test verifies native gas (ETH) is actually sent to `msg.sender`

**Location:** Lines 63-64

The context test sets `vm.deal(address(arbOrderTaker), 5e18)` and verifies that the gas balance appears in the context column. But there is no assertion that `msg.sender`'s ETH balance increased by `5e18` after the call, or that the arb contract's balance is zero afterward. A bug in the `Address.sendValue` call (e.g., sending to `address(this)` instead of `msg.sender`) would not be caught.

**Impact:** The ETH sweep is a critical part of arb profit extraction. Without balance assertions, a loss-of-funds bug could go undetected.

### A12-P2-4 [LOW] No test exercises `finalizeArb` with realistic non-zero balances of both tokens simultaneously

**Location:** Lines 33-54

The context test sets non-zero balances for both input and output tokens, but uses mocked (non-real) ERC20 tokens. The fuzz tests in `GenericPool*` and `RouteProcessor*` use real `Token` contracts but the mock orderbook's `takeOrders4` is a no-op that never produces token balances. No test creates a scenario where `finalizeArb` actually has real ERC20 balances to sweep.

**Impact:** The interaction between `balanceOf`, `safeTransfer`, and the post-task context values is never tested with real token contracts. Subtle issues like token transfer fees, re-entrancy through ERC777-like tokens, or `safeTransfer` revert behavior on non-standard tokens are completely unexercised.

### A12-P2-5 [INFO] `fromFixedDecimalLossyPacked` lossy conversion is explicitly silenced

**Location:** Lines 39-40, 51-52

The `lossless` return value from `fromFixedDecimalLossyPacked` is captured but discarded: `(lossless);`. This means the context column values passed to the post-task expression may be imprecise representations of the actual token balances. There is no test that verifies lossless conversion for common decimal values (6, 8, 18) or that documents the expected precision loss for extreme values.

### A12-P2-6 [INFO] `IERC20Metadata` import is unused

**Location:** Line 10

`IERC20Metadata` is imported but never used in the library. This was also noted in Pass 1 (A11-1). No test impact, but it adds a spurious dependency.

## Summary

| ID | Severity | Title |
|----|----------|-------|
| A12-P2-1 | LOW | `NonZeroBeforeArbStack` and `BadLender` are dead code with zero coverage |
| A12-P2-2 | LOW | No test verifies token transfer recipient and amount in `finalizeArb` |
| A12-P2-3 | LOW | No test verifies native gas is sent to `msg.sender` |
| A12-P2-4 | LOW | No test exercises `finalizeArb` with real non-zero token balances |
| A12-P2-5 | INFO | Lossy float conversion is silently discarded, untested for precision |
| A12-P2-6 | INFO | Unused `IERC20Metadata` import (confirms A11-1) |

The library has zero direct test coverage. All coverage is indirect through consumer contracts (`OrderBookV6ArbOrderTaker` and `OrderBookV6FlashBorrower`), and the existing tests rely heavily on mocked ERC20 behavior. The mocks verify that `finalizeArb` produces correct context column values for the post-task, but do not verify the actual token/ETH transfers that constitute the library's core purpose.
