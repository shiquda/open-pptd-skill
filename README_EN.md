# Open PPTD (`open-pptd`)

[简体中文](README.md) | [English](README_EN.md)

[![GitHub Release](https://img.shields.io/github/v/release/shiquda/open-pptd-skill?label=release)](https://github.com/shiquda/open-pptd-skill/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/shiquda/open-pptd-skill/actions/workflows/ci.yml/badge.svg)](https://github.com/shiquda/open-pptd-skill/actions/workflows/ci.yml)
[![Node >= 18](https://img.shields.io/badge/node-%3E%3D18-brightgreen)](https://nodejs.org)
[![Rust >= 1.85](https://img.shields.io/badge/rust-%3E%3D1.85-orange)](https://www.rust-lang.org)

**Open PPTD** is an open-source declarative presentation toolkit and AI Agent skill tailored for AI coding agents (Claude Code, Cursor, Codex, WorkBuddy, Windsurf, Multica) and developers. Built on the open **PPTD** (Declarative Presentation Language) YAML specification, it provides a **100% offline, sub-second native Rust compiler (`pptd` CLI)** and an out-of-the-box agent skill.

It completely eliminates brittle dependencies on web sessions, headless browsers, cookies, or cloud API tokens, delivering fully editable, vector-sharp native PowerPoint (`.pptx`) decks and round-trippable project directories directly on your local machine.

---

## ⚡ Quick Setup: Agent-First Installation (Recommended)

> [!TIP]
> **Zero Manual Environment Hassle!** We **strongly and exclusively recommend** letting your AI Coding Agent install this skill and follow its instructions to configure the native `pptd` compiler automatically.

### 📋 Copy and send this instruction to your AI Agent:

```text
Please install and set up the open-pptd presentation skill for me:
1. Run the skill installation command (if git clone is slow or times out, use the fallback command):
   npx skills add https://github.com/shiquda/open-pptd-skill -g -y
   # Fallback command (skips full git history clone):
   # npx github:shiquda/open-pptd-skill install -y
2. Run the skill installer script to ensure the native `pptd` compiler is ready (use the absolute path or cd to skill dir):
   python3 ~/.agents/skills/open-pptd/scripts/install_binary.py
3. Verify compiler: run `pptd --version` in terminal; if command not found, run `export PATH="$HOME/.local/bin:$PATH"`;
4. Notify me when setup is complete and ready.
```

> **Installation Command Notes**:
> - **Standard Ecosystem Command**: Use standard skill managers via `npx skills add https://github.com/shiquda/open-pptd-skill -g -y` (or `npx skills add shiquda/open-pptd-skill -g -y`). That command performs a full clone: the complete git history is ~110MB (the current working tree is ~31MB). If the clone times out under constrained networks, use the fallback command below.
> - **Direct npm/npx Compatibility (Recommended Fallback)**: Use `npx github:shiquda/open-pptd-skill install -y` to install directly and avoid full git clone timeouts.
---

## 💬 Generating Presentations with AI (Daily Usage)

Once configured, simply describe your presentation goals to your agent. **The agent always delivers both**: a fully editable PPTD source project folder and the compiled, production-ready `.pptx` file.

### 📋 Copy and send to your Agent:

```text
Use open-pptd to create an 8-slide keynote presentation on "Distributed Systems Architecture Evolution":
- Theme style: Dark Fintech (ebony-ledger) or Deep Blue Academic (deep-blue-atlas)
- Content requirements:
  1. Cover, architectural pain points, blueprint diagram (with arrowed connectors and vector cards), and performance metric comparison table;
  2. Every slide must include speaker rehearsal notes (notes: "...");
  3. Format algorithmic formulations using native LaTeX math;
- Deliverables: Complete PPTD project directory and compiled PPTX deck, verified with validate_pptx.py.
```

---

## 🖼️ Showcase

Open PPTD adheres to a dual-deliverable model: **editable PPTD source code + production-ready native PPTX**. From enterprise keynotes to academic defense slides, it consistently produces professional typography:

| Xiaomi YU7 Keynote Deck (Rich Editorial Layout) | DJI Pocket 4 Product Deck (Panoramic Visuals) |
| :---: | :---: |
| [![Xiaomi YU7 Presentation](docs/images/example-yu7-pptx.png)](docs/images/example-yu7-pptx.png) | [![DJI Pocket 4 Presentation](docs/images/example-dji-pocket4-pptx.png)](docs/images/example-dji-pocket4-pptx.png) |

| DeepSeek Liquid Glass Presentation | Minimal Starter Benchmark (Tables / Lines / Notes) |
| :---: | :---: |
| [![DeepSeek Liquid Glass](docs/images/example-deepseek-liquid-glass.png)](docs/images/example-deepseek-liquid-glass.png) | [![Starter Template](example/starter/starter_expected.png)](example/starter/starter_expected.png) |

---

## 💡 Why Open PPTD?

Common AI presentation generators suffer from major tradeoffs: programmatic OOXML builders easily misalign layouts, whole-page raster image generators cannot be edited in PowerPoint, and web HTML slides are not standard enterprise/academic PPTX files.

| Metric | **Open PPTD** | Programmatic OOXML (e.g. pptxgenjs) | Whole-Page Image PPT | Web HTML Slides |
|---|---|---|---|---|
| **Default Deliverables** | **PPTD Project + Native PPTX** | Usually PPTX only | Single raster image in PPTX | Single HTML file |
| **Agent Authoring** | **Declarative YAML**, clean AST | Verbose coordinates & brittle APIs | Prompt-dependent, unpredictable | Complex HTML/CSS constraints |
| **PowerPoint Editing** | **100% Vector Lossless**, editable text & shapes | Editable, but brittle layout | Bitmap image; text cannot be edited | Not a native PPTX file |
| **Build Speed & Deps** | **<0.05s** (Native Rust, 100% offline) | 1 ~ 5s (Node.js runtime) | 10 ~ 30s (Model queue) | Requires browser runtime |
| **Math Typography** | **Native OOXML Math** (`<latex>` / `\(…\)` into editable equations with real fraction bars, radicals & operators; hard error on unsupported syntax) | Unsupported or requires plugins | Relies on raster text | MathJax / KaTeX rendering |
| **Security & Privacy** | **Zero network calls**, sandbox-safe | Local runtime | Sends slide content to cloud LLM | Usually requires web renderer |

---

## 📋 Features

- 📐 **DrawingML Vector Geometries**: Native support for `rect`, `roundRect`, `ellipse`, `triangle`, and `diamond` with stroke, fill, and drop shadows.
- 🔗 **Connector & Arrow Lines**: Relative vector coordinates `points: "0,0 dx,dy"` with start/end arrowheads (`triangle`, `stealth`, etc.).
- 🔢 **Native OOXML Math (`<a14:m>`)**: `<latex>…</latex>` and `\(…\)` compile to real editable Office Math equations with genuine fraction bars (`\frac`), radical roots (`\sqrt`), scripts (`^`/`_`), big operators (`\sum`, `\int`), and matrices. Standalone formulas render in display mode (centered); inline formulas also carry an `mc:Fallback` text rendering, so PowerPoint/WPS show an editable equation while readers without the `a14` math extension (LibreOffice among them) show the equivalent Unicode text instead of losing the formula. Unsupported LaTeX commands fail the build with a hard error rather than silently degrading.
- 📊 **Native Bi-directional Tables**: Full support for cell merging (`rowSpan`/`colSpan`), column width ratios, zebra striping, and round-tripping.
- 🗣️ **PowerPoint Speaker Notes**: Page-level `notes: "..."` attributes seamlessly generate standard OOXML `notesSlide` relationships.
- 🎨 **1422 Built-in Vector Icons**: Font Awesome 6.7.2 Solid vector dataset with automatic `fa-` prefix normalization and validation.
- 🔒 **`.sync.hash` Overwrite Protection**: The compiler tracks content hashes to prevent accidental overwrites between manual edits and automated builds.
- 🔤 **Cross-Platform Safe Fonts**: Intelligent Chinese and Latin font mapping prevents cursive/casual script fallbacks on Linux and Windows.


---

## ⚠️ Current Boundaries & Unsupported Features

To guarantee deterministic builds and avoid silent degradations, the following features are not currently supported in the compiler and are strictly intercepted during `pptd check` or `pptd build`:
- 📊 **Charts (`elementType: chart`)**: Native chart elements are not yet supported. Both `pptd check` and `build` reject them with an explicit error. Workaround: use native tables (`elementType: table`) or export charts to images via script and embed them as `elementType: image`.
- 🎬 **Animations (`animations`)**: Animation syntax is parsed and validated in AST, but the compiler does not yet emit OOXML `p:timing` blocks (🟡 parsed but not emitted). Decks render as clean, complete static slides.
- 🌐 **HTML Document & Extended Tags**:
  - HTML list tags (`<ul>`, `<ol>`, `<li>`) are not supported; write normal paragraphs with bullet prefixes (e.g. `1. first` or `• first`).
  - HTML hyperlinks (`<a>`) are not supported; write URLs as plain text or annotate sources in slide footers/notes.
  - HTML superscript/subscript tags (`<sup>`, `<sub>`) are not supported; use native LaTeX math (`<latex>x^2</latex>`, `\( H_2O \)`).
  - HTML headings and structural containers (`<h1>`~`<h6>`, `<table>`, `<div>`, `<img>`) are not supported inside rich text; use top-level PPTD elements instead.
- 🖼️ **Shape Image Fills**: Shapes only support solid (`type: solid`) and gradient (`type: gradient`) fills; image fills on shapes (`fill: {type: image}`) are rejected (`a:blipFill` is only supported on slide backgrounds). Use separate `image` elements for photos.
- 📐 **Out-of-Canvas Elements**: Element coordinates must reside within slide bounds; out-of-canvas boxes are caught by `pptd check` (error for entirely out-of-bounds, warning for partially out-of-bounds).
---

## 📁 PPTD Directory Structure

A canonical, self-contained PPTD project directory looks like this:

```text
my_presentation/
├── deck.pptd             # Manifest: slide size, theme tokens, typography, page list
├── pages/
│   ├── 01_cover.page     # Single slide AST (shapes, text, tables, connectors)
│   ├── 02_problem.page
│   └── 03_solution.page
└── assets/               # Local images and media assets
```

For detailed specifications and guides, see:
- [Agent Quickstart Guide (SKILL.md)](skills/open-pptd/SKILL.md)
- [PPTD v2 Specification (pptd.md)](skills/open-pptd/reference/pptd.md)
- [Pre-designed Theme Palettes (theme.md)](theme.md)
- [CLI Reference Manual (cli.md)](skills/open-pptd/reference/cli.md)
- [Typography & Font Guidelines (fonts.md)](skills/open-pptd/reference/fonts.md)

---

## 🛠️ Appendix: Manual CLI Installation & Reference (Advanced Debugging)

<details>
<summary>Click to expand: Standalone CLI installation without an agent</summary>

If you need to invoke the `pptd` compiler directly in your terminal for debugging:

### 1. Manual CLI Installation
```bash
# Method A: Automatic download script to ~/.local/bin/pptd
curl -fsSL https://raw.githubusercontent.com/shiquda/open-pptd-skill/main/skills/open-pptd/scripts/install_binary.sh | bash

# Method B: Cross-platform Python installer
python3 skills/open-pptd/scripts/install_binary.py

# Method C: Install from source using Cargo
cargo install --git https://github.com/shiquda/open-pptd-skill.git
```

### 2. Core CLI Commands
- **Static Diagnostics**: `pptd check my_project/deck.pptd`
- **Native PPTX Build**: `pptd build my_project/deck.pptd -o presentation.pptx`
- **Decompile PPTX**: `pptd convert presentation.pptx ./decompiled_project/`
- **Automated QA**: `python3 skills/open-pptd/scripts/validate_pptx.py presentation.pptx --expected-pages 8 --check-notes`

</details>

---

## 📄 License & Attribution

This project is licensed under the [MIT License](LICENSE).

### Acknowledgements & Upstream Projects:
1. **`open-kimi-ppt-skill`**:
   Evolved and refactored from `open-kimi-ppt-skill` by **Binaryify Zhuang** (MIT License, [GitHub](https://github.com/acnlie/open-kimi-ppt-skill)). We gratefully acknowledge Binaryify's foundational contribution to the PPTD declarative DSL and AI agent presentation skills.
2. **`slideforge` (Native Rust Compiler Engine)**:
   The underlying native Rust compiler engine was originally created by **meatRoll** ([`meatRoll/slideforge`](https://github.com/meatRoll/slideforge), MIT License). Further secondary development by shiquda and contributors added rich text formatting, LaTeX math typography, PowerPoint speaker notes (`notesSlide`), expanded Font Awesome icon datasets, and strict semantic validation, before being unified into Open PPTD under the `pptd` CLI. We gratefully acknowledge meatRoll's original engineering.
3. **Font Awesome Free 6 (Solid Icons)**:
   Vector icons embedded in the compiler are sourced from Font Awesome 6.7.2 under the CC BY 4.0 license ([License](https://fontawesome.com/license/free)).
