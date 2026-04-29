# A12 - LibOrderBookArb.sol - Pass 4 (Code Quality)

**File:** `src/lib/LibOrderBookArb.sol`
**Lines:** 77

## Evidence Inventory

### Library
- `LibOrderBookArb` (library) - line 14

### Functions
- `finalizeArb(TaskV2 memory task, address ordersInputToken, uint8 inputDecimals, address ordersOutputToken, uint8 outputDecimals)` (internal) - line 20

### Imports (lines 5-11)
- `TaskV2` from `rain.raindex.interface/interface/IRaindexV6.sol` - line 5
- `IERC20` from `openzeppelin-contracts/contracts/token/ERC20/IERC20.sol` - line 6
- `LibOrderBook` from `./LibOrderBook.sol` - line 7
- `Address` from `openzeppelin-contracts/contracts/utils/Address.sol` - line 8
- `SafeERC20` from `openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol` - line 9
- **`IERC20Metadata`** from `openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol` - line 10
- `LibDecimalFloat`, `Float` from `rain.math.float/lib/LibDecimalFloat.sol` - line 11

### Using-for
- `SafeERC20 for IERC20` - line 15

### Constants / Types / Errors
- (none)

## Findings

### A12-1: Unused Import `IERC20Metadata` [LOW]

Line 10 imports `IERC20Metadata` from OpenZeppelin, but it is never referenced in the file body. The `finalizeArb` function receives token decimals as pre-resolved `uint8` parameters rather than calling `IERC20Metadata.decimals()` on-chain. This is dead code that will generate a compiler warning in Solidity 0.8.25+ (the version used by the concrete contracts that compile this library).

**Recommendation:** Remove the unused import on line 10.

---

**No other findings.** No commented-out code (the `//slither-disable` and `// forge-lint:` annotations are legitimate suppression directives, not commented-out code). No bare `src/` imports. The pragma `^0.8.19` matches sibling library files. The internal structure is clean: scoped blocks keep local variables tight, and the post-task delegation to `LibOrderBook.doPost` is a well-chosen abstraction boundary.
