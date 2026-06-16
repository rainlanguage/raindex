# Pass 0: Process Review

**Date:** 2026-03-14
**Files reviewed:** CLAUDE.md, AGENTS.md, ~/.claude/CLAUDE.md, ~/.claude/skills/audit/GENERAL_RULES.md

No changes to process documents since prior audit (2026-03-13-01). All prior findings still apply:

## A01-1: CLAUDE.md uses `@AGENTS.md` syntax — LOW

**File:** CLAUDE.md:1

The `@` prefix (`@AGENTS.md`) is IDE-specific mention syntax. Under context compression or in environments that don't expand `@` references, the instruction could be interpreted as literal text rather than a file reference.

## A01-2: No nix shell guidance in AGENTS.md — MEDIUM

**File:** AGENTS.md

AGENTS.md lists build commands but doesn't mention the nix development environment. Commands must not be prefixed with `nix develop -c` when already inside a nix shell (`$IN_NIX_SHELL` is set).

## A01-3: Preflight check omits `forge fmt` — LOW

**File:** AGENTS.md:29

The "Quick preflight" is `npm run lint-format-check:all && rainix-rs-static`, which covers JS/TS and Rust but not Solidity formatting (`forge fmt`).

## A01-4: Global CLAUDE.md applies audit rules universally — LOW

**File:** ~/.claude/CLAUDE.md:4

"Follow the coding and testing standards in `~/.claude/skills/audit/GENERAL_RULES.md` at all times" applies audit-specific rules to non-audit contexts where they don't apply.

## A01-5: `ast-grep`/`sg` availability not guaranteed — INFO

**File:** AGENTS.md:35

Agent-Specific Instructions prefer `sg` (ast-grep) but it's a nix-provided tool with no fallback instruction.
