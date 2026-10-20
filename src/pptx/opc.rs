//! OPC (Open Packaging Conventions) helpers: namespaces, relationship types,
//! and the builders for `[Content_Types].xml` and `.rels` files.

use super::package::PackageEntry;
use super::xml::Xml;

/// XML namespaces used by the generated parts.
pub mod ns {
    pub const PRESENTATIONML: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
    pub const DRAWINGML: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
    pub const RELATIONSHIPS: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    pub const CONTENT_TYPES: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
    pub const PACKAGE_REL: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
    pub const CORE_PROPS: &str =
        "http://schemas.openxmlformats.org/package/2006/metadata/core-properties";
    pub const DC: &str = "http://purl.org/dc/elements/1.1/";
    pub const DCTERMS: &str = "http://purl.org/dc/terms/";
    pub const DCMITYPE: &str = "http://purl.org/dc/dcmitype/";
    pub const XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";
    pub const EXTENDED_PROPS: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties";
    /// The Kimi/PPTD extension namespace (icons, element metadata).
    pub const PPTD: &str = "https://kimi-design.msh.team/pptd/2026";
}

/// Relationship type URIs used by the writer.
pub mod rel_kind {
    pub const OFFICE_DOCUMENT: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
    pub const CORE_PROPERTIES: &str =
        "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
    pub const SLIDE: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";
    pub const NOTES_SLIDE: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide";
    pub const SLIDE_LAYOUT: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";
    pub const SLIDE_MASTER: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";
    pub const THEME: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";
    pub const IMAGE: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
    pub const MEDIA: &str = "http://schemas.microsoft.com/office/2007/relationships/media";
    pub const VIDEO: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/video";
    pub const AUDIO: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/audio";
    pub const NOTES_MASTER: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster";
    pub const HYPERLINK: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
    pub const EXTENDED_PROPERTIES: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";
    pub const CHART: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart";
    /// Chart → embedded workbook (`<c:externalData>`).
    pub const PACKAGE: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/package";
}
/// One relationship inside a `.rels` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rel {
    pub id: String,
    pub kind: &'static str,
    pub target: String,
    /// `Some("External")` for targets outside the package (hyperlinks).
    pub target_mode: Option<&'static str>,
}

impl Rel {
    pub fn new(id: &str, kind: &'static str, target: impl Into<String>) -> Self {
        Self {
            id: id.to_owned(),
            kind,
            target: target.into(),
            target_mode: None,
        }
    }

    /// An external relationship (`TargetMode="External"`). External targets
    /// are written verbatim — never resolved against the part's directory,
    /// never fetched.
    pub fn external(id: &str, kind: &'static str, target: impl Into<String>) -> Self {
        Self {
            id: id.to_owned(),
            kind,
            target: target.into(),
            target_mode: Some("External"),
        }
    }
}

/// Render a `.rels` file.
pub fn rels_xml(rels: &[Rel]) -> String {
    let mut xml = Xml::new();
    xml.start("Relationships", &[("xmlns", ns::PACKAGE_REL)]);
    for rel in rels {
        let mut attrs = vec![
            ("Id", rel.id.as_str()),
            ("Type", rel.kind),
            ("Target", rel.target.as_str()),
        ];
        if let Some(mode) = rel.target_mode {
            attrs.push(("TargetMode", mode));
        }
        xml.leaf("Relationship", &attrs);
    }
    xml.end("Relationships");
    xml.into_string()
}

/// Render `[Content_Types].xml` from the package entries. Adopts the OPC
/// rule: a `Default` for `xml` / `rels` extensions (plus image extensions
/// collected from [`MediaRegistry`](super::media::MediaRegistry)), and an
/// `Override` for every typed part.
pub fn content_types_xml(entries: &[PackageEntry], media_defaults: &[String]) -> String {
    let mut xml = Xml::new();
    xml.start("Types", &[("xmlns", ns::CONTENT_TYPES)]);
    xml.leaf(
        "Default",
        &[
            ("Extension", "rels"),
            (
                "ContentType",
                "application/vnd.openxmlformats-package.relationships+xml",
            ),
        ],
    );
    xml.leaf(
        "Default",
        &[("Extension", "xml"), ("ContentType", "application/xml")],
    );
    for extension in media_defaults {
        let mime = match extension.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "mp4" | "m4v" => "video/mp4",
            "mov" => "video/quicktime",
            "mp3" => "audio/mpeg",
            "m4a" => "audio/mp4",
            "wav" => "audio/x-wav",
            "wma" => "audio/x-ms-wma",
            _ => continue,
        };
        xml.leaf(
            "Default",
            &[("Extension", extension), ("ContentType", mime)],
        );
    }
    for entry in entries {
        if let Some(content_type) = &entry.content_type {
            xml.leaf(
                "Override",
                &[
                    ("PartName", &format!("/{}", entry.path)),
                    ("ContentType", content_type.mime()),
                ],
            );
        }
    }
    xml.end("Types");
    xml.into_string()
}
