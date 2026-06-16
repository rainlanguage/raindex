# Pass 3: Documentation -- A01 OrderBookV6ArbCommon

**File:** `src/abstract/OrderBookV6ArbCommon.sol`

## Evidence of Reading

**Structs:**
- `OrderBookV6ArbConfig` (line 13): fields `task`, `implementationData`

**Errors:**
- `WrongTask()` (line 19)

**Constants:**
- `BEFORE_ARB_SOURCE_INDEX` (line 23): `SourceIndexV2.wrap(0)`

**Contract:** `OrderBookV6ArbCommon` (abstract, line 29)

**State variables:**
- `iTaskHash` (line 39): `bytes32 public immutable`, default `0`

**Functions/modifiers:**
- `constructor(OrderBookV6ArbConfig memory config)` (line 42)
- `modifier onlyValidTask(TaskV2 memory task)` (line 54)

**Events:**
- `Construct(address sender, OrderBookV6ArbConfig config)` (line 35)

## Findings

### A01-1: Unused constant `BEFORE_ARB_SOURCE_INDEX` is documented but never referenced [INFO]

**Lines:** 21-23

The constant `BEFORE_ARB_SOURCE_INDEX` at line 23 has a `@dev` doc comment explaining it is "evaluated before the arb is executed" for "access control to the arb." However, this constant is never referenced anywhere within `OrderBookV6ArbCommon.sol` or its direct inheritors (`OrderBookV6ArbOrderTaker.sol`, `OrderBookV6FlashBorrower.sol`). The doc comment describes a purpose that is not implemented in this codebase. This is a documentation accuracy concern: the comment implies runtime behavior that does not exist in these contracts.

**Severity:** INFO

No fix needed -- the constant may be consumed by external tooling or downstream contracts outside this repository.

### A01-2: `OrderBookV6ArbConfig.implementationData` field lacks usage documentation [INFO]

**Lines:** 11-12

The `@param implementationData` doc says "The constructor data for the specific implementation of the arb contract" but the field is never read in `OrderBookV6ArbCommon`'s constructor (only `config.task` is used). The struct doc does not explain that `implementationData` is consumed by inheriting constructors, which could confuse readers.

**Severity:** INFO

No fix proposed -- the comment is technically accurate (it is "for the specific implementation"), just could be clearer.

### A01-3: All public/external items are documented [INFO]

Every public-facing declaration has NatSpec documentation:

| Item | Has Doc | Doc Quality |
|------|---------|-------------|
| `OrderBookV6ArbConfig` struct | Yes (`@param` for both fields) | Adequate |
| `WrongTask` error | Yes (line 18) | Adequate |
| `BEFORE_ARB_SOURCE_INDEX` constant | Yes (`@dev`, line 21-22) | See A01-1 |
| `Construct` event | Yes (`@notice`, `@param` x2, lines 32-34) | Good |
| `iTaskHash` state variable | Yes (`@notice`, line 37-38) | Good |
| `constructor` | Yes (`@param`, line 41) | Minimal but adequate |
| `onlyValidTask` modifier | Yes (`@notice`, lines 51-53) | Good -- describes revert condition and pass-through |

No undocumented public functions or types.

**Severity:** INFO
