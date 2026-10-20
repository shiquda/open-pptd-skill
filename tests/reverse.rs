//! PPTX → PPTD → PPTX round-trip tests (hermetic, fixture-based).
//!
//! The workflow under test is the reverse compiler's contract: rebuild the
//! fixture deck, reverse it back into PPTD, then rebuild again. Geometry must
//! be preserved to the EMU and every element class must survive.

use std::fs;
use std::io::Read;
use std::path::PathBuf;

use open_pptd::pptd;
use open_pptd::pptx;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/buildable")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("slideforge-roundtrip-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn buildable_fixture_roundtrips() {
    let work = temp_dir("fixture");
    let first_pptx = work.join("first.pptx");
    let project = pptd::load_project(&fixture_dir().join("buildable.pptd")).unwrap();
    pptx::writer::PptxWriter::new(&project)
        .build(&first_pptx)
        .unwrap();

    let converted = work.join("converted");
    let report = pptx::import::convert_pptx_to_pptd(&first_pptx, &converted).unwrap();
    // The buildable fixture contains only supported constructs.
    assert!(
        report.skipped.is_empty(),
        "unexpected skips: {:?}",
        report.skipped
    );
    assert_eq!(report.page_count, 1, "buildable fixture has one page");

    // Every element index must survive the reverse.
    let project = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    assert_eq!(project.presentation.pages.len(), 1);

    // Rebuild from the converted PPTD; must open and re-parse.
    let second_pptx = work.join("second.pptx");
    let project = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    pptx::writer::PptxWriter::new(&project)
        .build(&second_pptx)
        .unwrap();

    // The regenerated package has the same number of slides and media.
    let first = zip::ZipArchive::new(fs::File::open(&first_pptx).unwrap()).unwrap();
    let second = zip::ZipArchive::new(fs::File::open(&second_pptx).unwrap()).unwrap();
    let slides_of = |zip: &zip::ZipArchive<fs::File>| -> usize {
        zip.file_names()
            .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
            .count()
    };
    assert_eq!(slides_of(&first), slides_of(&second));
    assert_eq!(slides_of(&second), 1);
}

#[test]
fn reverse_keeps_theme_colors_and_size() {
    let work = temp_dir("theme");
    let pptx_path = work.join("t.pptx");
    let project = pptd::load_project(&fixture_dir().join("buildable.pptd")).unwrap();
    pptx::writer::PptxWriter::new(&project)
        .build(&pptx_path)
        .unwrap();

    let converted = work.join("converted");
    pptx::import::convert_pptx_to_pptd(&pptx_path, &converted).unwrap();
    let project = pptd::load_project(&converted.join("deck.pptd")).unwrap();

    let theme = project.presentation.theme.as_ref().expect("theme survives");
    assert!(
        theme.colors.contains_key("primary"),
        "theme colors lost: {:?}",
        theme.colors.keys().collect::<Vec<_>>()
    );
    let size = project.presentation.size;
    assert!(
        (size.width - 960.0).abs() < 1e-6 && (size.height - 540.0).abs() < 1e-6,
        "size drifted: {size:?}"
    );
}

/// Tables survive the reverse direction: `a:tbl` → PPTD `table` with grid
/// ratios, spans, text and resolved cell fills; the converted project must
/// also rebuild.
#[test]
fn table_fixture_roundtrips_through_convert() {
    let work = temp_dir("tables");
    let first_pptx = work.join("first.pptx");
    let project = pptd::load_project(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tables/tables.pptd"),
    )
    .unwrap();
    pptx::writer::PptxWriter::new(&project)
        .build(&first_pptx)
        .unwrap();

    let converted = work.join("converted");
    let report = pptx::import::convert_pptx_to_pptd(&first_pptx, &converted).unwrap();
    assert!(
        report.skipped.is_empty(),
        "tables must not be skipped any more: {:?}",
        report.skipped
    );

    let project = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let page = project
        .pages
        .first()
        .expect("converted project keeps its page");
    let tables: Vec<_> = page
        .elements
        .iter()
        .filter(|e| e.type_name() == "table")
        .collect();
    assert_eq!(tables.len(), 1, "exactly one table expected");
    let pptd::Element::Table(table) = tables[0] else {
        unreachable!()
    };
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.column_widths.len(), 3);
    // The `colSpan=2` origin survives; the covered cell is dropped.
    assert_eq!(table.rows[1].len(), 2);
    assert_eq!(table.rows[1][0].col_span, Some(2));
    let texts: Vec<&str> = table
        .rows
        .iter()
        .flat_map(|r| r.iter())
        .filter_map(|c| c.text.as_deref())
        .collect();
    for needle in ["名称", "合并单元格", "右侧"] {
        assert!(
            texts.iter().any(|t| t.contains(needle)),
            "cell text `{needle}` lost"
        );
    }
    // Header fill was resolved from the theme `$primary` reference.
    assert!(
        table.rows[0][0].fill.as_ref().is_some_and(
            |f| matches!(f, pptd::shared::Fill::Solid { color } if color.0 == "#2F5D50")
        ),
        "header fill must survive as solid #2F5D50"
    );

    // And the converted project builds again.
    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&project)
        .build(&second)
        .expect("converted project must rebuild");
    let _ = fs::remove_dir_all(&work);
}
#[test]
fn reverse_preserves_speaker_notes() {
    let work = temp_dir("notes");
    let pptx_path = work.join("with_notes.pptx");
    let mut project = pptd::load_project(&fixture_dir().join("buildable.pptd")).unwrap();
    project.pages[0].notes = Some("Here are the speaker rehearsal notes for slide 1.".to_string());

    pptx::writer::PptxWriter::new(&project)
        .build(&pptx_path)
        .unwrap();

    let converted = work.join("converted_notes");
    let report = pptx::import::convert_pptx_to_pptd(&pptx_path, &converted).unwrap();
    assert_eq!(report.page_count, 1);

    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    assert_eq!(
        decompiled.pages[0].notes.as_deref(),
        Some("Here are the speaker rehearsal notes for slide 1.")
    );

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn math_formulas_roundtrip_through_convert() {
    let work = temp_dir("math");
    let pptx_path = work.join("with_math.pptx");
    let mut project = pptd::load_project(&fixture_dir().join("buildable.pptd")).unwrap();

    if let Some(pptd::Element::Text(t)) = project.pages[0].elements.first_mut() {
        t.content.text = "<p><latex>\\frac{a+b}{c+d}</latex></p><p><latex>\\sqrt{x^2+y^2}</latex></p><p><latex>\\sum_{i=1}^{n} i^2</latex></p>".to_string();
    }

    pptx::writer::PptxWriter::new(&project)
        .build(&pptx_path)
        .unwrap();

    let converted = work.join("converted_math");
    let report = pptx::import::convert_pptx_to_pptd(&pptx_path, &converted).unwrap();
    assert_eq!(report.page_count, 1);

    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let decompiled_text = match &decompiled.pages[0].elements[0] {
        pptd::Element::Text(t) => &t.content.text,
        _ => panic!("expected text element"),
    };

    assert!(
        decompiled_text.contains(r"\frac{a+b}{c+d}"),
        "got {decompiled_text}"
    );
    assert!(
        decompiled_text.contains(r"\sqrt{x^{2}+y^{2}}")
            || decompiled_text.contains(r"\sqrt{x^2+y^2}"),
        "got {decompiled_text}"
    );
    assert!(
        decompiled_text.contains(r"\sum_{i=1}^{n} i^{2}")
            || decompiled_text.contains(r"\sum_{i=1}^{n} i^2"),
        "got {decompiled_text}"
    );

    // Rebuild from the converted PPTD
    let rebuild_pptx = work.join("rebuild_math.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&rebuild_pptx)
        .expect("converted math project must rebuild");

    let _ = fs::remove_dir_all(&work);
}

fn read_part(path: &std::path::Path, name: &str) -> String {
    let mut zip = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    let mut data = String::new();
    zip.by_name(name)
        .unwrap()
        .read_to_string(&mut data)
        .unwrap();
    data
}

fn read_part_bytes(path: &std::path::Path, name: &str) -> Vec<u8> {
    let mut zip = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    let mut data = Vec::new();
    zip.by_name(name)
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    data
}

#[test]
fn richtext_hyperlinks_roundtrip_through_convert_and_build() {
    let work = temp_dir("hyperlink");
    let dir = work.join("src");
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(
        dir.join("deck.pptd"),
        r#"
version: v2
title: Link Roundtrip
size: [960, 540]
pages: [pages/p.page]
"#,
    )
    .unwrap();
    fs::write(
        dir.join("pages/p.page"),
        r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [50, 50, 500, 120]
    content:
      text: |
        <p>Read <a href="https://arxiv.org/abs/2605.27223">arXiv:2605.27223</a>.</p>
"#,
    )
    .unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let first = work.join("first.pptx");
    pptx::writer::PptxWriter::new(&project)
        .build(&first)
        .unwrap();
    assert_eq!(
        read_part(&first, "ppt/slides/slide1.xml")
            .matches("<a:hlinkClick")
            .count(),
        1
    );

    let converted = work.join("converted");
    pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let text = match &decompiled.pages[0].elements[0] {
        pptd::Element::Text(t) => &t.content.text,
        other => panic!("expected text element, got {other:?}"),
    };
    assert!(
        text.contains("<a href=\"https://arxiv.org/abs/2605.27223\">"),
        "hyperlink markup must be restored: {text}"
    );

    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&second)
        .unwrap();
    let slide = read_part(&second, "ppt/slides/slide1.xml");
    assert_eq!(slide.matches("<a:hlinkClick").count(), 1, "{slide}");
    let rels = read_part(&second, "ppt/slides/_rels/slide1.xml.rels");
    assert!(rels.contains("TargetMode=\"External\""), "{rels}");
    assert!(
        rels.contains("Target=\"https://arxiv.org/abs/2605.27223\""),
        "{rels}"
    );
}

#[test]
fn element_link_roundtrips_through_convert() {
    let work = temp_dir("element-link");
    let dir = work.join("src");
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(
        dir.join("deck.pptd"),
        "version: v2\ntitle: Link\nsize: [960, 540]\npages: [pages/p.page]\n",
    )
    .unwrap();
    fs::write(
        dir.join("pages/p.page"),
        "elements:\n  - elementId: card\n    elementType: shape\n    shapeName: roundRect\n    bounds: [50, 50, 300, 120]\n    link: \"https://example.com/source\"\n    fill: {type: solid, color: \"#EEF2FF\"}\n",
    )
    .unwrap();
    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let first = work.join("first.pptx");
    pptx::writer::PptxWriter::new(&project)
        .build(&first)
        .unwrap();
    let converted = work.join("converted");
    pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    assert_eq!(
        decompiled.pages[0].elements[0].common().link.as_deref(),
        Some("https://example.com/source")
    );
}

/// Native charts survive `build → convert → build`: the chart XML and the
/// embedded workbook are byte-identical after the round trip, the slide and
/// chart relationships are regenerated, and a second convert still yields
/// exactly one chart element (no loss, no duplication).
#[test]
fn chart_parts_roundtrip_through_convert() {
    let work = temp_dir("chart");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/chart/chart.pptd");
    let fixture_dir = fixture.parent().unwrap();
    let chart_bytes = fs::read(fixture_dir.join("media/chart1.xml")).unwrap();
    let workbook_bytes = fs::read(fixture_dir.join("media/Microsoft_Excel_Sheet1.xlsx")).unwrap();

    let first = work.join("first.pptx");
    let project = pptd::load_project(&fixture).unwrap();
    pptx::writer::PptxWriter::new(&project)
        .build(&first)
        .unwrap();

    assert_eq!(
        read_part_bytes(&first, "ppt/charts/chart1.xml"),
        chart_bytes,
        "chart XML must be emitted verbatim"
    );
    assert_eq!(
        read_part_bytes(&first, "ppt/embeddings/Microsoft_Excel_Sheet1.xlsx"),
        workbook_bytes,
        "embedded workbook must be emitted verbatim"
    );
    let slide = read_part(&first, "ppt/slides/slide1.xml");
    assert!(slide.contains("<c:chart"), "{slide}");
    assert!(slide.contains("r:id=\"rId2\""), "{slide}");
    let slide_rels = read_part(&first, "ppt/slides/_rels/slide1.xml.rels");
    assert!(
        slide_rels.contains("relationships/chart")
            && slide_rels.contains("../charts/chart1.xml"),
        "{slide_rels}"
    );
    let chart_rels = read_part(&first, "ppt/charts/_rels/chart1.xml.rels");
    assert!(
        chart_rels.contains("relationships/package")
            && chart_rels.contains("../embeddings/Microsoft_Excel_Sheet1.xlsx"),
        "{chart_rels}"
    );
    let content_types = read_part(&first, "[Content_Types].xml");
    assert!(
        content_types.contains("drawingml.chart+xml"),
        "{content_types}"
    );
    assert!(
        content_types.contains("spreadsheetml.sheet"),
        "{content_types}"
    );

    // Reverse: the chart element keeps both part paths and the plot hint.
    let converted = work.join("converted");
    let report = pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    assert!(report.skipped.is_empty(), "skips: {:?}", report.skipped);
    assert_eq!(report.element_count, 1);
    for part in [
        "media/chart1.xml",
        "media/chart1.xml.rels",
        "media/Microsoft_Excel_Sheet1.xlsx",
    ] {
        assert!(converted.join(part).is_file(), "missing {part}");
    }
    assert_eq!(
        fs::read(converted.join("media/chart1.xml")).unwrap(),
        chart_bytes,
        "convert must preserve the chart bytes"
    );

    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let charts: Vec<_> = decompiled.pages[0]
        .elements
        .iter()
        .filter_map(|e| match e {
            pptd::Element::Chart(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(charts.len(), 1, "exactly one chart expected");
    assert_eq!(charts[0].chart_part.as_deref(), Some("media/chart1.xml"));
    assert_eq!(
        charts[0].workbook_part.as_deref(),
        Some("media/Microsoft_Excel_Sheet1.xlsx")
    );
    assert_eq!(charts[0].chart_kind.as_deref(), Some("bar"));
    assert_eq!(charts[0].common.bounds.x, 60.0);
    assert_eq!(charts[0].common.bounds.width, 840.0);

    // Forward again: identical chart payloads, valid rels, no duplicates.
    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&second)
        .unwrap();
    assert_eq!(
        read_part_bytes(&second, "ppt/charts/chart1.xml"),
        chart_bytes
    );
    assert_eq!(
        read_part_bytes(&second, "ppt/embeddings/Microsoft_Excel_Sheet1.xlsx"),
        workbook_bytes
    );

    let converted_again = work.join("converted-again");
    let report = pptx::import::convert_pptx_to_pptd(&second, &converted_again).unwrap();
    assert!(report.skipped.is_empty(), "skips: {:?}", report.skipped);
    let again = pptd::load_project(&converted_again.join("deck.pptd")).unwrap();
    assert_eq!(
        again.pages[0]
            .elements
            .iter()
            .filter(|e| e.type_name() == "chart")
            .count(),
        1,
        "chart must neither vanish nor duplicate"
    );

    let _ = fs::remove_dir_all(&work);
}

/// Cell insets (`a:tcPr@marL/marR/marT/marB`) and per-cell autofit must
/// survive build → convert → build. Before this, the writer always emitted
/// PowerPoint's default insets and dropped the cell's `a:bodyPr` autofit
/// child, which reflowed every converted table.
#[test]
fn cell_margins_and_autofit_roundtrip() {
    let work = temp_dir("cell-margin-autofit");
    let dir = work.join("src");
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(
        dir.join("deck.pptd"),
        "version: v2\nsize: [960, 540]\npages: [pages/p.page]\n",
    )
    .unwrap();
    fs::write(
        dir.join("pages/p.page"),
        r#"
elements:
  - elementId: t1
    elementType: table
    bounds: [100, 100, 600, 300]
    columnWidths: [0.5, 0.5]
    rowHeights: [0.5, 0.5]
    rows:
      - - text: Padded
          margin: {left: 4.3, right: 3.6, top: 2.5, bottom: 2.2}
          autofit: fit_shape
        - text: Shrunk
          autofit: fit_text
      - - text: Default insets
        - text: ""
          autofit: fit_shape
"#,
    )
    .unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let first = work.join("first.pptx");
    pptx::writer::PptxWriter::new(&project)
        .build(&first)
        .unwrap();
    let first_xml = read_part(&first, "ppt/slides/slide1.xml");
    for attr in [
        r#"marL="54610""#,
        r#"marR="45720""#,
        r#"marT="31750""#,
        r#"marB="27940""#,
    ] {
        assert!(
            first_xml.contains(attr),
            "cell margin {attr} missing from built table: {first_xml}"
        );
    }
    assert_eq!(
        first_xml.matches("<a:spAutoFit/>").count(),
        2,
        "both fit_shape cells must keep spAutoFit"
    );
    assert_eq!(
        first_xml.matches("<a:normAutofit/>").count(),
        1,
        "the fit_text cell must keep normAutofit"
    );

    // Import direction: the converted AST carries both attributes.
    let converted = work.join("converted");
    pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let table = decompiled.pages[0]
        .elements
        .iter()
        .find_map(|e| match e {
            pptd::Element::Table(t) => Some(t),
            _ => None,
        })
        .expect("converted table");
    assert_eq!(
        table.rows[0][0].margin,
        Some(pptd::CellMargin {
            left: Some(4.3),
            right: Some(3.6),
            top: Some(2.5),
            bottom: Some(2.2),
        })
    );
    assert_eq!(table.rows[0][0].autofit, Some(pptd::TextAutofit::FitShape));
    assert_eq!(table.rows[0][1].autofit, Some(pptd::TextAutofit::FitText));
    assert_eq!(table.rows[1][1].autofit, Some(pptd::TextAutofit::FitShape));
    assert_eq!(
        table.rows[1][0].margin, None,
        "cells without explicit insets must not invent a margin"
    );

    // Render direction again: the rebuilt deck emits the same values.
    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&second)
        .unwrap();
    let second_xml = read_part(&second, "ppt/slides/slide1.xml");
    assert!(second_xml.contains(r#"marL="54610""#), "{second_xml}");
    assert!(second_xml.contains(r#"marR="45720""#), "{second_xml}");
    assert!(second_xml.contains(r#"marT="31750""#), "{second_xml}");
    assert!(second_xml.contains(r#"marB="27940""#), "{second_xml}");
    assert_eq!(second_xml.matches("<a:spAutoFit/>").count(), 2);

    let _ = fs::remove_dir_all(&work);
}

/// GIF pictures survive `build → convert → build`: the bytes are copied
/// verbatim into `ppt/media/imageN.gif`, declared as `image/gif`, and
/// re-imported into the project `media/` directory instead of being skipped.
#[test]
fn gif_images_roundtrip_through_convert() {
    let work = temp_dir("gif");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gif/gif.pptd");
    let fixture_dir = fixture.parent().unwrap();
    let gif_bytes = fs::read(fixture_dir.join("media/anim.gif")).unwrap();

    let first = work.join("first.pptx");
    let project = pptd::load_project(&fixture).unwrap();
    pptx::writer::PptxWriter::new(&project)
        .build(&first)
        .unwrap();

    assert_eq!(
        read_part_bytes(&first, "ppt/media/image1.gif"),
        gif_bytes,
        "GIF body must be emitted verbatim (no re-encoding)"
    );
    let content_types = read_part(&first, "[Content_Types].xml");
    assert!(
        content_types.contains("Extension=\"gif\"")
            && content_types.contains("ContentType=\"image/gif\""),
        "{content_types}"
    );
    let slide_rels = read_part(&first, "ppt/slides/_rels/slide1.xml.rels");
    assert!(
        slide_rels.contains("relationships/image") && slide_rels.contains("../media/image1.gif"),
        "{slide_rels}"
    );

    // Reverse: the GIF is copied into the project, not reported as unsupported.
    let converted = work.join("converted");
    let report = pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    assert!(report.skipped.is_empty(), "skips: {:?}", report.skipped);
    assert_eq!(
        fs::read(converted.join("media/image1.gif")).unwrap(),
        gif_bytes,
        "convert must copy the GIF bytes unchanged"
    );
    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let images: Vec<_> = decompiled.pages[0]
        .elements
        .iter()
        .filter_map(|e| match e {
            pptd::Element::Image(image) => Some(image),
            _ => None,
        })
        .collect();
    assert_eq!(images.len(), 1, "the GIF picture must survive the reverse");
    assert_eq!(images[0].src, "media/image1.gif");

    // Forward again: same bytes, same part name.
    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&second)
        .unwrap();
    assert_eq!(read_part_bytes(&second, "ppt/media/image1.gif"), gif_bytes);

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn richtext_baseline_roundtrips_exactly() {
    let work = temp_dir("baseline");
    let dir = work.join("src");
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(
        dir.join("deck.pptd"),
        "version: v2\ntitle: Baseline Roundtrip\nsize: [960, 540]\npages: [pages/p.page]\n",
    )
    .unwrap();
    // The shape of the GWTC-5 source deck: eleven scripted runs, six raised
    // 30% and five lowered 25%, plus one non-canonical shift that must keep
    // its exact percentage.
    let mut text = String::from("<p>");
    for i in 1..=6 {
        text.push_str(&format!("M<sup>{i}</sup>"));
    }
    for i in 1..=5 {
        text.push_str(&format!("f<sub>{i}</sub>"));
    }
    text.push_str("<span style=\"baseline:20000\">x</span></p>");
    let page = format!(
        "elements:\n  - elementId: t1\n    elementType: text\n    bounds: [40, 40, 800, 120]\n    content:\n      fontSize: 18\n      text: |\n        {text}\n"
    );
    fs::write(dir.join("pages/p.page"), page).unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let first = work.join("first.pptx");
    pptx::writer::PptxWriter::new(&project).build(&first).unwrap();
    let first_xml = read_part(&first, "ppt/slides/slide1.xml");
    assert_eq!(
        first_xml.matches("baseline=\"30000\"").count(),
        6,
        "{first_xml}"
    );
    assert_eq!(
        first_xml.matches("baseline=\"-25000\"").count(),
        5,
        "{first_xml}"
    );
    assert_eq!(
        first_xml.matches("baseline=\"20000\"").count(),
        1,
        "a non-canonical shift must keep its exact value: {first_xml}"
    );

    let converted = work.join("converted");
    pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let decompiled_text = match &decompiled.pages[0].elements[0] {
        pptd::Element::Text(t) => t.content.text.clone(),
        _ => panic!("expected text element"),
    };
    assert_eq!(
        decompiled_text.matches("<sup>").count(),
        6,
        "raised runs lost on import: {decompiled_text}"
    );
    assert_eq!(
        decompiled_text.matches("<sub>").count(),
        5,
        "lowered runs lost on import: {decompiled_text}"
    );
    assert!(
        decompiled_text.contains("baseline:20000"),
        "non-canonical shift must be carried verbatim: {decompiled_text}"
    );

    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&second)
        .unwrap();
    let second_xml = read_part(&second, "ppt/slides/slide1.xml");
    assert_eq!(
        second_xml.matches("baseline=\"30000\"").count(),
        6,
        "{second_xml}"
    );
    assert_eq!(
        second_xml.matches("baseline=\"-25000\"").count(),
        5,
        "{second_xml}"
    );
    assert_eq!(
        second_xml.matches("baseline=\"20000\"").count(),
        1,
        "{second_xml}"
    );

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn whitespace_only_runs_roundtrip_through_convert_and_build() {
    let work = temp_dir("whitespace-runs");
    let dir = work.join("src");
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(
        dir.join("deck.pptd"),
        "version: v2\ntitle: Whitespace Roundtrip\nsize: [960, 540]\npages: [pages/p.page]\n",
    )
    .unwrap();
    fs::write(
        dir.join("pages/p.page"),
        r#"elements:
  - elementId: t1
    elementType: text
    bounds: [40, 40, 800, 120]
    content:
      fontSize: 18
      text: |
        <p>Intro<span style="font-weight:400"> </span><a href="https://arxiv.org/abs/2605.27223">arXiv:2605.27223</a><span style="font-weight:400"> · </span>Methods</p>
"#,
    )
    .unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let first = work.join("first.pptx");
    pptx::writer::PptxWriter::new(&project).build(&first).unwrap();
    let first_xml = read_part(&first, "ppt/slides/slide1.xml");
    let intro_at = first_xml.find("<a:t>Intro</a:t>").expect(&first_xml);
    let spacer_at = first_xml
        .find("<a:t xml:space=\"preserve\"> </a:t>")
        .expect(&first_xml);
    let link_at = first_xml
        .find("<a:t>arXiv:2605.27223</a:t>")
        .expect(&first_xml);
    assert!(
        intro_at < spacer_at && spacer_at < link_at,
        "spacer run must stay between Intro and arXiv: {first_xml}"
    );
    assert!(
        first_xml.contains("<a:t xml:space=\"preserve\"> · </a:t>"),
        "edge spaces need xml:space=preserve: {first_xml}"
    );

    let converted = work.join("converted");
    pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let text = match &decompiled.pages[0].elements[0] {
        pptd::Element::Text(t) => t.content.text.clone(),
        other => panic!("expected text element, got {other:?}"),
    };
    assert!(
        text.contains("Intro <a href=\"https://arxiv.org/abs/2605.27223\">"),
        "converted rich text lost the spacer run: {text}"
    );
    assert!(
        text.contains(" · Methods"),
        "converted rich text trimmed the run edges: {text}"
    );

    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&second)
        .unwrap();
    let second_xml = read_part(&second, "ppt/slides/slide1.xml");
    assert!(
        second_xml.contains("<a:t xml:space=\"preserve\">Intro </a:t>"),
        "rebuilt run must keep its trailing space: {second_xml}"
    );
    assert!(
        second_xml.contains("<a:t xml:space=\"preserve\"> · Methods</a:t>"),
        "rebuilt run must keep both edge spaces: {second_xml}"
    );

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn space_after_roundtrips_in_boxes_and_cells() {
    let work = temp_dir("space-after");
    let dir = work.join("src");
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(
        dir.join("deck.pptd"),
        "version: v2\ntitle: Space After\nsize: [960, 540]\npages: [pages/p.page]\n",
    )
    .unwrap();
    fs::write(
        dir.join("pages/p.page"),
        r#"elements:
  - elementId: uniform
    elementType: text
    bounds: [40, 40, 400, 120]
    content:
      spaceAfter: 7
      text: |
        <p>one</p>
        <p>two</p>
  - elementId: mixed
    elementType: text
    bounds: [40, 200, 400, 120]
    content:
      text: |
        <p style="margin-bottom:7px">spaced</p>
        <p>tight</p>
  - elementId: tbl
    elementType: table
    bounds: [40, 340, 400, 120]
    columnWidths: [1]
    rowHeights: [1]
    rows:
      - - text: |-
            <p style="margin-bottom:7px">cell a</p><p>cell b</p>
"#,
    )
    .unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let first = work.join("first.pptx");
    pptx::writer::PptxWriter::new(&project).build(&first).unwrap();
    let first_xml = read_part(&first, "ppt/slides/slide1.xml");
    // uniform: 2 paragraphs, mixed: 1, cell: 1.
    assert_eq!(
        first_xml.matches("<a:spcAft>").count(),
        4,
        "space-after must reach every paragraph that declares it: {first_xml}"
    );
    assert!(
        first_xml.contains(r#"<a:spcPts val="700"/>"#),
        "7pt must be written as 700 hundredths: {first_xml}"
    );

    let converted = work.join("converted");
    pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let uniform = decompiled.pages[0]
        .elements
        .iter()
        .find_map(|e| match e {
            pptd::Element::Text(t) if t.common.element_id == "uniform" => Some(t),
            _ => None,
        })
        .expect("uniform text element");
    assert_eq!(
        uniform.content.space_after,
        Some(7.0),
        "a uniform space-after must hoist to `spaceAfter` in pt"
    );
    assert!(
        !uniform.content.text.contains("margin-bottom"),
        "hoisted space-after must not be duplicated per paragraph: {}",
        uniform.content.text
    );
    let mixed = decompiled.pages[0]
        .elements
        .iter()
        .find_map(|e| match e {
            pptd::Element::Text(t) if t.common.element_id == "mixed" => Some(t),
            _ => None,
        })
        .expect("mixed text element");
    assert_eq!(mixed.content.space_after, None);
    assert!(
        mixed.content.text.contains("margin-bottom:7px"),
        "mixed space-after must stay paragraph-level: {}",
        mixed.content.text
    );
    let table = decompiled.pages[0]
        .elements
        .iter()
        .find_map(|e| match e {
            pptd::Element::Table(t) => Some(t),
            _ => None,
        })
        .expect("converted table");
    let cell_text = table.rows[0][0].text.as_deref().unwrap_or_default();
    assert!(
        cell_text.contains("margin-bottom:7px"),
        "table cells have no `spaceAfter` field, so theirs must stay per paragraph: {cell_text}"
    );

    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&second)
        .unwrap();
    let second_xml = read_part(&second, "ppt/slides/slide1.xml");
    assert_eq!(
        second_xml.matches("<a:spcAft>").count(),
        4,
        "{second_xml}"
    );
    assert!(second_xml.contains(r#"<a:spcPts val="700"/>"#), "{second_xml}");

    let _ = fs::remove_dir_all(&work);
}

#[test]
fn math_fallback_and_rich_baseline_stay_separate() {
    let work = temp_dir("baseline-math");
    let dir = work.join("src");
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(
        dir.join("deck.pptd"),
        "version: v2\ntitle: Baseline Math\nsize: [960, 540]\npages: [pages/p.page]\n",
    )
    .unwrap();
    fs::write(
        dir.join("pages/p.page"),
        r#"elements:
  - elementId: t1
    elementType: text
    bounds: [40, 40, 800, 120]
    content:
      fontSize: 18
      text: |
        <p><latex>x^2</latex> + a<sup>2</sup> + <latex>y_1</latex></p>
"#,
    )
    .unwrap();

    let project = pptd::load_project(&dir.join("deck.pptd")).unwrap();
    let first = work.join("first.pptx");
    pptx::writer::PptxWriter::new(&project).build(&first).unwrap();
    let first_xml = read_part(&first, "ppt/slides/slide1.xml");
    // One raised fallback segment for `x^2` plus the explicit `<sup>` run.
    assert_eq!(
        first_xml.matches("baseline=\"30000\"").count(),
        2,
        "{first_xml}"
    );
    let fallback = first_xml
        .split("<mc:Fallback>")
        .nth(1)
        .expect("fallback segment");
    assert!(
        fallback.contains("baseline=\"30000\""),
        "the formula fallback keeps its own shift: {fallback}"
    );

    let converted = work.join("converted");
    pptx::import::convert_pptx_to_pptd(&first, &converted).unwrap();
    let decompiled = pptd::load_project(&converted.join("deck.pptd")).unwrap();
    let text = match &decompiled.pages[0].elements[0] {
        pptd::Element::Text(t) => t.content.text.clone(),
        _ => panic!("expected text element"),
    };
    assert_eq!(
        text.matches("<latex>").count(),
        2,
        "both formulas must survive: {text}"
    );
    assert_eq!(
        text.matches("<sup>").count(),
        1,
        "exactly one rich-text superscript: {text}"
    );
    for body in text.split("<latex>").skip(1) {
        let body = body.split("</latex>").next().unwrap_or("");
        assert!(
            !body.contains("sup") && !body.contains("baseline"),
            "rich-text tags must not leak into a LaTeX body: {body}"
        );
    }

    let second = work.join("second.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&second)
        .unwrap();
    let second_xml = read_part(&second, "ppt/slides/slide1.xml");
    assert_eq!(
        second_xml.matches("baseline=\"30000\"").count(),
        2,
        "round-trip must not duplicate or drop either shift: {second_xml}"
    );

    let _ = fs::remove_dir_all(&work);
}
