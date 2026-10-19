//! Integration tests: semantic validation of loaded PPTD projects.

use std::fs;
use std::path::Path;

use open_pptd::pptd;
use open_pptd::pptd::validate::validate_project;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/demo/demo.pptd");

/// Write a minimal deck into a unique temp directory and return the entry path.
fn tmp_deck(tag: &str, deck_yaml: &str, pages: &[(&str, &str)]) -> String {
    let dir = std::env::temp_dir().join(format!("slideforge-test-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("pages")).unwrap();
    fs::write(dir.join("deck.pptd"), deck_yaml).unwrap();
    for (name, body) in pages {
        fs::write(dir.join("pages").join(name), body).unwrap();
    }
    dir.join("deck.pptd").to_string_lossy().into_owned()
}

const MINIMAL_DECK: &str = "version: v2\nsize: [960, 540]\npages:\n  - pages/p.page\n";

#[test]
fn display_only_math_environment_must_own_its_paragraph() {
    let inline_page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [0, 0, 400, 80]
    content:
      text: '<p>text <latex>\begin{align} a &= b \end{align}</latex> more</p>'
"#;
    let entry = tmp_deck("latex-align-inline", MINIMAL_DECK, &[("p.page", inline_page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues.iter().any(|d| d.message.contains("display-only")),
        "inline align must be reported the way the compiler rejects it: {issues:?}"
    );

    // The same environment on a line of its own is what the compiler renders as
    // display math, and `aligned` may be used inside running text.
    let display_page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [0, 0, 400, 80]
    content:
      text: '<p><latex>\begin{align} a &= b \\ c &= d \end{align}</latex></p>'
  - elementId: t2
    elementType: text
    bounds: [0, 100, 400, 80]
    content:
      text: '<p>where \( \begin{aligned} a &= b \\ c &= d \end{aligned} \) holds</p>'
"#;
    let entry = tmp_deck("latex-align-display", MINIMAL_DECK, &[("p.page", display_page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(issues.is_empty(), "display math must stay clean: {issues:?}");
}

#[test]
fn macro_definitions_are_not_reported_as_unsupported() {
    let page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [0, 0, 400, 80]
    content:
      text: '<latex>\newcommand{\R}{\mathbb{R}}\def\norm#1{\left|#1\right|} \norm{x} \in \R^n</latex>'
"#;
    let entry = tmp_deck("latex-macros", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(issues.is_empty(), "user macros must validate cleanly: {issues:?}");

    let bad_page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [0, 0, 400, 80]
    content:
      text: '<latex>\newcommand{\R}{a}\newcommand{\R}{b} \R</latex>'
"#;
    let entry = tmp_deck("latex-macros-bad", MINIMAL_DECK, &[("p.page", bad_page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("already defined") && !d.is_warning()),
        "a broken macro definition must be an error: {issues:?}"
    );
}

#[test]
fn demo_fixture_reports_unsupported_chart() {
    let project = pptd::load_project(Path::new(FIXTURE)).unwrap();
    let issues = validate_project(&project);
    assert_eq!(
        issues.len(),
        1,
        "demo fixture contains 1 unsupported chart: {issues:?}"
    );
    assert!(issues[0].message.contains("chart `trend` is not supported"));
}

#[test]
fn duplicate_element_id_is_reported() {
    let page = r#"
elements:
  - elementId: a
    elementType: text
    bounds: [0, 0, 100, 50]
    content: {text: first}
  - elementId: a
    elementType: text
    bounds: [0, 60, 100, 50]
    content: {text: second}
"#;
    let entry = tmp_deck("dup-id", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("duplicate elementId")),
        "expected a duplicate-elementId finding, got {issues:?}"
    );
}

#[test]
fn non_positive_bounds_are_reported() {
    let page = r#"
elements:
  - elementId: bad
    elementType: text
    bounds: [0, 0, 0, 50]
    content: {text: nope}
"#;
    let entry = tmp_deck("zero-bounds", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("positive width and height")),
        "expected a bounds finding, got {issues:?}"
    );
}

#[test]
fn missing_theme_reference_is_reported() {
    let page = r#"
elements:
  - elementId: t
    elementType: text
    bounds: [0, 0, 100, 50]
    content:
      style: "$nope"
      text: hi
"#;
    let entry = tmp_deck("missing-style", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("theme.textStyles")),
        "expected a style-reference finding, got {issues:?}"
    );
}

#[test]
fn chart_row_length_mismatch_is_reported() {
    let page = r#"
elements:
  - elementId: c
    elementType: chart
    bounds: [0, 0, 400, 300]
    data:
      cols: [a, b]
      rows:
        - [1, 2]
        - [1, 2, 3]
    series:
      - type: bar
        encode: {x: a, y: b}
"#;
    let entry = tmp_deck("chart-rows", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("row 1 has 3 cells, expected 2")),
        "expected a row-length finding, got {issues:?}"
    );
}

#[test]
fn unknown_chart_series_type_is_reported() {
    let page = r#"
elements:
  - elementId: c
    elementType: chart
    bounds: [0, 0, 400, 300]
    data:
      cols: [a, b]
      rows:
        - [1, 2]
    series:
      - type: radar
        encode: {category: a, y: b}
"#;
    let entry = tmp_deck("unknown-series", MINIMAL_DECK, &[("p.page", page)]);
    let result = pptd::load_project(Path::new(&entry));
    let message = result
        .expect_err("unknown series type must fail to parse")
        .to_string();
    assert!(
        message.contains("unknown variant `radar`"),
        "expected the serde unknown-variant finding, got: {message}"
    );
}

#[test]
fn table_ratio_sum_mismatch_is_reported() {
    let page = r#"
elements:
  - elementId: t
    elementType: table
    bounds: [0, 0, 400, 300]
    columnWidths: [0.5, 0.6]
    rowHeights: [1.0]
    rows:
      - - text: a
        - text: b
"#;
    let entry = tmp_deck("table-ratios", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("columnWidths must sum to 1")),
        "expected a ratio-sum finding, got {issues:?}"
    );
}

#[test]
fn animation_must_reference_existing_element() {
    let page = r#"
elements:
  - elementId: t
    elementType: text
    bounds: [0, 0, 100, 50]
    content: {text: hi}
animations:
  - elementId: ghost
    effect: fade-in
"#;
    let entry = tmp_deck("animation-target", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("references unknown elementId")),
        "expected an animation-target finding, got {issues:?}"
    );
}

#[test]
fn undefined_theme_color_is_reported() {
    let page = r#"
elements:
  - elementId: t
    elementType: text
    bounds: [0, 0, 100, 50]
    content:
      text: hi
      color: "$nonexistentColor"
"#;
    let entry = tmp_deck("undef-color", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues.iter().any(|d| d
            .message
            .contains("references undefined color `$nonexistentColor`")),
        "expected undefined color diagnostic, got {issues:?}"
    );
}

#[test]
fn missing_image_file_is_reported() {
    let page = r#"
elements:
  - elementId: img1
    elementType: image
    bounds: [0, 0, 100, 50]
    src: "assets/nonexistent_image.png"
"#;
    let entry = tmp_deck("missing-img", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues.iter().any(|d| d
            .message
            .contains("references missing file `assets/nonexistent_image.png`")),
        "expected missing image file diagnostic, got {issues:?}"
    );
}

#[test]
fn invalid_shape_name_is_reported() {
    let page = r#"
elements:
  - elementId: s1
    elementType: shape
    bounds: [0, 0, 100, 50]
    shapeName: "circle"
"#;
    let entry = tmp_deck("bad-shape", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues.iter().any(|d| d
            .message
            .contains("has unknown shapeName `circle` (did you mean `ellipse`?)")),
        "expected invalid shape diagnostic with suggestion, got {issues:?}"
    );
}

#[test]
fn unsupported_rich_text_tags_are_reported() {
    let page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [0, 0, 100, 50]
    content:
      text: "<p><ul><li>leaked list item</li></ul></p>"
"#;
    let entry = tmp_deck("unsupported-tag", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("contains unsupported tag `<ul>`")),
        "expected unsupported tag diagnostic, got {issues:?}"
    );
}

#[test]
fn unknown_misspelled_fields_are_reported() {
    let page = r##"
elements:
  - elementId: t1
    elementType: text
    bounds: [0, 0, 100, 50]
    colro: "#ff0000"
    content:
      text: "hello"
      font_sizee: 24
"##;
    let entry = tmp_deck("unknown-field", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("element `t1` has unknown field `colro`")),
        "expected unknown field colro diagnostic, got {issues:?}"
    );
    assert!(
        issues.iter().any(|d| d
            .message
            .contains("text `t1` content has unknown field `font_sizee`")),
        "expected unknown field font_sizee diagnostic, got {issues:?}"
    );
}

#[test]
fn potential_text_overflow_is_reported() {
    let page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [0, 0, 100, 20]
    content:
      fontSize: 24
      text: |
        这是一段非常非常长的文字内容，在一个只有20pt高度的极小容器框里面排版，肯定会造成严重的上下文字视觉溢出，系统应当发出明确的文本溢出警告。
"#;
    let entry = tmp_deck("overflow", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    assert!(
        issues
            .iter()
            .any(|d| d.message.contains("potential text overflow in `t1`")),
        "expected text overflow warning, got {issues:?}"
    );
}

#[test]
fn unsupported_latex_command_is_reported() {
    let page = r#"
elements:
  - elementId: t1
    elementType: text
    bounds: [0, 0, 400, 80]
    content:
      text: 'energy <latex>\frac{E}{\foo}</latex> and \(\bar{x}\)'
"#;
    let entry = tmp_deck("latex-unsupported", MINIMAL_DECK, &[("p.page", page)]);
    let project = pptd::load_project(Path::new(&entry)).unwrap();
    let issues = validate_project(&project);
    let latex_issue = issues
        .iter()
        .find(|d| d.message.contains("unsupported LaTeX command"));
    let diag = latex_issue.unwrap_or_else(|| panic!("expected LaTeX error, got {issues:?}"));
    assert!(diag.message.contains("`\\foo`"), "got {}", diag.message);
    assert!(diag.message.contains("t1"), "got {}", diag.message);
    assert!(!diag.is_warning(), "unsupported LaTeX must be an error");
    assert!(
        !issues.iter().any(|d| d.message.contains("`\\bar`")),
        "supported commands must stay clean: {issues:?}"
    );
}
