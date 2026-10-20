# 09 · Kimi K3 Showcase Reproduction — Controlled Nuclear Fusion Industry Research

A 16-page, 16:9 (960 × 540 pt) consulting-research deck reproducing the structure,
typographic register and information density of the Kimi K3 showcase deck
"受控核聚变行业研究", built entirely in PPTD v2 and compiled with `pptd` 1.4.1.

## 1. Design brief

| Item | Decision |
| --- | --- |
| Scenario | Industry / technology research readout for an investment-committee and policy audience |
| Audience | Institutional investors, utility strategy teams, industrial policy analysts |
| Canvas | 960 × 540 pt (16:9), 10 pt = 1 unit |
| Theme | Custom inline theme in `deck.pptd` (colours + `textStyles` + `tableStyles`) |
| Reference design system | `skills/open-pptd/reference/design_system/consulting/marine-blue-research/design.md` (palette + typographic register adopted; the preset's palette is followed closely, its layout skeletons were re-derived by hand) |
| Display face | 思源宋体 (Source Han Serif) — titles, serif display register |
| Body face | MiSans — kickers, subtitles, body, captions |

### Palette

| Token | Hex | Use |
| --- | --- | --- |
| `ink` | `#0F172A` | primary body text |
| `titleNavy` | `#00295F` | page titles, card header bars |
| `accentBlue` | `#0064BC` | second-tier headers, accents |
| `cyanAccent` | `#00ACEE` | chart series, tertiary accents |
| `orangeAccent` | `#F17E00` | third-tier headers, watch-indicator rails |
| `panelBg` | `#F1F5F9` | synthesis strips, watch-indicator bands |
| `cardBg` | `#F8FAFC` | card bodies, alternating table rows |
| `border` | `#CBD5E1` | hairlines, card outlines |
| `borderDark` | `#94A3B8` | table outer rules |
| `textMuted` | `#475569` | secondary body |
| `textLight` | `#64748B` | captions, source lines |
| `successGreen` | `#15803D` | positive claim verdicts |
| `tagRed` | `#DC2626` | falsification / negative verdicts |

### Grid & typography decisions

* Single left margin at `x = 50`, right margin at `x = 910` (860 pt content width) on every page.
* Fixed header stack: kicker `y = 26` (h 16) → title `y = 42` (h 30) → subtitle `y = 74` (h 16) → hairline divider `y = 92`.
* Fixed footer stack: synthesis strip `y = 472` (h 44) → footer hairline `y = 514` → source line left / `Slide NN / 16` right at `y = 520`.
* Body region `y = 106 … 466` (360 pt) is occupied on every body page: cards (`y = 106`, h 268) plus a
  **watch-indicator band** (`y = 384`, h 74) carrying three or four quantitative monitoring metrics.
  The band is the mechanism that guarantees no empty region in the body area.
* Body-page titles are declarative takeaways ("Divertor Heat Flux Rivals Rocket Nozzles…"), never section labels.
* Type scale: title 19 pt serif semibold, kicker 8.5 pt caps with letterspacing, subtitle 9.5 pt, card header 9.5 pt caps,
  card body 8.1 pt / 1.28 line-height, caption 7.5 pt, source 7.5 pt.

### Evidence discipline

Every figure on every page carries a source line naming an organisation and a year
(IAEA, UKAEA/JET, LLNL, Max Planck IPP, FIA, ITER, US NRC, EPRI, IOP, CFS/MIT PSFC…).
Values that are forward-looking projections rather than measurements are labelled
"Pilot Target", "NOAK Target", "claimed", "~" or "示意" in the surrounding sentence.
No statistic in this deck is invented: where a number could not be attributed it was
replaced by a qualitative statement (see §4 for the one place where a chart band is
explicitly annotated as a modelled projection rather than data).

## 2. Syntax-coverage checklist

| PPTD feature | Where | Notes |
| --- | --- | --- |
| `version: v2`, `size`, `theme.colors`, `theme.textStyles`, `theme.tableStyles` | `deck.pptd` | 16 colours, 6 text styles, 1 table style |
| `pages:` manifest with external `pages/*.page` | all | 16 files, one page per file |
| `pageType: content` | 02–16 | |
| `notes:` speaker notes | 01–16 | validator `--check-notes` clean |
| `elementType: text` + `content.style` / `fontSize` / `color` / `align` | all pages | 200 text elements |
| HTML subset `<p> <b> <i> <br>` | all pages | |
| `elementType: shape` — `rect` | 01–16 | card bodies, header bars, divider rails |
| `elementType: shape` — `ellipse` | 01 | cover schematic (plasma cross-section) |
| `elementType: shape` — `arc` | 01 | cover schematic (toroidal field coils) |
| `elementType: shape` — `donut` | 01 | cover schematic (vacuum vessel ring) |
| `elementType: shape` — `diamond` | 13 | milestone-gate markers on the scenario timeline |
| `elementType: line` with `viewBox` + `points` + arrowheads | 01–16 | 33 connector/arrow elements (timeline arrows, gate leads, tree connectors) |
| `elementType: image` (`src`, fit within `bounds`) | 04, 08, 12 | 3 matplotlib PNGs, 300 dpi |
| `elementType: table` (`columnWidths`, `rowHeights`, `rows`, `rowSpan`, `colSpan`, per-cell `fill`, per-cell `border`) | 07, 14 | two dense native tables, incl. merged header cells and alternating row fills |
| `tableStyles` style reference (`denseConsulting`) | 07, 14 | navy header row, `#F8FAFC` alternating body rows, thin rules |
| Inline LaTeX `\( … \)` inside `<p>` (mixed with text) | 03, 09, 10, 13 | D-T reaction, Lawson triple product, `Q_plasma` / `Q_eng`, TBR, `P_fus ∝ β²B⁴V`, `J_c` degradation |
| `&amp;` / `&gt;` / `&lt;` / `&ge;` entity handling | 03, 07, 10, 11, 12, 13, 15 | |
| Unicode superscripts for nuclides (`⁶Li`, `⁷Li`, `⁴He`, `10²²`) | 04, 10 | avoids LaTeX isotope workaround |
| `elementId` on every element | all | required by the compiler |

Not used (deliberately or unavailable): `elementType: chart` (documented unsupported),
`elementType: icon`, `animations`, `shapeName: custom` + `viewBox` custom paths,
`cropShape`, gradients, shadows.

## 3. Verification evidence

All commands run from the repository root.

```
# 1. static check
$ pptd check open-pptd-skill/tests/decks/09-kimi-fusion-industry/deck.pptd
check passed with 7 warning(s)
# stderr, x7: "- page N: warning: potential text overflow in `title`: estimated text
#              height ~45pt exceeds box height 30pt by 51%"   (N = 2,3,4,5,6,8,10)

# 2. build
$ pptd build open-pptd-skill/tests/decks/09-kimi-fusion-industry/deck.pptd \
      -o open-pptd-skill/tests/decks/09-kimi-fusion-industry/build/deck.pptx --force
wrote .../build/deck.pptx
  sync point recorded: ... = 4c9041f35efffe3514a7cc00569fdade24ec944acdf34eb76989714fa69f1701

# 3. validate + render
$ .venv/bin/python open-pptd-skill/skills/open-pptd/scripts/validate_pptx.py \
      open-pptd-skill/tests/decks/09-kimi-fusion-industry/build/deck.pptx \
      --expected-pages 16 --check-notes \
      --render-images open-pptd-skill/tests/decks/09-kimi-fusion-industry/build/preview
Status:       [PASS]
Slides:       16
Errors:       0
Warnings:     0
```

Observed results: 16 slides, 0 validator errors, 0 warnings, speaker notes present on
all 16 pages, no leaked HTML tags. `build/preview/slide-01.png … slide-16.png` plus
`contact-sheet.jpg` were generated and every page was inspected at full size
(2001 × 1125 px) — not from thumbnails.

## 4. Defects and rough edges

### D1 — `pptd check` reports 7 false-positive title-overflow warnings

**Symptom.** Seven body pages carry a title that renders on a single line in the
exported PPTX, yet `check` claims a 51% overflow.

**Repro.** Any page with a 19 pt serif title inside `bounds: [50, 42, 860, 30]`.

```
$ pptd check open-pptd-skill/tests/decks/09-kimi-fusion-industry/deck.pptd
check passed with 7 warning(s)
- page 2: warning: potential text overflow in `title`: estimated text height ~45pt exceeds box height 30pt by 51%
```

**Observed.** The warning fires for 7 of the 16 pages (2, 3, 4, 5, 6, 8, 10) and not
for the others, although all 16 titles use identical geometry. The estimated height
(~45 pt) is independent of the actual string length — a 40-character title and an
84-character title both estimate 45 pt. LibreOffice rendering shows every title on
one line with no clipping.

**Suspected layer.** Compiler (`check` heuristic). **Severity.** Low (cosmetic/noise),
but it prevents a genuinely "clean" `pptd check`, and the estimator appears to ignore
the actual glyph advance, so it cannot be silenced by legitimate tuning.

### D2 — `elementType: chart` is rejected with a schema error, not the documented "unsupported" error

**Symptom.** `reference/pptd.md:1044` states that `pptd check` "actively intercepts
chart elements and reports an error" and that `build` raises
`Error::Unsupported("element type 'chart'")`. Neither happens: the element is
deserialised against the *full* (future) chart schema first, so the user gets a
generic missing-field error instead.

**Repro.**

```yaml
- elementId: e1
  elementType: chart
  bounds: [40, 40, 500, 300]
  data: { cols: [a, b], rows: [[1, 2]] }
```

```
$ pptd check /tmp/pptd-repro-chart/deck.pptd
error: failed to parse /tmp/pptd-repro-chart/pages/a.page as PPTD: elements: missing field `series` at line 4 column 3
```

Omitting `data` entirely yields `missing field `cols``; adding `cols` yields
`missing field `series``. **Observed:** the documented `Unsupported` message is
unreachable for an incomplete chart; the user is led to believe the chart is *almost*
valid and only a field away from working.

**Suspected layer.** Compiler (parse order) vs. skill docs (docs promise a dedicated
error). **Severity.** Medium — a documented hard-error path is unreachable and the
substitute message actively misleads.

### D3 — `size:` accepts only the tuple form, but the map form fails with a Rust-serde error

**Repro.**

```yaml
version: v2
size: { width: 960, height: 540 }
```

```
$ pptd build deck.pptd -o out.pptx --force
error: failed to parse deck.pptd as PPTD: size: invalid type: map, expected a tuple of size 2 at line 2 column 7
```

**Observed:** the message leaks the Rust type name (`tuple of size 2`) instead of
naming the expected PPTD syntax (`size: [width, height]`). **Suspected layer.**
Compiler (error presentation). **Severity.** Low — self-correcting after one attempt,
but it is the first error a new author hits.

### D4 — `pptd build` prints no per-page progress or summary; the only build-time signal is the sync-point hash

**Observed.** A successful 16-page build emits exactly two lines: `wrote …` and
`sync point recorded: …`. There is no page count, no element count, and no indication
that the `notes:` blocks were carried through. The only way to confirm notes survived
is to run the Python validator. **Suspected layer.** CLI ergonomics. **Severity.** Low.

### D5 — `--render-images` never overwrites existing preview PNGs

**Repro.** Run the validator twice with the same `--render-images DIR` after changing
the deck. The second run prints `Generated contact sheet: …` but `slide-NN.png` keeps
the first run's pixels; the contact sheet is rebuilt from the stale PNGs, so both the
thumbnails and the sheet show the *previous* revision.

**Observed.** Deleting the directory first (`rm -rf build/preview/*`) is required for
every re-review. **Suspected layer.** Validator script. **Severity.** Medium for the
review loop — silently reviewing stale output is an easy way to sign off on a broken page.

### D6 — Display-math fallback: hypothesis tested and **disproven**

An earlier draft of this deck converted standalone display math
(`<p>\( … \)</p>` on its own line, which the compiler emits as
`<a14:m><m:oMathPara>`) into inline math, on the theory that the `oMathPara` path
lacked an `mc:AlternateContent`/`mc:Fallback` wrapper and would therefore render as a
blank box in LibreOffice. A minimal repro with one inline and one display equation
was built to confirm this:

```
$ pptd build /tmp/pptd-repro/deck.pptd -o out.pptx --force
$ unzip -p out.pptx ppt/slides/slide1.xml | grep -c AlternateContent   # → 2
$ unzip -p out.pptx ppt/slides/slide1.xml | grep -c oMathPara          # → 2 (1 paragraph)
$ unzip -p out.pptx ppt/slides/slide1.xml | grep -c '<m:oMath>'        # → 2
```

**Observed:** *both* paragraphs carry one `mc:AlternateContent` each — the display
path is not defective. The hypothesis was wrong; the inline-with-bold-label form was
kept anyway because it reads better in a consulting layout, not as a workaround. No
defect is claimed here.

## 5. Known limitations of this reproduction

* The Kimi benchmark's cover uses a full-bleed photographic plate; this reproduction
  uses a vector schematic (ellipse / arc / donut / line) instead, because external
  imagery may not be downloaded.
* Residual bottom whitespace of roughly 15–25% remains inside the taller cards on
  pages 2, 3, 6, 9, 11, 15 and 16 after the final geometry pass; tightening further
  would have required padding the card copy beyond what is defensible.
* Charts are matplotlib rasters (`elementType: image`), not native PPTX chart parts —
  required, since `elementType: chart` is unsupported.
