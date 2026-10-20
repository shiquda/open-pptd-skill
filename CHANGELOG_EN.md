# Changelog

[简体中文](CHANGELOG.md) | [English](CHANGELOG_EN.md)

This project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Native chart round-trip (`elementType: chart`)**: `pptd convert` copies `ppt/charts/chartN.xml`, its `_rels` and any embedded `ppt/embeddings/*.xlsx` byte-for-byte into the project `media/` directory and emits a chart element carrying `chartPart` / `workbookPart` / `chartKind`. `pptd build` re-emits `p:graphicFrame` + `<c:chart r:id>`, the slide rels, the chart rels and the `[Content_Types].xml` overrides without rewriting the chart XML. `pptd check` verifies the part exists, stays inside the project and has positive bounds; the declarative `data`/`series` DSL is still rejected explicitly
- **Rich-text hyperlinks**: `<a href="https://…">text</a>` compiles to a run-level `<a:hlinkClick>` plus an external relationship in `slideN.xml.rels` (`TargetMode="External"`, URLs deduplicated). `pptd convert` restores the same inline markup, and the element-level `link` field round-trips as well
- **Font names pass through by default**: `pptd build` writes the PPTD typeface verbatim; the Kimi web-font → system-font safety map is now the explicit `safeFonts: true` switch (off for `pptd convert` output, so rebuilt decks keep their source typography)

### Fixed

- **Table cell insets and autofit were lost on round-trip**: `Cell` gains `margin: {left,right,top,bottom}` (pt) and `autofit`. `pptd convert` reads `a:tcPr@marL/marR/marT/marB` plus the cell `a:bodyPr` child (`spAutoFit` / `normAutofit` / `noAutofit`), and `pptd build` writes them back verbatim (default insets collapse to "not written", preserving the previous default behaviour). Previously every cell was forced to 7.2/7.2/3.6/3.6 pt, which reflowed cell text; `pptd check` now rejects negative insets

## [1.4.1] - 2026-10-19

### Added

- **Multi-line aligned equations**: `\begin{align}`, `align*`, `aligned`, `gather`, `gathered`, `split` and `eqnarray` compile into a native OMML equation array (`<m:eqArr>`, one `<m:e>` per line, with the alignment point written as an `&` run — the same encoding Word's linear format uses and Microsoft's `MML2OMML.XSL` emits). `align`/`gather`/`eqnarray` are display environments and must own their paragraph; mixing one into a line of text is an explicit error
- **Per-formula macros**: `\newcommand` (including `[n]` and `[n][default]`), `\renewcommand`, `\providecommand` and `\def` (including `#1#2` parameters) are expanded before parsing. Scope is one formula, recursive definitions hit an expansion-limit error, and `pptd check` expands first so user-defined macros are no longer reported as unsupported
- `pptd convert` decompiles `<m:eqArr>` back into `\begin{aligned} … \end{aligned}`, alignment points preserved

### Fixed

- **Braces destroyed formula structure**: `{\frac{a}{b}} + c` used to collapse into the text `a/b+c`; the fraction structure now survives (`{x + \frac{a}{b}}` likewise)
- **Big operators swallowed their neighbours**: `\sum_{i=1}^{n} A + \sum_{j=1}^{m} B` nested the second sum inside the first one's operand; a big operator now takes the single atom that follows it, as TeX does
- The slide root now declares `mc:Ignorable="a14"`, so readers that do not know the `a14` math extension skip it per the OOXML rules instead of treating the part as corrupt

## [1.4.0] - 2026-10-19

### Added

- `<latex>…</latex>` and inline `\(…\)` compile to **native OOXML math** (`<a14:m>` / OMML): genuine fraction bars (`\frac`), radical roots (`\sqrt`, `\sqrt[n]{}`), scripts, big operators (`\sum`, `\int`, `\lim`), matrices and `\left…\right` delimiters. A standalone formula becomes a centered display equation; an inline formula additionally carries an `mc:Fallback` text rendering so viewers without the `a14` math extension (LibreOffice among them) still show the formula instead of losing it
- `pptd convert` decompiles native math back into `<latex>` source
- `letterSpacing` and `textDirection` are now actually emitted (`a:rPr@spc`, `a:bodyPr@vert`)
- New `pptd check` diagnostics: shape image fill, element outside the canvas, declared animations, unsupported LaTeX commands, icon prefixes and unknown icons
- `validate_pptx.py --render-images` writes per-page `slide-NN.png` plus an overview `contact-sheet.jpg`
- New "Current Boundaries" section in the README

### Fixed

- The installer now compares versions: an older `pptd` (or legacy `slideforge`) on `PATH` is upgraded to the target version instead of being reported as ready; `--force` is no longer required
- `.sync.hash`: in-project outputs are recorded relative, one entry per output path, no `.//…` path mangling for out-of-project outputs, and "changed by an external edit" is distinguished from "no sync point recorded"
- Reproducible builds: theme major/minor fonts no longer depend on hash-map iteration order
- Table borders: an unspecified edge is no longer written as `<a:lnX><a:noFill/>`, which used to erase the neighbouring row's bottom border
- `validate_pptx.py --strict` status line now matches its exit code (PASS / WARNINGS / FAIL); title detection no longer reports the first text box on a slide
- `pptd build` no longer refuses to build because of warnings (animations, out-of-canvas elements); only errors block a build
- Node CLI `build` correctly forwards to the native binary; `package.json` no longer exposes a `pptd` bin that could shadow the native compiler
- Fixed the 404 cargo install URL and the relative install command in `reference/cli.md`

### Changed

- Icons ship as Font Awesome Solid (`fas:`) only; `far:`/`fab:` prefixes and unknown icon names now fail `pptd check` explicitly instead of silently ignoring the prefix
- Repository slimmed: `docs/` 83MB → 14MB, `example/` 42MB → 17MB (regenerable example outputs removed); theme previews now ship with the skill (`skills/open-pptd/theme_previews/`, ~2.5MB), fixing the broken preview links in an installed `theme.md`
- README now recommends `npx github:shiquda/open-pptd-skill install -y` when a full clone times out
- Timeline animations are still parsed but not emitted (`check` warns)

## [1.2.0] - 2026-08-06

### Added

- CLI supports `-h` / `--help` and `-V` / `--version`

### Changed

- Renamed the npm package and CLI from `open-kimi-ppt-skills` to `open-kimi-ppt-skill` to match the GitHub repo. Use `npx open-kimi-ppt-skill@latest` going forward; the old package name will no longer receive updates

## [1.1.3] - 2026-08-06

### Added

- Interactive `install` checklist for `.agents` / `.codex` / `.claude` / `.cursor` / `.workbuddy` skill directories (space to multi-select)
- `-y/--yes` (non-interactive default) and repeatable `--target`
- `--all` installs only into detected agent directories; missing agents are skipped with a notice instead of being created
- Windows export auto-starts a persistent debug browser (Chrome, falling back to Edge) to work around agent-browser failing to launch Chrome itself; the instance stays resident and is reused across exports, and `AGENT_BROWSER_CDP` can point to a debug browser you started yourself

### Fixed

- Tolerate files vanishing mid-scan in `find_download` when Chrome renames `.crdownload` entries in Downloads, avoiding `FileNotFoundError` aborting export (Related to #4)

### Changed

- README now steers agents to `npx open-kimi-ppt-skill@latest install -y` instead of cloning the repo

## [1.1.2] - 2026-08-06

### Fixed

- Fix Chinese-locale Windows export hang / GBK decode errors in `export_pptx.py` / `export_images.py` (capture stdout via temp file + UTF-8)
- Work around agent-browser `--download-path` silently canceling Chrome downloads: click download and poll the default Downloads folder
- Stop shipping `__pycache__/*.pyc` in the npm package (list script sources explicitly in `files`)

## [1.1.1] - 2026-08-06

### Added

- Align PPTD with official element-level `animations`; Skill notes for animation / `notes` usage bounds
- Align image priority, anti-AI copy rules, clarification asks, replicate guidance, and parallel page writes
- Ship ~30 preset design systems, invoked only when named
- Restore `customFonts` (Google Fonts) and poster size recommendations
- Root theme catalogs: `theme.md` / `theme_EN.md` (with preview images)
- Sample project `example/xiaomi-yu7-ppt-animation` (on-slide entrance animations)

### Changed

- Sync scenario docs with official animation guidance and `customFonts` references
- README: document element animations, preset themes, and sample prompts

## [1.0.2] - 2026-08-06

### Changed

- `install` overwrites an existing skill by default; `--force` is no longer required (still accepted for compatibility)

## [1.0.1] - 2026-08-06

### Added

- Skill workflow **step0 prerequisite check**: verify Node.js 18+, npm/npx, and python3 before generation; note that a Chromium-based browser is required for export
- Export scripts check **Node.js 18+** and **npm** at startup, with clear install guidance when missing or too old
- CLI (`open-kimi-ppt-skill`) refuses to start when the Node.js major version is below 18
- Auto-install **PyYAML** via `pip install --user pyyaml` when missing (same pattern as Pillow / websocket-client)

### Docs

- Added multi-agent / multi-model example screenshots (ChatGPT·Codex + 5.6 Luna, Reasonix + DeepSeek, WorkBuddy, and more)
- Clarified install as “automatic or manual — pick one”, with Windows path notes
- Updated README structure and example images

## [1.0.0] - 2026-08-05

### Added

- Initial release of `open-kimi-ppt-skill`
- PPTD create / edit / replicate, delivering both an editable PPTD project and a PPTX by default
- Browser-side PPTX export (embedded fonts, fade transitions) with optional multimodal visual QA before export
- Local in-browser PPTD editor (`npx open-kimi-ppt-skill serve`)
- CLI to install the skill into `~/.agents/skills` (or another agent directory via `--target`)
