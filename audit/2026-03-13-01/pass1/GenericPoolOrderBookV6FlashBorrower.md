# Pass 1 Audit: GenericPoolOrderBookV6FlashBorrower.sol

**Agent:** A06
**File:** `src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol`
**Date:** 2026-03-13

## Evidence of Thorough Reading

### Contract Name
`GenericPoolOrderBookV6FlashBorrower` (inherits `OrderBookV6FlashBorrower`)

### Functions and Line Numbers
| Function | Line |
|---|---|
| `constructor(OrderBookV6ArbConfig memory config)` | 31 |
| `_exchange(TakeOrdersConfigV5 memory takeOrders, bytes memory exchangeData) internal virtual override` | 34 |
| `fallback() external` | 48 |

### Types, Errors, and Constants
- No types, errors, or constants defined in this file. All are inherited from parent contracts.

### Imports (lines 5-15)
- `IERC3156FlashLender` (line 5)
- `IERC3156FlashBorrower` (line 6)
- `OrderBookV6FlashBorrower`, `SafeERC20`, `IERC20`, `Address`, `TakeOrdersConfigV5`, `OrderBookV6ArbConfig` (lines 8-15)

### Parent Contract Context Reviewed
- `OrderBookV6FlashBorrower` (`src/abstract/OrderBookV6FlashBorrower.sol`): provides `arb4`, `onFlashLoan`, `_exchange` (virtual hook), `supportsInterface`. Uses `ReentrancyGuard`.
- `OrderBookV6ArbCommon` (`src/abstract/OrderBookV6ArbCommon.sol`): provides `iTaskHash`, `onlyValidTask` modifier, `Construct` event.
- `LibOrderBookArb` (`src/lib/LibOrderBookArb.sol`): provides `finalizeArb` which sends remaining tokens and gas to `msg.sender`.

---

## Findings

### A06-1 [MEDIUM] — Unlimited approval to arbitrary spender with no validation

**Location:** `src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol`, line 40

**Description:**
In `_exchange`, the contract approves `type(uint256).max` tokens to an arbitrary `spender` address decoded from `exchangeData`:

```solidity
IERC20(borrowedToken).forceApprove(spender, type(uint256).max);
```

The `spender` is entirely caller-controlled (decoded from `exchangeData` at line 35-36). While the approval is reset to 0 on line 44, if the `pool.functionCallWithValue` call on line 41 reverts, the approval is never cleared. More importantly, if the `pool` call re-enters (the pool is also arbitrary), it could exploit the standing max approval before it is cleared.

However, mitigating factors exist:
1. The parent `arb4` function has `nonReentrant` guard, but this only blocks re-entry into `arb4` itself, not into `_exchange` via `onFlashLoan` which is already inside the guarded section.
2. The `onFlashLoan` checks `initiator == address(this)`, so the flash loan callback path is protected.
3. After the entire arb completes, `finalizeArb` sweeps all remaining tokens to `msg.sender`, so there shouldn't be residual tokens for a future approval exploit.

The risk is that during the `pool.functionCallWithValue` call, the arbitrary pool contract has `type(uint256).max` approval and could drain any pre-existing token balance the contract holds from a prior step or concurrent state. Since the contract is designed to hold zero balance between arbs, this is primarily a theoretical risk during the arb execution window.

**Impact:** An attacker who controls or interacts with a malicious pool address could exploit the max approval window. However, since the `arb` caller chooses the pool and spender, this requires the arb caller to self-attack (or be tricked via a frontrunning attack that substitutes exchangeData, which is not possible since data is already committed in the tx).

### A06-2 [LOW] — fallback function accepts arbitrary calls without payable

**Location:** `src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol`, line 48

**Description:**
The contract defines a `fallback()` function:

```solidity
fallback() external {}
```

The comment says "Allow receiving gas" but the `fallback` is not `payable`. In Solidity 0.8.25, a non-payable `fallback` will revert when called with `msg.value > 0`. To receive ETH, the contract would need either `fallback() external payable` or a `receive() external payable` function.

This means the contract cannot actually receive plain ETH transfers via the fallback, contradicting the stated intent. The `arb4` function is `payable` and can receive ETH, and `pool.functionCallWithValue(encodedFunctionCall, address(this).balance)` forwards any ETH balance to the pool. If the pool needs to return ETH to this contract (e.g., as change), it cannot do so via a plain transfer.

However, ETH can still arrive at the contract via `selfdestruct` from another contract, or as block rewards / coinbase. The practical impact is that any pool that tries to send ETH back to this contract via a plain transfer will have that transfer revert.

### A06-3 [INFO] — Unused import IERC3156FlashLender

**Location:** `src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol`, line 5

**Description:**
`IERC3156FlashLender` is imported but never used in this contract. The contract implements `IERC3156FlashBorrower` (via inheritance), not the lender interface. This is a code hygiene issue with no security impact.
