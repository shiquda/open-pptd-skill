//! Semantic validation of a loaded [`Project`].
//!
//! The loader guarantees the document parses and the version is `v2`; this
//! module checks the *semantics* the spec requires: unique element ids,
//! positive bounds, canvas boundary containment, resolvable theme references,
//! well-formed tables and coherent chart data.
//!
//! # Diagnostic Severities
//!
//! Findings are reported as [`Diagnostic`]s rather than fatal panics, so
//! `pptd check` can list every issue in one pass:
//! - **Errors** (no `warning:` prefix): fatal structural or semantic problems
//!   that prevent building a valid PPTX, such as duplicate element IDs, image
//!   fills on non-background elements, unsupported icon prefixes, or elements
//!   placed entirely outside the slide canvas.
//! - **Warnings** (prefixed with `warning:`): non-fatal issues where PPTX
//!   generation can technically proceed but the output may be visually degraded
//!   or feature-incomplete, such as potential text overflow, elements extending
//!   partially outside the slide canvas, or declared animations (which the
//!   compiler parses but does not yet emit as `p:timing`).
use std::collections::HashSet;
use std::path::Path;

use super::chart::Chart;
use super::elements::{Element, Shape, Text};
use super::shared::{Border, BorderSpec, Color, Fill, Shadow};
use super::theme::{TableStyleRef, Theme};
use super::{Page, Project};

/// A single validation finding.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// One-based page index if the finding belongs to a page.
    pub page: Option<usize>,
    /// Human-readable description.
    pub message: String,
}

impl Diagnostic {
    /// True when this finding is a warning (non-fatal advisory) rather than an error.
    pub fn is_warning(&self) -> bool {
        self.message.starts_with("warning:")
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.page {
            Some(page) => write!(f, "page {page}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

/// Run every check over the project; an empty result means "valid".
pub fn validate_project(project: &Project) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    let size = &project.presentation.size;
    if !size.width.is_finite()
        || !size.height.is_finite()
        || size.width <= 0.0
        || size.height <= 0.0
    {
        out.push(Diagnostic {
            page: None,
            message: format!("size must be positive and finite, got {size:?}"),
        });
    }

    check_path_containment(project, &mut out);
    check_presentation_raw_yaml(project, &mut out);

    for (idx, page) in project.pages.iter().enumerate() {
        validate_page(project, idx, page, &mut out);
    }

    out
}

/// Verify every referenced page path stays inside the project root (the
/// canonical PPTD "fully self-contained" rule, spec §Multi-File Structure).
/// A page path that escapes via `..` is reported as a `Diagnostic` rather
/// than failing the load. Both sides are canonicalized so symlinks and
/// on-disk case are handled consistently.
fn check_path_containment(project: &Project, out: &mut Vec<Diagnostic>) {
    let Ok(root) = project.root_dir.canonicalize() else {
        return; // best-effort; an unreadable root surfaces elsewhere.
    };
    for (idx, path) in project.page_paths.iter().enumerate() {
        let Ok(resolved) = path.canonicalize() else {
            continue; // already-read files resolve; skip if not.
        };
        if !resolved.starts_with(&root) {
            let rel = project
                .presentation
                .pages
                .get(idx)
                .map(|s| s.as_str())
                .unwrap_or("?");
            diag(
                out,
                idx,
                format!(
                    "page path `{rel}` escapes the project directory (PPTD projects must be self-contained)"
                ),
            );
        }
    }
}

fn validate_page(project: &Project, page_idx: usize, page: &Page, out: &mut Vec<Diagnostic>) {
    let theme = project.presentation.theme.as_ref();

    check_page_raw_yaml(project, page_idx, out);
    check_fill(
        project,
        theme,
        page.background.as_ref(),
        page_idx,
        out,
        "page background",
    );
    if let Some(Fill::Image { src, .. }) = &page.background {
        check_image_file(&project.root_dir, src, page_idx, out, "page background");
    }
    // SlideForge extension: `Page.layout` must reference a key declared in
    // `Presentation.layouts`. Unset pages are canonical (flat) and skipped.
    if let Some(layout_key) = &page.layout {
        let resolved = project
            .presentation
            .layouts
            .as_ref()
            .is_some_and(|m| m.contains_key(layout_key));
        if !resolved {
            diag(
                out,
                page_idx,
                format!("layout `{layout_key}` is not defined in presentation.layouts"),
            );
        }
    }
    // TODO(P2): once convert sets `Text.placeholder`, check it resolves against
    // the page's layout placeholders (needs the layout context plumbed in).

    // SlideForge group extension: every `groupId` on an element must
    // resolve to an entry in `Page.groups`.
    for element in &page.elements {
        if let Some(gid) = element.common().group_id.as_deref() {
            let resolved = page.groups.as_ref().is_some_and(|m| m.contains_key(gid));
            if !resolved {
                diag(
                    out,
                    page_idx,
                    format!(
                        "element `{}` references groupId `{}` not defined in page.groups",
                        element.element_id(),
                        gid
                    ),
                );
            }
        }
    }

    let mut ids: HashSet<&str> = HashSet::new();
    for element in &page.elements {
        let element_id = element.element_id();
        if !ids.insert(element_id) {
            diag(out, page_idx, format!("duplicate elementId `{element_id}`"));
        }

        let bounds = element.bounds();
        let degenerate = match element {
            // A straight connector is legitimately degenerate in one axis
            // (horizontal: height 0, vertical: width 0), so a flattened
            // box is allowed as long as at least one axis is non-zero.
            Element::Line(_) => {
                let w = bounds.width;
                let h = bounds.height;
                !(w.is_finite() && h.is_finite() && (w > 0.0 || h > 0.0))
            }
            _ => {
                !(bounds.width.is_finite()
                    && bounds.height.is_finite()
                    && bounds.width > 0.0
                    && bounds.height > 0.0)
            }
        };
        if degenerate {
            diag(
                out,
                page_idx,
                format!(
                    "element `{element_id}` must have positive width and height, got {bounds:?}"
                ),
            );
        } else {
            check_element_canvas_bounds(
                element_id,
                bounds,
                element,
                project.presentation.size,
                page_idx,
                out,
            );
        }

        if let Some(opacity) = element.opacity() {
            if !(0.0..=1.0).contains(&opacity) {
                diag(
                    out,
                    page_idx,
                    format!("element `{element_id}` opacity must be in [0, 1], got {opacity}"),
                );
            }
        }

        validate_element(project, page_idx, element, theme, out);
    }

    if let Some(animations) = &page.animations {
        if !animations.is_empty() {
            diag(
                out,
                page_idx,
                format!(
                    "warning: page declares {} animation(s), but animations are not yet supported by the compiler (p:timing will not be emitted)",
                    animations.len()
                ),
            );
        }
        for animation in animations {
            if !ids.contains(animation.element_id.as_str()) {
                diag(
                    out,
                    page_idx,
                    format!(
                        "animation references unknown elementId `{}`",
                        animation.element_id
                    ),
                );
            }
        }
    }
}

fn validate_element(
    project: &Project,
    page_idx: usize,
    element: &Element,
    theme: Option<&Theme>,
    out: &mut Vec<Diagnostic>,
) {
    match element {
        Element::Text(text) => {
            check_text_style_ref(
                theme,
                text.content.style.as_deref(),
                page_idx,
                out,
                format!("text `{}`", text.common.element_id),
            );
            check_color(
                theme,
                text.content.color.as_ref(),
                page_idx,
                out,
                &text.common.element_id,
                "text color",
            );
            check_color(
                theme,
                text.content.background_color.as_ref(),
                page_idx,
                out,
                &text.common.element_id,
                "text background color",
            );
            check_fill(
                project,
                theme,
                text.fill.as_ref(),
                page_idx,
                out,
                &text.common.element_id,
            );
            check_border(
                theme,
                text.border.as_ref(),
                page_idx,
                out,
                &text.common.element_id,
            );
            check_rich_text_tags(text, page_idx, out);
            check_latex_commands(text, page_idx, out);
            check_text_overflow(text, theme, page_idx, out);
        }
        Element::Shape(shape) => {
            check_shape_name(shape, page_idx, out);
            check_fill(
                project,
                theme,
                shape.fill.as_ref(),
                page_idx,
                out,
                &shape.common.element_id,
            );
            check_border(
                theme,
                shape.border.as_ref(),
                page_idx,
                out,
                &shape.common.element_id,
            );
            check_shadow(
                theme,
                shape.shadow.as_ref(),
                page_idx,
                out,
                &shape.common.element_id,
            );
        }
        Element::Line(line) => {
            check_border(
                theme,
                line.border.as_ref(),
                page_idx,
                out,
                &line.common.element_id,
            );
            check_shadow(
                theme,
                line.shadow.as_ref(),
                page_idx,
                out,
                &line.common.element_id,
            );
        }
        Element::Icon(icon) => {
            if let Err(err) = crate::pptx::fa::lookup_glyph(&icon.icon_name) {
                diag(
                    out,
                    page_idx,
                    format!("element `{}`: {err}", icon.common.element_id),
                );
            }
            check_fill(
                project,
                theme,
                icon.fill.as_ref(),
                page_idx,
                out,
                &icon.common.element_id,
            );
            check_border(
                theme,
                icon.border.as_ref(),
                page_idx,
                out,
                &icon.common.element_id,
            );
            check_shadow(
                theme,
                icon.shadow.as_ref(),
                page_idx,
                out,
                &icon.common.element_id,
            );
        }
        Element::Image(image) => {
            check_image_file(
                &project.root_dir,
                &image.src,
                page_idx,
                out,
                &image.common.element_id,
            );
            check_border(
                theme,
                image.border.as_ref(),
                page_idx,
                out,
                &image.common.element_id,
            );
            check_shadow(
                theme,
                image.shadow.as_ref(),
                page_idx,
                out,
                &image.common.element_id,
            );
        }
        Element::Table(table) => {
            validate_table(page_idx, table, theme, out);
            for row in &table.rows {
                for cell in row {
                    check_color(
                        theme,
                        cell.background_color.as_ref(),
                        page_idx,
                        out,
                        &table.common.element_id,
                        "table cell background",
                    );
                    check_border_spec(
                        theme,
                        cell.border.as_ref(),
                        page_idx,
                        out,
                        &table.common.element_id,
                    );
                }
            }
        }
        Element::Video(m) | Element::Audio(m) => {
            check_image_file(
                &project.root_dir,
                &m.src,
                page_idx,
                out,
                &m.common.element_id,
            );
            if let Some(poster) = &m.poster {
                check_image_file(
                    &project.root_dir,
                    poster,
                    page_idx,
                    out,
                    &m.common.element_id,
                );
            }
        }
        Element::Chart(chart) => {
            validate_chart(page_idx, chart, out);
            diag(
                out,
                page_idx,
                format!(
                    "chart `{}` is not supported by pptd compiler yet (native PPTX charts are under development)",
                    chart.common.element_id
                ),
            );
        }
    }
}

/// `style: "$key"` must exist in `theme.textStyles`.
fn check_text_style_ref(
    theme: Option<&Theme>,
    style: Option<&str>,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
    context: String,
) {
    if let Some(style) = style {
        let key = style.trim_start_matches('$');
        let resolved = theme.is_some_and(|theme| theme.text_styles.contains_key(key));
        if !resolved {
            diag(
                out,
                page_idx,
                format!("style `{style}` is not defined in theme.textStyles ({context})"),
            );
        }
    }
}

fn validate_table(
    page_idx: usize,
    table: &super::elements::Table,
    theme: Option<&Theme>,
    out: &mut Vec<Diagnostic>,
) {
    let element_id = &table.common.element_id;

    if let Some(TableStyleRef::Ref(style)) = &table.style {
        let key = style.trim_start_matches('$');
        let resolved = theme.is_some_and(|theme| theme.table_styles.contains_key(key));
        if !resolved {
            diag(
                out,
                page_idx,
                format!(
                    "table style `{style}` is not defined in theme.tableStyles (`{element_id}`)"
                ),
            );
        }
    }

    check_ratios(
        &table.column_widths,
        "columnWidths",
        page_idx,
        out,
        element_id,
    );
    check_ratios(&table.row_heights, "rowHeights", page_idx, out, element_id);

    let columns = table.column_widths.len();
    if table.rows.len() != table.row_heights.len() {
        diag(
            out,
            page_idx,
            format!(
                "table `{element_id}` has {} rows of data but {} rowHeights",
                table.rows.len(),
                table.row_heights.len()
            ),
        );
    }
    for (row_idx, row) in table.rows.iter().enumerate() {
        if row.len() > columns {
            diag(
                out,
                page_idx,
                format!(
                    "table `{element_id}` row {row_idx} has {} cells but the table only has \
                     {columns} columns (merged cells should be omitted)",
                    row.len()
                ),
            );
        }
    }
}

/// Each ratio must be within `(0, 1]` and the whole slice must sum to `1`.
fn check_ratios(
    ratios: &[f64],
    field: &str,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
    element_id: &str,
) {
    if ratios.is_empty() {
        diag(
            out,
            page_idx,
            format!("table `{element_id}` has no {field}"),
        );
        return;
    }
    for (i, ratio) in ratios.iter().enumerate() {
        if *ratio <= 0.0 || *ratio > 1.0 || !ratio.is_finite() {
            diag(
                out,
                page_idx,
                format!("table `{element_id}` {field}[{i}] must be in (0, 1], got {ratio}"),
            );
        }
    }
    let sum: f64 = ratios.iter().sum();
    if (sum - 1.0).abs() > 1e-3 {
        diag(
            out,
            page_idx,
            format!("table `{element_id}` {field} must sum to 1, got {sum}"),
        );
    }
}

fn validate_chart(page_idx: usize, chart: &Chart, out: &mut Vec<Diagnostic>) {
    let element_id = &chart.common.element_id;

    if chart.series.is_empty() {
        diag(
            out,
            page_idx,
            format!("chart `{element_id}` must define at least one series"),
        );
    }

    let cols = &chart.data.cols;
    let mut seen: HashSet<&str> = HashSet::new();
    for col in cols {
        if col.is_empty() {
            diag(
                out,
                page_idx,
                format!("chart `{element_id}` has an empty column name"),
            );
        }
        if !seen.insert(col.as_str()) {
            diag(
                out,
                page_idx,
                format!("chart `{element_id}` has duplicate column `{col}`"),
            );
        }
    }

    for (row_idx, row) in chart.data.rows.iter().enumerate() {
        if row.len() != cols.len() {
            diag(
                out,
                page_idx,
                format!(
                    "chart `{element_id}` row {row_idx} has {} cells, expected {}",
                    row.len(),
                    cols.len()
                ),
            );
        }
    }

    for series in &chart.series {
        for col in series.encode_columns() {
            if !cols.iter().any(|c| c == col) {
                diag(
                    out,
                    page_idx,
                    format!(
                        "chart `{element_id}` {} series references unknown column `{col}`",
                        series.type_name()
                    ),
                );
            }
        }
        // TODO: check stacking-mode consistency, numeric channels, theme
        // color references in series fills, and the per-type series
        // constraints from the spec.
    }
}

fn diag(out: &mut Vec<Diagnostic>, page: usize, message: String) {
    out.push(Diagnostic {
        page: Some(page + 1),
        message,
    });
}

const VALID_PRESET_SHAPES: &[&str] = &[
    "rect",
    "roundRect",
    "ellipse",
    "triangle",
    "rtTriangle",
    "parallelogram",
    "trapezoid",
    "diamond",
    "pentagon",
    "hexagon",
    "heptagon",
    "octagon",
    "decagon",
    "dodecagon",
    "star4",
    "star5",
    "star6",
    "star7",
    "star8",
    "star10",
    "star12",
    "star16",
    "star24",
    "star32",
    "round1Rect",
    "round2SameRect",
    "round2DiagRect",
    "snipRoundRect",
    "snip1Rect",
    "snip2SameRect",
    "snip2DiagRect",
    "plaque",
    "teardrop",
    "homePlate",
    "chevron",
    "pie",
    "chord",
    "arc",
    "donut",
    "can",
    "cube",
    "bevel",
    "blockArc",
    "foldedCorner",
    "smileyFace",
    "heart",
    "lightningBolt",
    "sun",
    "moon",
    "cloud",
    "rightArrow",
    "leftArrow",
    "upArrow",
    "downArrow",
    "leftRightArrow",
    "upDownArrow",
    "quadArrow",
    "leftRightUpArrow",
    "bentArrow",
    "uturnArrow",
    "leftUpArrow",
    "bentUpArrow",
    "curvedRightArrow",
    "curvedLeftArrow",
    "curvedUpArrow",
    "curvedDownArrow",
    "stripedRightArrow",
    "notchedRightArrow",
    "line",
];

fn check_shape_name(shape: &Shape, page_idx: usize, out: &mut Vec<Diagnostic>) {
    let element_id = &shape.common.element_id;
    if shape.shape_name == "custom" {
        if shape.view_box.is_none() {
            diag(
                out,
                page_idx,
                format!("custom shape `{element_id}` requires viewBox"),
            );
        }
        if shape.path.is_none() {
            diag(
                out,
                page_idx,
                format!("custom shape `{element_id}` requires path"),
            );
        }
    } else if !VALID_PRESET_SHAPES.contains(&shape.shape_name.as_str()) {
        let suggestion = match shape.shape_name.as_str() {
            "circle" => " (did you mean `ellipse`?)",
            "rectangle" => " (did you mean `rect`?)",
            "rounded_rect" | "roundedRectangle" => " (did you mean `roundRect`?)",
            _ => "",
        };
        diag(
            out,
            page_idx,
            format!(
                "shape `{element_id}` has unknown shapeName `{}`{suggestion}",
                shape.shape_name
            ),
        );
    }
}

fn check_color(
    theme: Option<&Theme>,
    color: Option<&Color>,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
    element_id: &str,
    field: &str,
) {
    let Some(c) = color else { return };
    if c.0.starts_with('$') {
        let key = c.0.trim_start_matches('$');
        if !theme.is_some_and(|t| t.colors.contains_key(key)) {
            diag(
                out,
                page_idx,
                format!(
                    "element `{element_id}` {field} references undefined color `{}` (not defined in theme.colors)",
                    c.0
                ),
            );
        }
    }
}

fn check_fill(
    project: &Project,
    theme: Option<&Theme>,
    fill: Option<&Fill>,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
    element_id: &str,
) {
    let Some(f) = fill else { return };
    match f {
        Fill::Solid { color: c } => {
            check_color(theme, Some(c), page_idx, out, element_id, "fill color");
        }
        Fill::Gradient { stops, .. } => {
            for stop in stops {
                check_color(
                    theme,
                    Some(&stop.color),
                    page_idx,
                    out,
                    element_id,
                    "gradient stop color",
                );
            }
        }
        Fill::Image { src, .. } => {
            if element_id != "page background" {
                diag(
                    out,
                    page_idx,
                    format!(
                        "element `{element_id}`: image fill is not supported on shapes (only page backgrounds support image fills)"
                    ),
                );
            }
            check_image_file(&project.root_dir, src, page_idx, out, element_id);
        }
    }
}

fn check_border(
    theme: Option<&Theme>,
    border: Option<&Border>,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
    element_id: &str,
) {
    let Some(b) = border else { return };
    check_color(
        theme,
        b.color.as_ref(),
        page_idx,
        out,
        element_id,
        "border color",
    );
}

fn check_border_spec(
    theme: Option<&Theme>,
    border: Option<&BorderSpec>,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
    element_id: &str,
) {
    let Some(b) = border else { return };
    match b {
        BorderSpec::Uniform(border) => check_border(theme, Some(border), page_idx, out, element_id),
        BorderSpec::VerticalHorizontal([tb, lr]) => {
            check_border(theme, tb.as_ref(), page_idx, out, element_id);
            check_border(theme, lr.as_ref(), page_idx, out, element_id);
        }
        BorderSpec::Clockwise([t, r, b, l]) => {
            check_border(theme, t.as_ref(), page_idx, out, element_id);
            check_border(theme, r.as_ref(), page_idx, out, element_id);
            check_border(theme, b.as_ref(), page_idx, out, element_id);
            check_border(theme, l.as_ref(), page_idx, out, element_id);
        }
        _ => {}
    }
}

fn check_shadow(
    theme: Option<&Theme>,
    shadow: Option<&Shadow>,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
    element_id: &str,
) {
    let Some(s) = shadow else { return };
    check_color(
        theme,
        Some(&s.color),
        page_idx,
        out,
        element_id,
        "shadow color",
    );
}

fn check_image_file(
    root_dir: &Path,
    src: &str,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
    element_id: &str,
) {
    if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
        return;
    }
    let path = root_dir.join(src);
    if !path.is_file() {
        diag(
            out,
            page_idx,
            format!(
                "image `{element_id}` references missing file `{src}` (file not found in project)"
            ),
        );
    }
}

fn check_rich_text_tags(text: &Text, page_idx: usize, out: &mut Vec<Diagnostic>) {
    let raw = &text.content.text;
    let mut rest = raw.as_str();
    while let Some(lt) = rest.find('<') {
        let tag_slice = &rest[lt..];
        if let Some(gt) = tag_slice.find('>') {
            let tag_content = tag_slice[1..gt].trim();
            let name = tag_content
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('/');
            let name_clean = name.trim_start_matches('/');
            const VALID_TAGS: &[&str] = &[
                "p", "span", "strong", "b", "em", "i", "u", "s", "strike", "del", "br", "code", "latex",
            ];
            if !VALID_TAGS.contains(&name_clean) {
                diag(
                    out,
                    page_idx,
                    format!(
                        "text `{}` contains unsupported tag `<{name}>` (supported: <p>, <span>, <strong>, <em>, <b>, <i>, <u>, <s>, <code>, <latex>, <br>)",
                        text.common.element_id
                    ),
                );
            }
            rest = &tag_slice[gt + 1..];
        } else {
            break;
        }
    }
}

/// Validate the LaTeX bodies embedded in rich text (`<latex>…</latex>` and
/// `\(…\)`). The emitter cannot render commands outside its supported subset,
/// and quietly printing the raw command on the slide is the failure mode this
/// check exists to prevent, so every unsupported command is a hard error.
fn check_latex_commands(text: &Text, page_idx: usize, out: &mut Vec<Diagnostic>) {
    let raw = text.content.text.as_str();
    for span in latex_bodies(raw) {
        for problem in crate::pptx::math::latex_problems(span.body) {
            diag(
                out,
                page_idx,
                format!(
                    "text `{}`: {problem} (inside `{}`)",
                    text.common.element_id, span.open
                ),
            );
        }
    }
    for paragraph in paragraphs(raw) {
        let spans = latex_bodies(paragraph);
        let [span] = spans.as_slice() else {
            continue;
        };
        if !crate::pptx::math::requires_display_paragraph(span.body) {
            continue;
        }
        // The environment owns its paragraph when nothing but whitespace and
        // tags surrounds it — that is the shape the compiler renders as display
        // math. Anything else would be compiled inline and fail.
        let remainder = paragraph.replacen(
            &format!("{}{}{}", span.open, span.body, span.close),
            "",
            1,
        );
        if strip_tags(&remainder).trim().is_empty() {
            continue;
        }
        diag(
            out,
            page_idx,
            format!(
                "text `{}`: a display-only math environment (align/gather/eqnarray) needs a paragraph of its own (inside `{}`)",
                text.common.element_id, span.open
            ),
        );
    }
}

/// A LaTeX formula span inside rich text, with the delimiters that introduced it.
struct LatexSpan<'a> {
    open: &'static str,
    close: &'static str,
    body: &'a str,
}

/// Every LaTeX body in a rich-text string: `<latex>…</latex>` tags and
/// `\(…\)` inline delimiters, paired with the delimiters that introduced them.
fn latex_bodies(raw: &str) -> Vec<LatexSpan<'_>> {
    const TAG_OPEN: &str = "<latex>";
    const TAG_CLOSE: &str = "</latex>";
    let mut out = Vec::new();
    let mut rest = raw;
    while let Some(start) = rest.find(TAG_OPEN) {
        let after = &rest[start + TAG_OPEN.len()..];
        let Some(end) = after.find(TAG_CLOSE) else {
            break;
        };
        out.push(LatexSpan {
            open: TAG_OPEN,
            close: TAG_CLOSE,
            body: &after[..end],
        });
        rest = &after[end + TAG_CLOSE.len()..];
    }
    let mut rest = raw;
    while let Some(start) = rest.find("\\(") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("\\)") else {
            break;
        };
        out.push(LatexSpan {
            open: "\\(",
            close: "\\)",
            body: &after[..end],
        });
        rest = &after[end + 2..];
    }
    out
}

/// The paragraphs of a rich-text string: the contents of each `<p>…</p>`, or the
/// whole string when it carries no paragraph tags.
fn paragraphs(raw: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = raw;
    while let Some(start) = rest.find("<p>") {
        let after = &rest[start + 3..];
        let Some(end) = after.find("</p>") else {
            break;
        };
        out.push(&after[..end]);
        rest = &after[end + 4..];
    }
    if out.is_empty() {
        out.push(raw);
    }
    out
}

fn strip_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    out
}

fn check_text_overflow(
    text: &Text,
    theme: Option<&Theme>,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
) {
    let bounds = text.common.bounds;
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }
    let font_size = text
        .content
        .font_size
        .or_else(|| {
            text.content.style.as_deref().and_then(|s| {
                theme.and_then(|t| {
                    t.text_styles
                        .get(s.trim_start_matches('$'))
                        .and_then(|st| st.font_size)
                })
            })
        })
        .unwrap_or(18.0);

    let plain = strip_tags(&text.content.text);
    if plain.trim().is_empty() {
        return;
    }

    let mut total_h = 0.0;
    let avail_w = (bounds.width - 8.0).max(10.0);
    for line in plain.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            total_h += font_size * 0.8;
            continue;
        }
        let mut visual_units = 0.0;
        for c in trimmed.chars() {
            if c as u32 > 0x2E80 {
                visual_units += 1.0;
            } else {
                visual_units += 0.55;
            }
        }
        let est_line_px = visual_units * font_size;
        let line_count = (est_line_px / avail_w).ceil().max(1.0);
        let line_h = text.content.line_height.unwrap_or(1.25) * font_size;
        total_h += line_count * line_h + 4.0;
    }

    if total_h > bounds.height * 1.35 && bounds.height >= 20.0 {
        let pct = ((total_h - bounds.height) / bounds.height * 100.0).round() as i64;
        diag(
            out,
            page_idx,
            format!(
                "warning: potential text overflow in `{}`: estimated text height ~{:.0}pt exceeds box height {:.0}pt by {pct}%",
                text.common.element_id, total_h, bounds.height
            ),
        );
    }
}

fn check_presentation_raw_yaml(project: &Project, out: &mut Vec<Diagnostic>) {
    let mut entry_candidates = vec![
        project.root_dir.join("deck.pptd"),
        project.root_dir.join("presentation.pptd"),
    ];
    if let Some(title) = &project.presentation.title {
        entry_candidates.push(project.root_dir.join(title));
    }
    for path in &entry_candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(serde_yaml::Value::Mapping(map)) =
                serde_yaml::from_str::<serde_yaml::Value>(&content)
            {
                const VALID_PRES_KEYS: &[&str] = &[
                    "version",
                    "title",
                    "size",
                    "theme",
                    "pages",
                    "layouts",
                    "author",
                    "description",
                    "customFonts",
                ];
                for (k, _) in &map {
                    if let serde_yaml::Value::String(s) = k {
                        if !VALID_PRES_KEYS.contains(&s.as_str()) {
                            out.push(Diagnostic {
                                page: None,
                                message: format!("unknown field `{s}` in deck manifest"),
                            });
                        }
                    }
                }
                break;
            }
        }
    }
}

fn check_page_raw_yaml(project: &Project, page_idx: usize, out: &mut Vec<Diagnostic>) {
    let Some(path) = project.page_paths.get(page_idx) else {
        return;
    };
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
        return;
    };
    let serde_yaml::Value::Mapping(map) = val else {
        return;
    };

    const VALID_PAGE_KEYS: &[&str] = &[
        "version",
        "title",
        "pageType",
        "page_type",
        "background",
        "layout",
        "notes",
        "elements",
        "animations",
        "groups",
    ];
    for (k, _) in &map {
        if let serde_yaml::Value::String(s) = k {
            if !VALID_PAGE_KEYS.contains(&s.as_str()) {
                diag(out, page_idx, format!("unknown field `{s}` on page"));
            }
        }
    }

    if let Some(serde_yaml::Value::Sequence(elems)) = map.get("elements") {
        for elem in elems {
            if let serde_yaml::Value::Mapping(emap) = elem {
                let elem_id = emap
                    .get("elementId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let elem_type = emap
                    .get("elementType")
                    .or_else(|| emap.get("type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("text");

                const COMMON_KEYS: &[&str] = &[
                    "elementId",
                    "elementType",
                    "type",
                    "bounds",
                    "rotation",
                    "opacity",
                    "flip",
                    "groupId",
                    "groupBounds",
                    "className",
                    "style",
                    "order",
                ];
                let type_keys: &[&str] = match elem_type {
                    "text" => &[
                        "content",
                        "fill",
                        "border",
                        "shadow",
                        "padding",
                        "shape",
                        "placeholder",
                    ],
                    "shape" => &[
                        "shapeName",
                        "fill",
                        "border",
                        "shadow",
                        "viewBox",
                        "path",
                        "adjustments",
                    ],
                    "line" => &[
                        "viewBox",
                        "points",
                        "curve",
                        "arrow",
                        "border",
                        "shadow",
                        "stroke",
                        "fill",
                        "startArrow",
                        "endArrow",
                        "startPoint",
                        "endPoint",
                    ],
                    "icon" => &["iconName", "viewBox", "path", "fill", "border", "shadow"],
                    "image" => &[
                        "src",
                        "crop",
                        "fit",
                        "border",
                        "shadow",
                        "opacity",
                        "cropShape",
                        "softEdge",
                    ],
                    "table" => &[
                        "rows",
                        "columnWidths",
                        "rowHeights",
                        "style",
                        "fill",
                        "border",
                        "shadow",
                        "cellPadding",
                    ],
                    "video" | "audio" => &["src", "poster", "autoplay", "loop"],
                    "chart" => &[
                        "data",
                        "series",
                        "seriesDefaults",
                        "title",
                        "legend",
                        "dataLabels",
                        "xAxis",
                        "yAxis",
                        "barWidth",
                        "barGap",
                        "categoryGap",
                        "fill",
                        "border",
                        "fontFamily",
                    ],
                    _ => &[],
                };

                for (k, _) in emap {
                    if let serde_yaml::Value::String(s) = k {
                        if !COMMON_KEYS.contains(&s.as_str()) && !type_keys.contains(&s.as_str()) {
                            diag(
                                out,
                                page_idx,
                                format!("element `{elem_id}` has unknown field `{s}`"),
                            );
                        }
                    }
                }

                if elem_type == "text" {
                    if let Some(serde_yaml::Value::Mapping(cmap)) = emap.get("content") {
                        const CONTENT_KEYS: &[&str] = &[
                            "text",
                            "style",
                            "color",
                            "fontFamily",
                            "fontSize",
                            "bold",
                            "italic",
                            "backgroundColor",
                            "lineHeight",
                            "lineHeightPx",
                            "letterSpacing",
                            "align",
                            "wrap",
                            "direction",
                            "textDirection",
                            "autofit",
                            "marginTop",
                            "marginLeft",
                            "marginRight",
                            "marginBottom",
                            "bodyPrExtras",
                        ];
                        for (k, _) in cmap {
                            if let serde_yaml::Value::String(s) = k {
                                if !CONTENT_KEYS.contains(&s.as_str()) {
                                    diag(
                                        out,
                                        page_idx,
                                        format!("text `{elem_id}` content has unknown field `{s}`"),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_element_canvas_bounds(
    element_id: &str,
    bounds: super::shared::Bounds,
    element: &Element,
    canvas_size: super::shared::Size,
    page_idx: usize,
    out: &mut Vec<Diagnostic>,
) {
    let cw = canvas_size.width;
    let ch = canvas_size.height;
    if !cw.is_finite() || !ch.is_finite() || cw <= 0.0 || ch <= 0.0 {
        return;
    }
    let x = bounds.x;
    let y = bounds.y;
    let w = bounds.width;
    let h = bounds.height;
    if !x.is_finite() || !y.is_finite() || !w.is_finite() || !h.is_finite() {
        return;
    }

    let extends_outside = x < 0.0 || y < 0.0 || (x + w) > cw || (y + h) > ch;
    if !extends_outside {
        return;
    }

    let entirely_outside = match element {
        Element::Line(_) => {
            (w > 0.0 && (x + w <= 0.0 || x >= cw))
                || (w == 0.0 && (x < 0.0 || x > cw))
                || (h > 0.0 && (y + h <= 0.0 || y >= ch))
                || (h == 0.0 && (y < 0.0 || y > ch))
        }
        _ => (x + w <= 0.0) || (x >= cw) || (y + h <= 0.0) || (y >= ch),
    };

    if entirely_outside {
        diag(
            out,
            page_idx,
            format!(
                "element `{element_id}` is entirely outside the slide canvas (bounds: [{x}, {y}, {w}, {h}], canvas: [{cw}, {ch}])"
            ),
        );
    } else {
        diag(
            out,
            page_idx,
            format!(
                "warning: element `{element_id}` extends outside the slide canvas (bounds: [{x}, {y}, {w}, {h}], canvas: [{cw}, {ch}])"
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_project(size: [f64; 2], page_yaml: &str) -> Project {
        let pres_yaml = format!(
            "version: v2\nsize: [{}, {}]\npages: [pages/1.page]\n",
            size[0], size[1]
        );
        let presentation: super::super::Presentation = serde_yaml::from_str(&pres_yaml).unwrap();
        let page: Page = serde_yaml::from_str(page_yaml).unwrap();
        Project {
            root_dir: PathBuf::from("."),
            presentation,
            pages: vec![page],
            page_paths: vec![PathBuf::from("pages/1.page")],
        }
    }

    #[test]
    fn shape_image_fill_is_reported_as_error() {
        let page_yaml = r#"
elements:
  - elementId: shape-with-img
    elementType: shape
    shapeName: rect
    bounds: [50, 50, 200, 100]
    fill:
      type: image
      src: sample.png
"#;
        let proj = make_project([960.0, 540.0], page_yaml);
        let diags = validate_project(&proj);
        let img_fill_diag = diags
            .iter()
            .find(|d| d.message.contains("image fill is not supported on shapes"));
        assert!(
            img_fill_diag.is_some(),
            "expected image fill error on shape, got: {diags:?}"
        );
        let diag = img_fill_diag.unwrap();
        assert!(diag.message.contains("shape-with-img"));
        assert!(!diag.is_warning(), "image fill on shape must be an error");
    }

    #[test]
    fn page_background_image_fill_does_not_trigger_shape_error() {
        let page_yaml = r#"
background:
  type: image
  src: bg.png
elements:
  - elementId: t1
    elementType: text
    bounds: [50, 50, 200, 100]
    content:
      text: hello
"#;
        let proj = make_project([960.0, 540.0], page_yaml);
        let diags = validate_project(&proj);
        assert!(
            !diags
                .iter()
                .any(|d| d.message.contains("image fill is not supported")),
            "page background image fill must not be reported as unsupported: {diags:?}"
        );
    }

    #[test]
    fn element_canvas_bounds_error_when_entirely_outside() {
        let page_yaml = r#"
elements:
  - elementId: way-off-right
    elementType: text
    bounds: [1000, 50, 100, 50]
    content: {text: off}
  - elementId: way-off-left
    elementType: shape
    shapeName: rect
    bounds: [-200, 50, 100, 50]
  - elementId: way-off-bottom
    elementType: shape
    shapeName: rect
    bounds: [50, 600, 100, 50]
  - elementId: way-off-top
    elementType: shape
    shapeName: rect
    bounds: [50, -100, 100, 50]
"#;
        let proj = make_project([960.0, 540.0], page_yaml);
        let diags = validate_project(&proj);

        for id in &["way-off-right", "way-off-left", "way-off-bottom", "way-off-top"] {
            let d = diags
                .iter()
                .find(|d| d.message.contains(id) && d.message.contains("entirely outside"))
                .unwrap_or_else(|| panic!("expected entirely outside error for {id}, got: {diags:?}"));
            assert!(!d.is_warning(), "{id} must be an error, not a warning");
        }
    }

    #[test]
    fn element_canvas_bounds_warning_when_partially_outside() {
        let page_yaml = r#"
elements:
  - elementId: bleed-right
    elementType: text
    bounds: [900, 50, 100, 50]
    content: {text: bleed}
  - elementId: bleed-left
    elementType: shape
    shapeName: rect
    bounds: [-20, 50, 100, 50]
  - elementId: clean-element
    elementType: text
    bounds: [50, 50, 100, 50]
    content: {text: clean}
"#;
        let proj = make_project([960.0, 540.0], page_yaml);
        let diags = validate_project(&proj);

        let right_diag = diags
            .iter()
            .find(|d| d.message.contains("bleed-right"))
            .expect("expected warning for bleed-right");
        assert!(right_diag.is_warning());
        assert!(right_diag.message.contains("extends outside the slide canvas"));

        let left_diag = diags
            .iter()
            .find(|d| d.message.contains("bleed-left"))
            .expect("expected warning for bleed-left");
        assert!(left_diag.is_warning());
        assert!(left_diag.message.contains("extends outside the slide canvas"));

        assert!(
            !diags.iter().any(|d| d.message.contains("clean-element")),
            "clean element must not produce canvas bounds diagnostic"
        );
    }

    #[test]
    fn icon_prefix_validation_rejects_unsupported_prefixes() {
        let page_yaml = r#"
elements:
  - elementId: valid-fas
    elementType: icon
    iconName: "fas:bolt"
    bounds: [50, 50, 30, 30]
  - elementId: valid-unprefixed
    elementType: icon
    iconName: "bolt"
    bounds: [100, 50, 30, 30]
  - elementId: bad-far
    elementType: icon
    iconName: "far:heart"
    bounds: [150, 50, 30, 30]
  - elementId: bad-fab
    elementType: icon
    iconName: "fab:github"
    bounds: [200, 50, 30, 30]
  - elementId: nonexistent-fas
    elementType: icon
    iconName: "fas:nonexistent_icon_xyz"
    bounds: [250, 50, 30, 30]
"#;
        let proj = make_project([960.0, 540.0], page_yaml);
        let diags = validate_project(&proj);

        assert!(
            !diags.iter().any(|d| d.message.contains("valid-fas")),
            "valid fas icon should have no diagnostics: {diags:?}"
        );
        assert!(
            !diags.iter().any(|d| d.message.contains("valid-unprefixed")),
            "valid unprefixed icon should have no diagnostics: {diags:?}"
        );

        let far_diag = diags
            .iter()
            .find(|d| d.message.contains("bad-far"))
            .expect("expected error for far:heart");
        assert!(far_diag.message.contains("unsupported icon style prefix `far:` on `far:heart`"));
        assert!(!far_diag.is_warning());

        let fab_diag = diags
            .iter()
            .find(|d| d.message.contains("bad-fab"))
            .expect("expected error for fab:github");
        assert!(fab_diag.message.contains("unsupported icon style prefix `fab:` on `fab:github`"));
        assert!(!fab_diag.is_warning());

        let not_found_diag = diags
            .iter()
            .find(|d| d.message.contains("nonexistent-fas"))
            .expect("expected error for nonexistent icon");
        assert!(not_found_diag.message.contains("not in the embedded Font Awesome dataset"));
        assert!(!not_found_diag.is_warning());
    }

    #[test]
    fn declared_animations_emit_warning() {
        let page_yaml = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [50, 50, 100, 50]
    content: {text: animated}
animations:
  - elementId: t1
    effect: fade-in
"#;
        let proj = make_project([960.0, 540.0], page_yaml);
        let diags = validate_project(&proj);
        let anim_diag = diags
            .iter()
            .find(|d| d.message.contains("animations are not yet supported"))
            .expect("expected animation warning");
        assert!(anim_diag.is_warning());
        assert!(anim_diag.message.contains("warning: page declares 1 animation(s)"));
    }
}
