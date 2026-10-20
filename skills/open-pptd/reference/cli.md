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
- Unsupported rich text tags (such as `<ul>`, `<ol>`, `<li>`)
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
pptd convert <INPUT_PPTX> <OUTPUT_DIR> [--json] [--fail-on-loss]
```

**Arguments:**
- `<INPUT_PPTX>`: Path to input PowerPoint presentation.
- `<OUTPUT_DIR>`: Output directory for the generated PPTD project.

**Options:**
- `--json`: Print the conversion report as JSON (counts, loss list, unsupported constructs) instead of the human-readable summary.
- `--fail-on-loss`: Exit with code `2` when any lossy mapping is detected. Useful in CI to fail a round-trip that silently downgrades a deck.

**Example:**
```bash
pptd convert input.pptx ./project_dir
pptd convert input.pptx ./project_dir --json > convert-report.json
pptd convert input.pptx ./project_dir --fail-on-loss
```

#### Loss report

`convert` separates two kinds of downgrade:

- **Unsupported constructs** — an element the PPTD pipeline cannot represent at all (embedded OLE objects, image fills, unknown graphic frames, unsupported media clips). Each is reported with its slide and shape name.
- **Lossy mappings** — the element survives but one attribute does not. These are aggregated per category, with a count and the first occurrence:

| `--json` category | What is lost | First-occurrence locator |
|---|---|---|
| `chart` | Native `<c:chart>` whose part or relationship is missing from the package | slide, shape |
| `hyperlink` | Run-level `<a:hlinkClick>` that is internal, action-based (`ppaction://…`), or unresolved; also a `<a:endParaRPr>` click marker extending a link over the paragraph mark | slide, shape |
| `media` | Picture/clip whose extension is outside PNG / JPEG / GIF (pictures) or MP4 / MOV / MP3 / WAV (clips) | slide, shape |
| `font` | `+mj-lt` / `+mn-lt` alias that the theme cannot resolve, so the run falls back to the default face | slide, shape |
| `table_cell_margin` | Cell insets that could not be carried into the PPTD model | slide, table |
| `table_cell_autofit` | `a:normAutofit@fontScale` / `@lnSpcReduction` (the shrink factors PowerPoint applied) | slide, table |
| `baseline` | Run-level `a:rPr@baseline` with no representation | slide, shape |
| `space_after` | `a:spcAft` expressed as `spcPct`; only absolute `spcPts` maps to `spaceAfter` / `margin-bottom` | slide, shape |
| `placeholder` | Slide or layout placeholder with no `<a:xfrm>` and no matching layout placeholder (one aggregated line instead of one per shape) | slide or `layout/master`, shape |

Every category is reported only when it actually occurs; an empty `losses` array means the conversion is lossless for the supported corpus. Layout/master content uses `page: 0` in JSON and `layout/master` in the human summary.

Human-readable output stays compact: one line per category plus a detail line, and no per-shape dump for categories that are already aggregated.

`--json` schema:

```json
{
  "input": "deck.pptx",
  "output_dir": "out",
  "skipped_unchanged": false,
  "page_count": 11,
  "element_count": 342,
  "media_count": 15,
  "src_hash": "…",
  "migrated_sidecars": false,
  "loss_count": 33,
  "losses": [
    {
      "category": "placeholder",
      "label": "unresolved placeholder",
      "detail": "placeholder has no transform and no matching layout entry",
      "count": 33,
      "first": { "page": 0, "element": "Date Placeholder 1" }
    }
  ],
  "skipped_count": 33,
  "unsupported_count": 0,
  "skipped": [
    {
      "page": 0,
      "element": "Date Placeholder 1",
      "reason": "placeholder has no a:xfrm and no matching layout placeholder",
      "category": "placeholder"
    }
  ]
}
```

`skipped_count` includes entries already aggregated into a loss category; `unsupported_count` counts only the unclassified remainder.

**Exit codes:** `0` = conversion completed (and, without `--fail-on-loss`, losses do not fail the run); `1` = the package could not be read or written; `2` = `--fail-on-loss` was passed and at least one lossy mapping was reported. An unchanged source (`skipped_unchanged: true`) performs no conversion and exits `0`.

---

### `pptd dump`
Dumps a diagnostic summary of the parsed project structure and element counts to stdout.

```bash
pptd dump <MANIFEST>
```
