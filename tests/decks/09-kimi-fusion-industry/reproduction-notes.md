# Reproduction notes — Kimi K3 "受控核聚变行业研究"

These notes accompany `README.md`. They record, per page, what the declarative model
made easy, where manual coordinate arithmetic was required, how many render-review
iterations each page needed, and where fidelity to the Kimi K3 benchmark was traded away.

## Toolchain capability × verdict

| Capability under test | Verdict | Basis |
| --- | --- | --- |
| Declarative multi-page manifest (`version/pages`) | ✅ works | single 16-entry manifest, no issues |
| Theme colours / textStyles / tableStyles | ✅ works | one definition, 16 pages consistent |
| Text element + HTML subset (`<p> <b> <i> <br>`) | ✅ works | ~200 elements, no leaks, entities decode |
| Inline LaTeX `\( … \)` | ✅ works | renders via `mc:AlternateContent` fallback in LibreOffice |
| Display LaTeX (`<p>\(…\)</p>` alone) | ✅ works (hypothesis disproven, see README §4/D6) | fallback present in both paths |
| Shape presets (`rect/ellipse/arc/donut/diamond`) | ✅ works | cover schematic + gate markers |
| Line element with `viewBox`/`points`/arrowheads | ✅ works | 33 arrows/connectors, arrowheads render correctly |
| Native table (merge/fill/border, `columnWidths`/`rowHeights`) | ✅ works | two dense tables (slides 07, 14) |
| Matplotlib raster chart as `elementType: image` | ✅ works | slides 04, 08, 12; axis labels/units/source legible |
| `elementType: chart` (native) | ❌ unsupported (documented) | rejected at parse — with a misleading schema error, not the documented `Unsupported` error (README §4/D2) |
| Custom vector path (`shapeName: custom` + `viewBox`) | ⚠️ not exercised | cover schematic achievable with ellipse/arc/donut; no page demanded a bezier path |
| Auto-layout / constraints | ❌ absent by design | every element is absolutely positioned; all spacing is hand-computed |
| Animation | ❌ not used | out of scope for a static research deck |
| `pptd check` overflow estimation | ⚠️ false positives | 7 single-line titles flagged as 51% over (README §4/D1) |
| `pptd check` / `build` / `validate` / render loop | ✅ works end-to-end | final state: check 7 warnings → build OK → validate PASS 16/16 → render OK |
| Validator `--render-images` idempotency | ❌ does not overwrite stale PNGs | README §4/D5 |

## Per-page log

| # | Page | Iterations | Declarative ease | Manual arithmetic / workarounds | Fidelity gap vs Kimi benchmark |
| --- | --- | --- | --- | --- | --- |
| 01 | Cover | 2 | high | right-hand schematic (ellipse + arc + donut + 2 lines) laid out by hand | photographic plate replaced by vector schematic; confidential strip removed |
| 02 | Executive synthesis | 3 | medium | 4 equal cards; watch-indicator band added to fill body void; cards resized twice | none |
| 03 | Scientific basis | 2 | medium | inline LaTeX (`D-T reaction`, `Q_plasma`, `Q_eng`) with bold label prefix | none |
| 04 | Triple-product trajectory | 2 | high | matplotlib figure: y-axis clipping fixed via `subplots_adjust(left=0.18)` | Kimi uses a denser log-log chart; simplified to a single log-y series |
| 05 | Confinement taxonomy | 1 | high | 3 cards + band | none |
| 06 | Approach deep dive | 1 | high | 3 cards + band; long subtitle kept to one line | none |
| 07 | Readiness matrix | 2 | high | native table with merged header cell + alternating fills; row heights hand-tuned to avoid wrapped-cell overflow | none |
| 08 | Private-capital landscape | 3 | medium | two-panel matplotlib figure; bar % labels moved inside bars >40%; `ax2.set_xlim(0,75)` | none |
| 09 | HTS superconductor | 2 | medium | inline LaTeX `P_fus ∝ β²B⁴V`, `J_c`; watch band 3-line item | none |
| 10 | Tritium / first wall | 2 | medium | 2 wide cards; `⁶Li/⁷Li/⁴He` as Unicode superscripts instead of LaTeX isotopes; TBR as inline math | Kimi uses a process-flow diagram; replaced by two deep text panels |
| 11 | Plant engineering BOP | 1 | high | 3 cards + band | none |
| 12 | Economics / LCOE | 3 | medium | matplotlib figure; LCOE curves + NOAK/FOAK zones; `xlim(2000, 14200)` to stop curve clipping; FOAK/NOAK bands explicitly annotated as modelled, not data | none |
| 13 | Commercialisation scenarios | 3 | medium | non-default split: left base case 435 pt + right column two 105 pt cases; 3 diamond gates + 4 line arrows as the milestone-gate timeline | none |
| 14 | Claim ledger | 2 | high | dense native table; green/red verdict cells | none |
| 15 | Falsification criteria | 2 | high | 4 cards + band; red accents | none |
| 16 | Strategic roadmap | 2 | high | 3 cards + band | none |

## Geometry workflow notes

* Every body page shares one coordinate contract: header `26/42/74/92`, body `106 … 466`,
  synthesis strip `472`, footer `514/520`. Defining it once made the 16 pages consistent,
  but it is re-typed by hand in every file — there is no `layout:` / slot mechanism in PPTD.
* The body region is split `cards (106, h 268) + watch-indicator band (384, h 74) + gap 10`.
  The band was the decisive device for the "no empty body region" rule: cards alone leave
  a void under short content, and stretching the cards produced 35% interior slack.
  The current balance leaves ~15–25% slack inside the tallest cards on six pages —
  accepted as the cost of not padding copy.
* Total coordinate-iteration cycles (build → render → inspect → adjust): 3 full passes
  plus 2 chart-only passes (charts 04, 08, 12) — about 5 rendered builds in total.

## Toolchain findings carried to the report

1. `pptd check` title-overflow estimator is length-independent and over-estimates → 7 false warnings (README D1).
2. `elementType: chart` is rejected by the *parser* (`missing field cols/series`) before the documented `Unsupported` path is ever reached (README D2).
3. `size:` rejects the map form with a Rust-serde message that never names the correct syntax (README D3).
4. `pptd build` prints no progress/page summary — only a sync-point hash (README D4).
5. The validator's `--render-images` never overwrites existing `slide-NN.png`; stale previews are an easy sign-off trap (README D5).
6. Display math (`oMathPara`) DOES carry an `mc:AlternateContent` fallback — an earlier "blank box" hypothesis was disproven by a minimal repro (README D6).
