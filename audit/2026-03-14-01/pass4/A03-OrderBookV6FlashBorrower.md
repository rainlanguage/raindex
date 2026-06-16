# A03 - Pass 4 (Code Quality) - OrderBookV6FlashBorrower.sol

**File:** `src/abstract/OrderBookV6FlashBorrower.sol`

## Evidence: Full Inventory

- **Contract:** `OrderBookV6FlashBorrower` (abstract, line 60)
  - Inherits: `IERC3156FlashBorrower`, `ReentrancyGuard`, `ERC165`, `OrderBookV6ArbCommon`
- **Errors:**
  - `BadInitiator(address)` (line 20)
  - `BadLender(address)` (line 25)
  - `FlashLoanFailed()` (line 28)
- **Constructor:** line 63
- **Functions:**
  - `supportsInterface(bytes4)` (line 66) -- public view virtual override
  - `_exchange(TakeOrdersConfigV5, bytes)` (line 79) -- internal virtual, no-op
  - `onFlashLoan(address, address, uint256, uint256, bytes)` (line 82) -- external
  - `arb4(IRaindexV6, TakeOrdersConfigV5, bytes, TaskV2)` (line 129) -- external payable, nonReentrant, onlyValidTask
- **Using directive:** `using SafeERC20 for IERC20` (line 61)
- **Imports:**
  - `ERC165`, `IERC165` from `openzeppelin-contracts/.../ERC165.sol` (line 5)
  - `SafeERC20` from `openzeppelin-contracts/.../SafeERC20.sol` (line 6)
  - `IERC20` from `openzeppelin-contracts/.../IERC20.sol` (line 7)
  - `ReentrancyGuard` from `openzeppelin-contracts/.../ReentrancyGuard.sol` (line 8)
  - `ON_FLASH_LOAN_CALLBACK_SUCCESS` from `rain.raindex.interface/.../IERC3156FlashBorrower.sol` (line 9)
  - `IRaindexV6`, `TakeOrdersConfigV5`, `TaskV2` from `rain.raindex.interface/.../IRaindexV6.sol` (line 10)
  - `IERC3156FlashBorrower` from `rain.raindex.interface/.../IERC3156FlashBorrower.sol` (line 11)
  - `OrderBookV6ArbConfig`, `OrderBookV6ArbCommon` from `./OrderBookV6ArbCommon.sol` (line 12)
  - `LibOrderBookArb` from `../lib/LibOrderBookArb.sol` (line 13)
  - `LibOrderBookDeploy` from `../lib/deploy/LibOrderBookDeploy.sol` (line 14)
  - `LibTOFUTokenDecimals` from `rain.tofu.erc20-decimals/.../LibTOFUTokenDecimals.sol` (line 15)
  - `LibDecimalFloat` from `rain.math.float/.../LibDecimalFloat.sol` (line 16)

## Findings

### A03-1: Production code transitively imports forge-std via `LibOrderBookDeploy` (LOW)

**Line 14:** `OrderBookV6FlashBorrower` imports `LibOrderBookDeploy` solely to access the `ORDERBOOK_DEPLOYED_ADDRESS` constant (used at line 84 for the `BadLender` check). However, `LibOrderBookDeploy` (`src/lib/deploy/LibOrderBookDeploy.sol`) imports `{Vm} from "forge-std/Vm.sol"` (a test-only dependency) at its own line 5.

This means production source code (`src/`) has a transitive dependency on `forge-std`, which is a test framework. While the Solidity compiler will likely not include the `Vm` interface in deployed bytecode (it's only used as a parameter type in the `etchOrderBook` function which is never called from production paths), this is a leaky abstraction: test infrastructure bleeds into the production dependency graph.

The `ORDERBOOK_DEPLOYED_ADDRESS` constant should be either:
1. Defined directly in `OrderBookV6FlashBorrower.sol` or a minimal constants-only file that does not import `forge-std`, or
2. The `LibOrderBookDeploy` library should be split so constants live separately from the `etchOrderBook` test helper.

### A03-2: Unused `IERC165` import (INFO)

**Line 5:** `IERC165` is imported alongside `ERC165` but is never referenced directly. Same pattern as in `OrderBookV6ArbOrderTaker.sol`. Harmless but unnecessary.

### A03-3: NatSpec references outdated contract name `OrderBookFlashBorrower` (INFO)

**Line 44:** The contract-level NatSpec comment says:
```
/// The `OrderBookFlashBorrower` can:
```
The actual contract name is `OrderBookV6FlashBorrower`. This is a minor documentation drift.
