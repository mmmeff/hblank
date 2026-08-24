#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const [mode, version] = process.argv.slice(2);
const validModes = new Set(["set", "check"]);
const semver = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

if (!validModes.has(mode) || !semver.test(version ?? "")) {
  console.error("usage: release-version.mjs <set|check> <semver>");
  process.exit(2);
}

const rootManifest = resolve("Cargo.toml");
const runtimeManifest = resolve("crates/hblank/Cargo.toml");

function workspaceVersion(contents) {
  const section = contents.match(
    /^\[workspace\.package\]\s*$([\s\S]*?)(?=^\[|(?![\s\S]))/m,
  );
  if (!section) throw new Error("Cargo.toml has no [workspace.package] section");

  const value = section[1].match(/^version\s*=\s*"([^"]+)"\s*$/m);
  if (!value) throw new Error("[workspace.package] has no string version");
  return value[1];
}

function setWorkspaceVersion(contents) {
  const sectionPattern =
    /(^\[workspace\.package\]\s*$[\s\S]*?^version\s*=\s*")[^"]+("\s*$)/m;
  if (!sectionPattern.test(contents)) {
    throw new Error("cannot update [workspace.package].version");
  }
  return contents.replace(sectionPattern, `$1${version}$2`);
}

const runtimeDependencies = ["hblank-core", "hblank-macros"];

function dependencyPattern(name, captureVersion) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const value = captureVersion ? '([^"]+)' : '[^"]+';
  return new RegExp(
    `(^${escaped}\\s*=\\s*\\{[^\\n}]*\\bversion\\s*=\\s*")${value}("[^\\n}]*\\}\\s*$)`,
    "m",
  );
}

function dependencyVersion(contents, name) {
  const dependency = contents.match(dependencyPattern(name, true));
  if (!dependency) {
    throw new Error(`hblank dependency on ${name} has no string version`);
  }
  return dependency[2];
}

function setDependencyVersion(contents, name) {
  const pattern = dependencyPattern(name, false);
  if (!pattern.test(contents)) {
    throw new Error(`cannot update ${name} dependency version`);
  }
  return contents.replace(pattern, `$1${version}$2`);
}

const rootContents = readFileSync(rootManifest, "utf8");
const runtimeContents = readFileSync(runtimeManifest, "utf8");
const currentWorkspaceVersion = workspaceVersion(rootContents);
const currentDependencyVersions = new Map(
  runtimeDependencies.map((name) => [name, dependencyVersion(runtimeContents, name)]),
);

if (mode === "check") {
  const mismatches = [];
  if (currentWorkspaceVersion !== version) {
    mismatches.push(
      `workspace version is ${currentWorkspaceVersion}, expected ${version}`,
    );
  }
  for (const [name, current] of currentDependencyVersions) {
    if (current !== version) {
      mismatches.push(`${name} dependency is ${current}, expected ${version}`);
    }
  }
  if (mismatches.length > 0) {
    for (const mismatch of mismatches) console.error(mismatch);
    process.exit(1);
  }
  console.log(`release manifests agree on ${version}`);
  process.exit(0);
}

const nextRootContents = setWorkspaceVersion(rootContents);
const nextRuntimeContents = runtimeDependencies.reduce(
  (contents, name) => setDependencyVersion(contents, name),
  runtimeContents,
);

if (nextRootContents !== rootContents) {
  writeFileSync(rootManifest, nextRootContents);
}
if (nextRuntimeContents !== runtimeContents) {
  writeFileSync(runtimeManifest, nextRuntimeContents);
}

console.log(
  `release manifests: ${currentWorkspaceVersion}/${[...currentDependencyVersions.values()].join("/")} -> ${version}`,
);
