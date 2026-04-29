# Pass 3: Documentation -- OrderBookV6ArbCommon.sol

**Agent:** A01
**File:** `src/abstract/OrderBookV6ArbCommon.sol`
**Date:** 2026-03-13

## Evidence of Thorough Reading

### Contract/Module Name
- `OrderBookV6ArbCommon` (abstract contract, line 34)

### Struct Definitions (file scope)
- `OrderBookV6ArbConfig` (lines 21-25): fields `orderBook` (address), `task` (TaskV2), `implementationData` (bytes)

### Error Definitions (file scope)
- `WrongTask()` (line 28)

### Constant Definitions (file scope)
- `BEFORE_ARB_SOURCE_INDEX` (line 32): `SourceIndexV2.wrap(0)`

### State Variables
- `iTaskHash` (line 39): `bytes32 public immutable`, initialized to `0`

### Events
- `Construct(address sender, OrderBookV6ArbConfig config)` (line 37)

### Functions/Modifiers (with line numbers)
- `constructor(OrderBookV6ArbConfig memory config)` (line 41)
- `modifier onlyValidTask(TaskV2 memory task)` (line 50)

### Imports (lines 5-14)
- `EvaluableV4`, `SignedContextV1` from `IInterpreterCallerV4.sol`
- `IInterpreterV4`, `SourceIndexV2`, `DEFAULT_STATE_NAMESPACE` from `IInterpreterV4.sol`
- `IRaindexV6`, `TaskV2` from `IRaindexV6.sol`
- `LibContext` from `LibContext.sol`
- `LibNamespace` from `LibNamespace.sol`
- `LibEvaluable` from `LibEvaluable.sol`
- `using LibEvaluable for EvaluableV4` (line 35)

## Documentation Inventory

### File-Level / License Headers
- SPDX license identifier: present (line 1)
- SPDX copyright: present (line 2)
- Pragma: `^0.8.19` (line 3)

### Struct: `OrderBookV6ArbConfig` (lines 16-25)
- **Top-level comment:** "Configuration for an arb contract to construct." (line 16) -- present and accurate.
- **`@param orderBook`:** "The `OrderBook` contract to arb against." (line 17) -- present and accurate.
- **`@param tasks`:** "The tasks to use as post for each arb." (line 18) -- **INACCURATE**: the param name in the doc says `tasks` (plural) but the actual struct field is `task` (singular, `TaskV2 task` on line 23). The description "as post for each arb" is also vague.
- **`@param implementationData`:** "The constructor data for the specific implementation of the arb contract." (lines 19-20) -- present and accurate.

### Error: `WrongTask()` (line 28)
- **Comment:** "Thrown when the task does not match the expected hash." (line 27) -- present and accurate.

### Constant: `BEFORE_ARB_SOURCE_INDEX` (line 32)
- **Comment:** `@dev` "Before arb" is evaluated before the flash loan is taken. Ostensibly allows for some kind of access control to the arb." (lines 30-31) -- present. Partially accurate. The description says "before the flash loan is taken" which is only true for the `OrderBookV6FlashBorrower` path. In `OrderBookV6ArbOrderTaker`, there is no flash loan; the before-arb logic runs before `takeOrders` is called directly. Also, this constant is defined but never used within this file -- it is redefined in `OrderBookV6ArbOrderTaker.sol` (line 29) with slightly different wording and consumed through `LibOrderBookArb`.

### Contract: `OrderBookV6ArbCommon` (line 34)
- **No NatDoc on the contract declaration.** There is no `@title`, `@notice`, or `@dev` comment on the `abstract contract OrderBookV6ArbCommon` line. For comparison, the inheriting `OrderBookV6FlashBorrower` has a multi-paragraph `@title`/`@notice` block (though it has its own doc accuracy issues -- see A01-P3-4).

### Event: `Construct` (line 37)
- **No documentation.** The event has no NatDoc comment describing its purpose or parameters.

### State Variable: `iTaskHash` (line 39)
- **No documentation.** The public immutable variable has no NatDoc or `@dev` comment explaining its purpose, its zero-default meaning, or how it is set.

### Constructor (lines 41-48)
- **No NatDoc.** The constructor has no `@param` or `@dev` documentation. There is one inline comment: "Emit events before any external calls are made." (line 42), which is accurate (though there are no external calls in this constructor, the comment likely establishes a pattern for inheriting contracts).

### Modifier: `onlyValidTask` (lines 50-55)
- **No documentation.** The modifier has no NatDoc comment describing its purpose, parameters, or revert conditions.

## Public/External Interface Documentation Audit

The contract exposes one public interface item via auto-generated getter:

| Item | Visibility | Documented? | Parameters Documented? | Return Documented? |
|------|-----------|-------------|----------------------|-------------------|
| `iTaskHash` (getter) | public | NO | N/A | NO |

The constructor and modifier are not external-facing in the traditional sense but are part of the contract's API for inheriting contracts:

| Item | Documented? | Parameters Documented? | Behavior Documented? |
|------|-------------|----------------------|---------------------|
| `constructor` | NO | NO (`config` param undocumented) | NO |
| `onlyValidTask` | NO | NO (`task` param undocumented) | NO (revert condition undocumented) |

## Documentation Accuracy Review

### Struct `@param tasks` vs actual field name `task`

The NatDoc on line 18 says `@param tasks` but the struct field on line 23 is `task` (singular `TaskV2 task`). This is a name mismatch between documentation and implementation. Solidity compilers do not currently enforce NatDoc param name matching for struct fields, so this compiles without warning but misleads readers.

### `BEFORE_ARB_SOURCE_INDEX` doc mentions "flash loan" only

The `@dev` comment says "evaluated before the flash loan is taken" but this constant is intended for use by both flash-loan-based and non-flash-loan-based arb contracts. The `OrderBookV6ArbOrderTaker` path has no flash loan. Furthermore, this exact constant is redefined in `OrderBookV6ArbOrderTaker.sol` line 29 with wording "evaluabled before the arb is executed" (note also the typo "evaluabled" in that file).

### `OrderBookV6FlashBorrower` title mismatch

While not in this file, the inheriting `OrderBookV6FlashBorrower.sol` (line 32) has `@title OrderBookV5FlashBorrower` -- a stale V5 reference. This is noted here because this pass reviews documentation accuracy for the `OrderBookV6ArbCommon` family.

## Findings

### A01-P3-1 [LOW] Struct `@param` Name Mismatch: `tasks` vs `task`

**Severity:** LOW

**Location:** `src/abstract/OrderBookV6ArbCommon.sol:18` (NatDoc) vs line 23 (struct field)

**Description:** The NatDoc comment for `OrderBookV6ArbConfig` documents `@param tasks` (plural) but the actual struct field is `task` (singular `TaskV2 task`). This mismatch between documentation and implementation could mislead readers into thinking the struct accepts multiple tasks. The prior version of this struct likely had a `TaskV2[] tasks` array field; the documentation was not updated when the field was changed to a single `TaskV2 task`.

**Impact:** Developers integrating with this contract may incorrectly believe they can pass multiple tasks, leading to compilation errors or incorrect configuration. As a documentation-only issue in an abstract contract primarily used by sophisticated arb bot operators, the practical impact is low but the inaccuracy is clear.

### A01-P3-2 [LOW] Missing Documentation on Contract, Event, State Variable, Constructor, and Modifier

**Severity:** LOW

**Location:** `src/abstract/OrderBookV6ArbCommon.sol:34` (contract), line 37 (event), line 39 (state variable), line 41 (constructor), line 50 (modifier)

**Description:** The following items have no NatDoc documentation:

1. **Contract `OrderBookV6ArbCommon`** (line 34): No `@title`, `@notice`, or `@dev`. Both inheriting contracts (`OrderBookV6FlashBorrower` and `OrderBookV6ArbOrderTaker`) have contract-level documentation, but the base abstract contract does not describe its purpose as the common base for arb contracts.

2. **Event `Construct`** (line 37): No documentation on what the event signals or when it is emitted. No `@param` for `sender` or `config`.

3. **State variable `iTaskHash`** (line 39): No documentation explaining that this holds the keccak256 hash of the configured task (or `bytes32(0)` if no task was configured), and that it is used by the `onlyValidTask` modifier for validation.

4. **Constructor** (line 41): No `@param config` documentation.

5. **Modifier `onlyValidTask`** (line 50): No documentation explaining the validation logic, the pass-through behavior when `iTaskHash == 0`, or the `WrongTask` revert condition.

**Impact:** This contract is abstract and meant to be inherited. Without documentation, developers building new arb contract variants must read the implementation to understand the contract's behavior. The modifier's dual behavior (pass-through when unconfigured, hash-check when configured) is particularly non-obvious without documentation.

### A01-P3-3 [LOW] `BEFORE_ARB_SOURCE_INDEX` Documentation Inaccurately Scoped to Flash Loans

**Severity:** LOW

**Location:** `src/abstract/OrderBookV6ArbCommon.sol:30-32`

**Description:** The `@dev` comment says: "Before arb is evaluated before the flash loan is taken." This describes only the `OrderBookV6FlashBorrower` usage path. The constant is equally intended for `OrderBookV6ArbOrderTaker`, which does not use flash loans. The documentation should describe the constant generically as being evaluated before the arb executes, regardless of mechanism.

Additionally, this constant is defined at file scope in `OrderBookV6ArbCommon.sol` but is never referenced within this file or by any contract that imports from it. It is independently redefined in `OrderBookV6ArbOrderTaker.sol` (line 29). The file-scope definition here is dead code with misleading documentation.

**Impact:** Misleading documentation could cause a developer to believe this constant is only relevant to flash-loan-based arb, or to modify it here expecting the change to propagate, when in fact the actual consumers define their own copies.

### A01-P3-4 [INFO] Stale V5 Reference in `OrderBookV6FlashBorrower` Title

**Severity:** INFO

**Location:** `src/abstract/OrderBookV6FlashBorrower.sol:32`

**Description:** The `@title` says `OrderBookV5FlashBorrower` but the actual contract name is `OrderBookV6FlashBorrower`. This is a stale reference from a prior version. While this is in a different file, it is noted here as part of the documentation accuracy review for the `OrderBookV6ArbCommon` inheritance tree.

**Impact:** No functional impact. Cosmetic documentation staleness.
