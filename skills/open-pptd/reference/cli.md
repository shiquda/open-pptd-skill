# pptd CLI Reference

`pptd` is the native compiler toolchain for PPTD presentations.

---

## Installation

### Automatic Binary Download
```bash
# From the repository root:
python3 skills/open-pptd/scripts/install_binary.py

# Or from the skill directory:
python3 <SKILL_DIR>/scripts/install_binary.py

# Or curl one-liner:
curl -fsSL https://raw.githubusercontent.com/shiquda/open-pptd-skill/main/skills/open-pptd/scripts/install_binary.sh | bash
```

### From Source (via Cargo)
```bash
cargo install --git https://github.com/shiquda/open-pptd-skill.git
```

### Installer Behavior & Caveats
1. **Automatic Download**: The script attempts to download precompiled binaries from GitHub Releases (`shiquda/open-pptd-skill`) matching your OS (`linux`, `darwin`, `windows`) and architecture (`x86_64`, `aarch64`/`arm64`) into `~/.local/bin/pptd` (or `--dir <PATH>`).
2. **Shell Installer Cargo Fallback**:
   - When prebuilt archive downloads fail, `install_binary.sh` falls back to `cargo install --git https://github.com/shiquda/open-pptd-skill.git --root "${INSTALL_DIR}/.." --force`.
   - **Target directory caveat**: `--root "${INSTALL_DIR}/.."` places the binary in `${INSTALL_DIR}/../bin/`. This resolves to `${INSTALL_DIR}` only when `INSTALL_DIR` is the default `~/.local/bin`. If a custom `--dir` is supplied (e.g. `/tmp/custom`), the cargo output will land in `/tmp/bin/pptd` rather than `/tmp/custom/pptd`.
   - **Tag caveat**: The cargo fallback command does not pass a `--tag` parameter, so it builds the current default branch (`main` HEAD) rather than a pinned release tag. For a pinned release, invoke cargo manually with `--tag`:
     ```bash
     cargo install --git https://github.com/shiquda/open-pptd-skill.git --tag v1.4.1
     ```
---
### `pptd check`
Performs strict semantic and structural validation of a PPTD project without building.

```bash
pptd check <MANIFEST>
```

Checks for:
- Unknown or misspelled YAML fields on page and element definitions
- Invalid DrawingML preset shape names with automatic suggestions
- Undefined theme color `$colorKey` references
- Missing local image or media files under `assets/`
- Unsupported rich text tags (such as `<ul>`, `<ol>`, `<li>`, `<a>`, `<sup>`, `<sub>`)
- Unsupported element types (such as `chart`)
- Image fills on shapes (`fill: {type: image}` is rejected; only backgrounds support image fills)
- Elements positioned outside the slide canvas boundaries (error for entirely out-of-bounds, warning for partially out-of-bounds)
- Declared animations (warns that `animations` are parsed in AST but not emitted to `p:timing`)
- Unsupported LaTeX commands in math blocks
- Potential vertical or horizontal text box overflows

**Exit codes:** `0` = clean (warnings may still have been printed), `2` = at least one error was reported. The summary line tells the two apart (`no issues` vs `N warning(s)` vs `check failed: N error(s)`).

**`check` is a lint, not a build dry-run.** These constructs pass `check` and only fail in `build`:

- LaTeX that the parser rejects but the lint does not scan for, e.g. an unclosed `{` (`error: invalid PPTD: LaTeX math syntax error: unclosed '{'`)
- Table merges that escape the grid (`not supported yet: table 'tb1' merge at row 1 col 1 escapes the grid`; coordinates are 0-based)
- A `line` with fewer than two points (`line 'l1' needs at least 2 points`)
- Unsupported SVG path commands
- Documented-but-unimplemented fields (`cropShape`, `softEdge`, `fill` on a table, `customFonts` in the manifest) — these were rejected by `check` before v1.4.2 and are accepted now

Manifest-level field checking only runs when the entry file is named `deck.pptd` or `presentation.pptd` (or matches the deck `title`); a manifest with any other file name is validated without that check.

Always run `pptd build` before declaring a deck done.
---


## Commands

### `pptd build`
Compiles a PPTD manifest into a native OpenXML `.pptx` presentation.

```bash
pptd build <MANIFEST> [OPTIONS]
```

**Arguments:**
- `<MANIFEST>`: Path to `deck.pptd` manifest.

**Options:**
- `-o, --output <PATH>`: Output `.pptx` file path (default: `<title>.pptx` or `presentation.pptx`).
- `-f, --force`: Bypass `.sync.hash` overwrite guard to overwrite an existing `.pptx` file that has been modified externally.
```bash
pptd build deck.pptd -o my_slides.pptx
```

---

### `pptd convert`
Decompiles an existing PowerPoint `.pptx` file into a clean, editable PPTD project.

```bash
pptd convert <INPUT_PPTX> <OUTPUT_DIR>
```

**Arguments:**
- `<INPUT_PPTX>`: Path to input PowerPoint presentation.
- `<OUTPUT_DIR>`: Output directory for the generated PPTD project.

**Example:**
```bash
pptd convert input.pptx ./project_dir
```

---

### `pptd dump`
Dumps a diagnostic summary of the parsed project structure and element counts to stdout.

```bash
pptd dump <MANIFEST>
```
