# 10b: Urban Heat Island Microclimate Dynamics Infographic

## 1. Design Brief
- **Scenario & Audience**: Scientific environmental infographic reproducing the K3 dark-mode showcase benchmark (`refs/heat-island-cover.jpg`). Target audience: municipal sustainability officers, urban microclimate researchers, civil engineers, and environmental policy directors.
- **Chosen Theme & Preset**: Dark thermodynamic thermal scheme.
- **Palette**:
  - Deep Charcoal Canvas: `#0B0F19`
  - Elevated Container Cards: `#131C2E` / Border: `#1E293B`, `#334155`
  - Hot Thermal Peak: `#F97316` (amber orange) / `#EF4444` (extreme red)
  - Cool Thermal Sinks: `#38BDF8` (waterfront cyan) / `#10B981` (canopy green)
  - High Contrast Typography: `#F8FAFC` (pure white text), `#94A3B8` (slate muted)
- **Grid & Typography**:
  - Dimensions: 960 × 540 pt (16:9 widescreen).
  - Left Rail (x: 0–160 pt): 2 visual navigation thumbnail cards reflecting the active slide state (orange border on active, muted border on inactive).
  - Slide 1: 4 equal-width micro-zone columns (Commercial CBD, Residential Built-Up, Urban Forest Park, Waterfront Corridor) with synthetic 30m thermal grid maps, surface vs 1.5m air temperature discrepancy badges, and energy partitioning formula bar.
  - Slide 2: Left 3-card stack of physical thermodynamic heat traps (Albedo absorption, Canyon sky view factor, Anthropogenic emissions $Q_F$) + Right classroom microclimate audit field survey (native PPTD table).
  - Typography: `Liter` sans-serif throughout.

## 2. Syntax-Coverage Checklist
- Manifest v2 & Dark Theme Color Token System (`deck.pptd`)
- Element Types:
  - `elementType: shape` (rect containers, thermal zone metric badges, formula sub-boxes) — Pages 1, 2
  - `elementType: text` (rich text spans with custom colors, bolding, font sizes, line heights) — Pages 1, 2
  - `elementType: image` (synthetic 30m grid maps with `fit: { mode: contain }`) — Pages 1, 2
  - `elementType: table` (native styled table with `cellStyle`, `firstRowStyle`, custom column widths and row heights) — Page 2
- Microclimate Content & Epistemology:
  - Surface energy balance equation: $R_n = H + \lambda E + G + Q_F$ — Page 1
  - Physical drivers: Albedo ($\alpha$), Sky View Factor ($\text{SVF} = \cos \theta$), Anthropogenic flux ($Q_F$) — Page 2
  - Field audit table: 5 surface materials with real delta T measurements — Page 2
  - Explicit separation of empirical measurements, physical interpretations, policy interventions, and open research questions — Page 2
  - Speaker notes on every page (`notes:`) — Pages 1, 2

## 3. Verification Evidence
```bash
# 1. Compiler check
pptd check tests/decks/10b-urban-heat-island/deck.pptd
# Result: open-pptd-skill/tests/decks/10b-urban-heat-island/deck.pptd: 2 page(s) validated — no issues

# 2. Native compilation to PPTX
pptd build tests/decks/10b-urban-heat-island/deck.pptd -o tests/decks/10b-urban-heat-island/build/deck.pptx --force
# Result: wrote tests/decks/10b-urban-heat-island/build/deck.pptx

# 3. PPTX validation suite & notes inspection
.venv/bin/python skills/open-pptd/scripts/validate_pptx.py tests/decks/10b-urban-heat-island/build/deck.pptx --expected-pages 2 --check-notes
# Result:
# Status:       [PASS]
# Slides:       2
# Errors:       0
# Warnings:     0
# Slide Overview:
#   Slide  1: [Notes: Yes]   (22 text, 6 img, 0 tbl)
#   Slide  2: [Notes: Yes]   (17 text, 2 img, 1 tbl)

# 4. Headless rendering & visual verification
.venv/bin/python skills/open-pptd/scripts/validate_pptx.py tests/decks/10b-urban-heat-island/build/deck.pptx --render-images tests/decks/10b-urban-heat-island/build/preview
# Result: slide-01.png, slide-02.png, contact-sheet.jpg rendered.
# Read and inspected every PNG via read tool.
```

## 4. Defects & Rough Edges Found
1. **HTML Entity `&nbsp;` Not Decoded in Rich Text**:
   - *Symptom*: Writing `&nbsp;` inside text content leaks the literal string `&nbsp;` onto the rendered slide.
   - *Repro*: `text: "<span>A</span>&nbsp;<span>B</span>"`
   - *Observed*: Renders as `A&nbsp;B`.
   - *Workaround*: Use literal unicode non-breaking space or standard spaces.

2. **Shape Border Does Not Accept 4-Element Array**:
   - *Symptom*: Writing `border: [null, { style: solid, ... }, null, null]` on a `shape` element causes `pptd check` to fail with `invalid type: map, expected f64`.
   - *Root Cause*: PPTD distinguishes `Border` (single object for shapes/images/lines) from `BorderSpec` (4-side array for table cells).
   - *Workaround*: Use a single `Border` object on shapes, or use a line element for single-edge dividers.
