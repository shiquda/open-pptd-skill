//! The PPTD language: a typed AST, a parser and a validator.
//!
//! PPTD (PPT-DSL) is a YAML-based abstraction layer over OOXML used to
//! describe, generate and edit presentations. The AST in [`ast`] mirrors the
//! format specification so that a project can be parsed, validated and
//! eventually written back losslessly.

/// Helper macro to attach a compile-time exhaustiveness guard and a static list
/// of known YAML field names to an AST struct.
///
/// If any field is added, removed, or renamed on `$struct_name` without updating
/// this declaration, `_exhaustiveness_guard` will produce a compile-time error.
#[macro_export]
macro_rules! define_field_names {
    (
        $struct_name:ident {
            $( $field:ident : $name:literal ),* $(,)?
        } $( extra: [ $( $extra:literal ),* $(,)? ] )?
    ) => {
        impl $struct_name {
            /// Known YAML field names accepted for this struct.
            pub const KNOWN_FIELDS: &'static [&'static str] = &[
                $( $name, )*
                $( $( $extra, )* )?
            ];

            #[doc(hidden)]
            #[inline(always)]
            pub fn _exhaustiveness_guard(dummy: &Self) {
                let Self { $( $field: _ ),* } = dummy;
            }
        }
    };
}

/// Helper macro to attach a compile-time exhaustiveness guard and a static list
/// of known YAML field names for an element struct that flattens `ElementCommon`.
#[macro_export]
macro_rules! define_element_fields {
    (
        $struct_name:ident {
            common: _,
            $( $field:ident : $name:literal ),* $(,)?
        } $( extra: [ $( $extra:literal ),* $(,)? ] )?
    ) => {
        impl $struct_name {
            /// Known YAML field names specific to this element type (excluding flattened common fields).
            pub const OWN_FIELDS: &'static [&'static str] = &[
                $( $name, )*
                $( $( $extra, )* )?
            ];

            #[doc(hidden)]
            #[inline(always)]
            pub fn _exhaustiveness_guard(dummy: &Self) {
                let Self { common: _, $( $field: _ ),* } = dummy;
            }
        }
    };
}

pub mod animation;
pub mod ast;
pub mod chart;
pub mod elements;
pub mod layout;
pub mod parser;
pub mod shared;
pub mod theme;
pub mod validate;

pub use animation::Animation;
pub use ast::{Page, Presentation};
pub use chart::*;
pub use elements::*;
pub use layout::*;
pub use parser::load_project;
pub use shared::*;
pub use theme::*;
pub use validate::{Diagnostic, validate_project};

use std::path::PathBuf;

/// A fully loaded PPTD project: the main entry file plus every page it
/// references, in declaration order.
#[derive(Debug, Clone)]
pub struct Project {
    /// Directory that contains the `.pptd` entry file. All paths inside the
    /// project are relative to this directory.
    pub root_dir: PathBuf,
    /// The `.pptd` entry file that was loaded, exactly as it was passed to
    /// [`load_project`]. Diagnostics that read the raw manifest must use this
    /// path instead of guessing a conventional file name.
    pub entry_path: PathBuf,
    /// The parsed main entry file.
    pub presentation: Presentation,
    /// Paths of the page files, parallel to [`Project::pages`].
    pub page_paths: Vec<PathBuf>,
    /// Parsed pages, in the order declared by `presentation.pages`.
    pub pages: Vec<Page>,
}
