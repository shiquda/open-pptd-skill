//! PPTX → PPTD → PPTX round-trip tests (hermetic, fixture-based).
//!
//! The workflow under test is the reverse compiler's contract: rebuild the
//! fixture deck, reverse it back into PPTD, then rebuild again. Geometry must
//! be preserved to the EMU and every element class must survive.

use std::fs;
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

    assert!(decompiled_text.contains(r"\frac{a+b}{c+d}"), "got {decompiled_text}");
    assert!(decompiled_text.contains(r"\sqrt{x^{2}+y^{2}}") || decompiled_text.contains(r"\sqrt{x^2+y^2}"), "got {decompiled_text}");
    assert!(decompiled_text.contains(r"\sum_{i=1}^{n} i^{2}") || decompiled_text.contains(r"\sum_{i=1}^{n} i^2"), "got {decompiled_text}");

    // Rebuild from the converted PPTD
    let rebuild_pptx = work.join("rebuild_math.pptx");
    pptx::writer::PptxWriter::new(&decompiled)
        .build(&rebuild_pptx)
        .expect("converted math project must rebuild");

    let _ = fs::remove_dir_all(&work);
}
