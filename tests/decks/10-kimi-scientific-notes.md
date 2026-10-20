# Kimi K3 Scientific Showcase Reproduction Notes: GWTC-5 & Urban Heat Island

## Executive Summary
This document synthesizes the implementation findings, layout strategies, and toolchain evaluation resulting from reproducing two flagship Kimi K3 scientific presentation decks using the native Rust `pptd` compiler (v1.4.1) and declarative PPTD YAML syntax:
1. `tests/decks/10a-gwtc5-gravitational-wave/`: 11-slide complete research deck (16:9, light academic paper-white preset, dual-recycled Michelson vector schematic, 5 matplotlib scientific figures, global network delay matrix, catalog growth, mass gap demographics, and open science workflow).
2. `tests/decks/10b-urban-heat-island/`: 2-slide thermodynamic environmental infographic (16:9, dark thermal scheme, 4-zone synthetic 30m grid maps, energy balance equation, and empirical classroom IR thermometer audit table).

---

## 1. Design & Architectural Reproduction Strategies

### 1.1 The Left Visual Navigation Rail
Both decks employ a persistent left-hand thumbnail rail (x: 0–160 pt, width: 134 pt cards) that grounds the viewer across dense scientific narratives.
- **Challenge**: PPTD does not have a native "master layout" or component template inheritance system. Every page file (.page) must declare the rail elements independently.
- **Implementation**: We standardized rail card geometries across all slides (`bounds: [13, 30 + 100*i, 134, 84]`).
- **Asset Fidelity**: Rail thumbnails were generated programmatically via matplotlib/Pillow with clean 100–200 DPI vector-like graphics and zero dark-band clipping.

### 1.2 Vector Schematics vs External Image Downloads
Per the project contract, zero external raster images were downloaded:
- **Slide 1 Michelson Interferometer**: Hand-crafted entirely from native PPTD shapes (`rect`, `ellipse`, `line` with round joins, and text labels). Test mass mirrors, power recycling cavity, signal recycling cavity, 50:50 beam splitter, photodetector dark port, and laser diode are fully editable PowerPoint vector shapes.
- **Scientific Plots**: Catalog growth bar charts, mass plane scatter plots with exclusion zones, Mollweide sky localization maps, cosmological luminosity distance siren curves, and spin demographics were synthesized via matplotlib in `generate_assets.py` with matched aspect ratios and tight margins.
- **Image Fit**: Chart elements declare `fit: { mode: contain }` inside matching aspect ratio bounding boxes, eliminating any edge clipping of axis labels or legends.

### 1.3 Epistemological Rigor in Scientific Decks
A core requirement was distinguishing empirical facts from interpretations and models:
- Every body slide incorporates a dedicated Epistemological Callout box classifying:
  - `[Catalog Fact]` / `[Empirical Observation]`: Verifiable measurements (e.g. 90 GWTC-3 events, 22.9°C delta T).
  - `[Visualization Choice]`: Projections, logarithmic scales, color mapping choices.
  - `[Physical Interpretation]`: Underlying astrophysics/thermodynamics (PISN, latent heat partitioning).
  - `[Open Question]` / `[Policy Implication]`: Unresolved frontiers and civic mitigation strategies.

---

## 2. Toolchain Defect Log & Reproductions

### Defect 1: Panicking XML Tag Mismatch in `curve: smooth`
- **Component**: Native Rust Compiler (`src/pptx/render.rs`, line 2001)
- **Severity**: High (generates invalid XML, breaks PPTX compilation/rendering)
- **Symptom**: Line elements declaring `curve: smooth` fail to parse or cause downstream XML validator crashes.
- **Root Cause**: The compiler emits an opening `<a:cubicBezTo>` tag but erroneously closes it with `</a:cTo>` instead of `</a:cubicBezTo>`.
- **Minimal Reproduction**:
  ```yaml
  - elementId: l1
    elementType: line
    bounds: [50, 50, 200, 200]
    points: "0,0 50,30 150,170 200,200"
    curve: smooth
  ```
- **Observed Behavior**: Office XML parse exception / element tag mismatch.
- **Workaround**: Use `curve: round` or `curve: sharp`.

### Defect 2: Combining Vector Arrow (`\vec{...}`) Produces Stray Glyph in Inline Text
- **Component**: Compiler LaTeX Fallback / LibreOffice Fallback Text Rendering
- **Severity**: Medium (visual defect on slides with inline physics formulas)
- **Symptom**: An inline formula containing `\vec{D}` produces a black unassigned glyph box (`▯`) overlapping the first character of the adjacent text run.
- **Minimal Reproduction**:
  ```yaml
  - elementId: t1
    elementType: text
    bounds: [40, 40, 600, 40]
    content:
      text: "<b>Delay:</b> Geometric timing \\( \\Delta t = \\vec{D}\\cdot\\vec{n}/c \\) defines circles."
  ```
- **Root Cause**: The compiler emits Unicode combining arrow `U+20D7` in the fallback `<a:t>` run. When parsed in text engines without complex text layout hooks, the zero-width combining accent triggers a font fallback missing-glyph box.
- **Workaround**: Avoid `\vec` in inline text runs; use bold or italic notation.

### Defect 3: LaTeX Math Ignores Element Text Color (Always Emits Black)
- **Component**: Compiler OMML Math Generator
- **Severity**: Medium-High (breaks dark-mode scientific decks)
- **Symptom**: Standalone and inline LaTeX formulas in `<m:oMathPara>` omit `<a:solidFill>` inside `<a:rPr>`, defaulting to document black (`#000000`).
- **Observed Impact**: On dark slides (`#0B0F19`), LaTeX formulas are nearly invisible.
- **Minimal Reproduction**:
  ```yaml
  - elementId: f1
    elementType: text
    bounds: [100, 100, 400, 40]
    content:
      color: "#F8FAFC"
      text: "\( R_n = H + \lambda E + G + Q_F \)"
  ```
- **Observed XML**: `<a:rPr lang="en-US"><a:latin typeface="Cambria Math"/></a:rPr>` (lacks `<a:solidFill>`).
- **Workaround**: Enclose LaTeX formulas in light-colored sub-boxes, or use styled unicode rich text spans on dark slides.

### Defect 4: HTML Entities Leaking in Text Content
- **Component**: Rich Text Parser
- **Severity**: Low
- **Symptom**: HTML entities like `&nbsp;` are not decoded and appear verbatim on slides.
- **Workaround**: Use standard spaces or Unicode characters directly.

### Defect 5: Shape Border Does Not Support 4-Side Array
- **Component**: Schema Validator
- **Severity**: Low
- **Symptom**: `border: [null, { style: solid, ... }, null, null]` fails validation on `elementType: shape` (only accepted on table cells).
- **Workaround**: Use single `Border` object or separate `line` elements.

---

## 3. Usability & Maintenance Recommendations
1. **Component Templates**: Adding support for reusable components (e.g. `$navRail`, `$formulaCard`) would dramatically cut down repetition across multi-slide decks.
2. **Auto-Fit for LaTeX Display Math**: Display math in small text boxes should either scale to fit the box or trigger an overflow warning in `pptd check`.
3. **Color Inheritance for OMML**: The compiler should propagate `textStyle.color` and element `color` into the `<a:rPr>` properties of `<m:oMath>` runs.
