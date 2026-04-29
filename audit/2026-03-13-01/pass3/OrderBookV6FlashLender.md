# Pass 3: Documentation -- OrderBookV6FlashLender.sol

**Agent:** A04
**File:** `src/abstract/OrderBookV6FlashLender.sol`

## Evidence of Thorough Reading

- **Contract:** `OrderBookV6FlashLender` (abstract, line 29), inherits `IERC3156FlashLender`, `ERC165`
- **Imports:** `ERC165`, `IERC165` (OZ), `IERC20` (OZ), `SafeERC20` (OZ), `IERC3156FlashBorrower`, `ON_FLASH_LOAN_CALLBACK_SUCCESS`, `IERC3156FlashLender`
- **Error:** `FlashLenderCallbackFailed(bytes32 result)` (line 18)
- **Constant:** `FLASH_FEE = 0` (line 23, file-level)
- **Functions:**
  - `supportsInterface(bytes4)` -- public view virtual override, line 33
  - `flashLoan(IERC3156FlashBorrower, address, uint256, bytes)` -- external override, line 38
  - `flashFee(address, uint256)` -- external pure override, line 70
  - `maxFlashLoan(address)` -- external view override, line 78

## Documentation Coverage

### File-level

| Item | Has NatSpec | Quality |
|------|-------------|---------|
| Error `FlashLenderCallbackFailed` | Yes (lines 15-17) | Good. Describes when thrown and documents the `result` parameter. |
| Constant `FLASH_FEE` | Yes (lines 20-22) | Good. `@dev` explains rationale for zero fee. |
| Contract `OrderBookV6FlashLender` | Yes (lines 25-28) | Good. `@title` and `@notice` identify purpose and reference the EIP-3156 spec. |

### Function-level

| Function | Has NatSpec | Notes |
|----------|-------------|-------|
| `supportsInterface` (line 33) | `@inheritdoc IERC165` | Adequate -- inherited doc is sufficient for a standard override. |
| `flashLoan` (line 38) | `@inheritdoc IERC3156FlashLender` | Adequate -- inherited doc covers params and return. Inline comment (lines 50-62) adds valuable security rationale for the `safeTransferFrom` pattern and Slither suppression. |
| `flashFee` (line 70) | `@inheritdoc IERC3156FlashLender` | Acceptable, but see A04-P3-2 below regarding unnamed parameters. |
| `maxFlashLoan` (line 78) | Custom NatSpec (lines 74-76) + `@inheritdoc` (line 77) | Inaccurate -- see A04-P3-1 below. |

## Findings

### A04-P3-1 [LOW] Misleading NatSpec on `maxFlashLoan` -- claims active-debt guard that does not exist

**Location:** Lines 74-76

The NatSpec states:

> If there is an active debt then loans are disabled so the max becomes `0` until after repayment.

The actual implementation (line 79) simply returns `IERC20(token).balanceOf(address(this))`. There is no active-debt tracking, no boolean flag, and no logic that returns `0` during an outstanding loan. A `grep` for "active.*debt", "flashLoanActive", and related patterns across the entire `src/` tree returns zero matches beyond this comment itself.

This is the same finding as A04-1 from Pass 1, now confirmed from a documentation-accuracy perspective. The comment is not merely aspirational -- it describes behaviour that a reader would reasonably expect the code to enforce but that does not exist at all. This could mislead integrators into believing re-entrant flash loans are impossible when they are not.

**Recommendation:** Remove the sentence about active debt entirely. The accurate description is just the first sentence: there is no limit other than the current token balance held by the contract. Alternatively, if active-debt tracking is desired functionality, implement it.

### A04-P3-2 [INFO] `flashFee` drops parameter names, reducing documentation linkage

**Location:** Line 70

```solidity
function flashFee(address, uint256) external pure override returns (uint256) {
```

The interface (`IERC3156FlashLender`) documents `@param token` and `@param amount`. The implementation drops both parameter names. While valid Solidity (the parameters are unused and the fee is constant), the unnamed parameters make it harder for readers to correlate `@inheritdoc` documentation with the actual signature. This is purely informational since `@inheritdoc` still renders correctly in tooling.

### A04-P3-3 [INFO] No `@param` / `@return` on the custom `maxFlashLoan` comment

**Location:** Lines 74-77

The custom NatSpec block for `maxFlashLoan` does not include `@param token` or `@return` tags. The `@inheritdoc` on line 77 pulls these from the interface, so they are technically present, but the custom comment block preceding `@inheritdoc` adds prose without structured parameter documentation. This is standard practice and not a defect, but noting for completeness.

## Summary

| ID | Severity | Title |
|----|----------|-------|
| A04-P3-1 | LOW | Misleading NatSpec on `maxFlashLoan` claims active-debt guard that does not exist |
| A04-P3-2 | INFO | `flashFee` drops parameter names, reducing documentation linkage |
| A04-P3-3 | INFO | No `@param`/`@return` on custom `maxFlashLoan` comment |

One LOW finding (A04-P3-1) requires a fix file. The two INFO findings are cosmetic and do not warrant fix files.
