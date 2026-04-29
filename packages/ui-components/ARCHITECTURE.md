# @rainlanguage/ui-components — Architecture

This package is the reusable Svelte component library for building Rain Orderbook UIs. It composes the WASM‑backed SDK `@rainlanguage/orderbook` with UI primitives, domain components, providers, hooks, and utilities to implement common flows: listing and inspecting orders and vaults, showing charts, handling wallet connection and transactions, and guiding users through deploying algorithmic orders from a dotrain registry.


## Overview

- Purpose
  - Provide a cohesive, app‑ready set of Svelte components and helpers for Orderbook features: tables, detail views, charts, deployment builder, toasts, and transaction UX.
  - Ship provider and hook contexts so apps can wire wallet, client, registry, toasts, and transaction state consistently.
- Targets
  - Svelte 4 components, packaged with `svelte-package` for consumption in SvelteKit/Vite apps.
  - Single `dist/index.js` entry (ESM) with `svelte` and `types` fields; no SSR‑specific code required.
- Upstream libraries
  - SDK: `@rainlanguage/orderbook` (WASM)
  - State/query: `@tanstack/svelte-query`
  - Wallet: `wagmi`, `viem`, `@reown/appkit` + `@reown/appkit-adapter-wagmi`
  - UI: `flowbite-svelte` (+ icons), `tailwindcss`, `lightweight-charts`, `svelte-markdown`, `svelte-codemirror-editor`, `codemirror-rainlang`

Typical development

```bash
cd packages/ui-components
nix develop -c npm run dev
```


## Providers & State

The library exposes lightweight provider components that set Svelte contexts, plus hooks to access them:

- Raindex client
  - `RaindexClientProvider` — sets a `RaindexClient` from `@rainlanguage/orderbook` in context.
  - `useRaindexClient()` — retrieves the client and reports a user‑facing error if missing.
- Wallet
  - `WalletProvider` — injects a Svelte `Readable<Hex | null>` account store into context.
  - `useAccount()` — returns the account store from context.
- Transactions
  - `TransactionProvider` — constructs a `TransactionManager` using TanStack Query’s `QueryClient` (via `useQueryClient()`), a `wagmi` `Config`, and an `addToast` function.
  - `useTransactions()` — returns `{ manager, transactions }`, where `transactions` is a store of in‑flight `TransactionStore` instances.
  - UI components: `TransactionList`, `FixedBottomTransaction` to surface status, errors, and explorer links.
- Registry
  - `RegistryProvider` + `RegistryManager` — manage dotrain registry URL (persisted in `localStorage` and an optional `?registry=` query param). `useRegistry()` returns the manager.
- Toasts
  - `ToastProvider` — provides a toasts store and renders `ToastDetail` instances.
  - `useToasts()` — returns `{ toasts, addToast, removeToast, errToast }`.
- Builder (deploy flows)
  - `RaindexOrderBuilderProvider` — provides a `RaindexOrderBuilder` instance for deployment flows.
  - `useRaindexOrderBuilder()` — retrieves the builder instance; integrates with deployment components.

Notes

- Consumers should wrap their app in a TanStack Query `QueryClientProvider` (from `@tanstack/svelte-query`) so `TransactionProvider` can access the client.
- `TransactionProvider` requires an `addToast` callback. A small “wrapper” component that calls `useToasts()` is a convenient way to pass this down.


## Components & Features

- Tables & lists
  - `OrdersListTable`, `OrderTradesListTable`, `VaultsListTable`, `VaultBalanceChangesTable`, `OrderVaultsVolTable` driven by TanStack Query, with `TanstackAppTable` wrapper.
- Detail views & quotes
  - `OrderDetail`, `VaultDetail`, `TanstackOrderQuote`, `TanstackPageContentDetail`, `Hash`, `OrderOrVaultHash`.
- Charts
  - `LightweightChart`, `TanstackLightweightChartLine`, `OrderTradesChart`, `VaultBalanceChart` with time helpers and themes.
- Deployment Builder
  - `OrderPage`, `DeploymentsSection`, `DeploymentSteps`, `TokenIOInput`, `FieldDefinitionInput`, `SelectToken`, `DisclaimerModal`, `ValidOrdersSection`, `InvalidOrdersSection`.
  - Services: dotrain registry fetch/validate/share (`registry.ts`, `loadRegistryUrl.ts`, `handleShareChoices.ts`).
- Wallet UX & general UI
  - `WalletConnect`, `ButtonDarkMode`, dropdowns, inputs (`InputTokenAmount`, `InputHex`, `InputOrderHash`, `InputRegistryUrl`), tooltips, badges, icons.
- Editors
  - `CodeMirrorRainlang`, `CodeMirrorDotrain` with theme helpers.


## Services, Utils, Queries

- Time & indexing
  - `awaitTransactionIndexing` — generic polling helper to await subgraph indexing with success predicate; used by `TransactionStore`.
  - `formatTimestampSecondsAsLocal`, `timestampSecondsToUTCTimestamp`, `promiseTimeout`, `dateTimestamp`.
- Links & formatting
  - `getExplorerLink` (via `viem/chains`), `bigintStringToHex`, numeric/string utilities.
- Registry
  - `fetchParseRegistry`, `fetchRegistryDotrains`, `validateOrders`, `loadRegistryUrl`.
- Charts
  - `historicalOrderCharts.ts` for transforming trades to chartable data.
- Queries
  - `queries/constants.ts`, `queries/keys.ts`, `queries/queryClient.ts` (includes `invalidateTanstackQueries`).


## Exports & API Surface

`src/lib/index.ts` re‑exports by category:

- Components: tables, detail views, charts, editors, inputs, icons, wallet/connectivity, UI primitives.
- Providers: `RaindexOrderBuilderProvider`, `RaindexClientProvider`, `WalletProvider`, `RegistryProvider`, `ToastProvider`, `TransactionProvider`.
- Hooks: `useRaindexOrderBuilder`, `useRaindexClient`, `useAccount`, `useRegistry`, `useToasts`, `useTransactions`.
- Types: app stores, modal/transaction/toast/order typings.
- Functions: time helpers, explorer link, TanStack invalidation helpers, mocks for tests.
- Constants: query keys, default page sizes/intervals, chart/code editor themes.
- Stores: cached writable store helpers.
- Assets: light/dark logos.
- Classes: `RegistryManager`, `TransactionStore`, `TransactionManager`.


## Directory Layout

- `src/lib/components/` — Svelte components grouped by domain (`tables`, `detail`, `charts`, `deployment`, `transactions`, `wallet`, `input`, etc.).
- `src/lib/providers/` — Context providers for builder, client, wallet, registry, toasts, transactions.
- `src/lib/hooks/` — Accessors for provider contexts.
- `src/lib/models/` — State models such as `TransactionStore`.
- `src/lib/types/` — Type helpers and enums.
- `src/lib/services/` — Registry, indexing, sharing, time utilities.
- `src/lib/queries/` — Query keys, constants, query client helpers.
- `src/lib/utils/` — Formatting and misc utilities.
- `src/lib/assets/` — SVGs, logos.
- `src/lib/__mocks__/` — Test helpers and resolvable queries/stores.


## Build, Test, and Dev

Run inside a Nix shell for tool parity (`nix develop -c <cmd>`):

- Dev preview: `nix develop -c npm run dev`
- Build: `nix develop -c npm run build` (Vite) and `npm run package` (svelte‑package + publint)
- Package only: `nix develop -c npm run package`
- Lint/format/check: `npm run format`, `npm run lint`, `npm run check`
- Tests: `nix develop -c npm run test` (Vitest, jsdom)


## Usage Notes

- Provider wiring
  - Wrap your app in `QueryClientProvider`, then add `ToastProvider`, `WalletProvider`, `RaindexClientProvider`, `RegistryProvider`, and a small wrapper for `TransactionProvider` that supplies `addToast` from `useToasts()` and a `wagmi` `Config`.
  - See `packages/webapp` for a working composition with a fixed‑bottom transaction status bar.
- Tailwind setup
  - Ensure Tailwind’s `content` globs include this package and `flowbite-svelte` so styles tree‑shake correctly. Example (from the webapp):

```ts
// tailwind.config.ts (consumer app)
export default {
  content: [
    './src/**/*.{html,js,svelte,ts}',
    '../../node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}',
    '../../node_modules/@rainlanguage/ui-components/**/*.{html,js,svelte,ts}',
    '../ui-components/**/*.{html,js,svelte,ts}',
  ],
  // ...
}
```


## How It Fits The Workspace

- Rust crates under `crates/*` implement core logic and compile to a WASM surface consumed by `@rainlanguage/orderbook`.
- `@rainlanguage/ui-components` provides the reusable Svelte UI and provider layer that apps compose.
- The `packages/webapp` project consumes this library directly to implement the full Orderbook UI.


## Caveats & Tips

- Always run inside `nix develop` for consistent Node/tooling.
- Ensure the Query Client is available in context before mounting `TransactionProvider`.
- Pass a valid `wagmi` `Config` to `TransactionProvider` and ensure wallet connectors are initialized at the app level.
- Include this package in Tailwind `content` globs to avoid missing styles.
- For deployment flows, pass a `RaindexOrderBuilder` via `RaindexOrderBuilderProvider` and use the registry helpers to load/validate dotrain entries.

This document explains what `packages/ui-components` is for, how providers and components are organized, how to build and test the package, and how it integrates with the rest of the Rain Orderbook workspace.

