# Pass 3: Documentation -- A12 LibOrderBookArb

**File:** `src/lib/LibOrderBookArb.sol`

## Evidence of Reading

- **Library:** `LibOrderBookArb` (lines 14-76)
- **Imports:** `TaskV2` (line 5), `IERC20` (line 6), `LibOrderBook` (line 7), `Address` (line 8), `SafeERC20` (line 9), `IERC20Metadata` (line 10), `LibDecimalFloat`, `Float` (line 11)
- **Using directive:** `SafeERC20 for IERC20` (line 15)

### Public/Internal Functions

| Function | Visibility | Line | Has NatSpec |
|----------|-----------|------|-------------|
| `finalizeArb(TaskV2 memory task, address ordersInputToken, uint8 inputDecimals, address ordersOutputToken, uint8 outputDecimals)` | `internal` | 20 | Partial |

### Types / Errors / Constants

None declared in this file.

### Library-level Documentation

- `@title LibOrderBookArb` (line 13) -- present but has no `@notice` or `@dev` describing the library's purpose.

### Function-level Documentation

**`finalizeArb`** (lines 17-75):
- `@dev` (lines 17-19): "Sends all remaining token balances and native gas to `msg.sender`, then evaluates the post-arb task with a context column containing the amounts sent as Floats."
- No `@param` tags for any of the 5 parameters.
- No `@return` tag (function returns nothing, so this is correct).
- Inline comments document each step: sending input tokens (line 31), output tokens (line 42), native gas (lines 54-59), and context assembly (line 70-74).

### Unused Import

- `IERC20Metadata` is imported (line 10) but never used in this file.

## Findings

### A12-1: `finalizeArb` missing `@param` tags (INFO)

**Location:** Lines 17-26

The function has 5 parameters but no `@param` NatSpec tags:

- `task` -- the post-arb task to evaluate after sending funds
- `ordersInputToken` -- address of the input token (from the orders' perspective)
- `inputDecimals` -- decimals of the input token for Float conversion
- `ordersOutputToken` -- address of the output token (from the orders' perspective)
- `outputDecimals` -- decimals of the output token for Float conversion

The `@dev` comment provides a high-level description but does not describe individual parameters. Adding `@param` tags would clarify the role of each argument, especially the meaning of "input" vs "output" from the orders' perspective.

### A12-2: Library-level `@notice` or `@dev` missing (INFO)

**Location:** Line 13

The `@title` tag is present but there is no `@notice` or `@dev` tag describing the library's purpose. For consistency with `LibOrder` (which has both `@title` and `@notice`), a brief description should be added. For example: "Handles finalization of arbitrage operations by sweeping remaining balances and executing post-arb tasks."

### A12-3: Unused import `IERC20Metadata` (INFO)

**Location:** Line 10

`IERC20Metadata` is imported but never referenced in the library. This is a code hygiene issue rather than a documentation issue, but it could mislead readers into thinking the library uses metadata functionality.
