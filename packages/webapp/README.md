# sv

Everything you need to build a Svelte project, powered by
[`sv`](https://github.com/sveltejs/cli).

## Creating a project

If you're seeing this, you've probably already done this step. Congrats!

```bash
# create a new project in the current directory
npx sv create

# create a new project in my-app
npx sv create my-app
```

## Developing

Once you've created a project and installed dependencies with `npm install` (or
`pnpm install` or `yarn`), start a development server:

```bash
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```bash
npm run build
```

You can preview the production build with `npm run preview`.

> To deploy your app, you may need to install an
> [adapter](https://svelte.dev/docs/kit/adapters) for your target environment.

## Local DB snapshot POC

The snapshot path is opt-in and leaves the normal production bootstrap
unchanged. Build a checkout of the companion `sqlite-web` snapshot branch,
install its generated tarball locally, then build and run this repository:

```bash
# In sqlite-web
nix develop -c local-bundle

# In raindex
npm install --no-save --package-lock=false ../sqlite-web/pkg/rainlanguage-sqlite-web-0.0.2.tgz
npm run build -w @rainlanguage/raindex
npm run dev -w @rainlanguage/webapp -- --host 127.0.0.1
```

Open `http://127.0.0.1:5173/orders?snapshot-poc=1`. The first load streams and
authenticates the configured gzip snapshot into OPFS, performs cheap Raindex
schema checks, and catches up from its stored watermarks. Later loads render the
existing queryable database immediately while incremental sync continues.

The committed snapshot URL and digest are temporary POC inputs. Production
rollout requires the remote publisher to emit an immutable, browser-ready SQLite
artifact and a manifest containing its URL, compression, uncompressed size and
SHA-256, database schema version, and covered target watermarks.
