import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cli = join(projectRoot, "bin", "open-pptd-skill.js");

function createStubBinary(dir, exitCode = 42) {
  const isWin = process.platform === "win32";
  const stubJs = join(dir, "stub.js");
  writeFileSync(
    stubJs,
    `import { writeFileSync } from "node:fs";
const log = process.env.STUB_LOG;
if (log) {
  writeFileSync(log, JSON.stringify(process.argv.slice(2)), "utf8");
}
process.exit(${exitCode});
`,
    "utf8",
  );

  if (isWin) {
    const stubCmd = join(dir, "stub-pptd.cmd");
    writeFileSync(stubCmd, `@echo off\r\nnode "${stubJs}" %*\r\n`, "utf8");
    return stubCmd;
  }

  const stubSh = join(dir, "stub-pptd");
  writeFileSync(
    stubSh,
    `#!/usr/bin/env node
import { writeFileSync } from "node:fs";
const log = process.env.STUB_LOG;
if (log) {
  writeFileSync(log, JSON.stringify(process.argv.slice(2)), "utf8");
}
process.exit(${exitCode});
`,
    "utf8",
  );
  chmodSync(stubSh, 0o755);
  return stubSh;
}

test("build forwards to native binary specified by PPTD_BIN with unchanged arguments and exit code", () => {
  const tempDir = mkdtempSync(join(tmpdir(), "pptd-stub-test-"));
  const logFile = join(tempDir, "args.json");
  const stubBinary = createStubBinary(tempDir, 42);

  try {
    const result = spawnSync(
      process.execPath,
      [cli, "build", "deck.pptd", "-o", "out.pptx", "--force"],
      {
        env: {
          ...process.env,
          PPTD_BIN: stubBinary,
          STUB_LOG: logFile,
        },
        encoding: "utf8",
      },
    );

    assert.equal(result.status, 42, `stderr: ${result.stderr}`);
    const recordedArgs = JSON.parse(readFileSync(logFile, "utf8"));
    assert.deepEqual(recordedArgs, ["build", "deck.pptd", "-o", "out.pptx", "--force"]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("build forwards exit code 0 on success", () => {
  const tempDir = mkdtempSync(join(tmpdir(), "pptd-stub-test-"));
  const logFile = join(tempDir, "args.json");
  const stubBinary = createStubBinary(tempDir, 0);

  try {
    const result = spawnSync(process.execPath, [cli, "build", "deck.pptd"], {
      env: {
        ...process.env,
        PPTD_BIN: stubBinary,
        STUB_LOG: logFile,
      },
      encoding: "utf8",
    });

    assert.equal(result.status, 0, `stderr: ${result.stderr}`);
    const recordedArgs = JSON.parse(readFileSync(logFile, "utf8"));
    assert.deepEqual(recordedArgs, ["build", "deck.pptd"]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("missing native binary gives actionable install instructions and non-zero exit code (custom PPTD_BIN)", () => {
  const tempDir = mkdtempSync(join(tmpdir(), "pptd-missing-test-"));
  const missingBinary = join(tempDir, "missing-pptd-binary");

  try {
    const result = spawnSync(process.execPath, [cli, "build", "deck.pptd", "-o", "out.pptx"], {
      env: {
        ...process.env,
        PPTD_BIN: missingBinary,
      },
      encoding: "utf8",
    });

    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /native 'pptd' binary not found/);
    assert.match(result.stderr, /skills\/open-pptd\/scripts\/install_binary\.py/);
    assert.doesNotMatch(result.stderr, /spawnSync is not defined/);
    assert.doesNotMatch(result.stderr, /unknown argument/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("missing native binary gives actionable install instructions and non-zero exit code (no PATH or local bin)", () => {
  const tempDir = mkdtempSync(join(tmpdir(), "pptd-empty-env-test-"));

  try {
    const result = spawnSync(process.execPath, [cli, "build", "deck.pptd"], {
      env: {
        ...process.env,
        PPTD_BIN: "",
        HOME: tempDir,
        USERPROFILE: tempDir,
        PATH: "",
      },
      encoding: "utf8",
    });

    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /native 'pptd' binary not found/);
    assert.match(result.stderr, /skills\/open-pptd\/scripts\/install_binary\.py/);
    assert.doesNotMatch(result.stderr, /spawnSync is not defined/);
    assert.doesNotMatch(result.stderr, /unknown argument/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
