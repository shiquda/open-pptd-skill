# 01-analysis-industry-research — 全球储能电池（BESS）产业链格局与中国厂商机会

**10-page reading-type Analysis & Decision deck for an investment committee · 960×540 (16:9) · pptd v1.4.1**

---

## 1. Design Brief

- **Scenario**: `analysis-decision`（`reference/slides_categories/analysis-decision.md`）— reading-type deck; every title is a conclusion sentence; high density; data + structural diagrams as argument body.
- **Audience**: 投资决策委员会 (Investment Committee) — venture/PE members evaluating China BESS supply-chain investment opportunities.
- **Preset design system**: `consulting/indigo-due-diligence` — source file:
  `skills/open-pptd/reference/design_system/consulting/indigo-due-diligence/design.md`
  (also mirrored at `~/.agents/skills/open-pptd/reference/design_system/consulting/indigo-due-diligence/design.md`)
- **Palette** (verbatim from design.md PART B):
  - Background `#FFFFFF`; judgment panels `#F4F5F7` (cool-gray ladder instead of `#E5E6E5`, single-step substitution documented in deck.pptd `panelBg`)
  - Text/titles `#000000` / `#041C2C` (structural dark: table headers, dark label bars, numbered marks)
  - Primary accent `#2151FF`; secondary series `#00A9F4`; supplementary `#282C70`, `#99C4FF`, `#B2B3B2`; negative `#E4536C`; tint `#EEF2FF`; hairlines `#D1D5DB`/`#E5E7EB`
- **Typography**: serif titles `{ea: 思源宋体, latin fallback}` (bold, 16.5–26pt), sans body/data `MiSans` 8–10.5pt, footnotes 8pt — matches the preset's serif-title + sans-body contract.
- **Grid**: 40px left/right margins; 42px title block; hairline rule at y=89; content 98–496; fixed footer at y=514 (source left, page number right, identical coordinates on every body page). Section navigation across the top at fixed x-positions (40/180/320/460); the **active-section indicator bar moves position** (40 → 180 → 320 → 460), tab text never changes color semantics beyond muted/active weight.

### Content grounding (all figures sourced; illustrative values flagged)
- Global BESS additions 2020–2030E: BNEF 2024 H1 Energy Storage Outlook + CNESA 全球储能数据库 2024 (P3 chart; 2024E–2030E are **forecast/示意值** per BNEF mid-case).
- Li2CO3 spot price & EPC bid curve: 上海有色网(SMM), 索比储能网 2024Q3 大储中标统计 (P7).
- Cell maker shipments 2023/2024E: InfoLink Consulting + company annual reports (P6 table); cycle-life / system figures from vendor white papers (宁德时代天恒、阳光电源 PowerTitan 2.0、比亚迪 MC Cube-T, P5).
- Regional policy: USTR 301 tariff schedule (2026 → 25%), IRA §48 ITC, EU Battery Regulation (EU 2023/1542), AEMO PSSE/PSCAD rules (P8).
- 2×2 quadrant mapping of named companies is the **analysis team's own framework judgment** (P9, labelled 模型来源).

---

## 2. Syntax Coverage Checklist (features actually used, with pages)

| Feature | Pages |
|---|---|
| `version: v2` manifest, `theme.colors`/`textStyles`/`tableStyles` with `$ref` | all (deck.pptd) |
| Native `elementType: table` with `columnWidths`/`rowHeights` | P6 (7 col × 8 row), P7 (LCOS sensitivity), P8 (6 col × 7 row) |
| Merged header cells `rowSpan`/`colSpan` (omitted covered cells) | P6 (5×rowSpan + 1×colSpan), P8 (4×rowSpan + 1×colSpan) |
| 4-element `border` BorderSpec `[top, right, bottom, left]` hairline rules on cells | P6, P7, P8 (every cell uses `[{...}, null, {...}, null]`) |
| Dark header fill `#041C2C` + white bold text; right-aligned numeric columns | P6, P7, P8 |
| Per-cell `fill` for emphasis rows (tint `#EEF2FF`) | P6 (CATL/BYD rows), P7 (current-price row), P8 (US/EU rows) |
| `elementType: line` with `arrow: [null, "triangle"]` | P4 (driver-tree connectors), P5 (timeline backbone), P9 (axes), P10 (roadmap) |
| `elementType: line` with `arrow: [null, "stealth"]` | P4 (value-chain pipeline) |
| Relative-coordinate `points` (`"0,0 dx,dy"` and `"0,0 0,dy"`) | P4, P5, P9, P10 |
| Multi-point polyline path + `viewBox` | P4 (smile curve, 5 points), P5 (cycle-life curve, 4 points) |
| Shapes with `adjustments`: `hexagon` [20000/18000, 115470] | P4 (4-stage pipeline) |
| Framework diagram from `rect` + `line` (2×2 investment matrix, axes as arrowed lines) | P9 |
| `letterSpacing` | P1 (kicker 1.2), P2–P10 (nav tag 0.8) |
| `lineHeight` (multiple) | P2, P6, P7, P9, P10 (1.3–1.45) |
| `lineHeightPx` (fixed) | P4 (14px), P5 (14px) |
| `align: [h, v]` on cells + `<p style="text-align:center/right">` | P6, P7, P8, P9, P10 |
| `<span style="color/font-size/font-weight">` inline overrides | every page |
| `marginTop` on `<p style="margin-top: …">` | P1–P10 body blocks |
| `fit: { mode: contain }` on images | P3, P7 |
| Fixed footer (source + page number, identical coordinates) | P2–P10 |
| Section navigation highlight moving by position | P2–P10 (bar at x=40/180/320/460) |
| `notes:` speaker notes on every slide | P1–P10 |
| Raster charts generated locally (matplotlib → `assets/*.png`) | P3, P7 |

---

## 3. Verification Evidence

```bash
# 1. Syntax check
pptd check tests/decks/01-analysis-industry-research/deck.pptd
# → "check passed with 35 warning(s)"  (all 35 are text-overflow FALSE POSITIVES — see defect #3)

# 2. Compile
pptd build tests/decks/01-analysis-industry-research/deck.pptd \
    -o tests/decks/01-analysis-industry-research/build/deck.pptx --force
# → "wrote .../build/deck.pptx" (exit 0)

# 3. Structural validation
<venv>/python skills/open-pptd/scripts/validate_pptx.py \
    tests/decks/01-analysis-industry-research/build/deck.pptx \
    --expected-pages 10 --check-notes
# → Status: [PASS]  Slides: 10  Errors: 0  Warnings: 0
# → All 10 slides [Notes: Yes]; no leaked HTML tags; no boundary errors.

# 4. Headless render + full-size visual review
<venv>/python skills/open-pptd/scripts/validate_pptx.py \
    tests/decks/01-analysis-industry-research/build/deck.pptx \
    --render-images tests/decks/01-analysis-industry-research/build/preview
# → build/preview/slide-01.png … slide-10.png + contact-sheet.jpg
# → Every PNG reviewed at full size; issues found and fixed in-loop:
#    • chart PNGs initially cropped by default cover-fit → switched to fit:contain + matched aspect
#    • "&ge;"/"&le;" rendered literally → replaced with ≥/≤ characters
#    • page-04 smile-curve peak label overlapped title; rebalanced banner geometry
#    • page-05 TODAY marker sat on the timeline line → repositioned gate/tag
#    • page-09 quadrant-II rating line wrapped awkwardly → widened text boxes, shortened rating
#    • page-01/10 planned boxes had empty lower regions → cover pillars switched to rule+text
#      (consulting no-card rule) and page-10 pillars shrunk + Execution Roadmap strip added
# Re-rendered after each fix; final render shows no overflow/overlap/off-canvas elements.
```

---

## 4. Defects / Rough Edges Found (each with minimal repro)

### #1 · CRITICAL — `curve: smooth` emits mismatched XML tag (`</a:cTo>` instead of `</a:cubicBezTo>`) → corrupt PPTX
**Symptom**: `pptd check` says "no issues" and `pptd build` exits 0, but the produced PPTX is corrupt; only the external validator catches it.

Minimal repro (`/tmp/pptd_smooth/deck.pptd` + `pages/t.page`):
```yaml
- elementId: c
  elementType: line
  bounds: [100, 100, 400, 100]
  viewBox: [400, 100]
  points: "0,80 100,0 300,100 400,20"
  curve: smooth
  border: { style: solid, width: 2, color: "#2151FF" }
```
Commands: `pptd check …` → `no issues`; `pptd build … -o o.pptx --force` → `wrote o.pptx`;
`validate_pptx.py o.pptx --expected-pages 1` →
`[ERROR] Slide 1: Corrupt slide XML in ppt/slides/slide1.xml: mismatched tag: line 45, column 18`.
Raw `ppt/slides/slide1.xml` contains:
```xml
<a:cubicBezTo>
  <a:pt x="1270000" y="0"/> …
</a:cTo>          <!-- wrong closing tag -->
```
**Suspected layer**: compiler — `src/pptx/render.rs:2001` does `xml.end("a:cTo")` after `xml.start("a:cubicBezTo")` (`src/pptx/svg_path.rs:52` uses the correct `xml.end("a:cubicBezTo")`). LibreOffice tolerates it (renders fine), python-pptx/OOXML parsing does not.
**Severity**: critical (silent corruption of documented feature). **Workaround used in this deck**: `curve: round` (polyline).

### #2 · MEDIUM — HTML entities `&ge;` / `&le;` render literally (not decoded); `&gt;`/`&lt;` work
**Symptom**: text `维度 1：核心部件自研率 &ge; 40%` renders as `维度 1：核心部件自研率 &ge; 40%` (literal ampersand-entity) on slides; same for `&le;`. The XML built-ins `&gt;`/`&lt;` decode correctly.
Minimal repro: any `content.text: "<p>阈值 &ge; 40%</p>"` → build → render → literal `&ge;` visible.
**Suspected layer**: compiler rich-text entity table only handles XML predefined entities, not the HTML named set; the skill docs never state which entities are legal (`pptd.md` uses `&gt;` in examples but never mentions `&ge;` is unsupported). **Severity**: medium (silent visual corruption; `check` does not warn).
**Workaround used**: literal Unicode characters `≥` / `≤`.

### #3 · LOW-MEDIUM — `pptd check` text-overflow estimator ignores `<span style="font-size: …">` inline overrides → 35 false-positive warnings on this deck
**Symptom**: this deck produced 35 "potential text overflow" warnings; every single one is false (verified at full-size render — no overflow anywhere). Root cause isolated:
```yaml
# repro page (/tmp/pptd_ovf)
- elementId: w170              # inline span font-size → WARN (wrong)
  elementType: text
  bounds: [40, 40, 170, 20]
  content:
    text: "<p style=\"text-align: center;\"><span style=\"color: #64748B; font-size: 8pt;\">原材料加工 (10-15%)</span></p>"
- elementId: w170-8pt-field    # same text, fontSize field → no warn (correct)
  elementType: text
  bounds: [40, 160, 170, 20]
  content:
    text: "<p>原材料加工 (10-15%)</p>"
    fontSize: 8
```
`pptd check` → `warning: … w170 … estimated text height ~49pt exceeds box height 20pt by 145%` (only the inline-span element warns). The estimator falls back to the default 18pt when font size is set via `<span style>`, wraps the string into 2 lines and reports 49pt. 
**Suspected layer**: compiler `check` (text metrics path reads element-level `fontSize` only, skipping inline `<span style="font-size">` overrides). **Severity**: low-medium (noise trains users to ignore warnings; a real overflow would drown in them).

### #4 · MEDIUM — default image fit (`cover`) silently crops chart PNGs
**Symptom**: `elementType: image` with bounds aspect ≠ source aspect cropped off the matplotlib charts' Y-axis labels/annotations (Slide 3 & 7 left/right axes were missing in first render). `SKILL.md` matrix mentions `fit: {mode: cover/contain}` but does not state the default; `pptd.md` says `cover` "may crop".
Repro: any 7.2×4.6-inch chart PNG in `bounds: [40,98,540,398]` without `fit:` → cropped edges.
**Suspected layer**: compiler default + docs gap (default mode undocumented in SKILL.md). **Severity**: medium. **Workaround used**: `fit: { mode: contain }` + matplotlib `figsize` matched to the bounds aspect ratio.

### #5 · OBSERVATION — `points` coordinates outside `viewBox` are not rejected
`pptd.md` constrains "all coordinates must be within viewBox", but neither `check` nor `build` was seen to enforce it during debugging of curve bounds (large coordinates silently render/clamp). **Suspected layer**: compiler validation gap. **Severity**: low. *(Flagged as observed gap; a dedicated negative test was not kept.)*

---

## 5. Assets
- `assets/global_bess_additions.png` — stacked-column regional additions chart (P3), generated by `generate_assets.py`
- `assets/bess_cost_curve_and_lithium.png` — dual-axis Li2CO3 vs EPC bid chart (P7), generated by `generate_assets.py`
- `build/deck.pptx`, `build/preview/slide-01..10.png`, `build/preview/contact-sheet.jpg`
