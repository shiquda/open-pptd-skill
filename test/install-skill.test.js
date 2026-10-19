import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cli = join(projectRoot, "bin", "open-pptd-skill.js");

function runCli(args, env = process.env) {
  return spawnSync(process.execPath, [cli, ...args], {
    encoding: "utf8",
    env,
  });
}

test("installs the packaged skill into a custom skills directory", () => {
  const root = mkdtempSync(join(tmpdir(), "open-pptd-test-"));
  const target = join(root, "skills");

  try {
    const result = runCli(["install", "--target", target]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(existsSync(join(target, "open-pptd", "SKILL.md")), true);
    assert.equal(existsSync(join(target, "open-pptd", "scripts", "export_pptx.py")), true);
    assert.equal(existsSync(join(target, "open-pptd", "_user_meta.json")), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("installs into ~/.agents/skills when no target is provided (non-interactive)", () => {
  const root = mkdtempSync(join(tmpdir(), "open-pptd-test-"));

  try {
    const result = runCli([], { ...process.env, HOME: root, USERPROFILE: root, CODEX_HOME: undefined });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(existsSync(join(root, ".agents", "skills", "open-pptd", "SKILL.md")), true);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("supports -y for non-interactive default install", () => {
  const root = mkdtempSync(join(tmpdir(), "open-pptd-test-"));

  try {
    const result = runCli(["install", "-y"], {
      ...process.env,
      HOME: root,
      USERPROFILE: root,
      CODEX_HOME: undefined,
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(existsSync(join(root, ".agents", "skills", "open-pptd", "SKILL.md")), true);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("installs into multiple --target directories", () => {
  const root = mkdtempSync(join(tmpdir(), "open-pptd-test-"));
  const first = join(root, "codex", "skills");
  const second = join(root, "claude", "skills");

  try {
    const result = runCli(["install", "--target", first, "--target", second]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(existsSync(join(first, "open-pptd", "SKILL.md")), true);
    assert.equal(existsSync(join(second, "open-pptd", "SKILL.md")), true);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("--all installs into detected agent directories and skips missing ones", () => {
  const root = mkdtempSync(join(tmpdir(), "open-pptd-test-"));

  try {
    mkdirSync(join(root, ".agents"), { recursive: true });
    mkdirSync(join(root, ".codex"), { recursive: true });
    mkdirSync(join(root, ".claude"), { recursive: true });

    const result = runCli(["install", "--all"], {
      ...process.env,
      HOME: root,
      USERPROFILE: root,
      CODEX_HOME: undefined,
    });
    assert.equal(result.status, 0, result.stderr);

    for (const relative of [[".agents", "skills"], [".codex", "skills"], [".claude", "skills"]]) {
      assert.equal(
        existsSync(join(root, ...relative, "open-pptd", "SKILL.md")),
        true,
        relative.join("/"),
      );
    }
    for (const missing of [".cursor", ".workbuddy"]) {
      assert.equal(existsSync(join(root, missing)), false, missing);
    }
    assert.match(result.stdout, /Skipped .*\.cursor/);
    assert.match(result.stdout, /Skipped .*\.workbuddy/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("--all warns and installs nothing when no agent directory exists", () => {
  const root = mkdtempSync(join(tmpdir(), "open-pptd-test-"));

  try {
    const result = runCli(["install", "--all"], {
      ...process.env,
      HOME: root,
      USERPROFILE: root,
      CODEX_HOME: undefined,
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(existsSync(join(root, ".agents")), false);
    assert.match(result.stderr, /No known agent directories found/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("overwrites an existing installation by default", () => {
  const root = mkdtempSync(join(tmpdir(), "open-pptd-test-"));
  const target = join(root, "skills");
  const skillFile = join(target, "open-pptd", "SKILL.md");

  try {
    assert.equal(runCli(["--target", target]).status, 0);
    writeFileSync(skillFile, "modified", "utf8");

    const result = runCli(["--target", target]);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /Updated open-pptd/);
    assert.match(readFileSync(skillFile, "utf8"), /^---\nname: open-pptd/m);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("accepts legacy --force without changing overwrite behavior", () => {
  const root = mkdtempSync(join(tmpdir(), "open-pptd-test-"));
  const target = join(root, "skills");

  try {
    assert.equal(runCli(["--target", target]).status, 0);
    const result = runCli(["--target", target, "--force"]);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /Updated open-pptd/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("help documents interactive install and -y for agents", () => {
  const result = runCli(["install", "--help"]);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /space select/);
  assert.match(result.stdout, /-y, --yes/);
  assert.match(result.stdout, /--all/);
  assert.match(result.stdout, /npx github:shiquda\/open-pptd-skill install -y/);
});

test("-h shows help", () => {
  const result = runCli(["-h"]);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Usage:/);
  assert.match(result.stdout, /-h, --help/);
  assert.match(result.stdout, /-V, --version/);
});

test("--version and -V print the package version", () => {
  const { version } = JSON.parse(readFileSync(join(projectRoot, "package.json"), "utf8"));
  for (const args of [["--version"], ["-V"], ["serve", "--version"]]) {
    const result = runCli(args);
    assert.equal(result.status, 0, `${args.join(" ")}: ${result.stderr}`);
    assert.equal(result.stdout.trim(), version);
  }
});
