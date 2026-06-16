# Pass 3: Documentation -- A03 OrderBookV6FlashBorrower

**File:** `src/abstract/OrderBookV6FlashBorrower.sol`

## Evidence of Reading

**Errors:**
- `BadInitiator(address badInitiator)` (line 20)
- `BadLender(address badLender)` (line 25)
- `FlashLoanFailed()` (line 28)

**Contract:** `OrderBookV6FlashBorrower` (abstract, line 60)
- Inherits: `IERC3156FlashBorrower`, `ReentrancyGuard`, `ERC165`, `OrderBookV6ArbCommon`

**Functions:**
- `constructor(OrderBookV6ArbConfig memory config)` (line 63)
- `supportsInterface(bytes4 interfaceId) public view virtual override returns (bool)` (line 66)
- `_exchange(TakeOrdersConfigV5 memory takeOrders, bytes memory exchangeData) internal virtual` (line 79)
- `onFlashLoan(address initiator, address, uint256, uint256, bytes calldata data) external returns (bytes32)` (line 82)
- `arb4(IRaindexV6 orderBook, TakeOrdersConfigV5 calldata takeOrders, bytes calldata exchangeData, TaskV2 calldata task) external payable` (line 129)

## Findings

### A03-1: Constructor has no NatSpec documentation [LOW]

**Lines:** 63

The constructor at line 63 has no documentation. Same pattern as A02-1 -- it delegates to `OrderBookV6ArbCommon(config)` but provides no `@param` tag.

**Severity:** LOW

### A03-2: `onFlashLoan` parameter names are partially elided [LOW]

**Lines:** 82

The function signature at line 82 is:
```solidity
function onFlashLoan(address initiator, address, uint256, uint256, bytes calldata data) external returns (bytes32)
```

Three parameters are unnamed: the second (`token`), third (`amount`), and fourth (`fee`). The `@inheritdoc IERC3156FlashBorrower` tag provides interface-level docs, but the unnamed parameters make the implementation harder to read in isolation. The ERC-3156 interface documents these as `token`, `amount`, and `fee`. Since `token` and `amount` are conceptually important to understanding the flash loan flow (even if unused in the body), naming them would improve readability.

**Severity:** LOW

### A03-3: `_exchange` hook documentation does not use NatSpec tags properly [LOW]

**Lines:** 70-79

The `_exchange` function at line 79 has a plain comment block (lines 70-77) but does not use the `@notice` or `@dev` NatSpec tag prefix. The `@param` tags on lines 76-77 are present but the leading description text is a regular comment without `///` + NatSpec tag structure. Looking more carefully: the comments do use `///` (NatSpec-style) but the leading description lacks a `@notice` or `@dev` tag. NatSpec will treat the first untagged `///` block as the `@notice`, so this technically works, but it is inconsistent with the rest of the file which uses explicit tags.

Actually, re-reading: lines 70-77 all use `///` prefix, which means the untagged text becomes the implicit `@notice`. The `@param` tags are correctly placed. This is valid NatSpec. However, the documentation does not describe the return value (there is none -- it is void), which is correct.

**Severity:** LOW -- the missing explicit `@notice` or `@dev` tag is a minor style inconsistency.

### A03-4: `FlashLoanFailed` error documentation is minimal [INFO]

**Lines:** 27-28

The error `FlashLoanFailed()` at line 28 has a one-line doc: "Thrown when the flash loan fails somehow." The word "somehow" is vague. The only place this error is used is line 160, where it reverts if `orderBook.flashLoan(...)` returns `false`. The doc could be more precise: "Thrown when the orderbook's `flashLoan` call returns `false`."

**Severity:** INFO

### A03-5: Documentation completeness summary [INFO]

| Item | Has Doc | Doc Quality |
|------|---------|-------------|
| Contract-level `@title`/`@notice` | Yes (lines 30-59) | Excellent -- detailed example with numbers |
| `BadInitiator` error | Yes (lines 18-19) | Good, includes `@param` |
| `BadLender` error | Yes (lines 22-24) | Good, includes `@param` |
| `FlashLoanFailed` error | Yes (line 27) | Vague (A03-4) |
| `constructor` | No | **Missing** (A03-1) |
| `supportsInterface` | Yes (`@inheritdoc IERC165`) | Adequate via inheritance |
| `_exchange` | Yes (lines 70-78) | Good -- describes purpose, has `@param` tags |
| `onFlashLoan` | Yes (`@inheritdoc IERC3156FlashBorrower`) | Adequate via inheritance, unnamed params (A03-2) |
| `arb4` | Yes (lines 110-128) | Good -- describes flow, all `@param` tags present |

**Severity:** INFO
