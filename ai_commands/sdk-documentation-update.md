Title: Sync SDK Documentation with Current JS API and Package

You are an engineering assistant. Your task is to verify and update the SDK documentation so it matches the current codebase. The primary source of truth is the Rust JS API in `crates/js_api` (inline docs on items exported via the `wasm_export` macro). You must also validate and update the additional package-level docs in `packages/orderbook` (e.g., README and any in‑package docs) to stay consistent with the built TypeScript surface.

Goal
- Ensure every JS-facing API exported from `crates/js_api` is documented accurately: names, parameters, parameter descriptions, return types, return descriptions, error semantics, and examples.
- Ensure package documentation in `packages/orderbook` (e.g., README) reflects the current API surface and real usage patterns.
- Avoid changing behavior or public API; this task is documentation-only. If you find code/doc conflicts that cannot be resolved with doc edits alone, open/leave a clear TODO note in your summary.

Scope
- Rust JS API: `crates/js_api/**` — functions, classes, and types reachable from JS via `#[wasm_export]`, `#[wasm_bindgen]`, and `Tsify`.
- Package docs: `packages/orderbook/README.md` and any additional docs in that package. Validate against built declarations `packages/orderbook/{cjs.d.ts,esm.d.ts}` when helpful.
- Cross-check TS usage in `packages/orderbook/test/**/*` to keep examples canonical.

Constraints and Repo Conventions
- Always use a Nix shell for build/test commands: prefix commands with `nix develop -c` (or use appropriate shell attributes).
- Do not fetch network resources. Keep changes scoped to docs and docstrings.
- Prefer syntax-aware search with ast-grep for structured matching:
  - Rust: `sg --lang rust -p '<pattern>'`
  - TypeScript: `sg --lang ts -p '<pattern>'`
  - Use plain file reads only when structural matching is unnecessary (e.g., opening a README).
- Follow AGENTS.md for tone and style. Keep edits concise and consistent with existing documentation voice.

High-Level Process
1) Build and list the JS-facing API.
2) Compare inline Rust docs and export metadata to the built TypeScript surface.
3) Audit package README and examples against canonical usage from tests.
4) Apply focused documentation edits in Rust doc comments and package docs.
5) Validate with workspace builds and TS checks. Summarize changes and any gaps.

Procedure
1) Prep and build
   - Run the relevant builds and tests to ensure the code is in a good state:
     - `nix develop -c cargo build --workspace`
     - `nix develop -c cargo test`
     - `cd packages/orderbook && nix develop -c npm run build && nix develop -c npm run test`

2) Enumerate exported JS API (Rust)
   - Find all wasm-exported items and their metadata:
     - `sg --lang rust -p '#[wasm_export]' crates/js_api`
     - Also consider `#[wasm_bindgen]` classes/impls and `Tsify` types when relevant:
       - `sg --lang rust -p '#[wasm_bindgen]' crates/js_api`
       - `sg --lang rust -p 'derive(Tsify)' crates/js_api`
   - For each exported function/class/method, record:
     - JS name (`js_name` in the attribute, or inferred from Rust name if not set)
     - Parameter list and ordering; whether any `unchecked_param_type` is specified
     - Return type (`unchecked_return_type` if present), and whether `preserve_js_class` is set
     - Associated doc comments (`///`), `param_description`, and `return_description`

3) Compare Rust docs to the effective TS surface
   - Build the package and inspect generated declarations as a quick proxy for the public TS surface:
     - `cd packages/orderbook && nix develop -c npm run build`
     - Open `packages/orderbook/cjs.d.ts` and `packages/orderbook/esm.d.ts`.
   - Verify that each exported item’s JS name, parameter types/order, and return type match the Rust `wasm_export` metadata.
   - If you find a mismatch between Rust doc comments/metadata and the generated `.d.ts`, prefer fixing the Rust docs/metadata (not changing code semantics) so docs match actual exports.

4) Reconcile README and examples with canonical usage
   - Locate examples and usage references in the package README:
     - `packages/orderbook/README.md`
   - Cross-check against real usage in tests to keep documentation examples canonical:
     - `sg --lang ts -p 'import { $X } from \"@rainlanguage/orderbook\"' packages/orderbook/test`
     - Skim `packages/orderbook/test/js_api/*.test.ts` for calls and parameter shapes.
   - Ensure examples use correct names, parameter order and types, and demonstrate error handling with `WasmEncodedResult<T>` where relevant.

5) Apply focused edits
   - In Rust (`crates/js_api/**`):
     - Update `///` doc comments to describe current behavior precisely.
     - Ensure each exported function has accurate `param_description` and `return_description` attributes.
     - Make examples minimal and correct, using the JS-facing `js_name` and showing realistic inputs.
     - Keep tone consistent with existing docs in this crate.
   - In package docs (`packages/orderbook/README.md` and peers):
     - Fix any outdated names, parameters, or return shapes.
     - Align examples with patterns used in tests (preferred canonical usage).
     - Keep imports correct: `import { ... } from '@rainlanguage/orderbook'`.

6) Validate
   - Rust checks: `nix develop -c cargo fmt --all && nix develop -c rainix-rs-static`
   - Build artifacts needed for TS surface checks: `cd packages/orderbook && nix develop -c npm run build`
   - TS type-check the built output: `cd packages/orderbook && nix develop -c npm run check`
   - Run tests to confirm examples mirror real usage: `cd packages/orderbook && nix develop -c npm run test`

Practical ast-grep patterns
- List wasm exports: `sg --lang rust -p '#[wasm_export]' crates/js_api`
- Find JS name assignments: `sg --lang rust -p 'js_name = $NAME' crates/js_api`
- Param descriptions: `sg --lang rust -p 'param_description = $DESC' crates/js_api`
- Return descriptions: `sg --lang rust -p 'return_description = $DESC' crates/js_api`
- Tsify types: `sg --lang rust -p 'derive(Tsify)' crates/js_api`
- TS API imports in tests: `sg --lang ts -p 'import { $API } from "@rainlanguage/orderbook"' packages/orderbook/test`

Editing Guidelines
- Do not change any function signatures or runtime logic for this task.
- Prefer clarifying Rust doc comments and `wasm_export` attributes so the generated TypeScript aligns with docs.
- Use the `js_name` in all JS examples; avoid Rust identifiers in examples.
- Keep examples realistic but short; include basic error handling with `WasmEncodedResult<T>` where applicable.
- If multiple names or behaviors are plausible, defer to the generated `.d.ts` and existing tests.
- Preserve the existing voice and formatting used across current docs.

Acceptance Criteria
- All `#[wasm_export]` items in `crates/js_api` have accurate, up-to-date docs: names, params (with descriptions), return types/descriptions, and examples.
- `packages/orderbook/README.md` contains examples that compile conceptually against the current `.d.ts` and mirror test usage.
- TypeScript declarations (`cjs.d.ts`/`esm.d.ts`) match the documentation claims for names/types.
- Builds, tests, and type checks pass locally using Nix shell commands listed above.
- Your summary lists what changed and calls out any unresolved mismatches that require follow-up.

What to return
- A concise summary of updates made, the files touched, and any notable mismatches discovered that couldn’t be resolved via doc changes.

Notes
- This repo uses a macro named `wasm_export` in `crates/js_api` to define JS-visible APIs and attach TypeScript-specific metadata (`js_name`, `unchecked_return_type`, `param_description`, `return_description`, `preserve_js_class`). Treat those attributes as the contract for the generated TypeScript surface.
- Some examples in README demonstrate end-to-end flows (e.g., builder setup via `RaindexOrderBuilder`, order hash/calldata helpers). Prefer aligning those with current tests under `packages/orderbook/test/js_api` to avoid drift.
