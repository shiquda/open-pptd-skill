# Native Math & LaTeX Formulas in PPTD

Starting in version 1.4.0, `open-pptd` compiles LaTeX formulas directly into native PowerPoint Office Math (OOXML OMML via `<a14:m>`) with authentic typographical layout (real horizontal fraction bars, scalable radical roots, and properly aligned large operators). Standalone formulas are editable equations in every viewer that understands OOXML math; inline formulas stay editable in PowerPoint/WPS and degrade to correctly typeset text where the viewer does not implement the `a14` math extension (see §2).

---

## 1. Syntax

Formulas can be declared in PPTD rich text using either `<latex>...</latex>` tags or `\( ... \)` delimiters inside `<p>` paragraphs:

```yaml
- elementId: math-card
  elementType: text
  bounds: [60, 140, 840, 120]
  content:
    text: |
      <p>Inline formula in running text: \( h(s_t) \ge 0 \) defines the forward invariant set.</p>
      <p>\( \frac{a+b}{c+d} = \sqrt{x^2 + y^2} \)</p>
```

Both `<latex>\frac{1}{2}</latex>` and `\(\frac{1}{2}\)` are treated identically by the compiler.

---

## 2. Display vs. Inline Formatting Rules

The compiler automatically distinguishes between **display formulas** and **inline formulas** based on paragraph structure:

1. **Display Formulas (Standalone)**:
   - When a formula is the **sole content** of a paragraph (e.g. `<p>\( \sum_{i=1}^{n} i^2 = \frac{n(n+1)(2n+1)}{6} \)</p>`), it compiles into an Office Math display paragraph (`<m:oMathPara>` with `<m:jc m:val="centerGroup"/>`).
   - The equation is centered horizontally with dedicated vertical spacing.
2. **Inline Formulas (Mixed Text)**:
   - When a formula is mixed with surrounding text or other tags in the same paragraph (e.g. `<p>Given \( x \in \mathbb{R} \), we have ...</p>`), it is emitted as a markup-compatibility pair:
     - `<mc:Choice Requires="a14">` — the native math zone (`<a14:m>` / `<m:oMath>`) placed alongside `<a:r>` text runs. PowerPoint and WPS take this branch and keep the formula editable.
     - `<mc:Fallback>` — ordinary text runs rendering the same formula with Unicode symbols and baseline-shifted scripts (`x^{2}` → `x²`, `\frac{a+b}{c+d}` → `(a+b)/(c+d)`, `\sqrt{x^2+y^2}` → `√(x²+y²)`, `\sum_{i=1}^{n}` → `∑` with lowered/raised limits). Viewers that do not implement the `a14` extension — LibreOffice among them — take this branch.
   - This pairing exists because LibreOffice's PPTX importer drops a math zone that shares a paragraph with text runs; the fallback guarantees the formula is never silently lost.
   - Consequence: an inline formula is an editable equation in PowerPoint/WPS but plain (still correct) text in LibreOffice. If a formula must be an editable equation in every viewer, put it on its own line so it compiles as a display formula.
3. **Display-only environments**:
   - `\begin{align}`, `\begin{gather}` and `\begin{eqnarray}` are display environments in LaTeX and must own their paragraph. Using one inside a line of text fails the build with an explicit error instead of silently producing a broken inline equation.
   - The `…ed` variants (`\begin{aligned}`, `\begin{gathered}`, `\begin{split}`) may be used in either position.

---

## 3. Supported LaTeX Subset

The compiler includes a built-in LaTeX-to-OMML converter supporting standard mathematical typesetting:

### Fractions and Radicals
- Fractions: `\frac{a}{b}`, `\dfrac{a}{b}`, `\tfrac{a}{b}` (compiled to `<m:f>` with genuine horizontal fraction bar)
- Square roots: `\sqrt{x}`, `\sqrt{x^2 + y^2}` (compiled to `<m:rad>`)
- N-th roots: `\sqrt[n]{x}` (compiled to `<m:rad>` with explicit `<m:deg>`)

### Subscripts and Superscripts
- Subscripts: `x_i`, `x_{t+1}`, `o_{\le t}`
- Superscripts: `x^2`, `A^T`, `e^{-\lambda t}`
- Combined: `A_t^H`, `\sigma_{i,j}^2` (compiled to `<m:sSubSup>`)

### Big Operators and Limits
- Summation, product, and integrals: `\sum`, `\prod`, `\int`, `\oint`, `\lim` (compiled to `<m:nary>`)
- Upper and lower limits attached via `_` and `^`:
  ```latex
  \sum_{i=1}^{n} x_i, \quad \int_{0}^{\infty} e^{-x} dx, \quad \lim_{x \to 0} \frac{\sin x}{x}
  ```

### Delimiters & Grouping
- Scalable delimiters: `\left( ... \right)`, `\left[ ... \right]`, `\left\{ ... \right\}`, `\left| ... \right|`, `\left\langle ... \right\rangle`, `\left. ... \right|` (compiled to `<m:d>`)

### Matrices and Piecewise Cases
- Matrix environments: `\begin{matrix}`, `\begin{pmatrix}`, `\begin{bmatrix}`, `\begin{vmatrix}` using `&` for column separation and `\\` for rows (compiled to `<m:m>`).
- Piecewise cases: `\begin{cases} ... \end{cases}`.

### Multi-line Equations
- Aligned derivations: `\begin{aligned} ... \end{aligned}`, `\begin{split}`, `\begin{gather}`, `\begin{gathered}`, `\begin{align}`, `\begin{eqnarray}`. Rows are separated by `\\`; `&` marks the alignment point. They compile to an OMML equation array (`<m:eqArr>`) with one line per `<m:e>`, so PowerPoint and WPS keep a single editable multi-line equation:
  ```latex
  \begin{aligned} (a+b)^2 &= a^2 + 2ab + b^2 \\ &= a^2 + 2ab + b^2 \end{aligned}
  ```
- `align`, `align*`, `gather`, `gather*` and `eqnarray` are display environments: they must be the only content of their paragraph, exactly as in LaTeX. The `…ed` variants (`aligned`, `gathered`, `split`) may also appear inside running text.
- The alignment point is written as an `&` run inside each line — the same encoding Word's linear format uses and Microsoft's `MML2OMML.XSL` produces. LibreOffice renders the equation array as a stacked column of lines.

### Macro Definitions
Short aliases can be defined at the top of a formula; they are expanded before parsing, so a macro never leaks onto the slide and never trips the unsupported-command diagnostics:

```latex
\newcommand{\R}{\mathbb{R}}\newcommand{\norm}[1]{\left|#1\right|} \norm{x} \in \R^n
```

- `\newcommand{\name}[n]{body}`, `\newcommand\name[n]{body}`, and the optional-default form `\newcommand{\name}[n][default]{body}` (`\name[…]` may then be omitted).
- `\renewcommand` (the macro must already exist) and `\providecommand` (no-op when it does).
- `\def\name{body}` and `\def\name#1#2{body}`. Delimited parameter texts (`\def\x#1,{…}`) are rejected with a clear error.
- `#1`…`#9` are substituted with the arguments of the call; `##` yields a literal `#`.
- A definition is scoped to one formula: a macro defined in one `<latex>` body is not visible in the next.
- Recursive definitions (`\def\a{\a}`) fail with an expansion-limit error instead of hanging.

### Mathematical Symbols & Greek Letters
- **Greek lowercase**: `\alpha`, `\beta`, `\gamma`, `\delta`, `\epsilon`, `\theta`, `\pi`, `\sigma`, `\tau`, `\omega`, etc.
- **Greek uppercase**: `\Gamma`, `\Delta`, `\Theta`, `\Lambda`, `\Xi`, `\Pi`, `\Sigma`, `\Phi`, `\Psi`, `\Omega`.
- **Relations**: `\le`, `\ge`, `\neq`, `\approx`, `\equiv`, `\sim`, `\propto`, `\mid`, `\perp`, `\parallel`.
- **Set operations**: `\in`, `\notin`, `\subset`, `\subseteq`, `\supset`, `\cup`, `\cap`, `\emptyset`.
- **Logic & arrows**: `\forall`, `\exists`, `\neg`, `\wedge`, `\vee`, `\to`, `\leftarrow`, `\rightarrow`, `\Rightarrow`, `\Leftarrow`, `\mapsto`.
- **Arithmetic & calculus**: `\times`, `\cdot`, `\pm`, `\mp`, `\div`, `\infty`, `\partial`, `\nabla`.

### Font Styles & Functions
- Blackboard bold: `\mathbb{R}`, `\mathbb{N}`, `\mathbb{Z}`, `\mathbb{C}`, `\mathbb{Q}`
- Upright text / math operators: `\mathrm{...}`, `\text{...}`, `\operatorname{...}`
- Standard functions: `\sin`, `\cos`, `\tan`, `\ln`, `\log`, `\exp`, `\min`, `\max`

### Spacing, Accents & Escapes
- Spacing: `\,`, `\;`, `\:`, `\quad`, `\qquad`
- Escapes: `\{`, `\}`, `\_`, `\&`, `\%`, `\#`, `\$`
- Accents: `\bar`, `\overline`, `\hat`, `\vec`, `\tilde`, `\dot`

---

## 4. Hard-Error Policy (No Silent Degradation)

Prior compiler versions attempted to approximate unsupported LaTeX commands as plain text runs (e.g. converting `\frac{x+1}{y}` into text `x+1/y`).

In `open-pptd` v1.4.0+:
- **Strict Validation**: Every LaTeX command is validated against the supported vocabulary.
- **Hard Error on Unsupported Commands**: If an unsupported command (e.g. `\tikz`, `\color`, `\foo`) or malformed syntax is encountered, `pptd build` fails immediately with an explicit error naming the command.
- **No Leaked Raw LaTeX**: Raw backslash commands never leak onto the presentation slide.

---

## 5. Reverse Compilation (`pptd convert`)

When reverse-compiling an existing `.pptx` file via `pptd convert input.pptx output_dir/`:
- Native `<a14:m>` equation structures (`<m:f>`, `<m:rad>`, `<m:sSup>`, `<m:nary>`, `<m:m>`, `<m:eqArr>`, etc.) are automatically parsed and reconstructed into standard `<latex>...</latex>` PPTD rich text tags.
- Equation arrays (`<m:eqArr>`) come back as `\begin{aligned} … \end{aligned}` with `&` alignment points preserved.
- Equations round-trip between `.pptd` and `.pptx` losslessly.

