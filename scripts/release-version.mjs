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

function macroDependencyVersion(contents) {
  const dependency = contents.match(
    /^hblank-macros\s*=\s*\{[^\n}]*\bversion\s*=\s*"([^"]+)"[^\n}]*\}\s*$/m,
  );
  if (!dependency) {
    throw new Error("hblank dependency on hblank-macros has no string version");
  }
  return dependency[1];
}

function setMacroDependencyVersion(contents) {
  const dependencyPattern =
    /(^hblank-macros\s*=\s*\{[^\n}]*\bversion\s*=\s*")[^"]+("[^\n}]*\}\s*$)/m;
  if (!dependencyPattern.test(contents)) {
    throw new Error("cannot update hblank-macros dependency version");
  }
  return contents.replace(dependencyPattern, `$1${version}$2`);
}

const rootContents = readFileSync(rootManifest, "utf8");
const runtimeContents = readFileSync(runtimeManifest, "utf8");
const currentWorkspaceVersion = workspaceVersion(rootContents);
const currentMacroVersion = macroDependencyVersion(runtimeContents);

if (mode === "check") {
  const mismatches = [];
  if (currentWorkspaceVersion !== version) {
    mismatches.push(
      `workspace version is ${currentWorkspaceVersion}, expected ${version}`,
    );
  }
  if (currentMacroVersion !== version) {
    mismatches.push(
      `hblank-macros dependency is ${currentMacroVersion}, expected ${version}`,
    );
  }
  if (mismatches.length > 0) {
    for (const mismatch of mismatches) console.error(mismatch);
    process.exit(1);
  }
  console.log(`release manifests agree on ${version}`);
  process.exit(0);
}

const nextRootContents = setWorkspaceVersion(rootContents);
const nextRuntimeContents = setMacroDependencyVersion(runtimeContents);

if (nextRootContents !== rootContents) {
  writeFileSync(rootManifest, nextRootContents);
}
if (nextRuntimeContents !== runtimeContents) {
  writeFileSync(runtimeManifest, nextRuntimeContents);
}

console.log(
  `release manifests: ${currentWorkspaceVersion}/${currentMacroVersion} -> ${version}`,
);
