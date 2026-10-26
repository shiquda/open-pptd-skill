//! The root AST node of a PPTD project: [`Presentation`] and [`Page`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::animation::Animation;
use super::elements::{Element, GroupDef};
use super::layout::LayoutDef;
use super::shared::{Fill, Size};
use super::theme::{CustomFont, Theme};

/// The main entry file (`*.pptd`) of a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    /// Version identifier. The loader enforces `"v2"`.
    pub version: String,
    /// Optional presentation title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional presentation author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Optional presentation description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_fonts: Vec<CustomFont>,
    /// Canvas size in px; e.g. `[960, 540]` for 16:9.
    pub size: Size,
    /// Map web-only typefaces (`MiSans`, `Liter`, `QuattrocentoSans`, …) onto
    /// universally installed system fonts at build time. Default `false`:
    /// font names are written verbatim, so `convert` → `build` keeps the
    /// source typography. Decks that rely on the safety mapping opt in.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub safe_fonts: bool,
    /// Shared theme: colors, text styles and table styles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    /// SlideForge extension: named layouts a page may reference via
    /// [`Page::layout`]. See `docs/pptd-layout-extension.md`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layouts: Option<HashMap<String, LayoutDef>>,
    /// Relative paths to the page files, e.g. `pages/1_cover.page`.
    pub pages: Vec<String>,
}
crate::define_field_names!(Presentation {
    version: "version",
    title: "title",
    author: "author",
    description: "description",
    custom_fonts: "customFonts",
    size: "size",
    safe_fonts: "safeFonts",
    theme: "theme",
    layouts: "layouts",
    pages: "pages",
});


/// A single slide. Page files are loaded only through the main entry file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    /// Optional slide format version (e.g. `"v2"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Optional slide title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Category label; does not affect rendering.
    #[serde(default, alias = "pageType", skip_serializing_if = "Option::is_none")]
    pub page_type: Option<String>,
    /// SlideForge extension: references a key in
    /// [`Presentation::layouts`]. When set, the page inherits the layout's
    /// background (unless overridden), decorative `elements` (painted under
    /// the page's own) and `placeholders`. See
    /// `docs/pptd-layout-extension.md`. Unset → fully self-contained
    /// (canonical PPTD v2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    /// Page background; defaults to a white solid fill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<Fill>,
    /// Speaker notes (plain text).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Elements, in paint order: the later an element, the higher its layer.
    #[serde(default)]
    pub elements: Vec<Element>,
    /// Optional animation orchestrations referencing `elements[].element_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<Animation>>,
    /// SlideForge extension: reconstructed `<p:grpSp>` group metadata keyed by
    /// id. Member elements carry `groupId` + `groupBounds` (child-space) on
    /// [`ElementCommon`]; the PPTD `elements` array stays flat (slide-space
    /// `bounds`), and the writer rebuilds the nesting on output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groups: Option<HashMap<String, GroupDef>>,
}
crate::define_field_names!(Page {
    version: "version",
    title: "title",
    page_type: "page_type",
    layout: "layout",
    background: "background",
    notes: "notes",
    elements: "elements",
    animations: "animations",
    groups: "groups",
} extra: ["pageType"]);
