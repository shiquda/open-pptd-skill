# 04 · Academic LaTeX Defense — 面向长上下文推理的稀疏注意力机制

PPTD stress-test deck: **LaTeX / Office-Math fidelity** on a thesis-defense narrative.
Canvas 960×540 (16:9), 10 pages, `version: v2`.

---

## 1. Design brief

**Scenario** — CS/ML doctoral dissertation defense (`reference/slides_categories/academic-research.md`,
"Tsinghua / MIT-Style Elite-University Defense" grammar).
**Audience** — defense committee (5 professors) + advisor; also readable as a conference talk deck.
**Narrative** — background → evidence gap → research question → method → derivation → core evidence
→ ablations → limitations → conclusions + references (the fixed order mandated by PART A).

**Preset used** — `reference/design_system/academic/teal-green-academic-defense/design.md`
(the ETH/Tsinghua "black text on white, teal structure" signature; chosen over
`paper-white-courseware` because that preset mandates a *photograph* and a portrait-derived
layout, and this deck is deliberately image-free except for generated charts).

**Palette (hex)** — background `#FFFFFF`; primary text `#000000`; structural accent `#009682` (teal,
≤10 % area); supporting `#68BCE0` (goal/framework), `#939FB5` (inactive), `#868686` (neutral rules),
`#F7F7F4` (near-white panels); focal marker `#F0A81D`; risk/error `#B51E00`; rule colour `#E2E5E8`.
Chart series follow the preset: `#B51E00` (dense baseline), `#009682` (proposed), `#F0A81D`
(hash-routing), `#4A6572` (linear).

**Grid / typography** — 12-col grid, margins x=50 … x=910 (usable 860 px).
Fixed header block: kicker (10 pt, letterspaced, teal) at y=30, **conclusion-style title** at y=48
(17–21 pt bold), hairline rule at y=86. Body region y=98…484. Fixed footer: hairline at y=498,
running title + section + page number at y=506 (10 pt grey). Body 11–11.5 pt / line-height 1.4–1.48;
captions 9.5 pt; footnotes 10 pt. Two unequal columns (≈ 47:53 or 45:55) — no card walls, no
equal thirds, no 2×2 matrix; containers are used only where they carry information (formula panel,
topology comparison, Q1/Q2/Q3 items, contribution items, table).

**Assets (all generated locally, no downloads)** — `assets/kv_cache_scaling.png` (log-x KV-cache
memory vs context length, H100 80 GB ceiling marked), `assets/complexity_curves.png` (log-log FLOPs
scaling of four attention families), `assets/pareto_frontier.png` (NIAH accuracy vs speedup).
Script: matplotlib only; every axis carries units + a source line.

**Content integrity** — the topic, candidate, committee and institution are a *fictional defense
framing* (the deck is a toolchain test). Quantitative figures are explicitly marked either as cited
public results (`Llama-3 Technical Report 2024`, `Dao et al. NeurIPS 2022`, `Beltagy et al. 2020`,
`Zhang et al. NeurIPS 2023`) or as **示意数据** where they illustrate the proposed method
(DLR-Attn's 98.4 % / 3.82× / 60.4 % are labelled 示意数据 in the speaker notes and captions).

---

## 2. Syntax coverage checklist (feature → page)

| PPTD / LaTeX feature | Where | Verified |
|---|---|---|
| `<latex>` / `\(…\)` **display** formula (own paragraph → `<m:oMathPara>` centered) | p1, p2, p3, p4, p5, p6, p8, p9 | render ✔ |
| Inline formula inside running text (`mc:Choice` + `mc:Fallback`) | p1, p2, p3, p4, p5, p6, p7, p8, p9, p10 | render ✔ (text fallback) |
| `\frac`, `\dfrac`, `\tfrac` | p2, p3, p4, p5, p6, p8, p9 | `<m:f>` ✔ |
| `\sqrt{x}`, `\sqrt[n]{…}` | p1, p2, p3, p4, p6 | `<m:rad>` ✔ |
| `\sum`, `\prod`, `\int`, `\oint` with limits | p2, p4, p8, p9 | `<m:nary>` ✔ |
| `\lim_{…}` with limits | p1, p4, p6, p9 | `<m:nary chr="lim">` — **renders wrong, see D3** |
| `\left(…\right)`, `\left\{…\right\}`, `\left|…\right|`, `\left\langle` | p2, p4, p5, p8, p9 | `<m:d>` ✔ |
| `\begin{aligned}…\end{aligned}` (multi-line, `\\`, `&`) | **p5** | `<m:eqArr>` ×4 ✔ |
| `\begin{split}…\end{split}` | **p6** | `<m:eqArr>` ×4 ✔ |
| `\begin{cases}…\end{cases}` (piecewise) | p3 (×2) | `<m:m>` ✔ |
| `pmatrix` | **p8** | `<m:m>` ✔ |
| `bmatrix` | **p5** | `<m:m>` ✔ |
| `vmatrix` | **p4** | `<m:m>` ✔ |
| Per-formula `\newcommand{\R}{…}` + `\newcommand{\norm}[1]{…}` + use | **p4** | ✔ |
| Macro scoping (no leak to next formula) | p4 (probe: `/tmp/latex-probe`) | ✔ hard error if leaked |
| `\mathrm`, `\text`, `\operatorname` | p2, p3, p4, p5, p6, p8, p9 | ✔ |
| `\mathbb` (`\mathbb{R}`, `\mathbb{R}^{m\times d_k}`) | p1, p2, p4, p5 | ✔ |
| Accents `\hat`, `\bar`, `\vec` | p5 (`\hat{S}_i`, `\bar{k}_b`), p3 (`\hat{P}`) | ✔ |
| Spacing `\quad`, `\,`, `\;` | p2, p4, p5, p6, p9 | ✔ |
| `\det`, `\exp`, `\log`, `\sin`, `\operatorname{Softmax}`, `\operatorname{TopK}` | p2, p4, p5, p8 | ✔ |
| Formula numbering + symbol-explanation column | p2 (式 2.1), p3 (3.1–3.2), p5 (5.1–5.2), p6 (6.1–6.3), p8 (8.1–8.2), p9 (9.1–9.2) | ✔ |
| **Booktabs three-line table** via `BorderSpec` (top/bottom 2 pt, mid 1 pt, no verticals) | p7 | ✔ (`tableStyles.booktabs`) |
| Table cell `fill`, per-cell `<span style="color:…;font-weight:bold;">`, `rowHeights`, `columnWidths` | p7 | ✔ |
| `elementType: shape` (`rect`) for rules / panels / mid-rule | all pages | ✔ |
| `elementType: image` + `fit: {mode: contain}` | p2, p6, p8 | ✔ |
| `pageType: cover` / `content` / `final`, `notes:` on every page | all | ✔ |
| Rich text `<p> <strong> <em> <span style>` (no `<ul>/<ol>/<li>/<a>/<sup>/<sub>`) | all | ✔ |
| Hard-error probes (unsupported LaTeX) | `/tmp/latex-probe` | see §4 |

Every page has `notes:` (speaker notes) — validator `--check-notes` clean.

---

## 3. Verification evidence (exact commands + observed results)

```bash
# 1) static check
$ pptd check open-pptd-skill/tests/decks/04-academic-latex-defense/deck.pptd
open-pptd-skill/tests/decks/04-academic-latex-defense/deck.pptd: 10 page(s) validated — no issues

# 2) build
$ pptd build open-pptd-skill/tests/decks/04-academic-latex-defense/deck.pptd \
      -o open-pptd-skill/tests/decks/04-academic-latex-defense/build/deck.pptx --force
wrote .../build/deck.pptx
  sync point recorded: .../build/deck.pptx = afd73e1145ae9cd27f0d6bbb371c4c5d1f371ac97f19973cd9b508b627dc62e0

# 3) validate (10 pages + notes)
$ .venv/bin/python open-pptd-skill/skills/open-pptd/scripts/validate_pptx.py \
      open-pptd-skill/tests/decks/04-academic-latex-defense/build/deck.pptx \
      --expected-pages 10 --check-notes
Status: [PASS]   Slides: 10   Errors: 0   Warnings: 0
Slide 1..10: [Notes: Yes]
"All checks passed cleanly! No leaked tags or boundary errors detected."

# 4) OOXML evidence — equation arrays really are <m:eqArr>
$ for i in $(seq 1 10); do
    echo -n "slide$i "; unzip -p build/deck.pptx ppt/slides/slide$i.xml | grep -o 'm:eqArr' | wc -l
  done
slide1 0 / slide2 0 / slide3 0 / slide4 0 / slide5 4 / slide6 4 / slide7 0 / slide8 0 / slide9 0 / slide10 0

$ for i in $(seq 1 10); do
    echo -n "slide$i "; unzip -p build/deck.pptx ppt/slides/slide$i.xml | grep -o 'm:m>' | wc -l
  done
slide3 4 / slide4 2 / slide5 2 / slide8 2   (rest 0)

# 5) headless render + full-size review of every page
$ .venv/bin/python .../validate_pptx.py build/deck.pptx --expected-pages 10 --check-notes \
      --render-images build/preview
# → build/preview/slide-01.png … slide-10.png (2001×1125) + contact-sheet.jpg
```

All 10 PNGs were read at full size and iterated on. Fixes applied during review:
leaked `<strong style=…>` raw HTML (p3/p4/p5/p7/p8/p10), formula-number overlaps on p5 and p6,
a text-overflow warning on the p6 title, and a missing booktabs mid-rule on p7.

---

## 4. Defects / rough edges found

### D1 — `<strong style="…">` silently leaks raw HTML onto the slide (HIGH)
Minimal repro:
```yaml
- elementId: t1
  elementType: text
  bounds: [50, 50, 800, 100]
  content:
    text: |
      <p><strong style="color:#009682;">colored bold</strong></p>
```
`pptd check` → `no issues`; `pptd build` → exit 0, no warning.
`slide1.xml` contains `<a:t>&lt;strong style=&quot;color:#009682;&quot;&gt;colored bold</a:t>`;
the LibreOffice PNG shows the literal text `<strong style="color:#009682;">colored bold`.
Plain `<strong>` works, and `<span style="…">` works — only `<strong style=…>` leaks.
**Layer:** compiler rich-text parser (attribute-carrying `<strong>`/`<em>` falls through to literal
text instead of erroring or mapping to the span style). **Workaround:** `<span style="…; font-weight:bold;">`.
Observed on the real deck (p3, p7 etc.) before fixing.

### D2 — validator does not detect the D1 leak (MEDIUM)
`skills/open-pptd/scripts/validate_pptx.py` line 37: `LEAKED_TAG_REGEX` covers
`ul|ol|li|a|div|span|p|b|i|u|s|del|strike|font|code|h[1-6]|table|tr|td|th` — **`strong` and `em` are
missing**. A deck containing `<strong style=…>` literals reports
`Status: [PASS] … "No leaked tags or boundary errors detected."`
**Layer:** validator. **Fix:** add `strong|em` to the alternation.

### D3 — `\lim` compiles to an invalid `<m:nary>` and renders as garbage (HIGH)
Minimal repro: `<p>\( \lim_{N \to \infty} \frac{1}{N} = 0 \)</p>` → `pptd build` exit 0.
Emitted OOXML:
```xml
<m:nary><m:naryPr><m:chr m:val="lim"/><m:limLoc m:val="undOvr"/><m:supHide m:val="1"/></m:naryPr>
  <m:sub>…N→∞…</m:sub><m:sup/><m:e>…</m:e></m:nary>
```
`m:chr` is specified to hold a **single character**; a 3-char value is invalid OMML. LibreOffice
renders it as `¿ N→∞` (U+00BF) instead of `lim` under `N→∞`. Visible on p1, p4, p6, p9.
Correct encoding would be `<m:limLow><m:e><m:r><m:t>lim</m:t></m:r></m:e><m:lim>…</m:lim></m:limLow>`.
`reference/math.md` §3 documents this as intended (`\sum, \prod, \int, \oint, \lim` → `<m:nary>`),
so docs and implementation agree on a construct that renders wrong in a real viewer.
**Layer:** compiler (LaTeX→OMML) + docs. **Workaround:** `\mathrm{lim}_{N\to\infty}`.

### D4 — `<` and `>` inside `\(…\)` are scanned as HTML tags (MEDIUM, usability)
Minimal repro: `<p>\( a < b \)</p>`
→ `pptd check` → `- page 1: text 't1' contains unsupported tag '< b \)</p>' …`.
Real-deck instances (exact messages observed):
- `\( (m \ll d_k) \)` → `unsupported LaTeX command '\ll'`; rewriting as `(m < d_k)` →
  `contains unsupported tag '<d_k)>'`
- `\( \epsilon < 10^{-2} \)` → `contains unsupported tag '<10^{-2}>'`
- `\( \sum_{t < s} \)` → `contains unsupported tag '<s}>'`
- `\( N < 2048 \)` → `contains unsupported tag '<2048>'`
The error is technically correct but points at a "tag" while the real fix is `&lt;`. Nothing in
`pptd.md`'s rich-text section mentions escaping `<` inside math.
**Layer:** compiler (tag scanner runs over math text) + docs.

### D5 — unsupported-command vocabulary is narrower than the docs suggest (MEDIUM, docs)
These all hard-error (good policy, poor coverage). Exact outputs from `pptd check`:
```
unsupported LaTeX command `\pmod` (inside `\(`)
unsupported LaTeX command `\bmod` (inside `\(`)
unsupported LaTeX command `\iff` (inside `\(`)
unsupported LaTeX command `\implies` (inside `\(`)
unsupported LaTeX command `\ll` (inside `\(`)
unsupported LaTeX command `\middle` (inside `\(`)
unsupported LaTeX command `\Longleftrightarrow` (inside `\(`)
unsupported LaTeX command `\mathbf` (inside `\(`)
unsupported LaTeX command `\mathcal` (inside `\(`)
unsupported LaTeX command `\|` (reported as: error: invalid PPTD: unsupported LaTeX command: \|)
```
`math.md` §3 lists relation symbols but omits `\ll`, `\gg`, `\iff`, `\implies`, `\pmod`, `\bmod`,
`\middle`, `\|`; the font-style section omits `\mathbf`/`\mathcal` (only `\mathbb`, `\mathrm`,
`\text`, `\operatorname` are listed). Workarounds used in this deck: `\Leftrightarrow`,
`\Rightarrow`, `\ (\mathrm{mod}\ S)`, `\left|…\right|`, `\mathrm{…}`.
Note also the inconsistent surface: most errors come from `pptd check` with the
`page N: text 'id': …` prefix, but `\|` only surfaces at `pptd build` as
`error: invalid PPTD: unsupported LaTeX command: \|` — so `check` is not a complete gate.

### D6 — table mid-rule is swallowed by the shared edge (MEDIUM, styling trap)
Theme:
```yaml
tableStyles:
  booktabs:
    cellStyle: {border: [ {style: solid,width: 1,color: "#FFFFFF"}, … ]}   # all four sides
    firstRowStyle: {border: [ {…width: 2,color:"#000000"}, null, {…width: 1,color:"#000000"}, null ]}
```
Emitted XML (verified on slide 7): row 1 cells carry `lnT w="25400"` **and** `lnB w="12700"`; row 2
cells carry `lnT w="12700"` white. OOXML resolves the shared horizontal edge in favour of the
**lower** cell's top border, so the header's 1 pt bottom rule is painted over and the three-line
table loses its mid rule in the render. Workaround used: an explicit
`elementType: shape` / `shapeName: rect` 860×1.5 px black rule at y=137.
**Layer:** compiler/theme (no conflict resolution guidance) + docs.

### D7 — LibreOffice fidelity of native math (LOW, viewer-side, but undocumented)
- **Equation arrays:** `aligned`/`split` correctly emit `<m:eqArr>`, and LibreOffice *does* stack the
  rows — but it draws the alignment marker literally: p5/p6 render as `z_i & = W_r q_i`. `math.md`
  §3 says "LibreOffice renders the equation array as a stacked column of lines" and does not mention
  the literal `&`.
- **Matrices:** `pmatrix`/`bmatrix`/`vmatrix` emit `<m:m>` correctly. LibreOffice fidelity is
  inconsistent: p8's 4×4 `pmatrix` renders as a proper grid, but p5's `bmatrix` (a genuine column
  vector) and p4's `vmatrix` determinant collapse toward a single horizontal run. Since the OOXML is
  right, this is viewer-side — but it makes matrix-heavy slides unreadable in LibreOffice previews.
- **`\text{}` / `\operatorname{}`:** emit `<m:sty m:val="p"/>` (upright) yet LibreOffice still
  renders them italic (p3 `if`/`otherwise`).
- **Inline formulas** take the `mc:Fallback` text branch as documented, and the fallback is good:
  `\frac{1}{1+e^{-x}}` → `1/(1+e⁻ˣ)`, `\sum_{i=1}^{n}` → `∑` with raised/lowered limits, `\mathbb{R}` → `ℝ`.
- **Display formulas must not share a text box with inline ones:** when a paragraph containing
  `mc:AlternateContent` precedes a display `<a14:m>` paragraph *in the same text box*, LibreOffice
  silently drops the display formula. Reproduced in `/tmp/latex-probe` (1 text element, 2 paragraphs
  → only the first renders; split into 2 text elements → both render). Not a compiler bug, but a
  layout rule worth documenting.
- **Display formulas need vertical head-room:** a `bounds` height of 60 px triggered
  `<a:normAutofit>` to squeeze the equation vertically and stretch it horizontally into an
  unreadable smear; 120 px renders correctly. Practical guidance: ≥ 90–140 px for a display equation.

### D8 — `check` vs `build` disagreement (LOW)
`pptd check` on `07_empirical_evidence.page` accepted the file; the first `pptd build` failed with
`error: invalid PPTD: unsupported LaTeX command: \|`. A gate that passes what the builder rejects
undermines `check` as a pre-commit gate.

**None** of the above blocked delivery: the final deck builds, validates PASS, and every page was
inspected at full size.

---

## 5. Honest gaps / compromises

- **`\lim` renders as `¿`** in the delivered PNGs (D3). Kept deliberately, because `\lim` with
  limits is an explicitly required feature of this stress test; PowerPoint/WPS will likely render the
  invalid `<m:nary chr="lim">` differently (possibly also wrong). Only LibreOffice was available.
- **Equation-array alignment (`&`) and matrix grids** cannot be judged in PowerPoint from this
  environment — only the emitted OOXML (`<m:eqArr>`, `<m:m>`) proves they are correct.
- `pptd convert` round-trip was **not** exercised (out of scope for this deck; another agent covers it).
- The **content is a fabricated defense framing** over a *plausible but not published* method
  (DLR-Attn). Cited results are real papers; DLR-Attn's numbers are 示意数据 and are labelled as such
  in the speaker notes/captions. No invented citation is attributed to a real author for DLR-Attn.
- p7's table is set at 10 pt to fit six columns; below the 14 pt "projection readability" floor of
  PART A, traded for the mandatory booktabs comparison table on one page.
- The `academic-research.md` "no cards" rule is respected, but p4/p5/p8/p10 use light grey panels as
  *information containers* (formula panel, Q1/Q2/Q3, contributions) — the preset explicitly allows
  containers with a genuine information function.
