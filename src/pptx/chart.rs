//! Native chart parts: `ppt/charts/chartN.xml` plus the embedded workbook.
//!
//! Charts are carried verbatim: `pptd convert` copies the chart XML and its
//! `<c:externalData>` workbook into the project `media/` directory, and the
//! writer re-emits them as package parts, wiring the slide relationship,
//! the chart's own relationship to the workbook, and the content-type
//! overrides. The chart XML is reproduced byte-for-byte; only
//! `<c:externalData r:id>` is examined so the generated chart `.rels` keeps
//! the id the chart expects.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use xmltree::Element as XmlEl;

use crate::pptd::Chart;
use crate::{Error, Result};

use super::opc::ns;
use super::render::RenderCtx;
use super::xml::Xml;

/// One chart part staged for the package.
#[derive(Debug)]
pub struct ChartPart {
    /// Deck-relative source path (used for deduplication).
    pub src: String,
    /// `ppt/charts/chartN.xml`.
    pub package_path: String,
    pub data: Vec<u8>,
    /// `r:id` of `<c:externalData>` inside `data`, if the chart has one.
    pub external_rid: Option<String>,
    pub workbook: Option<WorkbookPart>,
}

/// The embedded workbook a chart's `<c:externalData>` points at.
#[derive(Debug)]
pub struct WorkbookPart {
    /// `ppt/embeddings/<name>.xlsx`.
    pub package_path: String,
    pub data: Vec<u8>,
}

/// Deduplicates and numbers chart parts referenced by a build.
#[derive(Debug)]
pub struct ChartRegistry {
    root_dir: PathBuf,
    parts: Vec<ChartPart>,
    index: HashMap<String, usize>,
    /// Workbook part names already taken (two charts may embed differently
    /// named or same-named workbooks).
    workbook_names: HashSet<String>,
}

impl ChartRegistry {
    pub fn new(root_dir: &Path) -> Self {
        Self {
            root_dir: root_dir.to_path_buf(),
            parts: Vec::new(),
            index: HashMap::new(),
            workbook_names: HashSet::new(),
        }
    }

    /// Register `src` (a chart part relative to the project root) together
    /// with its optional workbook part. Repeated sources reuse the part.
    pub fn register(&mut self, src: &str, workbook_src: Option<&str>) -> Result<usize> {
        if let Some(&index) = self.index.get(src) {
            return Ok(index);
        }

        let data = std::fs::read(self.root_dir.join(src)).map_err(|source| Error::Io {
            path: self.root_dir.join(src),
            source,
        })?;
        let xml_rid = external_data_rid(&data)?;

        let workbook = match workbook_src {
            Some(wsrc) => {
                let wdata = std::fs::read(self.root_dir.join(wsrc)).map_err(|source| Error::Io {
                    path: self.root_dir.join(wsrc),
                    source,
                })?;
                let name = unique_workbook_name(wsrc, &mut self.workbook_names);
                Some(WorkbookPart {
                    package_path: format!("ppt/embeddings/{name}"),
                    data: wdata,
                })
            }
            None => None,
        };

        let external_rid = match (&xml_rid, &workbook) {
            (Some(rid), Some(_)) => {
                // `pptd convert` copies the chart's own `.rels` next to the
                // chart part; it preserves the authored relationship id.
                let rels_src = format!("{src}.rels");
                let rels_path = self.root_dir.join(&rels_src);
                let rid = if rels_path.is_file() {
                    let rels = std::fs::read(&rels_path).map_err(|source| Error::Io {
                        path: rels_path,
                        source,
                    })?;
                    let wb_name = workbook_src
                        .and_then(|s| s.rsplit('/').next())
                        .unwrap_or("workbook.xlsx")
                        .to_string();
                    chart_package_rid(&rels, &wb_name)?
                } else {
                    rid.clone()
                };
                Some(rid)
            }
            (None, Some(_)) => {
                return Err(Error::Invalid(format!(
                    "chart part `{src}` declares `workbookPart` but the chart XML has no `<c:externalData>`"
                )));
            }
            (Some(_), None) => {
                return Err(Error::Invalid(format!(
                    "chart part `{src}` embeds a workbook (`<c:externalData>`) but the element declares no `workbookPart`"
                )));
            }
            (None, None) => None,
        };

        let index = self.parts.len();
        self.parts.push(ChartPart {
            src: src.to_owned(),
            package_path: format!("ppt/charts/chart{}.xml", index + 1),
            data,
            external_rid,
            workbook,
        });
        self.index.insert(src.to_owned(), index);
        Ok(index)
    }

    pub fn part(&self, index: usize) -> &ChartPart {
        &self.parts[index]
    }

    pub fn into_parts(self) -> Vec<ChartPart> {
        self.parts
    }
}

/// Keep the source basename (PowerPoint stores recognizable workbook names)
/// but make it unique and path-safe for the package.
fn unique_workbook_name(src: &str, used: &mut HashSet<String>) -> String {
    let mut base = src
        .rsplit('/')
        .next()
        .unwrap_or("workbook.xlsx")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    if base.is_empty() || base.starts_with('.') {
        base = "workbook.xlsx".to_string();
    }
    if !base.contains('.') {
        base.push_str(".xlsx");
    }
    if used.insert(base.clone()) {
        return base;
    }
    let (stem, ext) = match base.rsplit_once('.') {
        Some((stem, ext)) => (stem.to_string(), format!(".{ext}")),
        None => (base.clone(), String::new()),
    };
    let mut n = 2;
    loop {
        let candidate = format!("{stem}_{n}{ext}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}

/// Resolve the relationship id the chart XML uses for its embedded workbook,
/// from the chart's own `.rels`. Only the package (workbook) relationship is
/// supported; anything else would dangle once the part is renumbered.
fn chart_package_rid(rels: &[u8], workbook_name: &str) -> Result<String> {
    let root = XmlEl::parse(rels)
        .map_err(|e| Error::Invalid(format!("chart .rels is not well-formed XML: {e}")))?;
    let mut rid = None;
    for rel in root.children.iter().filter_map(|n| n.as_element()) {
        if rel.name != "Relationship" {
            continue;
        }
        let id = rel.attributes.get("Id").cloned().unwrap_or_default();
        let ty = rel.attributes.get("Type").cloned().unwrap_or_default();
        let target = rel.attributes.get("Target").cloned().unwrap_or_default();
        if rel.attributes.get("TargetMode").map(String::as_str) == Some("External") {
            return Err(Error::Unsupported(format!(
                "chart relationship `{id}` targets an external resource (`{target}`), which is not supported"
            )));
        }
        if !ty.ends_with("/package") {
            return Err(Error::Unsupported(format!(
                "chart relationship `{id}` (`{ty}`) is not supported; only the embedded workbook round-trips"
            )));
        }
        let name = target.rsplit('/').next().unwrap_or(&target);
        if name != workbook_name {
            return Err(Error::Invalid(format!(
                "chart .rels points at `{target}` but the element declares workbook `{workbook_name}`"
            )));
        }
        rid = Some(id);
    }
    rid.ok_or_else(|| {
        Error::Invalid("chart .rels declares no embedded-workbook relationship".to_string())
    })
}

/// The chart's `<c:externalData r:id>` relationship id, if present. A missing
/// `r:id` on an existing `<c:externalData>` is a hard error: the rebuilt
/// chart would point at nothing.
fn external_data_rid(data: &[u8]) -> Result<Option<String>> {
    let root = XmlEl::parse(data)
        .map_err(|e| Error::Invalid(format!("chart part is not well-formed XML: {e}")))?;
    match first_descendant(&root, "externalData") {
        Some(el) => el.attributes.get("id").cloned().map(Some).ok_or_else(|| {
            Error::Invalid("chart `<c:externalData>` has no `r:id`".to_string())
        }),
        None => Ok(None),
    }
}

/// Plot-family hint from the chart XML (`bar`, `line`, `pie`, …). The chart
/// XML stays authoritative; this is metadata for the PPTD model.
pub fn plot_kind(data: &[u8]) -> Option<String> {
    let root = XmlEl::parse(data).ok()?;
    let plot_area = first_descendant(&root, "plotArea")?;
    for child in plot_area.children.iter().filter_map(|n| n.as_element()) {
        let Some(stem) = child.name.strip_suffix("Chart") else {
            continue;
        };
        if !stem.is_empty() {
            return Some(stem.to_lowercase());
        }
    }
    None
}

fn first_descendant<'a>(el: &'a XmlEl, name: &str) -> Option<&'a XmlEl> {
    if el.name == name {
        return Some(el);
    }
    for child in el.children.iter().filter_map(|n| n.as_element()) {
        if let Some(found) = first_descendant(child, name) {
            return Some(found);
        }
    }
    None
}

/// Render a passthrough chart as `<p:graphicFrame>` + `<c:chart r:id>`.
pub(crate) fn render_chart(xml: &mut Xml, ctx: &mut RenderCtx<'_>, chart: &Chart) -> Result<()> {
    let src = chart.chart_part.as_deref().ok_or_else(|| {
        Error::Unsupported(format!(
            "chart `{}` has no `chartPart`",
            chart.common.element_id
        ))
    })?;
    let rid = ctx.chart_rid(src).ok_or_else(|| {
        Error::Unsupported(format!(
            "chart `{}` references `{src}`, which was not registered on this slide",
            chart.common.element_id
        ))
    })?;

    let id = ctx.next_id().to_string();
    xml.start("p:graphicFrame", &[]);
    xml.start("p:nvGraphicFramePr", &[]);
    super::render::cnv_pr(
        xml,
        ctx,
        &id,
        &chart.common.element_id,
        chart.common.link.as_deref(),
    );
    xml.start("p:cNvGraphicFramePr", &[]);
    xml.leaf("a:graphicFrameLocks", &[("noGrp", "1")]);
    xml.end("p:cNvGraphicFramePr");
    xml.leaf("p:nvPr", &[]);
    xml.end("p:nvGraphicFramePr");
    super::render::xfrm_element(
        xml,
        "p:xfrm",
        chart.common.bounds,
        chart.common.rotation,
        chart.common.flip,
    );
    xml.start("a:graphic", &[]);
    xml.start(
        "a:graphicData",
        &[(
            "uri",
            "http://schemas.openxmlformats.org/drawingml/2006/chart",
        )],
    );
    xml.leaf(
        "c:chart",
        &[
            ("xmlns:c", "http://schemas.openxmlformats.org/drawingml/2006/chart"),
            ("xmlns:r", ns::RELATIONSHIPS),
            ("r:id", &rid),
        ],
    );
    xml.end("a:graphicData");
    xml.end("a:graphic");
    xml.end("p:graphicFrame");
    Ok(())
}
