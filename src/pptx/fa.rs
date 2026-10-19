//! Font Awesome solid-icon glyph dataset.
//!
//! The SVG path data comes from the Font Awesome 6.7.2 metadata and is
//! embedded at compile time (`assets/fa-solid-icons.json`, CC BY 4.0 — see
//! `assets/README.md` for the source and a regeneration recipe). Icons
//! missing from the dataset fail the build with an explicit error.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

const ASSET: &str = include_str!("../../assets/fa-solid-icons.json");

/// One icon glyph: viewBox size + SVG path `d`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FaIcon {
    pub w: f64,
    pub h: f64,
    pub d: String,
}

static ICONS: OnceLock<HashMap<String, FaIcon>> = OnceLock::new();

/// The embedded dataset (`"mobile-screen" -> FaIcon`).
fn icons() -> &'static HashMap<String, FaIcon> {
    ICONS.get_or_init(|| {
        // The asset is a JSON document, which is valid YAML; parse it once.
        serde_yaml::from_str(ASSET).unwrap_or_default()
    })
}

/// An error encountered when resolving a Font Awesome icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaError {
    /// The icon uses an unsupported style prefix (e.g. `far:` regular or `fab:` brands).
    /// Currently only solid (`fas:`) icons are supported.
    UnsupportedPrefix { prefix: String, icon: String },
    /// The icon was not found in the embedded Font Awesome dataset.
    NotFound { icon: String },
}

impl std::fmt::Display for FaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FaError::UnsupportedPrefix { prefix, icon } => {
                write!(
                    f,
                    "unsupported icon style prefix `{prefix}:` on `{icon}` (only solid `fas:` icons are supported)"
                )
            }
            FaError::NotFound { icon } => {
                write!(
                    f,
                    "icon `{icon}` is not in the embedded Font Awesome dataset (1422 icons supported)"
                )
            }
        }
    }
}

impl std::error::Error for FaError {}

/// Look up an icon glyph by its PPTD name (`"fas:mobile-screen"`, `"mobile-screen"`).
///
/// Only solid icons (`fas:` or unprefixed) are supported. Unsupported prefixes
/// (`far:`, `fab:`, etc.) return `Err(FaError::UnsupportedPrefix)`.
pub fn lookup_glyph(name: &str) -> Result<&'static FaIcon, FaError> {
    let (prefix, mut key) = if let Some((p, rest)) = name.split_once(':') {
        (Some(p), rest)
    } else {
        (None, name)
    };

    if let Some(p) = prefix {
        if p != "fas" {
            return Err(FaError::UnsupportedPrefix {
                prefix: p.to_string(),
                icon: name.to_string(),
            });
        }
    }

    if let Some(stripped) = key.strip_prefix("fa-") {
        key = stripped;
    }

    icons().get(key).ok_or_else(|| FaError::NotFound {
        icon: name.to_string(),
    })
}

/// Look up an icon by its PPTD name (`"fas:mobile-screen"`); unknown style
/// prefixes or names return `None`.
pub fn lookup(name: &str) -> Option<&'static FaIcon> {
    lookup_glyph(name).ok()
}

/// Number of icons in the embedded dataset (exposed for tests).
pub fn dataset_len() -> usize {
    icons().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pptx::svg_path;

    #[test]
    fn every_embedded_icon_parses() {
        assert!(
            dataset_len() >= 32,
            "dataset unexpectedly small: {}",
            dataset_len()
        );
        for (name, icon) in icons() {
            svg_path::parse(&icon.d)
                .unwrap_or_else(|err| panic!("icon `{name}` fails to parse: {err}"));
        }
    }

    #[test]
    fn prefix_handling_solid_vs_unsupported() {
        assert!(lookup("fas:bolt").is_some());
        assert!(lookup("bolt").is_some());
        assert!(lookup("fa-bolt").is_some());
        assert!(lookup("fas:fa-bolt").is_some());

        // Unsupported prefixes must not resolve to solid glyphs
        assert_eq!(lookup("far:bolt"), None);
        assert_eq!(lookup("fab:bolt"), None);
        assert_eq!(lookup("far:heart"), None);
        assert_eq!(lookup("fab:github"), None);

        // Detailed error messages
        assert_eq!(
            lookup_glyph("far:bolt"),
            Err(FaError::UnsupportedPrefix {
                prefix: "far".to_string(),
                icon: "far:bolt".to_string(),
            })
        );
        assert_eq!(
            lookup_glyph("fab:github"),
            Err(FaError::UnsupportedPrefix {
                prefix: "fab".to_string(),
                icon: "fab:github".to_string(),
            })
        );
        assert_eq!(
            lookup_glyph("fas:nonexistent_icon_xyz"),
            Err(FaError::NotFound {
                icon: "fas:nonexistent_icon_xyz".to_string(),
            })
        );
    }
}
