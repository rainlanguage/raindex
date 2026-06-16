# A10 — LibOrder.sol — Pass 1 (Security)

## Evidence of Thorough Reading

**File:** `src/lib/LibOrder.sol` (19 lines)

**Library:** `LibOrder`

**Functions:**
- `hash(OrderV4 memory order)` — line 16 (internal pure)

**Types/Errors/Constants:** None defined in this file.

**Imports:**
- `OrderV4` from `rain.raindex.interface/interface/IRaindexV6.sol` — line 5

## Findings

No security findings identified.

### Analysis Notes

The `hash` function correctly uses `abi.encode` rather than `abi.encodePacked`, as noted in the NatSpec comment. This prevents collision attacks where different structs with variable-length fields could produce identical packed encodings. Since `OrderV4` contains dynamic arrays (`validInputs`, `validOutputs`) and a nested struct (`evaluable` with `bytes bytecode`), using `abi.encode` is the correct choice.

The function is `internal pure` and stateless, with no external call risks or state concerns.
