//! A tiny indenting XML writer for the OOXML parts.
//!
//! Kept deliberately small: OOXML parts are regular, so a ~80-line builder
//! with escaping is easier to audit than a full XML library.

/// Escape text/attribute content per XML 1.0.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// A minimal XML writer producing one document (with prolog) in memory.
#[derive(Debug, Default)]
pub struct Xml {
    buf: String,
    indent: usize,
    tag_stack: Vec<String>,
}

impl Xml {
    /// Create a writer that already carries the XML prolog.
    pub fn new() -> Self {
        Self {
            buf: String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n"),
            indent: 0,
            tag_stack: Vec::new(),
        }
    }

    /// Open an element with attributes (may be empty).
    pub fn start(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.push_open(name, attrs);
        self.indent += 1;
        self.buf.push('\n');
        self.tag_stack.push(name.to_string());
    }

    /// Close an element opened by [`Xml::start`].
    pub fn end(&mut self, name: &str) {
        let expected = self.tag_stack.pop().unwrap_or_else(|| {
            panic!("Xml::end(\"{name}\") called on empty tag stack");
        });
        assert_eq!(
            expected, name,
            "XML tag mismatch: attempted to close <{expected}> with </{name}>"
        );
        self.indent = self.indent.saturating_sub(1);
        self.write_indent();
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push_str(">\n");
    }
    /// A self-closing element.
    pub fn leaf(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.write_indent();
        self.buf.push('<');
        self.buf.push_str(name);
        for (key, value) in attrs {
            self.buf.push(' ');
            self.buf.push_str(key);
            self.buf.push_str("=\"");
            self.buf.push_str(&escape(value));
            self.buf.push('"');
        }
        self.buf.push_str("/>\n");
    }

    /// Escaped text content (no line break or indentation around it).
    pub fn text(&mut self, text: &str) {
        self.buf.push_str(&escape(text));
    }
    /// Append a caller-supplied, already well-formed XML fragment verbatim.
    ///
    /// The caller is responsible for escaping text content. Kept narrow:
    /// intended exclusively for pre-built XML fragments (such as native OMML
    /// math zones produced by `latex_to_omml`); no other caller should use
    /// it for user-controlled strings.
    pub fn raw(&mut self, raw: &str) {
        self.buf.push_str(raw);
    }


    /// `name` containing the escaped `text` as its only content, with no
    /// line breaks or indentation: in DrawingML the `<a:t>` content is
    /// significant whitespace, so a formatting newline would render as an
    /// empty first line and shift the whole text block (visible with
    /// `anchor="ctr"`).
    pub fn text_elem(&mut self, name: &str, text: &str) {
        // DrawingML treats leading/trailing spaces (and whitespace-only runs)
        // as significant, but consumers are allowed to trim them unless
        // `xml:space="preserve"` is present.
        let attrs: &[(&str, &str)] = if text.starts_with(char::is_whitespace)
            || text.ends_with(char::is_whitespace)
        {
            &[("xml:space", "preserve")]
        } else {
            &[]
        };
        self.inline_text(name, attrs, text);
    }

    /// `name` (with `attrs`) containing the escaped `text` as its only
    /// content and no surrounding formatting whitespace (same rule as
    /// [`Xml::text_elem`]).
    pub fn inline_text(&mut self, name: &str, attrs: &[(&str, &str)], text: &str) {
        self.write_indent();
        self.buf.push('<');
        self.buf.push_str(name);
        for (key, value) in attrs {
            self.buf.push(' ');
            self.buf.push_str(key);
            self.buf.push_str("=\"");
            self.buf.push_str(&escape(value));
            self.buf.push('"');
        }
        self.buf.push('>');
        self.buf.push_str(&escape(text));
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push_str(">\n");
    }

    /// Emit the document and return the final string.
    pub fn into_string(self) -> String {
        assert!(
            self.tag_stack.is_empty(),
            "unclosed XML tags remaining when finishing document: {:?}",
            self.tag_stack
        );
        self.buf
    }
    fn push_open(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.write_indent();
        self.buf.push('<');
        self.buf.push_str(name);
        for (key, value) in attrs {
            self.buf.push(' ');
            self.buf.push_str(key);
            self.buf.push_str("=\"");
            self.buf.push_str(&escape(value));
            self.buf.push('"');
        }
        self.buf.push('>');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str("  ");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_paired_tags_succeed() {
        let mut xml = Xml::new();
        xml.start("root", &[]);
        xml.start("child", &[("attr", "val")]);
        xml.leaf("leaf", &[]);
        xml.end("child");
        xml.end("root");
        let s = xml.into_string();
        assert!(s.contains("<root>"));
        assert!(s.contains("</root>"));
    }

    #[test]
    #[should_panic(expected = "XML tag mismatch: attempted to close <p:sp> with </p:grpSp>")]
    fn mismatched_tags_panic() {
        let mut xml = Xml::new();
        xml.start("p:sp", &[]);
        xml.end("p:grpSp");
    }

    #[test]
    #[should_panic(expected = "Xml::end(\"p:sp\") called on empty tag stack")]
    fn empty_stack_end_panics() {
        let mut xml = Xml::new();
        xml.end("p:sp");
    }

    #[test]
    #[should_panic(expected = "unclosed XML tags remaining when finishing document: [\"p:sld\", \"p:cSld\"]")]
    fn unclosed_tags_panic_on_into_string() {
        let mut xml = Xml::new();
        xml.start("p:sld", &[]);
        xml.start("p:cSld", &[]);
        let _ = xml.into_string();
    }
}
