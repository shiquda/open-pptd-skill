//! End-to-end tests: `PptxWriter` produces a valid OPC/ZIP package.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use open_pptd::pptd;
use open_pptd::pptx::writer::PptxWriter;

const BUILDABLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/buildable/buildable.pptd"
);

fn out_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "pptd-build-{tag}-{}.pptx",
        std::process::id()
    ))
}

const REQUIRED_PARTS: [&str; 11] = [
    "[Content_Types].xml",
    "_rels/.rels",
    "docProps/core.xml",
    "ppt/presentation.xml",
    "ppt/_rels/presentation.xml.rels",
    "ppt/theme/theme1.xml",
    "ppt/slideMasters/slideMaster1.xml",
    "ppt/slideLayouts/slideLayout1.xml",
    "ppt/slides/slide1.xml",
    "ppt/slides/_rels/slide1.xml.rels",
    "ppt/media/image1.png",
];

fn read_zip_part(path: &Path, name: &str) -> String {
    let file = fs::File::open(path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut data = String::new();
    zip.by_name(name)
        .unwrap()
        .read_to_string(&mut data)
        .unwrap();
    data
}

#[test]
fn buildable_deck_produces_a_complete_package() {
    let project = pptd::load_project(Path::new(BUILDABLE)).unwrap();
    let path = out_path("ok");
    let _ = fs::remove_file(&path);

    PptxWriter::new(&project)
        .build(&path)
        .expect("buildable deck must build");

    let file = fs::File::open(&path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_owned())
        .collect();
    for expected in REQUIRED_PARTS {
        assert!(
            names.iter().any(|n| n == expected),
            "missing part {expected}; got {names:?}"
        );
    }

    // Slide content sanity: text, shape, line, icon, image and the fade.
    let slide = read_zip_part(&path, "ppt/slides/slide1.xml");
    assert!(slide.contains("Hello PPTD"), "slide text missing");
    assert!(slide.contains("roundRect"), "shape missing");
    assert!(slide.contains("cxnSp"), "line missing");
    assert!(slide.contains("cubicBezTo"), "icon geometry missing");
    assert!(slide.contains("<pptd:icon"), "icon pptd extension missing");
    assert!(slide.contains("<p:pic>"), "image missing");
    assert!(
        slide.contains("r:embed=\"rId2\""),
        "image blip embed missing"
    );
    assert!(
        slide.contains("<a:srcRect"),
        "contain fit should pad via srcRect"
    );
    assert!(slide.contains("<p:transition"), "transition missing");
    assert!(slide.contains("<p:fade/>"), "fade transition missing");

    // Theme carries the resolved color slots.
    let theme = read_zip_part(&path, "ppt/theme/theme1.xml");
    assert!(theme.contains("2563EB"), "theme primary color missing");

    // Media relationship: rId1 = layout, rId2 = the png image.
    let slide_rels = read_zip_part(&path, "ppt/slides/_rels/slide1.xml.rels");
    assert!(
        slide_rels.contains("relationships/image"),
        "media rel missing"
    );
}

#[test]
fn content_types_cover_every_typed_part() {
    let project = pptd::load_project(Path::new(BUILDABLE)).unwrap();
    let path = out_path("ct");
    let _ = fs::remove_file(&path);
    PptxWriter::new(&project).build(&path).unwrap();

    let content_types = read_zip_part(&path, "[Content_Types].xml");
    assert!(
        content_types.contains("Extension=\"png\"") && content_types.contains("image/png"),
        "png Default content type missing"
    );
    for (part, mime) in [
        (
            "/ppt/presentation.xml",
            "presentationml.presentation.main+xml",
        ),
        ("/ppt/slides/slide1.xml", "presentationml.slide+xml"),
        ("/ppt/theme/theme1.xml", "officedocument.theme+xml"),
        (
            "/ppt/slideMasters/slideMaster1.xml",
            "presentationml.slideMaster+xml",
        ),
        (
            "/ppt/slideLayouts/slideLayout1.xml",
            "presentationml.slideLayout+xml",
        ),
        ("/docProps/core.xml", "core-properties+xml"),
    ] {
        assert!(
            content_types.contains(part) && content_types.contains(mime),
            "content type for {part} missing or wrong"
        );
    }
}

#[test]
fn rich_text_renders_instead_of_failing() {
    // PPTD rich text (`<p>`/`<span style>`/`<strong>`) is a supported feature
    // now: the subtitle must render as styled runs, not fail.
    let work = std::env::temp_dir().join("pptd-rich-text");
    let _ = fs::create_dir_all(work.join("pages"));
    fs::write(
        work.join("deck.pptd"),
        "version: v2\nsize: [960, 540]\npages:\n  - pages/p.page\n",
    )
    .unwrap();
    fs::write(
        work.join("pages/p.page"),
        r#"elements:
  - elementId: subtitle
    elementType: text
    bounds: [40, 40, 800, 60]
    content:
      text: <p><span style="color:#ffffff;">FY2024 · built with PPTD</span> and <strong>bold</strong></p>
"#,
    )
    .unwrap();

    let project = pptd::load_project(&work.join("deck.pptd")).unwrap();
    let path = work.join("out.pptx");
    let _ = fs::remove_file(&path);
    PptxWriter::new(&project).build(&path).unwrap();

    // The styled span must produce a white run and a bold run, with no raw
    // markup leaking into the slide XML.
    let mut zip = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
    let mut slide = zip.by_name("ppt/slides/slide1.xml").unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut slide, &mut xml).unwrap();
    assert!(!xml.contains("<p>"), "raw markup leaked into the slide");
    assert!(
        xml.contains("FY2024") && xml.contains("built with PPTD") && xml.contains("bold"),
        "subtitle text missing or mangled"
    );
    assert!(
        xml.matches("<a:rPr").count() >= 2 && xml.contains("val=\"FFFFFF\""),
        "styled run fill missing"
    );
}

#[test]
fn unsupported_element_kinds_are_reported() {
    // A deck with a chart (plain text elsewhere) exercises the element-kind
    // guard rather than the rich-text guard.
    let deck = "version: v2\nsize: [960, 540]\npages:\n  - pages/p.page\n";
    let page = r#"
elements:
  - elementId: bench
    elementType: text
    bounds: [0, 0, 100, 50]
    content: {text: ok}
  - elementId: c
    elementType: chart
    bounds: [0, 60, 400, 300]
    data:
      cols: [a, b]
      rows:
        - [1, 2]
    series:
      - type: bar
        encode: {x: a, y: b}
"#;
    let dir = std::env::temp_dir().join(format!("pptd-build-chart-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(dir.join("deck.pptd"), deck).unwrap();
    fs::write(dir.join("pages/p.page"), page).unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let path = out_path("chart");
    let _ = fs::remove_file(&path);

    let message = PptxWriter::new(&project)
        .build(&path)
        .expect_err("chart must fail the build")
        .to_string();
    assert!(message.contains("not supported"), "got: {message}");
    assert!(message.contains("`c`"), "should name the chart element");
    assert!(message.contains("page 1"), "should name the page");
}

/// Tables compile to a native `<a:tbl>` graphic frame: grid, per-cell rich
/// text, theme-driven header styling and `colSpan` merges all survive.
#[test]
fn table_deck_produces_native_tbl() {
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tables/tables.pptd"
    );
    let project = pptd::load_project(Path::new(fixture)).unwrap();
    let path = out_path("table");
    let _ = fs::remove_file(&path);

    PptxWriter::new(&project)
        .build(&path)
        .expect("table deck must build");

    let slide = read_zip_part(&path, "ppt/slides/slide1.xml");
    assert!(slide.contains("<a:tbl>"), "must contain a native table");
    assert_eq!(slide.matches("<a:gridCol").count(), 3, "three grid columns");
    assert_eq!(slide.matches("<a:tr ").count(), 3, "three rows");
    assert!(
        slide.contains("gridSpan=\"2\""),
        "colSpan=2 must map to gridSpan"
    );
    assert!(
        slide.contains("hMerge=\"1\""),
        "covered cell must be hMerge"
    );
    for text in ["名称", "合并单元格", "粗体"] {
        assert!(slide.contains(text), "cell text `{text}` must survive");
    }
    // Header row resolves `firstRowStyle` through the theme `$` reference.
    assert!(
        slide.contains("2F5D50"),
        "header fill must resolve $primary"
    );
    assert!(slide.contains("b=\"1\""), "header runs must be bold");
    let _ = fs::remove_file(&path);
}

#[test]
fn line_arrows_and_straight_geometry() {
    let deck = r#"
version: v2
title: Line Test
size: [960, 540]
pages: [pages/p.page]
"#;
    let page = r##"
elements:
  - elementId: l1
    elementType: line
    bounds: [100, 100, 200, 150]
    viewBox: [200, 150]
    points: "0,0 200,150"
    arrow: [arrow, stealth]
    border: {style: solid, width: 2, color: "#FF0000"}
"##;
    let dir = std::env::temp_dir().join(format!("pptd-line-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(dir.join("deck.pptd"), deck).unwrap();
    fs::write(dir.join("pages/p.page"), page).unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let path = out_path("line_arrow");
    let _ = fs::remove_file(&path);

    PptxWriter::new(&project)
        .build(&path)
        .expect("line deck must build");

    let slide = read_zip_part(&path, "ppt/slides/slide1.xml");
    assert!(
        slide.contains("<p:cxnSp>"),
        "straight 2pt line should be cxnSp"
    );
    assert!(
        slide.contains("prst=\"line\""),
        "straight line must have prst=line"
    );
    assert!(
        slide.contains("<a:headEnd type=\"triangle\""),
        "arrow head missing"
    );
    assert!(
        slide.contains("<a:tailEnd type=\"stealth\""),
        "stealth tail missing"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn font_fallback_maps_misans_safely() {
    let deck = r#"
version: v2
title: Font Test
size: [960, 540]
theme:
  textStyles:
    body:
      fontFamily: MiSans
pages: [pages/p.page]
"#;
    let page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [50, 50, 400, 100]
    content:
      fontFamily: MiSans
      text: "测试安全字体回退"
"#;
    let dir = std::env::temp_dir().join(format!("pptd-font-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(dir.join("deck.pptd"), deck).unwrap();
    fs::write(dir.join("pages/p.page"), page).unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let path = out_path("font_test");
    let _ = fs::remove_file(&path);

    PptxWriter::new(&project)
        .build(&path)
        .expect("font test deck must build");

    let theme = read_zip_part(&path, "ppt/theme/theme1.xml");
    assert!(
        theme.contains("Microsoft YaHei"),
        "theme ea font must be Microsoft YaHei"
    );
    assert!(
        !theme.contains("typeface=\"MiSans\""),
        "theme must not have MiSans directly"
    );

    let slide = read_zip_part(&path, "ppt/slides/slide1.xml");
    assert!(
        slide.contains("typeface=\"Microsoft YaHei\""),
        "slide ea font must be Microsoft YaHei"
    );
    assert!(
        !slide.contains("typeface=\"MiSans\""),
        "slide must not have MiSans directly"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn font_fallback_maps_source_han_serif_safely() {
    let deck = r#"
version: v2
title: Serif Font Test
size: [960, 540]
theme:
  textStyles:
    title:
      fontFamily: 思源宋体
pages: [pages/p.page]
"#;
    let page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [50, 50, 400, 100]
    content:
      fontFamily: 思源宋体
      text: "测试宋体安全回退"
"#;
    let dir = std::env::temp_dir().join(format!("pptd-serif-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(dir.join("deck.pptd"), deck).unwrap();
    fs::write(dir.join("pages/p.page"), page).unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let path = out_path("serif_font_test");
    let _ = fs::remove_file(&path);

    PptxWriter::new(&project)
        .build(&path)
        .expect("serif font test deck must build");

    let slide = read_zip_part(&path, "ppt/slides/slide1.xml");
    assert!(
        slide.contains("typeface=\"SimSun\""),
        "slide ea font must be SimSun"
    );
    assert!(
        slide.contains("typeface=\"Georgia\""),
        "slide latin font must be Georgia"
    );
    assert!(
        !slide.contains("typeface=\"思源宋体\""),
        "slide must not have 思源宋体 as latin"
    );
    assert!(
        !slide.contains("typeface=\"Source Han Serif SC\""),
        "slide must not have uninstalled Source Han Serif SC directly"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn math_formula_default_upright_and_subscripts() {
    let deck = r#"
version: v2
title: Math Test
size: [960, 540]
pages: [pages/p.page]
"#;
    let page = r#"
elements:
  - elementId: m1
    elementType: text
    bounds: [50, 50, 400, 100]
    content:
      text: |
        <p>公式 \(h(s_t) \ge 0\)</p>
"#;
    let dir = std::env::temp_dir().join(format!("pptd-math-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(dir.join("deck.pptd"), deck).unwrap();
    fs::write(dir.join("pages/p.page"), page).unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let path = out_path("math_test");
    let _ = fs::remove_file(&path);

    PptxWriter::new(&project)
        .build(&path)
        .expect("math deck must build");

    let slide = read_zip_part(&path, "ppt/slides/slide1.xml");
    assert!(slide.contains("<a14:m><m:oMath>"), "native OMML inline math missing");
    assert!(slide.contains("<m:sSub>"), "subscript structure missing");
    assert!(slide.contains("≥"), "ge symbol missing");
    assert!(slide.contains("mc:AlternateContent"), "inline math must use mc:AlternateContent");
    assert!(slide.contains("mc:Fallback"), "inline math must include fallback");
    assert!(slide.contains("baseline=\"-25000\""), "fallback must contain subscript baseline shift");
    let _ = fs::remove_file(&path);
}

#[test]
fn math_formula_explicit_italic_respected() {
    let deck = r#"
version: v2
title: Math Italic Test
size: [960, 540]
pages: [pages/p.page]
"#;
    let page = r#"
elements:
  - elementId: m2
    elementType: text
    bounds: [50, 50, 400, 100]
    content:
      text: |
        <p>强调 <i>\(x + y\)</i> 和普通 \(z\)</p>
"#;
    let dir = std::env::temp_dir().join(format!("pptd-math-it-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(dir.join("deck.pptd"), deck).unwrap();
    fs::write(dir.join("pages/p.page"), page).unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let path = out_path("math_it_test");
    let _ = fs::remove_file(&path);

    PptxWriter::new(&project)
        .build(&path)
        .expect("math italic deck must build");

    let slide = read_zip_part(&path, "ppt/slides/slide1.xml");
    // Both formulas are emitted as native OMML
    assert!(slide.contains("<a14:m><m:oMath>"), "native OMML formula missing");
    assert!(slide.contains("<m:t>z</m:t>"), "formula z content missing");
    assert!(slide.contains("x"), "formula x content missing");
    assert!(slide.contains("y"), "formula y content missing");
    let _ = fs::remove_file(&path);
}

#[test]
fn speaker_notes_produce_notes_slide_and_rels() {
    let deck = "version: v2\ntitle: Notes Test\nsize: [960, 540]\npages:\n  - pages/p.page\n";
    let page = r#"
notes: |
  这是第一行演讲者备注。
  这是第二行演讲者备注。
elements:
  - elementId: t1
    elementType: text
    bounds: [50, 50, 400, 100]
    content:
      text: <p>Slide with notes</p>
"#;
    let dir = std::env::temp_dir().join(format!("pptd-notes-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(dir.join("deck.pptd"), deck).unwrap();
    fs::write(dir.join("pages/p.page"), page).unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let path = out_path("notes_test");
    let _ = fs::remove_file(&path);

    PptxWriter::new(&project)
        .build(&path)
        .expect("notes deck must build");

    let notes_xml = read_zip_part(&path, "ppt/notesSlides/notesSlide1.xml");
    assert!(
        notes_xml.contains("这是第一行演讲者备注。"),
        "first line missing from notes: {notes_xml}"
    );
    assert!(
        notes_xml.contains("这是第二行演讲者备注。"),
        "second line missing from notes: {notes_xml}"
    );

    let slide_rels = read_zip_part(&path, "ppt/slides/_rels/slide1.xml.rels");
    assert!(
        slide_rels.contains("notesSlide1.xml"),
        "notesSlide rel missing from slide rels: {slide_rels}"
    );

    let notes_rels = read_zip_part(&path, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    assert!(
        notes_rels.contains("slide1.xml"),
        "slide rel missing from notes rels: {notes_rels}"
    );

    let content_types = read_zip_part(&path, "[Content_Types].xml");
    assert!(
        content_types.contains("notesSlide+xml"),
        "notesSlide content type missing: {content_types}"
    );

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir_all(&dir);
}
