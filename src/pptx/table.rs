//! PPTD `table` → native `<a:tbl>` inside a `<p:graphicFrame>`.
//!
//! Style resolution follows the PPTD inheritance chain (lowest first):
//! defaults → `TableStyleConfig.cellStyle` → `bodyStyles` (data rows) →
//! first/last column category → first/last row category (row wins unless
//! `rowOverColumn: false`) → `Cell.textStyle` (`theme.textStyles`, text
//! fields only) → `Cell` inline fields.
//!
//! Merged cells follow the spec's ragged `rows` convention: covered cells are
//! omitted from the source arrays, so they are reconstructed here as
//! `hMerge` / `vMerge` placeholder cells while the origin cell carries
//! `gridSpan` / `rowSpan`.

use crate::pptd::Theme;
use crate::pptd::elements::{Cell, Table};
use crate::pptd::shared::{
    Alignment, Border, BorderSpec, Color, Fill, FontFamily, HorizontalAlign, LineStyle,
    VerticalAlign,
};
use crate::pptd::theme::{CellStyle, TableStyleConfig, TableStyleRef, TextStyleConfig};
use crate::{Error, Result};

use super::render::{EffTextStyle, emu, fill_xml, render_rich_body, shadow_xml};
use super::xml::Xml;

/// Accumulated cell style while walking the inheritance chain.
#[derive(Default)]
struct CellAcc {
    color: Option<Color>,
    font_size: Option<f64>,
    font_family: Option<FontFamily>,
    bold: Option<bool>,
    italic: Option<bool>,
    background_color: Option<Color>,
    line_height: Option<f64>,
    line_height_px: Option<f64>,
    letter_spacing: Option<f64>,
    margin_top: Option<f64>,
    fill: Option<Fill>,
    border: Option<BorderSpec>,
    align: Option<Alignment>,
}

/// Apply a text-level style over the accumulator (set fields win).
fn acc_text(acc: &mut CellAcc, s: &TextStyleConfig) {
    if let Some(v) = &s.color {
        acc.color = Some(v.clone());
    }
    if let Some(v) = s.font_size {
        acc.font_size = Some(v);
    }
    if let Some(v) = &s.font_family {
        acc.font_family = Some(v.clone());
    }
    if let Some(v) = s.bold {
        acc.bold = Some(v);
    }
    if let Some(v) = s.italic {
        acc.italic = Some(v);
    }
    if let Some(v) = &s.background_color {
        acc.background_color = Some(v.clone());
    }
    if let Some(v) = s.line_height {
        acc.line_height = Some(v);
    }
    if let Some(v) = s.line_height_px {
        acc.line_height_px = Some(v);
    }
    if let Some(v) = s.letter_spacing {
        acc.letter_spacing = Some(v);
    }
    if let Some(v) = s.margin_top {
        acc.margin_top = Some(v);
    }
}

/// Apply a cell-level style (text part + fill / border / align).
fn acc_cell(acc: &mut CellAcc, s: &CellStyle) {
    acc_text(acc, &s.text);
    if let Some(v) = &s.fill {
        acc.fill = Some(v.clone());
    }
    if let Some(v) = &s.border {
        acc.border = Some(v.clone());
    }
    if let Some(v) = s.align {
        acc.align = Some(v);
    }
}

/// Resolve the effective style of the logical cell at `(r, c)`.
fn resolve_cell(
    theme: Option<&Theme>,
    cfg: Option<&TableStyleConfig>,
    cell: &Cell,
    r: usize,
    c: usize,
    n_rows: usize,
    n_cols: usize,
) -> CellAcc {
    let mut acc = CellAcc::default();
    let cfg = match cfg {
        Some(c) => c,
        None => {
            apply_inline(&mut acc, theme, cell);
            return acc;
        }
    };
    if let Some(s) = &cfg.cell_style {
        acc_cell(&mut acc, s);
    }
    let is_first = r == 0;
    let is_last = r + 1 == n_rows;
    if !is_first && !is_last && !cfg.body_styles.is_empty() {
        let idx = (r - 1) % cfg.body_styles.len();
        acc_cell(&mut acc, &cfg.body_styles[idx]);
    }
    let row_cat = if is_first {
        cfg.first_row_style.as_ref()
    } else if is_last {
        cfg.last_row_style.as_ref()
    } else {
        None
    };
    let col_cat = if c == 0 {
        cfg.first_column_style.as_ref()
    } else if c + 1 == n_cols {
        cfg.last_column_style.as_ref()
    } else {
        None
    };
    if cfg.row_over_column.unwrap_or(true) {
        if let Some(s) = col_cat {
            acc_cell(&mut acc, s);
        }
        if let Some(s) = row_cat {
            acc_cell(&mut acc, s);
        }
    } else {
        if let Some(s) = row_cat {
            acc_cell(&mut acc, s);
        }
        if let Some(s) = col_cat {
            acc_cell(&mut acc, s);
        }
    }
    apply_inline(&mut acc, theme, cell);
    acc
}

/// `Cell.textStyle` (theme text style, text fields only) then inline fields.
fn apply_inline(acc: &mut CellAcc, theme: Option<&Theme>, cell: &Cell) {
    if let Some(reference) = &cell.text_style {
        if let Some(s) = theme.and_then(|t| t.text_styles.get(reference.trim_start_matches('$'))) {
            acc_text(acc, s);
        }
    }
    if let Some(v) = &cell.color {
        acc.color = Some(v.clone());
    }
    if let Some(v) = cell.font_size {
        acc.font_size = Some(v);
    }
    if let Some(v) = &cell.font_family {
        acc.font_family = Some(v.clone());
    }
    if let Some(v) = cell.bold {
        acc.bold = Some(v);
    }
    if let Some(v) = cell.italic {
        acc.italic = Some(v);
    }
    if let Some(v) = &cell.background_color {
        acc.background_color = Some(v.clone());
    }
    if let Some(v) = cell.line_height {
        acc.line_height = Some(v);
    }
    if let Some(v) = cell.line_height_px {
        acc.line_height_px = Some(v);
    }
    if let Some(v) = cell.letter_spacing {
        acc.letter_spacing = Some(v);
    }
    if let Some(v) = cell.margin_top {
        acc.margin_top = Some(v);
    }
    if let Some(v) = &cell.fill {
        acc.fill = Some(v.clone());
    }
    if let Some(v) = &cell.border {
        acc.border = Some(v.clone());
    }
    if let Some(v) = cell.align {
        acc.align = Some(v);
    }
}

/// Per-side borders after resolving a [`BorderSpec`]: `[left, right, top, bottom]`.
/// `None` entry = explicitly cleared side. Spec default when nothing was set
/// anywhere: 1px solid black on all four sides.
fn border_sides(spec: Option<&BorderSpec>) -> [Option<Border>; 4] {
    match spec {
        None | Some(BorderSpec::Clear) => [None, None, None, None],
        Some(BorderSpec::Uniform(b)) => [
            Some(b.clone()),
            Some(b.clone()),
            Some(b.clone()),
            Some(b.clone()),
        ],
        Some(BorderSpec::VerticalHorizontal([tb, lr])) => {
            [lr.clone(), lr.clone(), tb.clone(), tb.clone()]
        }
        Some(BorderSpec::Clockwise([t, r, b, l])) => [l.clone(), r.clone(), t.clone(), b.clone()],
    }
}

/// One cell-side border: `<a:lnL/lnR/lnT/lnB>`.
/// When unspecified (`None`), the XML element is omitted so the neighbour's border survives.
/// When explicitly set to none (e.g. width == 0), `<a:noFill/>` is emitted.
fn cell_border_xml(
    xml: &mut Xml,
    theme: Option<&Theme>,
    tag: &str,
    border: Option<&Border>,
) -> Result<()> {
    let Some(border) = border else {
        return Ok(());
    };
    if border.width.is_some_and(|w| w == 0.0) {
        xml.start(tag, &[]);
        xml.leaf("a:noFill", &[]);
        xml.end(tag);
        return Ok(());
    }
    let width = emu(border.width.unwrap_or(1.0)).to_string();
    xml.start(tag, &[("w", &width), ("cap", "flat")]);
    if let Some(grad) = &border.gradient {
        fill_xml(
            xml,
            theme,
            &Fill::Gradient {
                gradient_type: grad.gradient_type,
                stops: grad.stops.clone(),
                angle: grad.angle,
                scaled: grad.scaled,
            },
            None,
        )?;
    } else if let Some(color) = &border.color {
        fill_xml(
            xml,
            theme,
            &Fill::Solid {
                color: color.clone(),
            },
            None,
        )?;
    }
    let dash = border.style.map(|style| match style {
        LineStyle::Solid => "solid",
        LineStyle::Dash => "dash",
        LineStyle::Dot => "dot",
        _ => "sysDot",
    });
    if let Some(dash) = dash {
        xml.leaf("a:prstDash", &[("val", dash)]);
    }
    xml.end(tag);
    Ok(())
}

/// `bodyPr`/`tcPr` vertical anchor from the resolved alignment (PPTD default
/// for cells is `[center, middle]`).
fn cell_anchor(align: Option<Alignment>) -> &'static str {
    match align.map(|a| a.vertical) {
        Some(VerticalAlign::Top) => "t",
        Some(VerticalAlign::Bottom) => "b",
        _ => "ctr",
    }
}

/// One logical grid slot after reconstructing merges.
enum Slot<'a> {
    /// No source cell claims this position (only possible past the declared
    /// rows); renders as an empty cell.
    Empty,
    /// Merge-covered placeholder; `(h_merge, v_merge)`.
    Covered(bool, bool),
    /// A real cell; `(cell, row_span, col_span)`.
    Origin(&'a Cell, u32, u32),
}

/// Render a `table` element as a native `<a:tbl>` graphic frame.
pub(crate) fn render_table(
    xml: &mut Xml,
    ctx: &mut super::render::RenderCtx<'_>,
    table: &Table,
) -> Result<()> {
    let theme = ctx.theme;
    let cfg: Option<TableStyleConfig> = match &table.style {
        Some(TableStyleRef::Ref(key)) => theme
            .and_then(|t| t.table_styles.get(key.trim_start_matches('$')))
            .cloned(),
        Some(TableStyleRef::Inline(cfg)) => Some((**cfg).clone()),
        None => None,
    };
    let bounds = &table.common.bounds;
    let n_cols = table.column_widths.len();
    let n_rows = table.row_heights.len();
    if n_rows == 0 || n_cols == 0 {
        return Err(Error::Unsupported(format!(
            "table `{}` has an empty grid",
            table.common.element_id
        )));
    }

    // Reconstruct the logical grid from the ragged `rows` array.
    let mut slots: Vec<Vec<Slot>> = (0..n_rows)
        .map(|_| (0..n_cols).map(|_| Slot::Empty).collect())
        .collect();
    for (r, row) in table.rows.iter().enumerate() {
        if r >= n_rows {
            return Err(Error::Unsupported(format!(
                "table `{}` declares more row entries than `rowHeights`",
                table.common.element_id
            )));
        }
        let mut c = 0usize;
        for cell in row {
            while c < n_cols && matches!(slots[r][c], Slot::Covered(..) | Slot::Origin(..)) {
                c += 1;
            }
            if c >= n_cols {
                return Err(Error::Unsupported(format!(
                    "table `{}` row {} has more cells than `columnWidths`",
                    table.common.element_id,
                    r + 1
                )));
            }
            let rs = cell.row_span.unwrap_or(1).max(1);
            let cs = cell.col_span.unwrap_or(1).max(1);
            for dr in 0..rs as usize {
                for dc in 0..cs as usize {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let (rr, cc) = (r + dr, c + dc);
                    if rr >= n_rows || cc >= n_cols {
                        return Err(Error::Unsupported(format!(
                            "table `{}` merge at row {} col {} escapes the grid",
                            table.common.element_id,
                            r + 1,
                            c + 1
                        )));
                    }
                    slots[rr][cc] = Slot::Covered(dc > 0, dr > 0);
                }
            }
            slots[r][c] = Slot::Origin(cell, rs, cs);
            c += cs as usize;
        }
    }

    let id = ctx.next_id().to_string();
    xml.start("p:graphicFrame", &[]);
    xml.start("p:nvGraphicFramePr", &[]);
    super::render::cnv_pr(
        xml,
        ctx,
        &id,
        &table.common.element_id,
        table.common.link.as_deref(),
    );
    xml.leaf("p:cNvGraphicFramePr", &[]);
    xml.leaf("p:nvPr", &[]);
    xml.end("p:nvGraphicFramePr");
    xml.start("p:xfrm", &[]);
    let (x, y) = (emu(bounds.x).to_string(), emu(bounds.y).to_string());
    let (cx, cy) = (
        emu(bounds.width).to_string(),
        emu(bounds.height).to_string(),
    );
    xml.leaf("a:off", &[("x", &x), ("y", &y)]);
    xml.leaf("a:ext", &[("cx", &cx), ("cy", &cy)]);
    xml.end("p:xfrm");
    xml.start("a:graphic", &[]);
    xml.start(
        "a:graphicData",
        &[(
            "uri",
            "http://schemas.openxmlformats.org/drawingml/2006/table",
        )],
    );
    xml.start("a:tbl", &[]);
    // No `tableStyleId`: keeps PowerPoint's default banded style out; every
    // fill/border below is explicit.
    xml.start("a:tblPr", &[("firstRow", "0"), ("bandRow", "0")]);
    if let Some(fill) = &table.fill {
        fill_xml(xml, theme, fill, None)?;
    }
    // `a:tblPr` accepts an `a:effectLst` slot; a graphicFrame has no `p:spPr`,
    // so this is the only place a table-level shadow can be expressed.
    if let Some(shadow) = &table.shadow {
        shadow_xml(xml, theme, shadow)?;
    }
    xml.end("a:tblPr");
    xml.start("a:tblGrid", &[]);
    for ratio in &table.column_widths {
        let w = emu(ratio * bounds.width).to_string();
        xml.leaf("a:gridCol", &[("w", &w)]);
    }
    xml.end("a:tblGrid");

    for (r, row_slots) in slots.iter().enumerate() {
        let h = emu(table.row_heights[r] * bounds.height).to_string();
        xml.start("a:tr", &[("h", &h)]);
        for (c, slot) in row_slots.iter().enumerate() {
            match slot {
                Slot::Empty => emit_cell(xml, theme, None, None, false, false)?,
                Slot::Covered(h_merge, v_merge) => {
                    emit_cell(xml, theme, None, None, *h_merge, *v_merge)?
                }
                Slot::Origin(cell, rs, cs) => {
                    let acc = resolve_cell(theme, cfg.as_ref(), cell, r, c, n_rows, n_cols);
                    emit_origin_cell(xml, ctx, cell, &acc, *rs, *cs)?;
                }
            }
        }
        xml.end("a:tr");
    }
    xml.end("a:tbl");
    xml.end("a:graphicData");
    xml.end("a:graphic");
    xml.end("p:graphicFrame");
    Ok(())
}

/// Build the box-level text style for the cell's `a:txBody`.
fn cell_eff_style(acc: &CellAcc) -> EffTextStyle {
    let align = acc.align.or(Some(Alignment::from((
        HorizontalAlign::Center,
        VerticalAlign::Middle,
    ))));
    EffTextStyle {
        color: acc.color.clone(),
        font_size: acc.font_size,
        font_family: acc.font_family.clone(),
        bold: acc.bold,
        italic: acc.italic,
        line_height: acc.line_height,
        line_height_px: acc.line_height_px,
        letter_spacing: acc.letter_spacing,
        align,
        wrap: None,
        direction: None,
        margin_top: acc.margin_top,
        margin_left: None,
        margin_right: None,
        margin_bottom: None,
        autofit: None,
        body_pr_extras: None,
        bullet_char: None,
        bullet_font: None,
        list_margin: None,
        list_indent: None,
        gradient: None,
        shadow: None,
    }
}

/// `<a:tcPr>`: cell margins (PowerPoint defaults 0.1in / 0.05in), anchor,
/// per-side borders (schema order lnL, lnR, lnT, lnB), then fill.
fn emit_tc_pr(
    xml: &mut Xml,
    theme: Option<&Theme>,
    acc: Option<&CellAcc>,
    anchor: &str,
) -> Result<()> {
    let (l, r, t, b) = (
        emu(7.2).to_string(),
        emu(7.2).to_string(),
        emu(3.6).to_string(),
        emu(3.6).to_string(),
    );
    xml.start(
        "a:tcPr",
        &[
            ("marL", &l),
            ("marR", &r),
            ("marT", &t),
            ("marB", &b),
            ("anchor", anchor),
        ],
    );
    if let Some(acc) = acc {
        let sides = border_sides(acc.border.as_ref());
        cell_border_xml(xml, theme, "a:lnL", sides[0].as_ref())?;
        cell_border_xml(xml, theme, "a:lnR", sides[1].as_ref())?;
        cell_border_xml(xml, theme, "a:lnT", sides[2].as_ref())?;
        cell_border_xml(xml, theme, "a:lnB", sides[3].as_ref())?;
        if let Some(fill) = &acc.fill {
            fill_xml(xml, theme, fill, None)?;
        }
    }
    xml.end("a:tcPr");
    Ok(())
}

/// An empty or merge-covered `<a:tc>`.
fn emit_cell(
    xml: &mut Xml,
    theme: Option<&Theme>,
    acc: Option<&CellAcc>,
    _text: Option<&str>,
    h_merge: bool,
    v_merge: bool,
) -> Result<()> {
    let mut attrs: Vec<(&str, &str)> = Vec::new();
    if h_merge {
        attrs.push(("hMerge", "1"));
    }
    if v_merge {
        attrs.push(("vMerge", "1"));
    }
    xml.start("a:tc", &attrs);
    xml.start("a:txBody", &[]);
    xml.leaf("a:bodyPr", &[]);
    xml.leaf("a:lstStyle", &[]);
    xml.leaf("a:p", &[]);
    xml.end("a:txBody");
    emit_tc_pr(xml, theme, acc, "ctr")?;
    xml.end("a:tc");
    Ok(())
}

/// A content cell with resolved style and optional merge spans.
fn emit_origin_cell(
    xml: &mut Xml,
    ctx: &mut super::render::RenderCtx<'_>,
    cell: &Cell,
    acc: &CellAcc,
    row_span: u32,
    col_span: u32,
) -> Result<()> {
    let theme = ctx.theme;
    let rs = row_span.to_string();
    let cs = col_span.to_string();
    let mut attrs: Vec<(&str, &str)> = Vec::new();
    if col_span > 1 {
        attrs.push(("gridSpan", &cs));
    }
    if row_span > 1 {
        attrs.push(("rowSpan", &rs));
    }
    xml.start("a:tc", &attrs);
    let anchor = cell_anchor(acc.align);
    let text = cell.text.as_deref().unwrap_or("");
    xml.start("a:txBody", &[]);
    if text.is_empty() {
        xml.leaf("a:bodyPr", &[]);
        xml.leaf("a:lstStyle", &[]);
        xml.leaf("a:p", &[]);
    } else {
        let eff = cell_eff_style(acc);
        render_rich_body(xml, ctx, &eff, text, anchor, false)?;
    }
    xml.end("a:txBody");
    emit_tc_pr(xml, theme, Some(acc), anchor)?;
    xml.end("a:tc");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unspecified_border_omits_xml_elements() {
        let mut xml = Xml::new();
        cell_border_xml(&mut xml, None, "a:lnT", None).unwrap();
        let out = xml.into_string();
        assert!(
            !out.contains("<a:lnT"),
            "unspecified border must not emit any XML element: `{out}`"
        );
    }

    #[test]
    fn explicit_zero_width_border_emits_nofill() {
        let mut xml = Xml::new();
        let zero = Border {
            width: Some(0.0),
            ..Default::default()
        };
        cell_border_xml(&mut xml, None, "a:lnT", Some(&zero)).unwrap();
        let out = xml.into_string();
        assert!(
            out.contains("<a:lnT") && out.contains("<a:noFill/>"),
            "width 0 must emit noFill: `{out}`"
        );
    }

    #[test]
    fn positive_width_border_emits_line() {
        let mut xml = Xml::new();
        let b = Border {
            width: Some(1.5),
            color: Some(Color("#FF0000".to_string())),
            ..Default::default()
        };
        cell_border_xml(&mut xml, None, "a:lnB", Some(&b)).unwrap();
        let out = xml.into_string();
        assert!(out.contains("<a:lnB"), "must emit lnB: `{out}`");
        assert!(out.contains("FF0000"), "must contain color: `{out}`");
        assert!(!out.contains("noFill"), "must not contain noFill");
    }

    #[test]
    fn header_bottom_only_and_body_clear_emit_correct_borders() {
        // Header cell with [null, null, border, null]
        let header_acc = CellAcc {
            border: Some(BorderSpec::Clockwise([
                None,
                None,
                Some(Border {
                    width: Some(2.0),
                    color: Some(Color("#112233".to_string())),
                    ..Default::default()
                }),
                None,
            ])),
            ..Default::default()
        };
        let mut xml = Xml::new();
        emit_tc_pr(&mut xml, None, Some(&header_acc), "ctr").unwrap();
        let header_out = xml.into_string();
        assert!(header_out.contains("<a:lnB"), "header must have lnB");
        assert!(!header_out.contains("<a:lnT"), "header must omit lnT");
        assert!(!header_out.contains("<a:lnL"), "header must omit lnL");
        assert!(!header_out.contains("<a:lnR"), "header must omit lnR");
        assert!(
            !header_out.contains("noFill"),
            "header must not contain noFill"
        );

        // Body cell with border: null (BorderSpec::Clear)
        let body_acc = CellAcc {
            border: Some(BorderSpec::Clear),
            ..Default::default()
        };
        let mut xml_body = Xml::new();
        emit_tc_pr(&mut xml_body, None, Some(&body_acc), "ctr").unwrap();
        let body_out = xml_body.into_string();
        assert!(
            !body_out.contains("<a:lnT"),
            "body cell must omit lnT so header lnB survives"
        );
        assert!(!body_out.contains("<a:lnB"), "body cell must omit lnB");
        assert!(!body_out.contains("<a:lnL"), "body cell must omit lnL");
        assert!(!body_out.contains("<a:lnR"), "body cell must omit lnR");
        assert!(
            !body_out.contains("noFill"),
            "body cell must not contain noFill"
        );
    }
}
