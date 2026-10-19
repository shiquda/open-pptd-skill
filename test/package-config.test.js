import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJsonPath = join(projectRoot, "package.json");
const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));

test("package.json bin does not expose pptd to avoid shadowing Rust binary", () => {
  assert.equal(packageJson.bin.pptd, undefined, "package.json bin must not have pptd entry");
  assert.equal(packageJson.bin["open-pptd"], "bin/open-pptd-skill.js");
  assert.equal(packageJson.bin["open-pptd-skill"], "bin/open-pptd-skill.js");
});

test("package.json files includes skills/open-pptd/theme_previews/", () => {
  assert(
    packageJson.files.includes("skills/open-pptd/theme_previews/"),
    "package.json files must include skills/open-pptd/theme_previews/",
  );
});
