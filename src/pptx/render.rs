//! Element → drawingml rendering inside a slide's `cSld/spTree`.
//!
//! Supported: `text`, `shape`, `line`, `icon` (Font Awesome glyphs as custom
//! geometry), `image` (blip media parts), `table` (native `<a:tbl>`) and
//! `chart` (native chart parts, emitted as `p:graphicFrame`). Shape
//! drop shadows (`a:effectLst > a:outerShdw`, incl. `sx`/`sy` scale + `algn`)
//! `<p:grpSp>` group reconstruction (SlideForge group extension) are
//! supported.

use crate::pptd::elements::{ArrowType, GroupDef, GroupXfrm};
use crate::pptd::shared::{
    Alignment, Border, Bounds, Color, Fill, FontFamily, GradientType, HorizontalAlign,
    ImageFitMode, LineStyle, Shadow, VerticalAlign,
};
use crate::pptd::{
    Element, Icon, Image, Line, LineCurve, Media, PlaceholderDef, Shape, Text, TextAutofit,
    TextContent, TextDirection, Theme,
};
use crate::{Error, Result};

use super::media::ImageSize;
use super::theme::{ResolvedColor, resolve_color};
use super::xml::Xml;
use super::{fa, svg_path};

/// Rendering context shared while painting one slide.
pub struct RenderCtx<'a> {
    pub theme: Option<&'a Theme>,
    shape_id: usize,
    /// Media sources used by the current slide, in usage order.
    pub media: Vec<String>,
    /// Media source → pixel dimensions (for `contain`/`cover` cropping).
    pub image_sizes: Vec<(String, ImageSize)>,
    /// Chart sources used by the current slide, in usage order. Chart
    /// relationships follow the media ones (`rId2` onwards).
    pub charts: Vec<String>,
    /// True when rendering a `slideLayout` (not a slide): layout placeholders
    /// emit `<p:ph type=…>` + their `lstStyle`/prompt so slide placeholders
    /// inherit them; slide placeholders stay plain shapes (`txBox="1"`).
    pub in_layout: bool,
    /// External hyperlink targets used by this part, in first-use order.
    /// Relationship ids follow the media ids (`rId2` onwards).
    pub hyperlinks: Vec<String>,
    /// `safeFonts: true` maps web-only typefaces onto system fonts. Default
    /// is `false`: PPTD font names are written verbatim so round-trips keep
    /// the source typography.
    pub safe_fonts: bool,
}

impl<'a> RenderCtx<'a> {
    pub fn new(theme: Option<&'a Theme>) -> Self {
        Self {
            theme,
            shape_id: 1,
            media: Vec::new(),
            image_sizes: Vec::new(),
            charts: Vec::new(),
            in_layout: false,
            hyperlinks: Vec::new(),
            safe_fonts: false,
        }
    }

    /// Relationship id for an external hyperlink, registering the target on
    /// first use. Ids start after the part's media and chart relationships.
    pub fn hyperlink_rid(&mut self, url: &str) -> String {
        let pos = match self.hyperlinks.iter().position(|u| u == url) {
            Some(pos) => pos,
            None => {
                self.hyperlinks.push(url.to_owned());
                self.hyperlinks.len() - 1
            }
        };
        format!("rId{}", self.media.len() + self.charts.len() + 2 + pos)
    }

    /// Relationship id for a chart source: after the media relationships.
    pub fn chart_rid(&self, src: &str) -> Option<String> {
        self.charts
            .iter()
            .position(|s| s == src)
            .map(|p| format!("rId{}", self.media.len() + 2 + p))
    }

    pub(crate) fn next_id(&mut self) -> usize {
        self.shape_id += 1;
        self.shape_id
    }

    /// Relationship id for a media source: `rId2` onwards (rId1 = layout).
    pub fn media_rid(&self, src: &str) -> Option<String> {
        self.media
            .iter()
            .position(|s| s == src)
            .map(|p| format!("rId{}", p + 2))
    }

    pub fn image_size(&self, src: &str) -> Option<ImageSize> {
        self.image_sizes
            .iter()
            .find(|(s, _)| s == src)
            .map(|(_, size)| *size)
    }
}

/// Render one page element into the slide's `spTree`.
pub fn render_element(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    element: &Element,
    page_index: usize,
) -> Result<()> {
    match element {
        Element::Text(text) => render_text(xml, ctx, text, page_index),
        Element::Shape(shape) => render_shape(xml, ctx, shape),
        Element::Line(line) => render_line(xml, ctx, line),
        Element::Icon(icon) => render_icon(xml, ctx, icon),
        Element::Image(image) => render_image(xml, ctx, image),
        Element::Table(table) => super::table::render_table(xml, ctx, table),
        Element::Chart(chart) => super::chart::render_chart(xml, ctx, chart),
        Element::Video(m) => render_media(xml, ctx, m, "video"),
        Element::Audio(m) => render_media(xml, ctx, m, "audio"),
    }
}

/// Render a flat `elements` array into a `p:spTree`, reconstructing
/// `<p:grpSp>` groups from the SlideForge group extension (`groupId` +
/// `groupBounds` on members, `Page.groups` metadata). Elements without a
/// `groupId` render inline (slide-space `bounds`); grouped members render
/// inside the rebuilt group with their child-space `groupBounds`.
pub fn render_sp_tree(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    elements: &[Element],
    groups: Option<&std::collections::HashMap<String, GroupDef>>,
    page_index: usize,
) -> Result<()> {
    let mut i = 0;
    while i < elements.len() {
        match elements[i].common().group_id.as_deref() {
            None => {
                render_element(xml, ctx, &elements[i], page_index)?;
                i += 1;
            }
            Some(gid) => {
                let is_top = groups
                    .and_then(|m| m.get(gid))
                    .map(|g| g.parent.is_none())
                    .unwrap_or(false);
                if !is_top {
                    // Orphan (parent group not preserved): fall back to flat.
                    render_element(xml, ctx, &elements[i], page_index)?;
                    i += 1;
                    continue;
                }
                let end = group_span_end(elements, groups, i, gid);
                render_group(xml, ctx, elements, groups, i, end, gid, page_index)?;
                i = end;
            }
        }
    }
    Ok(())
}

/// Emit a `<p:grpSp>` over `elements[start..end)` (members + nested groups).
// 8 args because each one names a different axis of the OOXML group model
// (the body, the surrounding context, the member range, the group metadata,
// and the page). Bundling them into a struct would lose the per-axis
// semantics at every call site.
#[allow(clippy::too_many_arguments)]
fn render_group(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    elements: &[Element],
    groups: Option<&std::collections::HashMap<String, GroupDef>>,
    start: usize,
    end: usize,
    gid: &str,
    page_index: usize,
) -> Result<()> {
    let def = groups
        .and_then(|m| m.get(gid))
        .expect("group metadata for member element");
    let id = ctx.next_id().to_string();
    let name = def.name.clone().unwrap_or_else(|| format!("Group {gid}"));
    xml.start("p:grpSp", &[]);
    xml.start("p:nvGrpSpPr", &[]);
    xml.start("p:cNvPr", &[("id", &id), ("name", &name)]);
    xml.end("p:cNvPr");
    xml.leaf("p:cNvGrpSpPr", &[]);
    xml.leaf("p:nvPr", &[]);
    xml.end("p:nvGrpSpPr");
    xml.start("p:grpSpPr", &[]);
    group_xfrm(xml, &def.xfrm);
    if let Some(fill) = &def.fill {
        fill_xml(xml, None, fill, None)?;
    }
    xml.end("p:grpSpPr");
    let mut i = start;
    while i < end {
        match elements[i].common().group_id.as_deref() {
            Some(child_gid) if child_gid == gid => {
                // Direct leaf: swap in child-space `groupBounds`.
                let mut child = elements[i].clone();
                if let Some(gb) = child.common().group_bounds {
                    child.common_mut().bounds = gb;
                }
                render_element(xml, ctx, &child, page_index)?;
                i += 1;
            }
            Some(child_gid) => {
                // Nested group (child_gid's parent is this group).
                let nested_end = group_span_end(elements, groups, i, child_gid);
                render_group(
                    xml, ctx, elements, groups, i, nested_end, child_gid, page_index,
                )?;
                i = nested_end;
            }
            None => {
                render_element(xml, ctx, &elements[i], page_index)?;
                i += 1;
            }
        }
    }
    xml.end("p:grpSp");
    Ok(())
}

/// A group `<a:xfrm>` (off/ext/chOff/chExt + rot/flip), px → EMU.
fn group_xfrm(xml: &mut Xml, x: &GroupXfrm) {
    let off_x = emu(x.off.0).to_string();
    let off_y = emu(x.off.1).to_string();
    let ext_cx = emu(x.ext.0).to_string();
    let ext_cy = emu(x.ext.1).to_string();
    let cho_x = emu(x.ch_off.0).to_string();
    let cho_y = emu(x.ch_off.1).to_string();
    let che_cx = emu(x.ch_ext.0).to_string();
    let che_cy = emu(x.ch_ext.1).to_string();
    let rot = x
        .rot
        .filter(|d| d.abs() > 1e-9)
        .map(|d| ((d * 60000.0).round()).to_string());
    let (fh, fv) = x.flip.unwrap_or((false, false));
    let mut attrs: Vec<(&str, &str)> = Vec::new();
    if let Some(r) = rot.as_deref() {
        attrs.push(("rot", r));
    }
    if fh {
        attrs.push(("flipH", "1"));
    }
    if fv {
        attrs.push(("flipV", "1"));
    }
    xml.start("a:xfrm", &attrs);
    xml.leaf("a:off", &[("x", &off_x), ("y", &off_y)]);
    xml.leaf("a:ext", &[("cx", &ext_cx), ("cy", &ext_cy)]);
    xml.leaf("a:chOff", &[("x", &cho_x), ("y", &cho_y)]);
    xml.leaf("a:chExt", &[("cx", &che_cx), ("cy", &che_cy)]);
    xml.end("a:xfrm");
}

/// End index (exclusive) of the run of elements belonging to `gid` or its
/// descendants, starting at `start`.
fn group_span_end(
    elements: &[Element],
    groups: Option<&std::collections::HashMap<String, GroupDef>>,
    start: usize,
    gid: &str,
) -> usize {
    let mut j = start;
    while j < elements.len() {
        match elements[j].common().group_id.as_deref() {
            Some(id) if id == gid || is_descendant(groups, id, gid) => j += 1,
            _ => break,
        }
    }
    j
}

/// Is `id` a (transitive) child group of `ancestor`?
fn is_descendant(
    groups: Option<&std::collections::HashMap<String, GroupDef>>,
    id: &str,
    ancestor: &str,
) -> bool {
    let g = match groups.and_then(|m| m.get(id)) {
        Some(g) => g,
        None => return false,
    };
    match &g.parent {
        Some(p) if p == ancestor => true,
        Some(p) => is_descendant(groups, p, ancestor),
        None => false,
    }
}

/// px → EMU (1px = 1pt = 12700 EMU).
pub fn emu(px: f64) -> i64 {
    (px * 12700.0).round() as i64
}

/// Emit `p:cNvPr` with its optional external click target.
pub(crate) fn cnv_pr(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    id: &str,
    name: &str,
    link: Option<&str>,
) {
    match link {
        Some(url) => {
            xml.start("p:cNvPr", &[("id", id), ("name", name)]);
            let rid = ctx.hyperlink_rid(url);
            xml.leaf("a:hlinkClick", &[("r:id", &rid)]);
            xml.end("p:cNvPr");
        }
        None => xml.leaf("p:cNvPr", &[("id", id), ("name", name)]),
    }
}

/// `a:xfrm` from bounds / rotation / flip.
/// `a:xfrm` from bounds / rotation / flip. `rot` / `flipH` / `flipV` are
/// attributes of the transform (CT_Transform2D), never child elements.
fn xfrm(xml: &mut Xml, bounds: Bounds, rotation: Option<f64>, flip: Option<(bool, bool)>) {
    xfrm_element(xml, "a:xfrm", bounds, rotation, flip);
}

/// Same transform, but under a caller-chosen element name: `p:graphicFrame`
/// carries `p:xfrm` with the identical child layout.
pub(crate) fn xfrm_element(
    xml: &mut Xml,
    name: &str,
    bounds: Bounds,
    rotation: Option<f64>,
    flip: Option<(bool, bool)>,
) {
    let x = emu(bounds.x).to_string();
    let y = emu(bounds.y).to_string();
    let cx = emu(bounds.width).to_string();
    let cy = emu(bounds.height).to_string();
    let rot = rotation
        .filter(|deg| deg.abs() > 1e-9)
        .map(|deg| ((deg * 60000.0).round()).to_string());
    let (flip_h, flip_v) = flip.unwrap_or((false, false));

    let mut attrs: Vec<(&str, &str)> = Vec::new();
    if let Some(rot) = rot.as_deref() {
        attrs.push(("rot", rot));
    }
    if flip_h {
        attrs.push(("flipH", "1"));
    }
    if flip_v {
        attrs.push(("flipV", "1"));
    }
    xml.start(name, &attrs);
    xml.leaf("a:off", &[("x", &x), ("y", &y)]);
    xml.leaf("a:ext", &[("cx", &cx), ("cy", &cy)]);
    xml.end(name);
}

/// `srgbClr` optionally carrying an alpha override.
fn srgb(xml: &mut Xml, rgb: &str, alpha: Option<f64>) {
    xml.start("a:srgbClr", &[("val", rgb)]);
    if let Some(alpha) = alpha {
        let alpha = ((alpha * 100000.0).round() as u64).to_string();
        xml.leaf("a:alpha", &[("val", &alpha)]);
    }
    xml.end("a:srgbClr");
}

fn emit_fill_color(xml: &mut Xml, color: &ResolvedColor) {
    xml.start("a:solidFill", &[]);
    srgb(xml, &color.rgb, color.alpha);
    xml.end("a:solidFill");
}

/// `a:ln` from a border spec (style / width / color).
/// `a:ln` from a border spec. Schema order inside `a:ln`: fill, then
/// `a:prstDash`, then the join. `prstDash` must never be an attribute.
fn border_xml(xml: &mut Xml, theme: Option<&Theme>, border: &Border) -> Result<()> {
    // Emit `w` whenever the border carries a fill, so a solid/gradient
    // outline keeps its source thickness (OOXML defaults `<a:ln w>` to
    // 0.75pt when the attribute is missing; PowerPoint/Impress respect
    // it). Only an empty `<a:ln>` (no fill child at all) may drop `w`,
    // matching the P2 gradient-frame case where the default value flipped
    // the segment gradient bounding box.
    let width_attr: Option<String> = border
        .width
        .filter(|_| border.color.is_some() || border.gradient.is_some())
        .map(|w| emu(w).to_string());
    let dash: Option<&str> = border.style.map(|style| match style {
        LineStyle::Solid => "solid",
        LineStyle::Dash => "dash",
        LineStyle::Dot => "dot",
        LineStyle::SysDot => "sysDot",
    });
    if let Some(width) = &width_attr {
        xml.start("a:ln", &[("w", width)]);
    } else {
        xml.start("a:ln", &[]);
    }
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
        let resolved = resolve_color(theme, color)?;
        emit_fill_color(xml, &resolved);
    }
    // `color` + `gradient` both None → empty `<a:ln>` with no fill child,
    // matching a source line that colourised via `p:style lnRef` (and
    // invisible in LibreOffice). Never default to a black ring here.
    if let Some(dash) = dash {
        xml.leaf("a:prstDash", &[("val", dash)]);
    }
    xml.end("a:ln");
    Ok(())
}

/// `a:effectLst > a:outerShdw` from a [`Shadow`]. Emitted inside `p:spPr`
/// after the border. `blur` → `blurRad` (EMU); `offset` → `dist`+`dir`
/// (dir in 1/60000 deg). The colour's alpha rides on the `<a:srgbClr>`.
pub(crate) fn shadow_xml(xml: &mut Xml, theme: Option<&Theme>, shadow: &Shadow) -> Result<()> {
    let resolved = resolve_color(theme, &shadow.color)?;
    xml.start("a:effectLst", &[]);
    let blur = emu(shadow.blur).to_string();
    let (dist, dir) = match shadow.offset {
        Some((x, y)) => {
            let d = emu((x * x + y * y).sqrt());
            let deg = if x == 0.0 && y == 0.0 {
                0.0
            } else {
                y.atan2(x).to_degrees().rem_euclid(360.0)
            };
            (d, (deg * 60000.0).round() as i64)
        }
        None => (0, 0),
    };
    let dist_s = dist.to_string();
    let dir_s = dir.to_string();
    let mut attrs: Vec<(&str, &str)> = vec![
        ("blurRad", blur.as_str()),
        ("dist", dist_s.as_str()),
        ("dir", dir_s.as_str()),
        ("rotWithShape", "0"),
    ];
    let scale_s;
    if let Some(s) = shadow.scale {
        scale_s = ((s * 100000.0).round() as i64).to_string();
        attrs.push(("sx", scale_s.as_str()));
        attrs.push(("sy", scale_s.as_str()));
        attrs.push(("algn", "ctr"));
    }
    let tag = if shadow.inner.unwrap_or(false) {
        "a:innerShdw"
    } else {
        "a:outerShdw"
    };
    xml.start(tag, &attrs);
    srgb(xml, &resolved.rgb, resolved.alpha);
    xml.end(tag);
    xml.end("a:effectLst");
    Ok(())
}
pub fn fill_xml(
    xml: &mut Xml,
    theme: Option<&Theme>,
    fill: &Fill,
    opacity: Option<f64>,
) -> Result<()> {
    match fill {
        Fill::Solid { color } => {
            let mut resolved = resolve_color(theme, color)?;
            if let Some(op) = opacity {
                let base = resolved.alpha.unwrap_or(1.0);
                resolved.alpha = Some(base * op);
            }
            emit_fill_color(xml, &resolved);
        }
        Fill::Gradient {
            gradient_type,
            stops,
            angle,
            scaled,
        } => {
            xml.start("a:gradFill", &[("rotWithShape", "1")]);
            xml.start("a:gsLst", &[]);
            for stop in stops {
                let mut resolved = resolve_color(theme, &stop.color)?;
                if let Some(op) = opacity {
                    let base = resolved.alpha.unwrap_or(1.0);
                    resolved.alpha = Some(base * op);
                }
                let pos = ((stop.position * 100000.0).round() as u64).to_string();
                xml.start("a:gs", &[("pos", &pos)]);
                srgb(xml, &resolved.rgb, resolved.alpha);
                xml.end("a:gs");
            }
            xml.end("a:gsLst");
            match gradient_type {
                GradientType::Linear => {
                    let angle = (angle.unwrap_or(0.0) * 60000.0).round().to_string();
                    xml.start(
                        "a:lin",
                        &[("ang", &angle), ("scaled", if *scaled { "1" } else { "0" })],
                    );
                    xml.end("a:lin");
                }
                GradientType::Radial => {
                    xml.start("a:path", &[("path", "circle")]);
                    xml.leaf(
                        "a:fillToRect",
                        &[
                            ("l", "50000"),
                            ("t", "50000"),
                            ("r", "50000"),
                            ("b", "50000"),
                        ],
                    );
                    xml.end("a:path");
                }
            }
            xml.end("a:gradFill");
        }
        Fill::Image { src, .. } => {
            return Err(Error::Unsupported(format!(
                "image fill `{src}` is not supported yet (only solid and gradient fills)"
            )));
        }
    }
    Ok(())
}

/// `<p:bg><p:bgPr>` fill: solid/gradient via [`fill_xml`], image via
/// `<a:blipFill>` (resolves `r:embed` through the render context's media).
/// Used for slide and layout backgrounds.
pub fn bg_fill_xml(
    xml: &mut Xml,
    ctx: &RenderCtx<'_>,
    theme: Option<&Theme>,
    fill: &Fill,
) -> Result<()> {
    match fill {
        Fill::Image { src, .. } => {
            let rid = ctx.media_rid(src).ok_or_else(|| {
                Error::Unsupported(format!(
                    "background image `{src}` was not registered before rendering"
                ))
            })?;
            xml.start("a:blipFill", &[]);
            xml.start("a:blip", &[("r:embed", rid.as_str())]);
            xml.end("a:blip");
            xml.leaf("a:stretch", &[]);
            xml.end("a:blipFill");
            Ok(())
        }
        other => fill_xml(xml, theme, other, None),
    }
}

/// Emit one `<p:sp>` placeholder definition for a `slideLayout`, from a
/// [`PlaceholderDef`]. Carries `<p:ph type>` + `xfrm` + an `<a:lstStyle>`
/// rebuilt from the placeholder's resolved run-style, so slide placeholders
/// that omit geometry/run-style inherit it (the full OOXML layout→slide
/// placeholder chain — `pptd-layout-extension.md` §8). Mirrors the
/// slide-side empty placeholder shape; an empty `<a:p/>` lets the consumer
/// render the "Click to edit …" prompt.
pub fn render_layout_placeholder(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    theme: Option<&Theme>,
    type_name: &str,
    def: &PlaceholderDef,
) -> Result<()> {
    let id = ctx.next_id().to_string();
    xml.start("p:sp", &[]);
    xml.start("p:nvSpPr", &[]);
    xml.start("p:cNvPr", &[("id", &id), ("name", type_name)]);
    xml.end("p:cNvPr");
    xml.leaf("p:cNvSpPr", &[]);
    xml.start("p:nvPr", &[]);
    xml.leaf("p:ph", &[("type", type_name), ("idx", "1")]);
    xml.end("p:nvPr");
    xml.end("p:nvSpPr");

    xml.start("p:spPr", &[]);
    xfrm(xml, def.bounds, None, None);
    xml.start("a:prstGeom", &[("prst", "rect")]);
    xml.leaf("a:avLst", &[]);
    xml.end("a:prstGeom");
    xml.leaf("a:noFill", &[]);
    xml.end("p:spPr");

    // Resolve the placeholder's run-style: the `style` ref (`$key` →
    // theme.textStyles) as the base, overridden by the explicit fields.
    let base = def
        .style
        .as_deref()
        .and_then(|s| theme.and_then(|t| t.text_styles.get(s.trim_start_matches('$'))));
    // Resolve any `$key` theme color reference to a hex literal so the
    // emitted `<a:srgbClr>` is valid OOXML (a raw `$primary` would be
    // rejected). Falls back to the raw value if the theme can't resolve it.
    let raw_color = def
        .color
        .clone()
        .or_else(|| base.and_then(|b| b.color.clone()));
    let color = raw_color
        .as_ref()
        .and_then(|c| resolve_color(theme, c).ok().map(|rc| Color(rc.rgb)))
        .or(raw_color);
    let style = EffTextStyle {
        color,
        font_size: def.font_size.or_else(|| base.and_then(|b| b.font_size)),
        font_family: def
            .font_family
            .clone()
            .or_else(|| base.and_then(|b| b.font_family.clone())),
        bold: def.bold.or_else(|| base.and_then(|b| b.bold)),
        italic: def.italic.or_else(|| base.and_then(|b| b.italic)),
        line_height: base.and_then(|b| b.line_height),
        line_height_px: None,
        letter_spacing: None,
        align: def.align,
        wrap: None,
        direction: None,
        margin_top: None,
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
    };

    xml.start("p:txBody", &[]);
    xml.leaf("a:bodyPr", &[]);
    emit_lst_style(xml, &style, true, ctx.safe_fonts);
    xml.leaf("a:p", &[]);
    xml.end("p:txBody");
    xml.end("p:sp");
    Ok(())
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

/// Effective text style after the inheritance chain
/// (content fields > `content.style` → theme.textStyles > defaults).
#[derive(Default)]
pub struct EffTextStyle {
    pub color: Option<Color>,
    pub font_size: Option<f64>,
    pub font_family: Option<FontFamily>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub line_height: Option<f64>,
    pub line_height_px: Option<f64>,
    pub letter_spacing: Option<f64>,
    pub align: Option<Alignment>,
    pub wrap: Option<bool>,
    pub direction: Option<TextDirection>,
    pub margin_top: Option<f64>,
    pub margin_left: Option<f64>,
    pub margin_right: Option<f64>,
    pub margin_bottom: Option<f64>,
    pub autofit: Option<TextAutofit>,
    /// Raw `a:bodyPr` attributes preserved verbatim from the source.
    pub body_pr_extras: Option<std::collections::BTreeMap<String, String>>,
    /// List bullet glyph (e.g. `•`); `None` → no bullet.
    pub bullet_char: Option<String>,
    /// Bullet typeface (e.g. `Arial`).
    pub bullet_font: Option<String>,
    /// Paragraph left margin (`marL`) in px.
    pub list_margin: Option<f64>,
    /// Hanging indent in px (negative).
    pub list_indent: Option<f64>,
    /// Gradient applied to the text fill (overrides `color`).
    pub gradient: Option<crate::pptd::shared::GradientFill>,
    /// Drop shadow applied to the text itself.
    pub shadow: Option<crate::pptd::shared::Shadow>,
}

pub fn effective_text_style(theme: Option<&Theme>, content: &TextContent) -> EffTextStyle {
    let base = content
        .style
        .as_deref()
        .and_then(|style| theme.and_then(|t| t.text_styles.get(style.trim_start_matches('$'))));
    EffTextStyle {
        color: content
            .color
            .clone()
            .or_else(|| base.and_then(|b| b.color.clone())),
        font_size: content.font_size.or_else(|| base.and_then(|b| b.font_size)),
        font_family: content
            .font_family
            .clone()
            .or_else(|| base.and_then(|b| b.font_family.clone())),
        bold: content.bold.or_else(|| base.and_then(|b| b.bold)),
        italic: content.italic.or_else(|| base.and_then(|b| b.italic)),
        line_height: content
            .line_height
            .or_else(|| base.and_then(|b| b.line_height)),
        line_height_px: content
            .line_height_px
            .or_else(|| base.and_then(|b| b.line_height_px)),
        letter_spacing: content
            .letter_spacing
            .or_else(|| base.and_then(|b| b.letter_spacing)),
        align: content.align,
        wrap: content.wrap,
        direction: content.text_direction,
        margin_top: content
            .margin_top
            .or_else(|| base.and_then(|b| b.margin_top)),
        margin_left: content.margin_left,
        margin_right: content.margin_right,
        margin_bottom: content.margin_bottom,
        autofit: content.autofit,
        body_pr_extras: content.body_pr_extras.clone(),
        bullet_char: content.bullet_char.clone(),
        bullet_font: content.bullet_font.clone(),
        list_margin: content.list_margin,
        list_indent: content.list_indent,
        gradient: content.gradient.clone(),
        shadow: content.shadow.clone(),
    }
}

/// Open `<a:pPr>` with `algn` plus the box-level `marL`/`indent` (the list
/// bullet's hanging indent) when the text box carries bullets.
fn ppr_open(xml: &mut Xml, algn: &str, style: &EffTextStyle) {
    let mar = style.list_margin.map(|v| emu(v).to_string());
    let ind = style.list_indent.map(|v| emu(v).to_string());
    let mut attrs: Vec<(&str, &str)> = vec![("algn", algn)];
    if let Some(m) = &mar {
        attrs.push(("marL", m));
    }
    if let Some(i) = &ind {
        attrs.push(("indent", i));
    }
    xml.start("a:pPr", &attrs);
}

/// Emit the bullet glyph (`<a:buFont>` + `<a:buChar>`) when the box is a
/// bulleted list. Schema order: after `a:lnSpc`/`a:spcBef`, before
/// `a:defRPr`.
fn emit_bullet(xml: &mut Xml, style: &EffTextStyle) {
    if style.bullet_char.is_none() {
        return;
    }
    if let Some(tf) = &style.bullet_font {
        xml.leaf("a:buFont", &[("typeface", tf)]);
    }
    if let Some(ch) = &style.bullet_char {
        xml.leaf("a:buChar", &[("char", ch)]);
    }
}

/// Emit `<a:lstStyle>` for a placeholder. Plain text boxes get an empty
/// `<a:lstStyle/>`; placeholders get a `<a:lvl1pPr>` rebuilt from the
/// effective style (algn/lnSpc/spcBef/defRPr) so slide placeholders inherit
/// the layout/master text style (font/size/colour/bold/align/spacing) —
/// e.g. an empty slide title shows the layout's prompt in blue/bold 24pt.
fn emit_lst_style(xml: &mut Xml, style: &EffTextStyle, is_placeholder: bool, safe_fonts: bool) {
    if !is_placeholder {
        xml.leaf("a:lstStyle", &[]);
        return;
    }
    let algn = style.align.map(|a| match a.horizontal {
        HorizontalAlign::Left => "l",
        HorizontalAlign::Center => "ctr",
        HorizontalAlign::Right => "r",
        HorizontalAlign::Justify => "just",
        HorizontalAlign::Distributed => "dist",
    });
    let sz = style
        .font_size
        .map(|s| ((s * 100.0).round() as u64).to_string());
    let mut rpr_attrs: Vec<(&str, &str)> = Vec::new();
    if let Some(ref s) = sz {
        rpr_attrs.push(("sz", s));
    }
    if style.bold == Some(true) {
        rpr_attrs.push(("b", "1"));
    }
    if style.italic == Some(true) {
        rpr_attrs.push(("i", "1"));
    }
    let has_rpr = !rpr_attrs.is_empty() || style.color.is_some() || style.font_family.is_some();
    let has_anything = algn.is_some() || style.line_height.is_some() || has_rpr;
    if !has_anything {
        // No style info → empty lstStyle so the placeholder inherits the
        // layout/master (don't emit an empty `<a:lvl1pPr>` that overrides).
        xml.leaf("a:lstStyle", &[]);
        return;
    }
    let mut lvl_attrs: Vec<(&str, &str)> = Vec::new();
    if let Some(a) = algn {
        lvl_attrs.push(("algn", a));
    }
    xml.start("a:lstStyle", &[]);
    xml.start("a:lvl1pPr", &lvl_attrs);
    if let Some(lh) = style.line_height {
        let pct = ((lh * 100000.0).round() as u64).to_string();
        xml.start("a:lnSpc", &[]);
        xml.leaf("a:spcPct", &[("val", &pct)]);
        xml.end("a:lnSpc");
    }
    if has_rpr {
        xml.start("a:defRPr", &rpr_attrs);
        if let Some(c) = &style.color {
            xml.start("a:solidFill", &[]);
            xml.leaf("a:srgbClr", &[("val", c.0.trim_start_matches('#'))]);
            xml.end("a:solidFill");
        }
        if let Some(f) = &style.font_family {
            match f {
                FontFamily::Single(name) => {
                    let lat = super::theme::latin_typeface(name, safe_fonts);
                    let ea = super::theme::ea_typeface(name, safe_fonts);
                    xml.leaf("a:latin", &[("typeface", lat)]);
                    xml.leaf("a:ea", &[("typeface", ea)]);
                }
                FontFamily::Bilingual { latin, ea } => {
                    let lat = super::theme::latin_typeface(latin, safe_fonts);
                    let ea = super::theme::ea_typeface(ea, safe_fonts);
                    xml.leaf("a:latin", &[("typeface", lat)]);
                    xml.leaf("a:ea", &[("typeface", ea)]);
                }
            }
        }
        xml.end("a:defRPr");
    }
    xml.end("a:lvl1pPr");
    xml.end("a:lstStyle");
}

fn render_text(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    text: &Text,
    _page_index: usize,
) -> Result<()> {
    let id = ctx.next_id().to_string();
    let name = text.common.element_id.clone();
    let style = effective_text_style(ctx.theme, &text.content);

    let is_empty_placeholder = text.placeholder.is_some() && text.content.text.is_empty();

    xml.start("p:sp", &[]);
    xml.start("p:nvSpPr", &[]);
    cnv_pr(xml, ctx, &id, &name, text.common.link.as_deref());
    if let Some(ph_type) = text.placeholder.as_deref() {
        // Re-emit the `<p:ph type>` so the rebuilt shape keeps inheriting
        // the master/layout placeholder styles (default fonts, line
        // spacing, colour) instead of becoming a plain `txBox` text box.
        // Non-empty placeholders must stay placeholders — writing them as
        // plain shapes drops the OOXML inheritance chain, which changes
        // paragraph metrics (e.g. subtitle line pitch) in every consumer.
        xml.leaf("p:cNvSpPr", &[]);
        xml.start("p:nvPr", &[]);
        xml.leaf("p:ph", &[("type", ph_type)]);
        xml.end("p:nvPr");
    } else {
        xml.leaf("p:cNvSpPr", &[("txBox", "1")]);
        xml.leaf("p:nvPr", &[]);
    }
    xml.end("p:nvSpPr");

    xml.start("p:spPr", &[]);
    xfrm(
        xml,
        text.common.bounds,
        text.common.rotation,
        text.common.flip,
    );
    xml.start("a:prstGeom", &[("prst", "rect")]);
    xml.leaf("a:avLst", &[]);
    xml.end("a:prstGeom");
    // A text box may carry its own background fill + outline (a coloured
    // card behind the text). Without an explicit fill, emit noFill so the
    // box does not inherit the automatic shape fill (kimi parity).
    match &text.fill {
        Some(fill) => fill_xml(xml, ctx.theme, fill, text.common.opacity)?,
        None => xml.leaf("a:noFill", &[]),
    }
    if let Some(border) = &text.border {
        border_xml(xml, ctx.theme, border)?;
    }
    xml.end("p:spPr");

    let anchor = match style
        .align
        .map(|a| a.vertical)
        .unwrap_or(VerticalAlign::Top)
    {
        VerticalAlign::Top => "t",
        VerticalAlign::Middle => "ctr",
        VerticalAlign::Bottom => "b",
    };

    // Empty typed placeholder: emit a minimal empty txBody so the renderer
    // shows the placeholder prompt ("Click to edit title"). The bodyPr
    // stays empty (`<a:bodyPr/>`) to match the source — an explicit anchor
    // on an empty placeholder makes WPS inherit the layout's bodyPr anchor
    // for the prompt, shifting it; an empty bodyPr uses the renderer's
    // default (top-anchored prompt), matching the source.
    if is_empty_placeholder {
        xml.start("p:txBody", &[]);
        xml.leaf("a:bodyPr", &[]);
        emit_lst_style(xml, &style, true, ctx.safe_fonts);
        xml.leaf("a:p", &[]);
        xml.end("p:txBody");
        xml.end("p:sp");
        return Ok(());
    }

    // Rich text (`<p>`/`<span>`/`<strong>` …) or math formulas: parse paragraph + run styles.
    if text.content.text.contains('<') || text.content.text.contains("\\(") {
        xml.start("p:txBody", &[]);
        render_rich_body(
            xml,
            ctx,
            &style,
            &text.content.text,
            anchor,
            text.placeholder.is_some(),
        )?;
        xml.end("p:txBody");
        xml.end("p:sp");
        return Ok(());
    }

    xml.start("p:txBody", &[]);
    let wrap = if style.wrap == Some(false) {
        "none"
    } else {
        "square"
    };
    let ins = |v: Option<f64>| -> String { emu(v.unwrap_or(0.0)).to_string() };
    let mut bpr_v: Vec<String> = vec![
        ins(style.margin_left),
        ins(style.margin_right),
        ins(style.margin_top),
        ins(style.margin_bottom),
        wrap.to_string(),
        "0".to_string(),
        anchor.to_string(),
    ];
    let mut bpr_k: Vec<&str> = vec!["lIns", "rIns", "tIns", "bIns", "wrap", "rtlCol", "anchor"];
    // textDirection: vertical emits vert on a:bodyPr (prefer eaVert per requirement 6)
    if style.direction == Some(TextDirection::Vertical) {
        bpr_k.push("vert");
        bpr_v.push("eaVert".to_string());
    }
    if let Some(extras) = &style.body_pr_extras {
        for (k, v) in extras {
            bpr_k.push(k.as_str());
            bpr_v.push(v.clone());
        }
    }
    let bpr_attrs: Vec<(&str, &str)> = bpr_k
        .into_iter()
        .zip(bpr_v.iter().map(String::as_str))
        .collect();
    xml.start("a:bodyPr", &bpr_attrs);
    match style.autofit {
        Some(TextAutofit::FitShape) => xml.leaf("a:spAutoFit", &[]),
        Some(TextAutofit::FitText) => xml.leaf("a:normAutofit", &[]),
        Some(TextAutofit::Fixed) => xml.leaf("a:noAutofit", &[]),
        // No autofit declared: OOXML means "do not autofit" (text overflows
        // rather than shrinking). Emitting `normAutofit` here silently changed
        // the semantics of every converted deck — PowerPoint would shrink text
        // on overflow where the source kept its declared size.
        None => {}
    }
    xml.end("a:bodyPr");
    emit_lst_style(xml, &style, text.placeholder.is_some(), ctx.safe_fonts);

    // Plain text: every line becomes one paragraph (the `<p>` equivalence).
    let sz = (style.font_size.unwrap_or(18.0) * 100.0)
        .round()
        .to_string();
    for line in text.content.text.split('\n') {
        xml.start("a:p", &[]);

        let align = style
            .align
            .map(|a| a.horizontal)
            .unwrap_or(HorizontalAlign::Left);
        let algn = match align {
            HorizontalAlign::Left => "l",
            HorizontalAlign::Center => "ctr",
            HorizontalAlign::Right => "r",
            HorizontalAlign::Justify => "just",
            HorizontalAlign::Distributed => "dist",
        };
        let line_height_pct: Option<String> = style
            .line_height
            .map(|multiple| ((multiple * 100000.0).round() as u64).to_string());
        let line_height_pts: Option<String> = style
            .line_height_px
            .map(|px| ((px * 100.0).round() as u64).to_string());

        // Kimi emits paragraph properties for every paragraph: explicit
        // alignment. Line spacing is only emitted when the PPTD carries an
        // explicit value; otherwise omit `<a:lnSpc>` so the renderer uses
        // its natural default (QuickLook/WPS ≈ 1.2× font), matching a source
        // box that had no `<a:lnSpc>`.
        ppr_open(xml, algn, &style);
        if let Some(pct) = line_height_pct {
            xml.start("a:lnSpc", &[]);
            xml.leaf("a:spcPct", &[("val", &pct)]);
            xml.end("a:lnSpc");
        } else if let Some(pts) = line_height_pts {
            xml.start("a:lnSpc", &[]);
            xml.leaf("a:spcPts", &[("val", &pts)]);
            xml.end("a:lnSpc");
        }
        emit_bullet(xml, &style);
        xml.end("a:pPr");

        emit_run(xml, ctx, &style, line)?;
        // Kimi pins the paragraph end-mark size to the run size; without it
        // the mark takes the theme default (44pt here) and shifts the
        // vertically-centered text off position.
        xml.leaf(
            "a:endParaRPr",
            &[("lang", "en-US"), ("sz", &sz), ("noProof", "1")],
        );
        xml.end("a:p");
    }

    xml.end("p:txBody");
    xml.end("p:sp");
    Ok(())
}

/// CSS declaration list → key/value pairs (`font-size:24px; color:#f00`).
fn css_pairs(style: &str) -> Vec<(String, String)> {
    style
        .split(';')
        .filter_map(|kv| {
            let mut it = kv.split(':');
            let key = it.next()?.trim();
            let value = it.next()?.trim();
            if key.is_empty() || value.is_empty() {
                None
            } else {
                Some((key.to_ascii_lowercase(), value.to_string()))
            }
        })
        .collect()
}

fn css_value<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// `24px` → 24.0; bare numbers stay numbers.
fn css_px(value: &str) -> Option<f64> {
    value
        .strip_suffix("px")
        .unwrap_or(value)
        .trim()
        .parse::<f64>()
        .ok()
}

fn css_align(value: &str) -> Option<&'static str> {
    match value {
        "left" => Some("l"),
        "center" => Some("ctr"),
        "right" => Some("r"),
        "justify" => Some("just"),
        "distributed" => Some("dist"),
        _ => None,
    }
}

/// Decode the five XML entities used by rich text.
fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Parse PPTD rich text into paragraphs with per-paragraph and per-run
/// style. Plain text (no tags) yields one paragraph per line.
fn parse_rich(text: &str) -> Vec<RichPara> {
    fn plain_para(line: &str) -> RichPara {
        RichPara {
            align: None,
            line_height: None,
            line_height_px: None,
            margin_top_px: None,
            items: parse_runs(line),
        }
    }

    let mut paras: Vec<RichPara> = Vec::new();
    let mut pos = 0usize;
    loop {
        let remaining = &text[pos..];
        let Some(p_start) = remaining.find("<p") else {
            // Remaining plain text: line-based paragraphs.
            for line in remaining.split('\n') {
                if !line.trim().is_empty() {
                    paras.push(plain_para(line));
                }
            }
            break;
        };
        let head = &text[pos..pos + p_start];
        for line in head.split('\n') {
            if !line.trim().is_empty() {
                paras.push(plain_para(line));
            }
        }
        let tag_start = pos + p_start + 2;
        let gt_rel = remaining[p_start + 2..]
            .find('>')
            .unwrap_or(remaining[p_start + 2..].len());
        let gt = tag_start + gt_rel;
        // `<p style="...">` — tolerate the space after the tag name.
        let style_attr = text[tag_start..gt].trim();
        let style = style_attr
            .strip_prefix("style=\"")
            .and_then(|s| s.strip_suffix('\"'))
            .unwrap_or("");
        let pairs = css_pairs(style);
        let align = css_value(&pairs, "text-align").and_then(css_align);
        let line_height = css_value(&pairs, "line-height").and_then(|v| {
            if v.contains("px") {
                None
            } else {
                v.parse::<f64>().ok()
            }
        });
        let line_height_px = css_value(&pairs, "line-height")
            .and_then(css_px)
            .filter(|_| css_value(&pairs, "line-height").is_some_and(|v| v.contains("px")));
        let margin_top_px = css_value(&pairs, "margin-top").and_then(css_px);
        let close_rel = text[gt + 1..].find("</p>").unwrap_or(text[gt + 1..].len());
        let close = gt + 1 + close_rel;
        paras.push(RichPara {
            align,
            line_height,
            line_height_px,
            margin_top_px,
            items: parse_runs(&text[gt + 1..close]),
        });
        let after = close + 4;
        if after > text.len() {
            break;
        }
        pos = after;
        if pos >= text.len() {
            break;
        }
    }
    paras
}

/// Split one `<p>` body into styled runs (`<span style>` / `<strong>` /
/// `<em>`); plain text chunks become unstyled runs.
fn parse_runs(inner: &str) -> Vec<ParaItem> {
    #[derive(Default, Clone)]
    struct Flags {
        size: Option<f64>,
        color: Option<String>,
        family: Option<String>,
        letter_spacing: Option<f64>,
        bold: Option<bool>,
        italic: Option<bool>,
        underline: Option<bool>,
        strike: Option<bool>,
        styled: bool,
        /// External hyperlink from `<a href="…">`.
        link: Option<String>,
    }

    fn flush_text(out: &mut Vec<ParaItem>, buf: &mut String, f: &Flags) {
        if buf.is_empty() {
            return;
        }
        let raw = unescape_xml(&std::mem::take(buf));
        let mut rest = raw.as_str();
        while let Some(start) = rest.find("\\(") {
            if start > 0 {
                out.push(ParaItem::Run(RichRun {
                    text: rest[..start].to_owned(),
                    size: f.size,
                    color: f.color.clone(),
                    family: f.family.clone(),
                    letter_spacing: f.letter_spacing,
                    bold: f.bold,
                    italic: f.italic,
                    underline: f.underline,
                    strike: f.strike,
                    styled: f.styled,
                    baseline: None,
                    link: f.link.clone(),
                }));
            }
            let after = &rest[start + 2..];
            if let Some(end) = after.find("\\)") {
                out.push(ParaItem::Math(after[..end].to_owned()));
                rest = &after[end + 2..];
            } else {
                out.push(ParaItem::Run(RichRun {
                    text: rest[start..].to_owned(),
                    size: f.size,
                    color: f.color.clone(),
                    family: f.family.clone(),
                    letter_spacing: f.letter_spacing,
                    bold: f.bold,
                    italic: f.italic,
                    underline: f.underline,
                    strike: f.strike,
                    styled: f.styled,
                    baseline: None,
                    link: f.link.clone(),
                }));
                rest = "";
                break;
            }
        }
        if !rest.is_empty() {
            out.push(ParaItem::Run(RichRun {
                text: rest.to_owned(),
                size: f.size,
                color: f.color.clone(),
                family: f.family.clone(),
                letter_spacing: f.letter_spacing,
                bold: f.bold,
                italic: f.italic,
                underline: f.underline,
                strike: f.strike,
                styled: f.styled,
                baseline: None,
                link: f.link.clone(),
            }));
        }
    }

    /// Split `<name attr="v" ...>` into the tag name and the raw attribute text.
    fn split_tag(raw: &str) -> (&str, &str) {
        match raw.find(char::is_whitespace) {
            Some(i) => (&raw[..i], raw[i..].trim()),
            None => (raw, ""),
        }
    }

    /// Value of a `style="..."` attribute inside a tag's attribute text.
    fn style_attr(attrs: &str) -> Option<&str> {
        let start = attrs.find("style=\"")? + "style=\"".len();
        let tail = &attrs[start..];
        let end = tail.find('"')?;
        Some(&tail[..end])
    }

    /// Value of an arbitrary `name="..."` attribute inside a tag.
    fn tag_attr<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
        let needle = format!("{name}=\"");
        let start = attrs.find(&needle)? + needle.len();
        let tail = &attrs[start..];
        let end = tail.find('"')?;
        Some(&tail[..end])
    }

    /// Apply the CSS declarations shared by `<span style>` and the inline tags.
    fn apply_css(flags: &mut Flags, style: &str) {
        let pairs = css_pairs(style);
        if let Some(v) = css_value(&pairs, "font-size").and_then(css_px) {
            flags.size = Some(v);
        }
        if let Some(v) = css_value(&pairs, "color") {
            flags.color = Some(v.to_string());
        }
        if let Some(v) = css_value(&pairs, "font-family") {
            flags.family = Some(v.to_string());
        }
        if let Some(v) = css_value(&pairs, "letter-spacing").and_then(css_px) {
            flags.letter_spacing = Some(v);
        }
        if let Some(w) = css_value(&pairs, "font-weight") {
            flags.bold = Some(matches!(
                w,
                "bold" | "bolder" | "600" | "700" | "800" | "900"
            ));
        }
        if let Some(v) = css_value(&pairs, "font-style") {
            flags.italic = Some(v != "normal");
        }
        if let Some(dec) = css_value(&pairs, "text-decoration") {
            if dec.contains("underline") {
                flags.underline = Some(true);
            }
            if dec.contains("line-through") {
                flags.strike = Some(true);
            }
        }
        flags.styled = flags.size.is_some()
            || flags.color.is_some()
            || flags.family.is_some()
            || flags.letter_spacing.is_some()
            || flags.bold.is_some()
            || flags.italic.is_some()
            || flags.underline.is_some()
            || flags.strike.is_some();
    }

    fn is_inline_tag(name: &str) -> bool {
        matches!(
            name,
            "span" | "strong" | "b" | "em" | "i" | "u" | "s" | "strike" | "del" | "code" | "a"
        )
    }

    let mut out: Vec<ParaItem> = Vec::new();
    let mut buf = String::new();
    let mut flags = Flags::default();
    // Open inline tags keep the flags they inherited, so a closing tag restores
    // the surrounding style instead of resetting everything.
    let mut stack: Vec<Flags> = Vec::new();
    let mut rest = inner;
    loop {
        let Some(lt) = rest.find('<') else {
            buf.push_str(rest);
            break;
        };
        buf.push_str(&rest[..lt]);
        let tag = &rest[lt..];
        if let Some(end) = tag.find('>') {
            let raw = tag[1..end].trim();
            let (name, attrs) = split_tag(raw);
            let closing = name.starts_with('/');
            let bare = name.trim_start_matches('/');

            if bare == "latex" && !closing {
                flush_text(&mut out, &mut buf, &flags);
                let latex_start = lt + end + 1;
                if let Some(latex_end_rel) = rest[latex_start..].find("</latex>") {
                    let latex_content = &rest[latex_start..latex_start + latex_end_rel];
                    out.push(ParaItem::Math(unescape_xml(latex_content)));
                    rest = &rest[latex_start + latex_end_rel + 8..];
                    continue;
                } else {
                    out.push(ParaItem::Math(unescape_xml(&rest[latex_start..])));
                    break;
                }
            } else if bare == "br/" || bare == "br" {
                buf.push('\n');
            } else if is_inline_tag(bare) {
                flush_text(&mut out, &mut buf, &flags);
                if closing {
                    flags = stack.pop().unwrap_or_default();
                } else {
                    stack.push(flags.clone());
                    if let Some(style) = style_attr(attrs) {
                        apply_css(&mut flags, style);
                    }
                    match bare {
                        "strong" | "b" => {
                            flags.bold = Some(true);
                            flags.styled = true;
                        }
                        "em" | "i" => {
                            flags.italic = Some(true);
                            flags.styled = true;
                        }
                        "u" => {
                            flags.underline = Some(true);
                            flags.styled = true;
                        }
                        "s" | "strike" | "del" => {
                            flags.strike = Some(true);
                            flags.styled = true;
                        }
                        "code" => {
                            flags.family = Some("Consolas".to_string());
                            flags.styled = true;
                        }
                        "a" => {
                            if let Some(href) = tag_attr(attrs, "href") {
                                flags.link = Some(unescape_xml(href));
                                flags.styled = true;
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                // Unknown tag: keep it as literal text (rare).
                buf.push_str(&tag[..=end]);
            }
            rest = &rest[lt + end + 1..];
        } else {
            buf.push_str(&rest[lt..]);
            break;
        }
    }
    flush_text(&mut out, &mut buf, &flags);

    // Merge adjacent runs with identical style (span noise reduction).
    let mut merged: Vec<ParaItem> = Vec::new();
    for item in out {
        match item {
            ParaItem::Run(run) => {
                if let Some(ParaItem::Run(last)) = merged.last_mut() {
                    let same = last.size == run.size
                        && last.color == run.color
                        && last.family == run.family
                        && last.letter_spacing == run.letter_spacing
                        && last.bold == run.bold
                        && last.italic == run.italic
                        && last.underline == run.underline
                        && last.strike == run.strike
                        && last.link == run.link;
                    if same && !run.styled {
                        last.text.push_str(&run.text);
                        continue;
                    }
                }
                merged.push(ParaItem::Run(run));
            }
            ParaItem::Math(m) => {
                merged.push(ParaItem::Math(m));
            }
        }
    }
    merged
}
/// Apply the text box's resolved colour to a math fragment. Math runs are
/// emitted without a fill, so without this a formula ignores `content.color`
/// and disappears on dark backgrounds.
fn math_colored(theme: Option<&Theme>, style: &EffTextStyle, omml: &str) -> Result<String> {
    match &style.color {
        Some(color) => {
            let resolved = resolve_color(theme, color)?;
            Ok(super::math::apply_text_color(omml, &resolved.rgb))
        }
        None => Ok(omml.to_owned()),
    }
}

/// Render a rich-text body (inside an open `p:txBody`): per-paragraph `pPr`
/// + styled runs.
pub(crate) fn render_rich_body(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    style: &EffTextStyle,
    text: &str,
    anchor: &str,
    is_placeholder: bool,
) -> Result<()> {
    let theme = ctx.theme;
    let safe_fonts = ctx.safe_fonts;
    // Box-level defaults (shared with the plain-text path).
    let wrap = if style.wrap == Some(false) {
        "none"
    } else {
        "square"
    };
    let ins = |v: Option<f64>| -> String { emu(v.unwrap_or(0.0)).to_string() };
    let mut bpr_v: Vec<String> = vec![
        ins(style.margin_left),
        ins(style.margin_right),
        ins(style.margin_top),
        ins(style.margin_bottom),
        wrap.to_string(),
        "0".to_string(),
        anchor.to_string(),
    ];
    let mut bpr_k: Vec<&str> = vec!["lIns", "rIns", "tIns", "bIns", "wrap", "rtlCol", "anchor"];
    if style.direction == Some(TextDirection::Vertical) {
        bpr_k.push("vert");
        bpr_v.push("eaVert".to_string());
    }
    if let Some(extras) = &style.body_pr_extras {
        for (k, v) in extras {
            bpr_k.push(k.as_str());
            bpr_v.push(v.clone());
        }
    }
    let bpr_attrs: Vec<(&str, &str)> = bpr_k
        .into_iter()
        .zip(bpr_v.iter().map(String::as_str))
        .collect();
    xml.start("a:bodyPr", &bpr_attrs);
    match style.autofit {
        Some(TextAutofit::FitShape) => xml.leaf("a:spAutoFit", &[]),
        Some(TextAutofit::FitText) => xml.leaf("a:normAutofit", &[]),
        Some(TextAutofit::Fixed) => xml.leaf("a:noAutofit", &[]),
        // No autofit declared: OOXML means "do not autofit" (text overflows
        // rather than shrinking). Emitting `normAutofit` here silently changed
        // the semantics of every converted deck — PowerPoint would shrink text
        // on overflow where the source kept its declared size.
        None => {}
    }
    xml.end("a:bodyPr");
    emit_lst_style(xml, style, is_placeholder, safe_fonts);

    for para in parse_rich(text) {
        xml.start("a:p", &[]);
        let algn = para
            .align
            .or_else(|| {
                style.align.map(|a| a.horizontal).and_then(|h| {
                    css_align(match h {
                        HorizontalAlign::Left => "left",
                        HorizontalAlign::Center => "center",
                        HorizontalAlign::Right => "right",
                        HorizontalAlign::Justify => "justify",
                        HorizontalAlign::Distributed => "distributed",
                    })
                })
            })
            .unwrap_or("l");
        let line_height_pct = para
            .line_height
            .or(style.line_height)
            .map(|multiple| ((multiple * 100000.0).round() as u64).to_string());
        let line_height_pts = para
            .line_height_px
            .or(style.line_height_px)
            .map(|px| ((px * 100.0).round() as u64).to_string());

        ppr_open(xml, algn, style);
        // Empty (spacer) paragraphs without an explicit style carry no line
        // spacing — matching Office, whose empty `<a:p>` renders a much
        // shorter blank line than an explicit lnSpc at the box font would.
        // A spacer that DOES carry an explicit line-height keeps it, so
        // round-tripped template spacing (lnSpc 10%–60% rules between
        // sections) doesn't inflate to full line height and push content
        if !para.items.is_empty() || para.line_height.is_some() || para.line_height_px.is_some() {
            if let Some(pct) = line_height_pct {
                xml.start("a:lnSpc", &[]);
                xml.leaf("a:spcPct", &[("val", &pct)]);
                xml.end("a:lnSpc");
            } else if let Some(pts) = line_height_pts {
                xml.start("a:lnSpc", &[]);
                xml.leaf("a:spcPts", &[("val", &pts)]);
                xml.end("a:lnSpc");
            }
        }
        if let Some(mt) = para.margin_top_px {
            let pts = ((mt * 100.0).round() as u64).to_string();
            xml.start("a:spcBef", &[]);
            xml.leaf("a:spcPts", &[("val", &pts)]);
            xml.end("a:spcBef");
        }
        emit_bullet(xml, style);
        xml.end("a:pPr");

        let math_count = para
            .items
            .iter()
            .filter(|i| matches!(i, ParaItem::Math(_)))
            .count();
        let has_non_empty_runs = para.items.iter().any(|i| match i {
            ParaItem::Run(r) => !r.text.trim().is_empty(),
            _ => false,
        });
        let is_display = math_count == 1 && !has_non_empty_runs;

        if is_display {
            if let Some(ParaItem::Math(latex)) =
                para.items.iter().find(|i| matches!(i, ParaItem::Math(_)))
            {
                let omml = super::math::latex_to_omml(latex, true)
                    .map_err(|e| crate::error::Error::Invalid(e.to_string()))?;
                xml.raw(&math_colored(theme, style, &omml)?);
            }
        } else {
            for item in &para.items {
                match item {
                    ParaItem::Run(run) => {
                        emit_run_styled(xml, ctx, style, Some(run), &run.text)?;
                    }
                    ParaItem::Math(latex) => {
                        let omml = super::math::latex_to_omml(latex, false)
                            .map_err(|e| crate::error::Error::Invalid(e.to_string()))?;
                        let omml = math_colored(theme, style, &omml)?;
                        let fallback_segs = super::math::latex_to_fallback_segments(latex)
                            .map_err(|e| crate::error::Error::Invalid(e.to_string()))?;

                        xml.start(
                            "mc:AlternateContent",
                            &[(
                                "xmlns:mc",
                                "http://schemas.openxmlformats.org/markup-compatibility/2006",
                            )],
                        );
                        xml.start("mc:Choice", &[("Requires", "a14")]);
                        xml.raw(&omml);
                        xml.end("mc:Choice");
                        xml.start("mc:Fallback", &[]);
                        for seg in fallback_segs {
                            let r = RichRun {
                                text: seg.text.clone(),
                                styled: true,
                                family: None,
                                size: None,
                                color: None,
                                bold: None,
                                italic: None,
                                underline: None,
                                strike: None,
                                baseline: seg.baseline,
                                letter_spacing: None,
                                link: None,
                            };
                            emit_run_styled(xml, ctx, style, Some(&r), &r.text)?;
                        }
                        xml.end("mc:Fallback");
                        xml.end("mc:AlternateContent");
                    }
                }
            }
        }

        if !para.items.is_empty() {
            let sz = para
                .items
                .iter()
                .find_map(|i| match i {
                    ParaItem::Run(r) => r.size,
                    _ => None,
                })
                .or(style.font_size)
                .unwrap_or(18.0);
            let sz = ((sz * 100.0).round() as u64).to_string();
            xml.leaf(
                "a:endParaRPr",
                &[("lang", "en-US"), ("sz", &sz), ("noProof", "1")],
            );
        }
        xml.end("a:p");
    }
    Ok(())
}

fn emit_run(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    style: &EffTextStyle,
    text: &str,
) -> Result<()> {
    emit_run_styled(xml, ctx, style, None, text)
}

#[derive(Clone)]
enum ParaItem {
    Run(RichRun),
    Math(String),
}

/// Effective per-run override parsed from PPTD rich text.
#[derive(Clone)]
struct RichRun {
    text: String,
    size: Option<f64>,
    color: Option<String>,
    family: Option<String>,
    letter_spacing: Option<f64>,
    /// `None` inherits the box/paragraph style; `Some` is an explicit reset.
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strike: Option<bool>,
    styled: bool,
    /// OOXML `a:rPr@baseline` shift (1000ths of a percent) for math scripts.
    baseline: Option<i32>,
    /// External hyperlink target (`<a href="…">`).
    link: Option<String>,
}

/// One rich-text paragraph: per-paragraph overrides + items.
struct RichPara {
    align: Option<&'static str>,
    line_height: Option<f64>,
    line_height_px: Option<f64>,
    margin_top_px: Option<f64>,
    items: Vec<ParaItem>,
}

#[cfg(test)]
impl RichPara {
    fn runs(&self) -> Vec<&RichRun> {
        self.items
            .iter()
            .filter_map(|i| match i {
                ParaItem::Run(r) => Some(r),
                _ => None,
            })
            .collect()
    }
}
/// `emit_run` with a per-run rich-text override; unspecified run fields fall
/// back to the box-level `EffTextStyle`.
fn emit_run_styled(
    xml: &mut Xml,
    ctx: &mut RenderCtx<'_>,
    style: &EffTextStyle,
    run: Option<&RichRun>,
    text: &str,
) -> Result<()> {
    let theme = ctx.theme;
    let safe_fonts = ctx.safe_fonts;
    // Link-only runs must survive the "explicit style" filter, otherwise
    // `<a href="…">plain text</a>` would be emitted without its click target.
    let run = run.filter(|r| r.styled || r.link.is_some());
    // Math script runs (baseline set) without an explicit size render at 70%.
    let baseline = run.and_then(|r| r.baseline);
    let mut font_size = run.and_then(|r| r.size).or(style.font_size).unwrap_or(18.0);
    if baseline.is_some() && run.and_then(|r| r.size).is_none() {
        font_size *= 0.7;
    }
    let font_family = run
        .and_then(|r| r.family.as_deref())
        .map(|f| FontFamily::Single(f.to_owned()))
        .or_else(|| style.font_family.clone())
        .unwrap_or_else(|| {
            let (latin, ea) = super::theme::default_fonts(theme, safe_fonts);
            FontFamily::Bilingual { latin, ea }
        });
    let color = run
        .and_then(|r| r.color.as_deref())
        .map(|c| Color(c.to_owned()))
        .or_else(|| style.color.clone())
        .unwrap_or(Color("#000000".to_owned()));
    let bold = run.and_then(|r| r.bold).unwrap_or(style.bold == Some(true));
    let italic = run
        .and_then(|r| r.italic)
        .unwrap_or(style.italic == Some(true));
    let underline = run.and_then(|r| r.underline).unwrap_or(false);
    let strike = run.and_then(|r| r.strike).unwrap_or(false);

    let sz = (font_size * 100.0).round().to_string();
    let mut attrs: Vec<(&str, &str)> = vec![("lang", "en-US"), ("sz", &sz), ("noProof", "1")];
    if bold {
        attrs.push(("b", "1"));
    }
    if italic {
        attrs.push(("i", "1"));
    }
    if underline {
        attrs.push(("u", "sng"));
    }
    if strike {
        attrs.push(("strike", "sngStrike"));
    }
    let baseline_attr = baseline.map(|b| b.to_string());
    if let Some(bl) = &baseline_attr {
        attrs.push(("baseline", bl));
    }
    let letter_spacing = run.and_then(|r| r.letter_spacing).or(style.letter_spacing);
    let spc_attr = letter_spacing.map(|ls| (ls * 75.0).round().to_string());
    if let Some(spc) = &spc_attr {
        attrs.push(("spc", spc));
    }

    xml.start("a:r", &[]);
    xml.start("a:rPr", &attrs);
    let resolved = resolve_color(theme, &color).unwrap_or(ResolvedColor {
        rgb: "000000".to_owned(),
        alpha: None,
    });
    match &style.gradient {
        Some(gradient) => {
            let fill = Fill::Gradient {
                gradient_type: gradient.gradient_type,
                stops: gradient.stops.clone(),
                angle: gradient.angle,
                scaled: gradient.scaled,
            };
            fill_xml(xml, theme, &fill, None)?;
        }
        None => emit_fill_color(xml, &resolved),
    }
    if let Some(shadow) = &style.shadow {
        shadow_xml(xml, theme, shadow)?;
    }
    match &font_family {
        FontFamily::Single(name) => {
            let lat = super::theme::latin_typeface(name, safe_fonts);
            let ea = super::theme::ea_typeface(name, safe_fonts);
            xml.leaf("a:latin", &[("typeface", lat)]);
            xml.leaf("a:ea", &[("typeface", ea)]);
            xml.leaf("a:cs", &[("typeface", lat)]);
        }
        FontFamily::Bilingual { latin, ea } => {
            let lat = super::theme::latin_typeface(latin, safe_fonts);
            let ea = super::theme::ea_typeface(ea, safe_fonts);
            xml.leaf("a:latin", &[("typeface", lat)]);
            xml.leaf("a:ea", &[("typeface", ea)]);
            xml.leaf("a:cs", &[("typeface", lat)]);
        }
    }
    if let Some(url) = run.and_then(|r| r.link.as_deref()) {
        let rid = ctx.hyperlink_rid(url);
        xml.leaf("a:hlinkClick", &[("r:id", &rid)]);
    }
    xml.end("a:rPr");
    xml.text_elem("a:t", text);
    xml.end("a:r");
    Ok(())
}

// ---------------------------------------------------------------------------
// Shape
// ---------------------------------------------------------------------------

fn render_shape(xml: &mut Xml, ctx: &mut RenderCtx<'_>, shape: &Shape) -> Result<()> {
    let id = ctx.next_id().to_string();
    let name = shape.common.element_id.clone();

    xml.start("p:sp", &[]);
    xml.start("p:nvSpPr", &[]);
    cnv_pr(xml, ctx, &id, &name, shape.common.link.as_deref());
    xml.leaf("p:cNvSpPr", &[]);
    xml.leaf("p:nvPr", &[]);
    xml.end("p:nvSpPr");

    xml.start("p:spPr", &[]);
    xfrm(
        xml,
        shape.common.bounds,
        shape.common.rotation,
        shape.common.flip,
    );
    if shape.shape_name == "custom" {
        // Path geometry: viewBox (px) is the path coordinate space, the
        // element bounds then scale it to the final box (same contract as
        // icons and lines).
        let Some(view_box) = shape.view_box else {
            return Err(Error::Invalid(format!(
                "custom shape `{}` requires viewBox",
                shape.common.element_id
            )));
        };
        let Some(path) = shape.path.as_deref() else {
            return Err(Error::Invalid(format!(
                "custom shape `{}` requires path",
                shape.common.element_id
            )));
        };
        // The custGeom path space is abstract (source `a:path w/h`); the
        // element bounds scale it to the final box. Multiplying by 12700 here
        // would shrink every point to 1/12700 of the viewBox (a dot).
        let w = (view_box.0.round() as i64).to_string();
        let h = (view_box.1.round() as i64).to_string();
        xml.start("a:custGeom", &[]);
        emit_adjustments(xml, shape.adjustments.as_deref());
        xml.leaf("a:gdLst", &[]);
        xml.leaf("a:ahLst", &[]);
        xml.leaf("a:cxnLst", &[]);
        xml.leaf("a:rect", &[("l", "0"), ("t", "0"), ("r", &w), ("b", &h)]);
        xml.start("a:pathLst", &[]);
        xml.start("a:path", &[("w", &w), ("h", &h)]);
        svg_path::emit_path_children(xml, path)?;
        xml.end("a:path");
        xml.end("a:pathLst");
        xml.end("a:custGeom");
        // Adjustments refer to the ornament coordinates baked into the path
        // and are already represented by the path itself.
    } else {
        xml.start("a:prstGeom", &[("prst", &shape.shape_name)]);
        emit_adjustments(xml, shape.adjustments.as_deref());
        xml.end("a:prstGeom");
    }
    if let Some(fill) = &shape.fill {
        fill_xml(xml, ctx.theme, fill, shape.common.opacity)?;
    }
    if let Some(border) = &shape.border {
        border_xml(xml, ctx.theme, border)?;
    }
    if let Some(shadow) = &shape.shadow {
        shadow_xml(xml, ctx.theme, shadow)?;
    }
    xml.end("p:spPr");

    // Kimi renders decorative shapes without a `p:txBody` at all; an empty
    // text-box body would also carry a bodyPr with default insets.
    xml.end("p:sp");
    Ok(())
}

/// Preset-adjustment list `a:avLst` with up to 10 `val` guides.
fn emit_adjustments(xml: &mut Xml, adjustments: Option<&[f64]>) {
    xml.start("a:avLst", &[]);
    if let Some(adjustments) = adjustments {
        for (i, &adj) in adjustments.iter().take(10).enumerate() {
            let name = if i == 0 {
                "adj".to_owned()
            } else {
                format!("adj{}", i + 1)
            };
            let value = (adj.round() as i64).to_string();
            let fmla = format!("val {value}");
            xml.leaf("a:gd", &[("name", &name), ("fmla", &fmla)]);
        }
    }
    xml.end("a:avLst");
}

// ---------------------------------------------------------------------------
// Line
// ---------------------------------------------------------------------------

fn line_join(style: LineCurve) -> &'static str {
    match style {
        LineCurve::Sharp => "miter",
        LineCurve::Round => "round",
        LineCurve::Smooth => "round",
    }
}

/// Parse `"x,y x,y ..."` into points.
fn parse_points(points: &str) -> Result<Vec<(f64, f64)>> {
    points
        .split_whitespace()
        .map(|pair| {
            let (x, y) = pair.split_once(',').ok_or_else(|| {
                Error::Invalid(format!("line points `{points}`: `{pair}` is not `x,y`"))
            })?;
            let x = x
                .parse()
                .map_err(|_| Error::Invalid(format!("line points `{points}`: bad x `{x}`")))?;
            let y = y
                .parse()
                .map_err(|_| Error::Invalid(format!("line points `{points}`: bad y `{y}`")))?;
            Ok((x, y))
        })
        .collect()
}

fn line_stroke_xml(
    xml: &mut Xml,
    theme: Option<&Theme>,
    border: Option<&Border>,
    arrow: Option<(Option<ArrowType>, Option<ArrowType>)>,
    curve: Option<LineCurve>,
) -> Result<()> {
    let width_attr = border
        .and_then(|b| b.width)
        .map(|w| emu(w).to_string())
        .unwrap_or_else(|| emu(1.0).to_string());
    let dash: Option<&str> = border.and_then(|b| b.style).map(|style| match style {
        LineStyle::Solid => "solid",
        LineStyle::Dash => "dash",
        LineStyle::Dot => "dot",
        LineStyle::SysDot => "sysDot",
    });
    let join = curve.map(line_join).unwrap_or("round");

    xml.start(
        "a:ln",
        &[("w", &width_attr), ("cmpd", "sng"), ("cap", "rnd")],
    );

    if let Some(border) = border {
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
            let resolved = resolve_color(theme, color)?;
            emit_fill_color(xml, &resolved);
        } else {
            let def = ResolvedColor {
                rgb: "000000".to_owned(),
                alpha: None,
            };
            emit_fill_color(xml, &def);
        }
    } else {
        let def = ResolvedColor {
            rgb: "000000".to_owned(),
            alpha: None,
        };
        emit_fill_color(xml, &def);
    }

    if let Some(dash) = dash {
        xml.leaf("a:prstDash", &[("val", dash)]);
    }

    xml.leaf(&format!("a:{join}"), &[]);

    if let Some((head, tail)) = arrow {
        if let Some(h) = head {
            let t = match h {
                ArrowType::Arrow => "triangle",
                ArrowType::Stealth => "stealth",
                ArrowType::Diamond => "diamond",
                ArrowType::Oval => "oval",
            };
            xml.leaf("a:headEnd", &[("type", t), ("w", "med"), ("len", "med")]);
        }
        if let Some(t_arrow) = tail {
            let t = match t_arrow {
                ArrowType::Arrow => "triangle",
                ArrowType::Stealth => "stealth",
                ArrowType::Diamond => "diamond",
                ArrowType::Oval => "oval",
            };
            xml.leaf("a:tailEnd", &[("type", t), ("w", "med"), ("len", "med")]);
        }
    }

    xml.end("a:ln");
    Ok(())
}

fn render_line(xml: &mut Xml, ctx: &mut RenderCtx<'_>, line: &Line) -> Result<()> {
    let id = ctx.next_id().to_string();
    let name = line.common.element_id.clone();
    let points = parse_points(&line.points)?;
    if points.len() < 2 {
        return Err(Error::Invalid(format!(
            "line `{}` needs at least 2 points",
            line.common.element_id
        )));
    }

    let smooth = matches!(line.curve, Some(LineCurve::Smooth));
    let is_straight_2pt = points.len() == 2 && !smooth;

    if is_straight_2pt {
        let (p0, p1) = (points[0], points[1]);
        let min_x = p0.0.min(p1.0);
        let max_x = p0.0.max(p1.0);
        let min_y = p0.1.min(p1.1);
        let max_y = p0.1.max(p1.1);

        let span_x = max_x - min_x;
        let span_y = max_y - min_y;

        let scale_x = if line.view_box.0 > 0.0 {
            line.common.bounds.width / line.view_box.0
        } else {
            1.0
        };
        let scale_y = if line.view_box.1 > 0.0 {
            line.common.bounds.height / line.view_box.1
        } else {
            1.0
        };

        let actual_x = line.common.bounds.x + min_x * scale_x;
        let actual_y = line.common.bounds.y + min_y * scale_y;
        let actual_w = (span_x * scale_x).max(1.0);
        let actual_h = (span_y * scale_y).max(1.0);

        let mut flip_h = line.common.flip.is_some_and(|f| f.0);
        let mut flip_v = line.common.flip.is_some_and(|f| f.1);
        if p1.0 < p0.0 {
            flip_h = !flip_h;
        }
        if p1.1 < p0.1 {
            flip_v = !flip_v;
        }

        let bounds = Bounds {
            x: actual_x,
            y: actual_y,
            width: if span_x < 1e-4 { 0.0 } else { actual_w },
            height: if span_y < 1e-4 { 0.0 } else { actual_h },
        };

        xml.start("p:cxnSp", &[]);
        xml.start("p:nvCxnSpPr", &[]);
        cnv_pr(xml, ctx, &id, &name, line.common.link.as_deref());
        xml.leaf("p:cNvCxnSpPr", &[]);
        xml.leaf("p:nvPr", &[]);
        xml.end("p:nvCxnSpPr");

        xml.start("p:spPr", &[]);
        xfrm(xml, bounds, line.common.rotation, Some((flip_h, flip_v)));
        xml.start("a:prstGeom", &[("prst", "line")]);
        xml.leaf("a:avLst", &[]);
        xml.end("a:prstGeom");

        line_stroke_xml(xml, ctx.theme, line.border.as_ref(), line.arrow, line.curve)?;
        xml.end("p:spPr");
        xml.end("p:cxnSp");
    } else {
        xml.start("p:sp", &[]);
        xml.start("p:nvSpPr", &[]);
        cnv_pr(xml, ctx, &id, &name, line.common.link.as_deref());
        xml.leaf("p:cNvSpPr", &[]);
        xml.leaf("p:nvPr", &[]);
        xml.end("p:nvSpPr");

        xml.start("p:spPr", &[]);
        xfrm(
            xml,
            line.common.bounds,
            line.common.rotation,
            line.common.flip,
        );

        let pw = if line.view_box.0 > 0.0 {
            line.view_box.0
        } else {
            line.common.bounds.width
        };
        let ph = if line.view_box.1 > 0.0 {
            line.view_box.1
        } else {
            line.common.bounds.height
        };
        let path_w = (pw.round() as i64).to_string();
        let path_h = (ph.round() as i64).to_string();
        xml.start("a:custGeom", &[]);
        xml.leaf("a:avLst", &[]);
        xml.leaf("a:gdLst", &[]);
        xml.leaf("a:ahLst", &[]);
        xml.leaf("a:cxnLst", &[]);
        xml.leaf("a:rect", &[("l", "0"), ("t", "0"), ("r", "0"), ("b", "0")]);
        xml.start("a:pathLst", &[]);
        xml.start("a:path", &[("w", &path_w), ("h", &path_h)]);

        let (first, rest) = points.split_first().expect("len >= 2 checked above");
        let first_x = (first.0.round() as i64).to_string();
        let first_y = (first.1.round() as i64).to_string();
        xml.start("a:moveTo", &[]);
        xml.leaf("a:pt", &[("x", &first_x), ("y", &first_y)]);
        xml.end("a:moveTo");

        if smooth {
            for chunk in rest.chunks_exact(3) {
                let p1 = emu(chunk[0].0).to_string();
                let p1y = emu(chunk[0].1).to_string();
                let p2 = emu(chunk[1].0).to_string();
                let p2y = emu(chunk[1].1).to_string();
                let p3 = emu(chunk[2].0).to_string();
                let p3y = emu(chunk[2].1).to_string();
                xml.start("a:cubicBezTo", &[]);
                xml.leaf("a:pt", &[("x", &p1), ("y", &p1y)]);
                xml.leaf("a:pt", &[("x", &p2), ("y", &p2y)]);
                xml.leaf("a:pt", &[("x", &p3), ("y", &p3y)]);
                xml.end("a:cubicBezTo");
            }
        } else {
            for (x, y) in rest {
                let x = (x.round() as i64).to_string();
                let y = (y.round() as i64).to_string();
                xml.start("a:lnTo", &[]);
                xml.leaf("a:pt", &[("x", &x), ("y", &y)]);
                xml.end("a:lnTo");
            }
        }
        xml.end("a:path");
        xml.end("a:pathLst");
        xml.end("a:custGeom");

        xml.leaf("a:noFill", &[]);
        line_stroke_xml(xml, ctx.theme, line.border.as_ref(), line.arrow, line.curve)?;
        xml.end("p:spPr");
        xml.end("p:sp");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Icon
// ---------------------------------------------------------------------------

/// Base64url (RFC 4648 §5, unpadded) — used for the `pptd:icon` extension.
fn base64url(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let n = ((chunk[0] as u32) << 16)
            | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
            | (chunk.get(2).copied().unwrap_or(0) as u32);
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(TABLE[n as usize & 63] as char);
        }
    }
    out
}

fn render_icon(xml: &mut Xml, ctx: &mut RenderCtx<'_>, icon: &Icon) -> Result<()> {
    let glyph = fa::lookup(&icon.icon_name).ok_or_else(|| {
        Error::Unsupported(format!(
            "icon `{}` is not in the embedded Font Awesome dataset \
             (see assets/README.md to regenerate it)",
            icon.icon_name
        ))
    })?;
    let id = ctx.next_id().to_string();
    let name = icon.common.element_id.clone();

    xml.start("p:sp", &[]);
    xml.start("p:nvSpPr", &[]);
    cnv_pr(xml, ctx, &id, &name, icon.common.link.as_deref());
    xml.leaf("p:cNvSpPr", &[]);
    // Keep the PPTD identity for round-tripping (mirrors the Kimi exporter).
    let payload = base64url(format!("{{\"iconName\":\"{}\"}}", icon.icon_name).as_bytes());
    xml.start("p:nvPr", &[]);
    xml.start("p:extLst", &[]);
    xml.start(
        "p:ext",
        &[("uri", "{F5677B7D-0D2A-4D9B-9A5C-6D8D7D0E5A5D}")],
    );
    xml.start("pptd:icon", &[("encoding", "base64url")]);
    xml.text(&payload);
    xml.end("pptd:icon");
    xml.end("p:ext");
    xml.end("p:extLst");
    xml.end("p:nvPr");
    xml.end("p:nvSpPr");

    xml.start("p:spPr", &[]);
    xfrm(
        xml,
        icon.common.bounds,
        icon.common.rotation,
        icon.common.flip,
    );
    let w = (glyph.w as i64).to_string();
    let h = (glyph.h as i64).to_string();
    xml.start("a:custGeom", &[]);
    xml.leaf("a:avLst", &[]);
    xml.leaf("a:gdLst", &[]);
    xml.leaf("a:ahLst", &[]);
    xml.leaf("a:cxnLst", &[]);
    xml.leaf("a:rect", &[("l", "l"), ("t", "t"), ("r", "r"), ("b", "b")]);
    xml.start("a:pathLst", &[]);
    xml.start("a:path", &[("w", &w), ("h", &h)]);
    svg_path::emit_path_children(xml, &glyph.d)?;
    xml.end("a:path");
    xml.end("a:pathLst");
    xml.end("a:custGeom");
    if let Some(fill) = &icon.fill {
        fill_xml(xml, ctx.theme, fill, icon.common.opacity)?;
    } else {
        let resolved = ResolvedColor {
            rgb: "000000".to_owned(),
            alpha: None,
        };
        emit_fill_color(xml, &resolved);
    }
    xml.start("a:ln", &[]);
    xml.leaf("a:noFill", &[]);
    xml.end("a:ln");
    xml.end("p:spPr");
    xml.end("p:sp");
    Ok(())
}

// ---------------------------------------------------------------------------
// Image
// ---------------------------------------------------------------------------

/// Compute the `a:srcRect` crop/pad per `fit` + `crop`. Positive values crop
/// inward, negative pad outward with transparent pixels; unit 100000 = 100%.
fn src_rect(image: &Image, size: Option<ImageSize>) -> (i64, i64, i64, i64) {
    let bw = image.common.bounds.width;
    let bh = image.common.bounds.height;
    let mode = image.fit.map(|f| f.mode).unwrap_or(ImageFitMode::Cover);

    let (mut l, mut t, mut r, mut b) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    if let Some(ImageSize { width, height }) = size {
        let (iw, ih) = (width as f64, height as f64);
        if iw > 0.0 && ih > 0.0 && bw > 0.0 && bh > 0.0 {
            let ratio_box = bw / bh;
            let ratio_img = iw / ih;
            match mode {
                ImageFitMode::Fill => {}
                ImageFitMode::Contain => {
                    if ratio_img > ratio_box {
                        // Image wider than the box: the scale is limited by
                        // width, so the slack is vertical → letterbox
                        // top/bottom with negative crops (kimi parity).
                        let scale = bw / iw;
                        let pad = (bh - ih * scale) / bh / 2.0;
                        t = -pad;
                        b = -pad;
                    } else if ratio_img < ratio_box {
                        // Image taller than the box: scale limited by height,
                        // slack is horizontal → letterbox left/right.
                        let scale = bh / ih;
                        let pad = (bw - iw * scale) / bw / 2.0;
                        l = -pad;
                        r = -pad;
                    }
                }
                ImageFitMode::Cover => {
                    if ratio_img > ratio_box {
                        // Crop left/right.
                        let scale = bh / ih;
                        let frac = (iw * scale - bw) / 2.0 / (iw * scale);
                        l = frac.max(0.0);
                        r = frac.max(0.0);
                    } else if ratio_img < ratio_box {
                        let scale = bw / iw;
                        let frac = (ih * scale - bh) / 2.0 / (ih * scale);
                        t = frac.max(0.0);
                        b = frac.max(0.0);
                    }
                }
            }
        }
    }
    if let Some(crop) = &image.crop {
        l += crop.left.unwrap_or(0.0);
        t += crop.top.unwrap_or(0.0);
        r += crop.right.unwrap_or(0.0);
        b += crop.bottom.unwrap_or(0.0);
    }
    let to_per = |f: f64| -> i64 { (f * 100000.0).round() as i64 };
    (to_per(l), to_per(t), to_per(r), to_per(b))
}

fn render_image(xml: &mut Xml, ctx: &mut RenderCtx<'_>, image: &Image) -> Result<()> {
    let id = ctx.next_id().to_string();
    let name = image.common.element_id.clone();
    let rid = ctx.media_rid(&image.src).ok_or_else(|| {
        Error::Invalid(format!(
            "internal error: media `{}` was not registered before rendering",
            image.src
        ))
    })?;

    xml.start("p:pic", &[]);
    xml.start("p:nvPicPr", &[]);
    cnv_pr(xml, ctx, &id, &name, image.common.link.as_deref());
    xml.leaf("p:cNvPicPr", &[]);
    xml.leaf("p:nvPr", &[]);
    xml.end("p:nvPicPr");

    xml.start("p:blipFill", &[]);
    xml.start("a:blip", &[("r:embed", &rid)]);
    xml.end("a:blip");
    let (l, t, r, b) = src_rect(image, ctx.image_size(&image.src));
    if l != 0 || t != 0 || r != 0 || b != 0 {
        let mut attrs: Vec<(&str, String)> = Vec::new();
        if l != 0 {
            attrs.push(("l", l.to_string()));
        }
        if t != 0 {
            attrs.push(("t", t.to_string()));
        }
        if r != 0 {
            attrs.push(("r", r.to_string()));
        }
        if b != 0 {
            attrs.push(("b", b.to_string()));
        }
        let attrs_ref: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        xml.start("a:srcRect", &attrs_ref);
        xml.end("a:srcRect");
    }
    xml.start("a:stretch", &[]);
    xml.leaf("a:fillRect", &[]);
    xml.end("a:stretch");
    xml.end("p:blipFill");

    xml.start("p:spPr", &[]);
    xfrm(
        xml,
        image.common.bounds,
        image.common.rotation,
        image.common.flip,
    );
    match &image.crop_shape {
        Some(shape) => {
            xml.start("a:prstGeom", &[("prst", &shape.shape_name)]);
            xml.start("a:avLst", &[]);
            if let Some(adjustments) = &shape.adjustments {
                for (i, &adj) in adjustments.iter().take(10).enumerate() {
                    let adj_name = if i == 0 {
                        "adj".to_owned()
                    } else {
                        format!("adj{}", i + 1)
                    };
                    let value = (adj.round() as i64).to_string();
                    let fmla = format!("val {value}");
                    xml.leaf("a:gd", &[("name", &adj_name), ("fmla", &fmla)]);
                }
            }
            xml.end("a:avLst");
            xml.end("a:prstGeom");
        }
        None => {
            xml.start("a:prstGeom", &[("prst", "rect")]);
            xml.leaf("a:avLst", &[]);
            xml.end("a:prstGeom");
        }
    }
    if let Some(border) = &image.border {
        border_xml(xml, ctx.theme, border)?;
    } else {
        xml.start("a:ln", &[]);
        xml.leaf("a:noFill", &[]);
        xml.end("a:ln");
    }
    if let Some(radius_px) = image.soft_edge.filter(|r| *r > 0.0) {
        let rad = emu(radius_px) as u64;
        xml.start("a:effectLst", &[]);
        xml.leaf("a:softEdge", &[("rad", &rad.to_string())]);
        xml.end("a:effectLst");
    }
    xml.end("p:spPr");
    xml.end("p:pic");
    Ok(())
}

/// Render an embedded audio/video clip: the OOXML "media picture" — a
/// `p:pic` whose `nvPr` carries the `a:videoFile`/`a:audioFile` reference
/// plus the `p14:media` extension (PowerPoint 2010+ proper media embed;
/// the legacy r:link fallback stays for old renderers), with the poster
/// image as the picture's blip.
fn render_media(xml: &mut Xml, ctx: &mut RenderCtx<'_>, m: &Media, kind: &str) -> Result<()> {
    let id = ctx.next_id().to_string();
    let name = m.common.element_id.clone();
    let media_rid = ctx.media_rid(&m.src).ok_or_else(|| {
        Error::Invalid(format!(
            "internal error: media `{}` was not registered before rendering",
            m.src
        ))
    })?;
    let poster_rid = m
        .poster
        .as_deref()
        .and_then(|p| ctx.media_rid(p))
        .ok_or_else(|| {
            Error::Invalid(format!(
                "media element `{}` needs a poster image (png/jpg) — \"
                 PowerPoint requires a visible frame for clips",
                m.common.element_id
            ))
        })?;

    let (av_element, _av_rel_kind) = if kind == "video" {
        ("a:videoFile", super::opc::rel_kind::VIDEO)
    } else {
        ("a:audioFile", super::opc::rel_kind::AUDIO)
    };
    // The .rels entry's video/audio type is chosen by the writer from the
    // media part's extension; av_rel_kind documents the pairing.
    let _ = _av_rel_kind;

    xml.start("p:pic", &[]);
    xml.start("p:nvPicPr", &[]);
    xml.start("p:cNvPr", &[("id", &id), ("name", &name)]);
    // Media clips carry a hyperlink-free clickable-object hint.
    xml.leaf(
        "a:hlinkClick",
        &[("r:id", ""), ("action", "ppaction://media")],
    );
    xml.end("p:cNvPr");
    xml.leaf("p:cNvPicPr", &[]);
    xml.start("p:nvPr", &[]);
    xml.leaf(av_element, &[("r:link", &media_rid)]);
    // PowerPoint 2010+ extension: marks the pic as a true media object.
    xml.start("a:extLst", &[]);
    xml.start(
        "a:ext",
        &[("uri", "{DAA4B4D4-6D71-4841-9C94-3DE7FCFB9230}")],
    );
    xml.start(
        "p14:media",
        &[
            (
                "xmlns:p14",
                "http://schemas.microsoft.com/office/powerpoint/2010/main",
            ),
            ("r:embed", &media_rid),
        ],
    );
    xml.end("p14:media");
    xml.end("a:ext");
    xml.end("a:extLst");
    xml.end("p:nvPr");
    xml.end("p:nvPicPr");

    // Poster frame as the picture fill.
    xml.start("p:blipFill", &[]);
    xml.start("a:blip", &[("r:embed", &poster_rid)]);
    xml.end("a:blip");
    xml.start("a:stretch", &[]);
    xml.leaf("a:fillRect", &[]);
    xml.end("a:stretch");
    xml.end("p:blipFill");

    xml.start("p:spPr", &[]);
    xfrm(xml, m.common.bounds, m.common.rotation, m.common.flip);
    xml.start("a:prstGeom", &[("prst", "rect")]);
    xml.leaf("a:avLst", &[]);
    xml.end("a:prstGeom");
    xml.end("p:spPr");
    xml.end("p:pic");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pptd::shared::ColorStop;

    fn border(width: Option<f64>) -> Border {
        Border {
            style: Some(LineStyle::Solid),
            width,
            color: Some(Color("000000".into())),
            gradient: None,
        }
    }

    #[test]
    fn border_width_policy_distinguishes_empty_and_filled() {
        // An empty `<a:ln>` (no solid/gradient fill) may drop `w`: the
        // P2 gradient-frame test confirmed that an explicit default
        // attribute flipped the line gradient bounding box in Impress.
        // A line with a fill, however, must keep `w`; OOXML otherwise
        // defaults the attribute to 0.75pt and a 1pt source frame ends
        // up visibly thinner in the rebuild.
        let mut xml = Xml::new();
        border_xml(&mut xml, None, &border(None)).unwrap();
        let empty = xml.into_string();
        assert!(empty.contains("<a:ln>"), "{empty}");
        assert!(!empty.contains(" w="), "{empty}");

        let mut xml = Xml::new();
        border_xml(&mut xml, None, &border(Some(1.0))).unwrap();
        let filled = xml.into_string();
        assert!(filled.contains("<a:ln w=\"12700\">"), "{filled}");

        let mut xml = Xml::new();
        border_xml(&mut xml, None, &border(Some(2.0))).unwrap();
        assert!(xml.into_string().contains("<a:ln w=\"25400\">"));
    }

    #[test]
    fn gradient_border_keeps_lin_scaled() {
        let b = Border {
            style: Some(LineStyle::Solid),
            width: None,
            color: None,
            gradient: Some(crate::pptd::shared::GradientFill {
                gradient_type: GradientType::Linear,
                stops: vec![
                    ColorStop {
                        position: 0.0,
                        color: Color("FFFFFF".into()),
                    },
                    ColorStop {
                        position: 1.0,
                        color: Color("2C94FB".into()),
                    },
                ],
                angle: Some(45.0),
                scaled: true,
            }),
        };
        let mut xml = Xml::new();
        border_xml(&mut xml, None, &b).unwrap();
        let out = xml.into_string();
        assert!(out.contains("scaled=\"1\""), "{out}");
    }
    #[test]
    fn parse_runs_handles_underline_and_strikethrough_without_panic() {
        let text = "<p>Normal <u>underlined</u> and <s>struck</s> and <strong>bold</strong></p>";
        let paras = parse_rich(text);
        assert_eq!(paras.len(), 1);
        let runs = paras[0].runs();
        assert!(
            runs.iter()
                .any(|r| r.text == "underlined" && r.underline == Some(true))
        );
        assert!(
            runs.iter()
                .any(|r| r.text == "struck" && r.strike == Some(true))
        );
        assert!(
            runs.iter()
                .any(|r| r.text == "bold" && r.bold == Some(true))
        );
    }

    #[test]
    fn parse_runs_handles_unknown_tags_without_slice_panic() {
        let text = "<p>Prefix <ul><li>item</li></ul> <a>link</a> suffix</p>";
        let paras = parse_rich(text);
        assert_eq!(paras.len(), 1);
        // Must not panic on slice index and must retain text
        let combined: String = paras[0].runs().iter().map(|r| r.text.as_str()).collect();
        assert!(combined.contains("Prefix"));
        assert!(combined.contains("suffix"));
    }

    #[test]
    fn render_rich_body_emits_underline_and_strike_attributes() {
        let style = EffTextStyle::default();
        let mut xml = Xml::new();
        render_rich_body(
            &mut xml,
            &mut RenderCtx::new(None),
            &style,
            "<p><u>underline</u> <s>strike</s></p>",
            "tl",
            false,
        )
        .unwrap();
        let out = xml.into_string();
        assert!(out.contains("u=\"sng\""), "{out}");
        assert!(out.contains("strike=\"sngStrike\""), "{out}");
    }

    #[test]
    fn render_rich_body_emits_letter_spacing_and_vertical_direction() {
        let style = EffTextStyle {
            letter_spacing: Some(2.0),
            direction: Some(TextDirection::Vertical),
            ..Default::default()
        };
        let mut xml = Xml::new();
        render_rich_body(
            &mut xml,
            &mut RenderCtx::new(None),
            &style,
            "<p>Vertical Text</p>",
            "tl",
            false,
        )
        .unwrap();
        let out = xml.into_string();
        // vert="eaVert" on a:bodyPr
        assert!(out.contains("vert=\"eaVert\""), "{out}");
        // spc="150" (2.0 * 75) on a:rPr
        assert!(out.contains("spc=\"150\""), "{out}");
    }

    #[test]
    fn render_rich_body_emits_native_omml_display_and_inline() {
        let style = EffTextStyle::default();
        let mut xml = Xml::new();
        // Display formula (whole paragraph)
        render_rich_body(
            &mut xml,
            &mut RenderCtx::new(None),
            &style,
            "<p><latex>\\frac{a+b}{c+d}</latex></p>",
            "tl",
            false,
        )
        .unwrap();
        let out = xml.into_string();
        assert!(out.contains("<a14:m><m:oMathPara>"), "{out}");
        assert!(out.contains("<m:f>"), "{out}");

        let mut xml_inline = Xml::new();
        // Inline formula
        render_rich_body(
            &mut xml_inline,
            &mut RenderCtx::new(None),
            &style,
            "<p>Formula: <latex>\\sqrt{x}</latex> inline</p>",
            "tl",
            false,
        )
        .unwrap();
        let out_inline = xml_inline.into_string();
        assert!(out_inline.contains("<a14:m><m:oMath>"), "{out_inline}");
        assert!(!out_inline.contains("<m:oMathPara>"), "{out_inline}");
        assert!(out_inline.contains("<m:rad>"), "{out_inline}");
    }
}
