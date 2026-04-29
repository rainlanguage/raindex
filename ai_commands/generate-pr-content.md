Title: Generate PR Title and Description from Diff

You are an engineering assistant. Your task is to generate a high‑quality Pull Request title and description using the repository’s PR template, based on a git diff. Start by asking the user which base branch to diff against and whether they want to provide additional motivation/context. Use the diff and any provided context to infer scope, summarize changes, and produce a conventional‑commit style title and a filled template.

Interactive Start
- Ask: “Which base branch should I compare against? Use `main` or specify another (e.g., `release/x.y` or a feature branch).”
- Ask: “Do you want to provide any specific motivation (issue links, goals, context) to include?”
- If motivation is unclear after the diff, ask a brief follow‑up before finalizing text.

Inputs
- Base branch to compare against (default: `main`).
- Optional motivation/context and links (issues/PRs).

Constraints & Repo Conventions
- Use Conventional Commits for the title: `feat:`, `fix:`, `chore:`, `docs:`, `test:`, `refactor:`, `perf:`, `ci:`, `build:`.
- Prefer syntax‑aware search with ast-grep when inspecting code structure:
  - Rust: `sg --lang rust -p '<pattern>'`
  - TypeScript: `sg --lang ts -p '<pattern>'`
- When running builds/tests locally, use Nix shells (if needed for context): `nix develop -c <cmd>`.
- Follow AGENTS.md tone: concise, accurate, minimal verbosity.

High‑Level Goal
1) Determine the correct base for the diff (default `main`).
2) Summarize the changes and infer scope (crates/packages/areas touched).
3) Generate a clear, conventional‑commit PR title with a scoped summary.
4) Fill the PR description using the template below, incorporating user‑provided motivation and a crisp solution summary derived from the diff.

Template (use exactly this in the PR body)
```
<!-- Thanks for your Pull Request, please read the contributing guidelines before submitting. -->

## Motivation

<!--
Explain the context and why you're making that change. What is the problem
you're trying to solve? In some cases there is not a problem and this can be
thought of as being the motivation for your change.
-->

## Solution

<!--
Summarize the solution and provide any necessary context needed to understand
the code change.
-->

## Checks
<!-- It's important you've done these, or your PR will not be considered for review -->
By submitting this for review, I'm confirming I've done the following:
- [ ] made this PR as small as possible
- [ ] unit-tested any new functionality
- [ ] linked any relevant issues or PRs
- [ ] included screenshots (if this involves a front-end change)
```

Procedure
1) Confirm base branch
   - If the user provides none, use `main`.
   - Compute a local diff without network access.
   - Preferred commands:
     - List changed files: `git --no-pager diff --name-only <BASE>...HEAD`
     - Short stats: `git --no-pager diff --stat <BASE>...HEAD`
     - Commit context (optional): `git --no-pager log --oneline <BASE>..HEAD`

2) Infer scope and impact
   - Group changes by top-level path to derive scope keywords, e.g.:
     - `crates/<name>` → Rust crate scope
     - `packages/<name>` → npm/TS package scope
     - `src/`, `test/`, `subgraph/`, `script/` → other well-known areas
   - Heuristics for commit type:
     - `fix:` if tests mention a bug or diffs reverse/guard behavior; otherwise
     - `feat:` if new files/exports/commands are added; otherwise
     - `refactor:` if changes are structural without behavior; otherwise
     - `docs:` if docs only; `test:` if tests only; `chore:` for tooling/infra.
   - Optionally use ast‑grep to pull high-signal names to improve the title:
     - Rust public items: `sg --lang rust -p 'pub (fn|struct|enum|trait) $NAME' <changed_paths>`
     - TS exports: `sg --lang ts -p 'export (function|class|interface|type) $NAME' <changed_paths>`

3) Draft the title
   - Format: `<type>: <scope>: <concise summary>`
   - Scope: prefer a short identifier like `common`, `js_api`, `raindex`, `webapp`.
   - Summary: 6–12 words, describe outcome, not implementation details.

4) Fill the description template
   - Motivation:
     - Use user-provided context verbatim when given.
     - If missing, infer a brief rationale from the diff; if unsure, add a one‑line prompt asking the user to confirm/clarify.
   - Solution:
     - Summarize what changed: key files/areas, notable functions/types touched, new behaviors or fixed defects.
     - Mention tests updated/added and any important follow‑ups.
     - If UI files changed under `packages/webapp`, include a note to attach screenshots.
   - Keep paragraphs short and skimmable.

5) Present results
   - Return both:
     - Title: a single line
     - Description: the filled template body
   - Optionally include a compact diff summary in your answer for the user’s review.

Edge Cases & Guidance
- If the diff is empty, say so and ask the user to confirm the base branch.
- If there are many unrelated changes, propose splitting into multiple PRs and draft multiple candidate titles if appropriate.
- Do not invent details. Prefer asking one concise follow‑up if motivation cannot be inferred reliably.
- Keep style consistent with this repo’s PR expectations in AGENTS.md.

Acceptance Criteria
- Asks the user for base branch (`main` by default) and optional motivation before generating output.
- Produces a conventional‑commit title reflecting scope and change intent.
- Returns the description filled with the exact template, customized with inferred/provided context.
- Summarizes solution grounded in the actual diff (files/areas touched), not speculation.
- Notes when screenshots are relevant for front‑end changes.

What to return
- Title: `<type>: <scope>: <summary>`
- Description: the filled template body text
- Optional: a short “Diff summary” section to help reviewers sanity‑check scope

