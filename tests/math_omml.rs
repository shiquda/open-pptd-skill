//! Integration tests for native OOXML OMML math compilation.

use std::fs;
use std::path::PathBuf;

use open_pptd::pptd;
use open_pptd::pptx;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/buildable")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("slideforge-math-omml-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn deck_with_math_emits_omml_and_no_raw_latex() {
    let work = temp_dir("deck");
    let pptx_path = work.join("math_deck.pptx");
    let mut project = pptd::load_project(&fixture_dir().join("buildable.pptd")).unwrap();

    let math_text = [
        "<p><latex>\\frac{a+b}{c+d}</latex></p>",
        "<p><latex>\\frac{S \\times 8}{T}</latex></p>",
        "<p><latex>\\sqrt{x^2+y^2}</latex></p>",
        "<p><latex>\\sum_{i=1}^{n} i^2</latex></p>",
        "<p>Sentence with <latex>x^2+y^2</latex> inline.</p>",
        "<p><latex>\\text{a&lt;b&amp;c}</latex></p>",
    ]
    .join("");
    if let Some(pptd::Element::Text(t)) = project.pages[0].elements.first_mut() {
        t.common.bounds.y = 10.0;
        t.common.bounds.height = 450.0;
        t.content.font_size = Some(14.0);
        t.content.text = math_text;
    }

    pptx::writer::PptxWriter::new(&project)
        .build(&pptx_path)
        .expect("build with math formulas must succeed");

    // Read slide1.xml from the generated pptx
    let file = fs::File::open(&pptx_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut slide_xml = String::new();
    {
        use std::io::Read;
        let mut entry = archive.by_name("ppt/slides/slide1.xml").unwrap();
        entry.read_to_string(&mut slide_xml).unwrap();
    }

    // Must declare a14 and m namespaces on p:sld
    assert!(
        slide_xml.contains("xmlns:a14=\"http://schemas.microsoft.com/office/drawing/2010/main\""),
        "slide1.xml must declare a14 namespace"
    );
    assert!(
        slide_xml.contains("xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\""),
        "slide1.xml must declare m namespace"
    );

    // Must contain native OMML structures
    assert!(slide_xml.contains("<a14:m>"), "must contain <a14:m>");
    assert!(slide_xml.contains("<m:f>"), "must contain <m:f>");
    assert!(slide_xml.contains("<m:rad>"), "must contain <m:rad>");
    assert!(slide_xml.contains("<m:nary>"), "must contain <m:nary>");
    assert!(slide_xml.contains("<m:sSup>"), "must contain <m:sSup>");

    // Must contain XML-escaped text for \text{a<b&c}
    assert!(slide_xml.contains("a&lt;b&amp;c"), "must escape special characters in <m:t>");
    assert!(!slide_xml.contains("<m:t>a<b"), "raw unescaped '<' must not exist in XML");

    // Must not leak literal LaTeX commands as text
    assert!(!slide_xml.contains("\\frac"), "literal '\\frac' must not leak into slide");
    assert!(!slide_xml.contains("\\times"), "literal '\\times' must not leak into slide");
    assert!(!slide_xml.contains("\\sqrt"), "literal '\\sqrt' must not leak into slide");
    assert!(!slide_xml.contains("\\sum"), "literal '\\sum' must not leak into slide");

    // Must be well-formed XML: verify xmltree parses it cleanly
    xmltree::Element::parse(slide_xml.as_bytes()).expect("slide1.xml must be valid XML");

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn unsupported_command_fails_build() {
    let work = temp_dir("unsupported");
    let pptx_path = work.join("bad.pptx");
    let mut project = pptd::load_project(&fixture_dir().join("buildable.pptd")).unwrap();

    if let Some(pptd::Element::Text(t)) = project.pages[0].elements.first_mut() {
        t.content.text = "<p><latex>\\foo{x}</latex></p>".to_string();
    }

    let result = pptx::writer::PptxWriter::new(&project).build(&pptx_path);
    assert!(result.is_err(), "build with unsupported command must fail");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("\\foo") || err_msg.contains("foo"),
        "error message must name the unsupported command: {err_msg}"
    );

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn letter_spacing_and_text_direction_emitted() {
    let work = temp_dir("styling");
    let pptx_path = work.join("styling.pptx");
    let mut project = pptd::load_project(&fixture_dir().join("buildable.pptd")).unwrap();

    if let Some(pptd::Element::Text(t)) = project.pages[0].elements.first_mut() {
        t.content.letter_spacing = Some(2.0);
        t.content.text_direction = Some(pptd::TextDirection::Vertical);
    }

    pptx::writer::PptxWriter::new(&project)
        .build(&pptx_path)
        .expect("build must succeed");

    let file = fs::File::open(&pptx_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut slide_xml = String::new();
    {
        use std::io::Read;
        let mut entry = archive.by_name("ppt/slides/slide1.xml").unwrap();
        entry.read_to_string(&mut slide_xml).unwrap();
    }

    // vert="eaVert" on a:bodyPr
    assert!(slide_xml.contains("vert=\"eaVert\""), "{slide_xml}");
    // spc="150" on a:rPr (2.0 * 75)
    assert!(slide_xml.contains("spc=\"150\""), "{slide_xml}");

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn inline_math_emits_alternate_content_with_fallback() {
    let work = temp_dir("alt_content");
    let pptx_path = work.join("alt_content.pptx");
    let mut project = pptd::load_project(&fixture_dir().join("buildable.pptd")).unwrap();

    let mixed_text = "<p>Formula <latex>x^2+y^2</latex> inline</p>";
    if let Some(pptd::Element::Text(t)) = project.pages[0].elements.first_mut() {
        t.content.text = mixed_text.to_string();
    }

    pptx::writer::PptxWriter::new(&project)
        .build(&pptx_path)
        .expect("build with inline math must succeed");

    let file = fs::File::open(&pptx_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut slide_xml = String::new();
    {
        use std::io::Read;
        let mut entry = archive.by_name("ppt/slides/slide1.xml").unwrap();
        entry.read_to_string(&mut slide_xml).unwrap();
    }

    // Inline formula must be wrapped in mc:AlternateContent
    assert!(
        slide_xml.contains("<mc:AlternateContent"),
        "inline math must use mc:AlternateContent"
    );
    assert!(
        slide_xml.contains("<mc:Choice Requires=\"a14\">"),
        "Choice block must require a14"
    );
    assert!(
        slide_xml.contains("<mc:Fallback>"),
        "Fallback block must be present"
    );
    // Choice branch contains native OMML
    assert!(
        slide_xml.contains("<a14:m><m:oMath>"),
        "Choice branch must contain native OMML"
    );
    // Fallback branch contains superscript baseline shift
    assert!(
        slide_xml.contains("baseline=\"30000\""),
        "Fallback branch must contain superscript baseline shift"
    );

    let _ = fs::remove_dir_all(&work);
}
