# 10a: GWTC-5 Gravitational-Wave Catalog Showcase Deck

## 1. Design Brief
- **Scenario & Audience**: Academic research & scientific astrophysics overview reproducing the Kimi K3 showcase benchmark (`refs/gwtc5-cover.jpg`). Target audience: astrophysicists, research peers, funding reviewers, and graduate seminar students.
- **Chosen Theme & Preset**: Derived from `academic/paper-white-courseware` (`skills/open-pptd/reference/design_system/academic/paper-white-courseware/design.md`).
- **Palette**:
  - Paper Background: `#FFFFFF` / Slate Hairlines `#E2E8F0` / `#CBD5E1`
  - Deep Ink: `#0F172A` / Muted Text: `#475569`, `#64748B`
  - Science Primary (Interferometer Blue): `#2563EB`
  - Secondary Highlight (Astrophysical Teal): `#0D9488`
  - Exotic / Breaker Accent (Crimson Red): `#DC2626`
  - Gold / Gap Accent (Amber): `#D97706`
- **Grid & Typography**:
  - Dimensions: 960 × 540 pt (16:9 widescreen standard).
  - Left Rail (x: 0–160 pt): 4 visual navigation thumbnail cards previewing key physical epochs.
  - Main Canvas (x: 185–605 pt): Hero architectural schematics, global observatory tables, and empirical matplotlib scientific plots.
  - Right Rail (x: 625–925 pt): Monospace/LaTeX formula cards, metric callouts, and physical interpretations.
  - Footers (y: 452–510 pt): Bold one-line `Takeaway · ...` conclusion followed by formal peer-reviewed citation lines and fixed page numbers.
  - Typography: `Liter` screen-optimized neo-grotesque sans-serif with fallback to DejaVu Sans / Arial.

## 2. Syntax-Coverage Checklist
- Manifest v2 & Theme textStyles / color inheritance (`deck.pptd`)
- Element Types:
  - `elementType: shape` (rect, ellipse, custom adjustments) — Pages 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
  - `elementType: line` (multi-point bezier with `curve: round`, arrows) — Pages 1, 10
  - `elementType: text` (rich text `<b>`, `<i>`, `<span>`, `<br/>`, `<p>`) — Pages 1–11
  - `elementType: image` (local matplotlib rasters, `fit: { mode: contain }`) — Pages 1–11
  - `elementType: table` (styled cells, custom row heights, columnWidths, firstRowStyle) — Pages 2, 8
- Mathematical Expressions:
  - Inline LaTeX (`\( ... \)`) — Pages 1, 4, 5, 6, 7, 9
  - Standalone / Display LaTeX (`\( ... \)` in dedicated formula text blocks) — Pages 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
- Navigation & Structure:
  - Consistent multi-slide navigation rail (fixed geometry across all 11 pages) — Pages 1–11
  - Fixed Takeaway and formal source attribution footers — Pages 1–11
  - Speaker notes on every page (`notes:`) — Pages 1–11

## 3. Verification Evidence
```bash
# 1. Compiler syntax & schema check
pptd check tests/decks/10a-gwtc5-gravitational-wave/deck.pptd
# Result: open-pptd-skill/tests/decks/10a-gwtc5-gravitational-wave/deck.pptd: 11 page(s) validated — no issues (1.36s)

# 2. Native compilation to PowerPoint PPTX
pptd build tests/decks/10a-gwtc5-gravitational-wave/deck.pptd -o tests/decks/10a-gwtc5-gravitational-wave/build/deck.pptx --force
# Result: wrote tests/decks/10a-gwtc5-gravitational-wave/build/deck.pptx

# 3. PPTX validation suite & notes inspection
.venv/bin/python skills/open-pptd/scripts/validate_pptx.py tests/decks/10a-gwtc5-gravitational-wave/build/deck.pptx --expected-pages 11 --check-notes
# Result:
# Status:       [PASS]
# Slides:       11
# Errors:       0
# Warnings:     0
# Slide Overview:
#   Slide  1: [Notes: Yes]   (30 text, 4 img, 0 tbl)
#   Slide  2: [Notes: Yes]   (19 text, 4 img, 1 tbl)
#   Slide  3: [Notes: Yes]   (21 text, 4 img, 0 tbl)
#   Slide  4: [Notes: Yes]   (15 text, 5 img, 0 tbl)
#   Slide  5: [Notes: Yes]   (15 text, 5 img, 0 tbl)
#   Slide  6: [Notes: Yes]   (15 text, 5 img, 0 tbl)
#   Slide  7: [Notes: Yes]   (15 text, 5 img, 0 tbl)
#   Slide  8: [Notes: Yes]   (19 text, 4 img, 1 tbl)
#   Slide  9: [Notes: Yes]   (17 text, 5 img, 0 tbl)
#   Slide 10: [Notes: Yes]   (21 text, 4 img, 0 tbl)
#   Slide 11: [Notes: Yes]   (22 text, 4 img, 0 tbl)

# 4. Headless rendering & visual verification
.venv/bin/python skills/open-pptd/scripts/validate_pptx.py tests/decks/10a-gwtc5-gravitational-wave/build/deck.pptx --render-images tests/decks/10a-gwtc5-gravitational-wave/build/preview
# Result: 11 slides rendered to PNG + contact-sheet.jpg generated.
# Read and inspected every PNG (slide-01.png through slide-11.png) via read tool.
```

## 4. Defects & Rough Edges Found
1. **XML Tag Mismatch in `curve: smooth` (`src/pptx/render.rs` line 2001)**:
   - *Symptom*: Building any line element with `curve: smooth` panics or generates corrupt PPTX that LibreOffice / Keynote cannot open due to an XML parse error.
   - *Repro*: Line with `curve: smooth`, e.g.:
     ```yaml
     - elementId: l1
       elementType: line
       bounds: [100, 100, 300, 200]
       points: "0,0 50,20 150,180 200,200"
       curve: smooth
     ```
   - *Root Cause*: Compiler source in `src/pptx/render.rs` emits `<a:cubicBezTo>` but terminates it with `</a:cTo>` instead of `</a:cubicBezTo>`.
   - *Workaround*: Use `curve: round` or `curve: sharp`.

2. **Inline LaTeX Combining Character Artifact (`\vec{...}` in text runs)**:
   - *Symptom*: An inline formula containing `\vec{...}` followed by a text run produces an unassigned glyph box (`▯`) overlapping the first character of the following text run.
   - *Repro*:
     ```yaml
     - elementId: t1
       elementType: text
       bounds: [40, 40, 600, 40]
       content:
         text: "<b>Delay:</b> Geometric timing \\( \\Delta t = \\vec{D}\\cdot\\vec{n}/c \\) defines circles."
     ```
   - *Root Cause*: Compiler's fallback branch emits combining right arrow above `U+20D7`. When rendered through headless LibreOffice, the combining accent glyph causes a character layout collision at the boundary of adjacent text runs.
   - *Workaround*: Avoid combining `\vec` in inline text runs; use upright or italic vector notation without arrows.

3. **Display Math Ignores Element Text Color (Emits Black XML)**:
   - *Symptom*: Standalone LaTeX display math (`<m:oMathPara>`) emits `<a:rPr>` without `<a:solidFill>`, causing math to render black (`#000000`) regardless of `color: "$var"` declared on the text element.
   - *Repro*: Any text element with `color: "#F8FAFC"` and a LaTeX formula block on a dark slide.
   - *Workaround*: In dark-theme decks, enclose math in high-contrast light sub-boxes or format equations using unicode rich text spans.
