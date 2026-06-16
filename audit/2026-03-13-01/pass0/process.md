# Pass 0: Process Review

**Date:** 2026-03-13
**Files reviewed:** CLAUDE.md, AGENTS.md, ~/.claude/CLAUDE.md, ~/.claude/skills/audit/GENERAL_RULES.md

## A01-1: CLAUDE.md uses `@AGENTS.md` syntax — LOW

**File:** CLAUDE.md:1

The `@` prefix (`@AGENTS.md`) is IDE-specific mention syntax (e.g., Cursor, VS Code). Under context compression or in environments that don't expand `@` references, the instruction could be interpreted as literal text rather than a file reference. Plain markdown link or explicit "read the file `AGENTS.md`" would be more robust.

## A01-2: No nix shell guidance in AGENTS.md — MEDIUM

**File:** AGENTS.md

AGENTS.md lists build commands (`cargo build`, `forge build`, `npm run dev`) but doesn't mention the nix development environment. The codebase relies on nix for tooling (`rainix-sol-prelude`, `rainix-rs-static`, `rainix-sol-artifacts`, etc.), and commands must not be prefixed with `nix develop -c` when already inside a nix shell (`$IN_NIX_SHELL` is set). Without this guidance, agents will either fail to find nix-provided tools or double-wrap commands unnecessarily.

## A01-3: Preflight check omits `forge fmt` — LOW

**File:** AGENTS.md:29

The "Quick preflight" is `npm run lint-format-check:all && rainix-rs-static`, which covers JS/TS linting and Rust static analysis. However, Solidity formatting (`forge fmt`) is listed under Coding Style (line 20) but not included in the preflight command. An agent following only the preflight instruction would miss Solidity formatting violations.

## A01-4: Global CLAUDE.md applies audit rules universally — LOW

**File:** ~/.claude/CLAUDE.md:4

The instruction "Follow the coding and testing standards in `~/.claude/skills/audit/GENERAL_RULES.md` at all times" applies audit-specific rules (fuzz run override policies, proposed fix file structure, finding severity classifications) to non-audit contexts where they don't apply. This could cause confusion — e.g., an agent adding a fuzz test might feel obligated to benchmark timing before adding a runs override, even during routine development.

## A01-5: `ast-grep`/`sg` availability not guaranteed — INFO

**File:** AGENTS.md:35

Agent-Specific Instructions prefer `sg` (ast-grep) for search, but `sg` is a nix-provided tool. If an agent is operating outside nix or `sg` is not installed, there's no fallback instruction. Standard tools (Grep, Glob) are always available.
