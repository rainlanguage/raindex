Title: Refresh ARCHITECTURE.md for a Given Directory

You are an engineering assistant. Your task is to refresh the ARCHITECTURE.md file for a directory specified by the user. The document must accurately describe the current code and behavior in that directory. If the code has changed since the doc was written, identify discrepancies and update the doc to reflect the current state.

Inputs
- A path to the target directory inside this repository, provided by the user (e.g., `crates/common`, `packages/raindex`).

Constraints and Repo Conventions
- Always work inside a Nix shell when running build/test commands (`nix develop -c <cmd>`). Do not fetch network resources.
- Prefer syntax-aware searches with ast-grep for structured matching:
  - Rust: `sg --lang rust -p '<pattern>'`
  - TypeScript: `sg --lang ts -p '<pattern>'`
  - Use plain file reads or simple text scans for languages not supported by ast-grep (e.g., Solidity, Svelte markup) when structural matching is not needed.
- Follow AGENTS.md: if an `ARCHITECTURE.md` exists in the directory, read it first and preserve its voice, structure, and intent where still accurate.
- Do not modify code; only update documentation. Do not commit secrets or change configs.

High-Level Goal
1) Read the existing `ARCHITECTURE.md` (or create a new one if missing) in the provided directory.
2) Build a “current state” snapshot by examining the directory’s code, configuration, and exported surfaces.
3) Compare the snapshot to the current document and list discrepancies.
4) Update `ARCHITECTURE.md` to accurately represent the current state, keeping the style consistent with nearby docs in this repo.

Procedure
1) Verify input and locate doc
   - Confirm the target path exists and is inside this repo.
   - Look for `ARCHITECTURE.md` and also accept `architecture.md` (case-insensitive). If none exist, you will create `ARCHITECTURE.md`.

2) Read local context
   - Open and read the current `ARCHITECTURE.md` fully (if present).
   - Skim `AGENTS.md` at repo root to align format and terminology.

3) Build a current-state snapshot of the directory
   - File/Folder layout: list primary files and subfolders (1–2 levels deep), excluding obvious build outputs (`target`, `dist`, `node_modules`, `out`).
   - Language-specific cues:
     - Rust: read `Cargo.toml`, list crate name, lib/bin targets, features. Use ast-grep to enumerate public surface and structure:
       - Public items: `pub struct`, `pub enum`, `pub trait`, `pub fn` (top-level, module-level).
       - CLI commands (if any): look for `clap::Parser` or `#[derive(Parser)]`.
       - WASM exposure: `#[wasm_bindgen]`, `tsify`, feature flags gating.
     - TypeScript/Svelte: read `package.json` (name, scripts, exports, types). Use ast-grep to find exported API (`export function`, `export class`, `export interface`, `export type`). Note Svelte components under `*.svelte` and any public entry points.
     - Solidity: scan `src/**/*.sol` for `contract`, `interface`, `event` names; note Foundry config in `foundry.toml` and ABIs under `out/` if relevant.
     - Subgraph: if present, capture `subgraph.yaml`, `schema.graphql`, and mapping entry points.
   - Behavior and flows: identify primary responsibilities, key data flows, important invariants, and external dependencies (internal crates, packages, providers) as reflected in code.
   - Build/Test commands: derive correct commands from this repo's conventions (Nix + cargo/forge/npm) relevant to this directory.

4) Detect discrepancies between doc and code
   - Outdated or missing sections: modules/types that no longer exist, new modules not documented, renamed/moved files, changed public APIs, added/removed commands, changed feature flags or build targets.
   - Behavior changes: different data flows, new invariants/constraints, updated error handling, WASM vs native surface changes.
   - Integration points: new or removed dependencies, changed entry points, updated environment variables.

5) Update the document
   - Keep the existing voice and section ordering when they still make sense. Update only what’s necessary to make the document true and useful.
   - If the doc is severely out of date, rewrite it using the Template below. Otherwise, surgically edit inaccurate parts.
   - When introducing new sections, mirror styles used by other `ARCHITECTURE.md` files in this repo (short headings, bullets, concise prose, code fences where helpful).
   - Prefer accuracy over exhaustiveness; link to code paths where appropriate rather than duplicating implementation details.

6) Validate and summarize
   - Ensure headings, lists, and code fences render cleanly.
   - At the end of the file, append a short “Last Updated: YYYY-MM-DD — Summary of changes” line.
   - In your output back to the user, include a brief summary of changes and any notable gaps or TODOs discovered.

Template (use when creating or fully rewriting the doc)
```
# <name> — Architecture

Summary
- Purpose: what this directory/crate/package does in the workspace.
- Scope: key responsibilities and boundaries; platforms/targets (native, WASM, browser).

Directory Layout
- Brief list of important files/folders and their roles.

Build & Test
- Commands relevant to this directory (use Nix):
  - Build: `nix develop -c <cmd>`
  - Test: `nix develop -c <cmd>`
  - Lint/format: `nix develop -c <cmd>`

Modules & Public Surface
- Rust/TS/Solidity modules with a concise description of responsibilities.
- Public API highlights (types, functions, classes, wasm exports, CLI commands).

Key Data Flows & Behavior
- Primary flows and how components interact. Note important invariants and error surfaces.

External & Internal Dependencies
- Workspace crates/packages it depends on and any notable third-party libs.

How It Fits The Workspace
- How this directory integrates with the rest of the project.

Limitations & TODOs
- Known gaps, future work, or caveats.

Last Updated: YYYY-MM-DD — Summary of changes
```

Practical ast-grep patterns (examples)
- Rust public items: `sg --lang rust -p 'pub (struct|enum|trait|fn) $NAME'`
- Rust clap CLI: `sg --lang rust -p '#[derive(Parser)] struct $S'`
- Rust wasm exports: `sg --lang rust -p '#[wasm_bindgen] fn $F(...)'`
- TS exports: `sg --lang ts -p 'export (function|class|interface|type) $NAME'`

Acceptance Criteria
- The updated `ARCHITECTURE.md` exists in the target directory and describes the current code accurately.
- Discrepancies between the previous document and the code have been resolved or explicitly noted.
- The tone and structure match other architecture docs in this repo (concise headings, bullets, minimal verbosity).
- The doc includes a “Last Updated” line with today’s date and a one-line summary.

What to return
- A short summary of what changed and why, plus the path to the updated file.
