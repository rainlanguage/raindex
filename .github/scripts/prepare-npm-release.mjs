#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync, writeFileSync } from "node:fs";

const dryRun = process.argv.includes("--dry-run");

const packages = [
  {
    name: "@rainlanguage/orderbook",
    manifestPath: "packages/orderbook/package.json",
  },
  {
    name: "@rainlanguage/ui-components",
    manifestPath: "packages/ui-components/package.json",
  },
];

function readJson(path) {
  const text = readFileSync(path, "utf8");
  return {
    data: JSON.parse(text),
    indent: text.match(/\n([ \t]+)"/)?.[1] ?? "  ",
  };
}

function writeJson(path, json, indent) {
  writeFileSync(path, `${JSON.stringify(json, null, indent)}\n`);
}

function parseAlpha(version) {
  const match = /^(\d+\.\d+\.\d+-alpha\.)(\d+)$/.exec(version);
  if (!match) {
    return undefined;
  }
  return {
    prefix: match[1],
    number: Number.parseInt(match[2], 10),
  };
}

function npmVersions(packageName) {
  const output = execFileSync("npm", ["view", packageName, "versions", "--json"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  });
  const versions = JSON.parse(output);
  return Array.isArray(versions) ? versions : [versions];
}

const manifests = new Map(
  packages.map((pkg) => {
    const manifest = readJson(pkg.manifestPath);
    return [pkg.name, manifest];
  }),
);

const localAlpha = parseAlpha(manifests.get("@rainlanguage/orderbook").data.version);
if (!localAlpha) {
  throw new Error("orderbook package version is not an alpha prerelease");
}

let maxPublishedAlpha = -1;
for (const pkg of packages) {
  for (const version of npmVersions(pkg.name)) {
    const alpha = parseAlpha(version);
    if (alpha?.prefix === localAlpha.prefix) {
      maxPublishedAlpha = Math.max(maxPublishedAlpha, alpha.number);
    }
  }
}

if (maxPublishedAlpha < 0) {
  throw new Error(`No published ${localAlpha.prefix}* versions found`);
}

const nextVersion = `${localAlpha.prefix}${maxPublishedAlpha + 1}`;
console.log(`Resolved npm release version: ${nextVersion}`);

if (!dryRun) {
  for (const pkg of packages) {
    const manifest = manifests.get(pkg.name);
    manifest.data.version = nextVersion;
  }

  const uiManifest = manifests.get("@rainlanguage/ui-components");
  uiManifest.data.dependencies["@rainlanguage/orderbook"] = nextVersion;

  for (const pkg of packages) {
    const manifest = manifests.get(pkg.name);
    writeJson(pkg.manifestPath, manifest.data, manifest.indent);
  }

  const lock = readJson("package-lock.json");
  lock.data.packages["packages/orderbook"].version = nextVersion;
  lock.data.packages["packages/ui-components"].version = nextVersion;
  lock.data.packages["packages/ui-components"].dependencies["@rainlanguage/orderbook"] =
    nextVersion;
  writeJson("package-lock.json", lock.data, lock.indent);
}

if (process.env.GITHUB_ENV) {
  appendFileSync(process.env.GITHUB_ENV, `NEW_VERSION=${nextVersion}\n`);
}
