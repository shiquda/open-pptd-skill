#!/usr/bin/env node

import {
  accessSync,
  constants,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, join, resolve, sep } from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { stdin as input, stdout as output } from "node:process";
import { fileURLToPath } from "node:url";
import { startEditorServer } from "../lib/editor-server.js";

const SKILL_NAME = "open-pptd";
const MIN_NODE_MAJOR = 18;
const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sourceDirectory = join(packageRoot, "skills", SKILL_NAME);
const packageVersion = JSON.parse(readFileSync(join(packageRoot, "package.json"), "utf8")).version;

const KNOWN_TARGETS = [
  { id: "agents", label: "Shared / default", relative: [".agents", "skills"] },
  { id: "codex", label: "Codex", relative: [".codex", "skills"] },
  { id: "claude", label: "Claude Code", relative: [".claude", "skills"] },
  { id: "cursor", label: "Cursor", relative: [".cursor", "skills"] },
  { id: "workbuddy", label: "WorkBuddy", relative: [".workbuddy", "skills"] },
];

function assertNodeVersion() {
  const major = Number.parseInt(process.versions.node.split(".")[0], 10);
  if (!Number.isInteger(major) || major < MIN_NODE_MAJOR) {
    throw new Error(
      `Node.js ${MIN_NODE_MAJOR}+ is required; found ${process.version}. Install from https://nodejs.org`,
    );
  }
}

function printHelp(command) {
  if (command === "serve") {
    console.log(`Start the local PPTD editor and exporter.

Usage:
  open-pptd-skill serve [options]

Options:
  --port <number>  HTTP port (default: 55173)
  --open           Open the editor in the default browser
  -h, --help       Show this help
  -V, --version    Show version
`);
    return;
  }

  console.log(`Install ${SKILL_NAME} for your AI coding agent, build PPTD decks, or start its local editor.

Usage:
  open-pptd-skill [install] [options]
  open-pptd-skill build [options] <file>
  open-pptd-skill serve [options]

Install options:
  --target <directory>  Skills directory (repeatable)
  -y, --yes             Non-interactive; install to ~/.agents/skills when no --target
  --all                 Install to all detected agent skill directories
                        (agents whose home directory is missing are skipped)
  -h, --help            Show this help
  -V, --version         Show version

In an interactive terminal (no --target / --yes / --all), a checklist is shown:
  ↑/↓ move  space select  a all  enter confirm

For agents / CI, prefer:
  npx github:shiquda/open-pptd-skill install -y

This command only distributes skill files (SKILL.md, references, scripts) into a skills
directory. It does not install, download, or run the pptd compiler, and it is not needed
to build decks. Install the native Rust compiler separately from the installed skill:
  python3 ~/.agents/skills/open-pptd/scripts/install_binary.py

Re-running install replaces an existing open-pptd installation.
Run "open-pptd-skill serve --help" for server options.
Run "open-pptd-skill build --help" for build options.
`);
}

function printVersion() {
  console.log(packageVersion);
}

function parseArguments(arguments_) {
  const raw = [...arguments_];
  const args = [...arguments_];
  const known = ["install", "serve", "build", "convert", "install-binary", "check", "dump"];
  const command = known.includes(args[0]) ? args.shift() : "install";
  const options = {
    command,
    rawArgs: raw.slice(known.includes(raw[0]) ? 1 : 0),
    targets: [],
    yes: false,
    all: false,
    open: false,
    port: 55173,
  };

  if (command !== "install" && command !== "serve") {
    return options;
  }
  while (args.length > 0) {
    const argument = args.shift();

    // Accepted for backward compatibility; install always overwrites.
    if (command === "install" && argument === "--force") {
      continue;
    }

    if (command === "install" && (argument === "-y" || argument === "--yes")) {
      options.yes = true;
      continue;
    }

    if (command === "install" && argument === "--all") {
      options.all = true;
      continue;
    }

    if (command === "install" && argument === "--target") {
      const target = args.shift();
      if (!target || target.startsWith("-")) {
        throw new Error("--target requires a directory");
      }
      options.targets.push(resolve(target));
      continue;
    }

    if (command === "serve" && argument === "--port") {
      const port = Number(args.shift());
      if (!Number.isInteger(port) || port < 1 || port > 65_535) {
        throw new Error("--port must be an integer between 1 and 65535");
      }
      options.port = port;
      continue;
    }

    if (command === "serve" && argument === "--open") {
      options.open = true;
      continue;
    }

    if (argument === "--help" || argument === "-h") {
      options.help = true;
      continue;
    }

    if (argument === "--version" || argument === "-V") {
      options.version = true;
      continue;
    }

    throw new Error(`unknown argument: ${argument}`);
  }

  return options;
}

function openBrowser(url) {
  const command = process.platform === "darwin" ? "open" : process.platform === "win32" ? "cmd" : "xdg-open";
  const args = process.platform === "win32" ? ["/c", "start", "", url] : [url];
  const child = spawn(command, args, { detached: true, stdio: "ignore" });
  child.on("error", (error) => console.warn(`Could not open the browser: ${error.message}`));
  child.unref();
}

function defaultSkillsDirectory() {
  return join(homedir(), ".agents", "skills");
}

function knownTargetDirectories() {
  const home = homedir();
  return KNOWN_TARGETS.map((entry) => ({
    ...entry,
    directory: join(home, ...entry.relative),
  }));
}

function displayPath(absolutePath) {
  const home = homedir();
  const normalized = absolutePath.split(sep).join("/");
  const homeNormalized = home.split(sep).join("/");
  if (normalized === homeNormalized || normalized.startsWith(`${homeNormalized}/`)) {
    return `~${normalized.slice(homeNormalized.length)}`;
  }
  return absolutePath;
}

function isInteractiveInstall() {
  return Boolean(input.isTTY && output.isTTY);
}

function uniqueDirectories(directories) {
  const seen = new Set();
  const unique = [];
  for (const directory of directories) {
    const key = resolve(directory);
    if (seen.has(key)) continue;
    seen.add(key);
    unique.push(key);
  }
  return unique;
}

async function promptInstallTargets() {
  const choices = knownTargetDirectories();
  const selected = new Set([0]);
  let cursor = 0;
  const lines = choices.length + 3;

  const render = (initial = false) => {
    if (!initial) {
      output.write(`\u001b[${lines}A`);
    }
    output.write("Install open-pptd to which skills directories?\n");
    output.write("↑/↓ move · space select · a all · enter confirm · ctrl+c cancel\n");
    for (const [index, choice] of choices.entries()) {
      const pointer = index === cursor ? "❯" : " ";
      const mark = selected.has(index) ? "◉" : "◯";
      const installed = existsSync(join(choice.directory, SKILL_NAME, "SKILL.md"))
        ? " (installed)"
        : "";
      const line = `${pointer}${mark} ${displayPath(choice.directory).padEnd(28)} ${choice.label}${installed}`;
      output.write(`\u001b[2K${line}\n`);
    }
  };

  return new Promise((resolvePromise, reject) => {
    if (typeof input.setRawMode !== "function") {
      resolvePromise([defaultSkillsDirectory()]);
      return;
    }

    render(true);
    input.setRawMode(true);
    input.resume();
    input.setEncoding("utf8");

    const cleanup = () => {
      input.off("data", onData);
      input.setRawMode(false);
      input.pause();
    };

    const onData = (key) => {
      if (key === "\u0003") {
        cleanup();
        output.write("\n");
        reject(new Error("install cancelled"));
        return;
      }

      if (key === "\u001b[A" || key === "k") {
        cursor = (cursor - 1 + choices.length) % choices.length;
        render();
        return;
      }

      if (key === "\u001b[B" || key === "j") {
        cursor = (cursor + 1) % choices.length;
        render();
        return;
      }

      if (key === " ") {
        if (selected.has(cursor)) selected.delete(cursor);
        else selected.add(cursor);
        render();
        return;
      }

      if (key === "a" || key === "A") {
        if (selected.size === choices.length) selected.clear();
        else for (let index = 0; index < choices.length; index += 1) selected.add(index);
        render();
        return;
      }

      if (key === "\r" || key === "\n") {
        if (selected.size === 0) {
          output.write("\u0007");
          return;
        }
        cleanup();
        output.write("\n");
        resolvePromise([...selected].sort((a, b) => a - b).map((index) => choices[index].directory));
      }
    };

    input.on("data", onData);
  });
}

function detectedTargetDirectories() {
  const home = homedir();
  const detected = [];
  const skipped = [];
  for (const entry of knownTargetDirectories()) {
    if (existsSync(join(home, entry.relative[0]))) detected.push(entry);
    else skipped.push(entry);
  }
  return { detected, skipped };
}

async function resolveInstallTargets(options) {
  if (options.targets.length > 0) {
    return uniqueDirectories(options.targets);
  }

  if (options.all) {
    const { detected, skipped } = detectedTargetDirectories();
    for (const entry of skipped) {
      console.log(`Skipped ${entry.directory} (${entry.label} not found)`);
    }
    if (detected.length === 0) {
      console.warn(
        "No known agent directories found; nothing installed. Use -y for ~/.agents/skills or --target <directory>.",
      );
    }
    return uniqueDirectories(detected.map((entry) => entry.directory));
  }

  if (options.yes || !isInteractiveInstall()) {
    return [defaultSkillsDirectory()];
  }

  return promptInstallTargets();
}

function installSkillTo(skillsDirectory) {
  if (!existsSync(join(sourceDirectory, "SKILL.md"))) {
    throw new Error(`packaged skill is incomplete: ${sourceDirectory}`);
  }

  const destination = join(skillsDirectory, SKILL_NAME);
  const replaced = existsSync(destination);

  mkdirSync(skillsDirectory, { recursive: true });
  const stagingRoot = mkdtempSync(join(skillsDirectory, `.${SKILL_NAME}-`));
  const stagedSkill = join(stagingRoot, SKILL_NAME);

  try {
    cpSync(sourceDirectory, stagedSkill, {
      recursive: true,
      filter: (source) => ![".DS_Store", "_user_meta.json"].includes(basename(source)),
    });

    rmSync(destination, { recursive: true, force: true });
    renameSync(stagedSkill, destination);
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }

  console.log(
    replaced
      ? `Updated ${SKILL_NAME} at ${destination}`
      : `Installed ${SKILL_NAME} to ${destination}`,
  );
}

async function installSkill(options) {
  const targets = await resolveInstallTargets(options);
  for (const target of targets) {
    installSkillTo(target);
  }
}

function findOnPath(binName) {
  const pathEnv = process.env.PATH || "";
  const delimiter = process.platform === "win32" ? ";" : ":";
  const extensions = process.platform === "win32"
    ? (process.env.PATHEXT || ".EXE;.CMD;.BAT;.COM").split(";").map((e) => e.toLowerCase())
    : [""];
  const dirs = pathEnv.split(delimiter);

  for (const dir of dirs) {
    if (!dir) continue;
    for (const ext of extensions) {
      const candidate = join(dir, process.platform === "win32" ? `${binName}${ext}` : binName);
      try {
        if (existsSync(candidate)) {
          if (process.platform !== "win32") {
            accessSync(candidate, constants.X_OK);
          }
          return candidate;
        }
      } catch {
        // ignore
      }
    }
  }
  return null;
}

function resolvePptdBinary() {
  if (process.env.PPTD_BIN) {
    const custom = resolve(process.env.PPTD_BIN);
    if (existsSync(custom)) {
      return custom;
    }
    return null;
  }

  const home = homedir();
  const localBin = join(home, ".local", "bin", process.platform === "win32" ? "pptd.exe" : "pptd");
  if (existsSync(localBin)) {
    try {
      if (process.platform !== "win32") {
        accessSync(localBin, constants.X_OK);
      }
      return localBin;
    } catch {
      // ignore
    }
  }

  return findOnPath("pptd");
}

async function main() {
  assertNodeVersion();
  const options = parseArguments(process.argv.slice(2));
  if (options.version) {
    printVersion();
    return;
  }
  if (options.help) {
    printHelp(options.command);
    return;
  }
  if (options.command === "install-binary") {
    const script = join(packageRoot, "skills", SKILL_NAME, "scripts", "install_binary.py");
    const res = spawnSync("python3", [script, ...options.rawArgs], { stdio: "inherit" });
    if (res.error) {
      console.error(`Error: failed to run installer: ${res.error.message}`);
      process.exit(1);
    }
    process.exit(res.status ?? 0);
  }
  if (["build", "check", "dump"].includes(options.command)) {
    const binary = resolvePptdBinary();
    if (!binary) {
      console.error("Error: native 'pptd' binary not found.");
      console.error("Please install it by running:");
      console.error("  python3 skills/open-pptd/scripts/install_binary.py");
      const installScript = join(packageRoot, "skills", SKILL_NAME, "scripts", "install_binary.py");
      if (existsSync(installScript)) {
        console.error(`  (or: python3 ${installScript})`);
      }
      process.exit(1);
    }
    const res = spawnSync(binary, [options.command, ...options.rawArgs], { stdio: "inherit" });
    if (res.error) {
      console.error(`Error: failed to execute '${binary}': ${res.error.message}`);
      process.exit(1);
    }
    process.exit(res.status ?? (res.signal ? 1 : 0));
  }
  if (options.command === "convert") {
    const script = join(packageRoot, "skills", SKILL_NAME, "scripts", "convert_pptx.py");
    const res = spawnSync("python3", [script, ...options.rawArgs], { stdio: "inherit" });
    if (res.error) {
      console.error(`Error: failed to run converter: ${res.error.message}`);
      process.exit(1);
    }
    process.exit(res.status ?? 0);
  }

  if (options.command === "install") {
    await installSkill(options);
    return;
  }
  const { server, url } = await startEditorServer({ port: options.port });
  console.log(`Open Kimi PPT editor is running at ${url}`);
  console.log("Press Ctrl+C to stop the server.");
  if (options.open) openBrowser(url);

  const shutdown = () => server.close(() => process.exit(0));
  process.once("SIGINT", shutdown);
  process.once("SIGTERM", shutdown);
}

main().catch((error) => {
  console.error(`Error: ${error.message}`);
  process.exitCode = 1;
});
