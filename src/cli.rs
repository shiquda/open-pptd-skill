//! Command-line interface for the `pptd` binary.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::Result;
use crate::hash;
use crate::pptd;
use crate::pptx;

#[derive(Debug, Parser)]
#[command(
    name = "pptd",
    version,
    about = "Edit and build PPTX files from the PPTD language",
    long_about = "pptd parses, validates and compiles PPTD \
                  (a YAML-based presentation DSL) into editable PPTX files."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Load a PPTD project and report every validation issue found.
    Check {
        /// Path to the `.pptd` main entry file.
        file: PathBuf,
    },
    /// Dump a summary of the parsed project (debugging aid).
    Dump {
        /// Path to the `.pptd` main entry file.
        file: PathBuf,
    },
    /// Compile a PPTD project into a `.pptx` package.
    ///
    /// Overwrite guard: an existing output file may only be overwritten when
    /// its current hash (computed live) matches the value recorded in the
    /// project's `.sync.hash` sidecar when this project last wrote it;
    /// otherwise the build is refused (exit code 2) instead of destroying
    /// unknown edits. Every successful build refreshes the record.
    Build {
        /// Path to the `.pptd` main entry file.
        file: PathBuf,
        /// Output `.pptx` path; defaults to `<entry basename>.pptx`.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Force build, bypassing .sync.hash overwrite protection.
        #[arg(short, long)]
        force: bool,
    },
    /// Reverse-compile an existing `.pptx` package into a PPTD project.
    ///
    /// The source hash is compared against the `.sync.hash` records in the
    /// output directory: when it matches the last sync point the conversion
    /// is skipped ("unchanged → skipped convert", exit code 0) and the
    /// existing PPTD is left untouched.
    Convert {
        /// Input `.pptx` package.
        file: PathBuf,
        /// Output directory for the PPTD project (created if missing).
        output: PathBuf,
        /// Emit a machine-readable report (counts, loss list, unsupported
        /// constructs) as JSON on stdout.
        #[arg(long)]
        json: bool,
        /// Exit with code 2 when the conversion loses any source information.
        #[arg(long = "fail-on-loss")]
        fail_on_loss: bool,
    },
}

impl Cli {
    /// Execute the parsed command line and return the process exit code.
    pub fn run(self) -> i32 {
        match self.command {
            Command::Check { file } => run_check(&file),
            Command::Dump { file } => run_dump(&file),
            Command::Build { file, output, force } => run_build(&file, output.as_deref(), force),
            Command::Convert {
                file,
                output,
                json,
                fail_on_loss,
            } => run_convert(&file, &output, json, fail_on_loss),
        }
    }
}

/// Human-readable location for a loss: `page 0` is layout/master content.
fn loss_location(page: usize) -> String {
    if page == 0 {
        "layout/master".to_string()
    } else {
        format!("page {page}")
    }
}

fn run_convert(input: &Path, out_dir: &Path, json: bool, fail_on_loss: bool) -> i32 {
    let report = match pptx::import::convert_pptx_to_pptd(input, out_dir) {
        Ok(report) => report,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };

    let loss_total: usize = report.losses.iter().map(|loss| loss.count).sum();
    if json {
        let losses: Vec<_> = report
            .losses
            .iter()
            .map(|loss| {
                serde_json::json!({
                    "category": loss.kind.code(),
                    "label": loss.kind.label(),
                    "detail": loss.kind.detail(),
                    "count": loss.count,
                    "first": {
                        "page": loss.page,
                        "element": loss.element,
                    },
                })
            })
            .collect();
        let skipped: Vec<_> = report
            .skipped
            .iter()
            .map(|skip| {
                serde_json::json!({
                    "page": skip.page,
                    "element": skip.name,
                    "reason": skip.reason,
                    "category": skip.loss.map(|kind| kind.code()),
                })
            })
            .collect();
        let payload = serde_json::json!({
            "input": input.display().to_string(),
            "output_dir": report.output_dir.display().to_string(),
            "skipped_unchanged": report.skipped_unchanged,
            "page_count": report.page_count,
            "element_count": report.element_count,
            "media_count": report.media_count,
            "src_hash": report.src_hash,
            "migrated_sidecars": report.migrated_sidecars,
            "loss_count": loss_total,
            "losses": losses,
            "skipped_count": report.skipped.len(),
            "unsupported_count": report.skipped.len() - report.skipped.iter().filter(|s| s.loss.is_some()).count(),
            "skipped": skipped,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).expect("JSON report")
        );
        if fail_on_loss && !report.skipped_unchanged && loss_total > 0 {
            return 2;
        }
        return 0;
    }

    if report.skipped_unchanged {
        println!(
            "unchanged → skipped convert ({} already in sync with {})",
            out_dir.join("deck.pptd").display(),
            input.display()
        );
        println!("  hint: the existing PPTD reflects this exact .pptx; edit it directly.");
        if report.migrated_sidecars {
            println!("  note: legacy .src.hash/.build.hash records migrated to .sync.hash");
        }
        return 0;
    }
    println!(
        "converted {} → {}",
        input.display(),
        out_dir.join("deck.pptd").display()
    );
    println!(
        "  {} page(s), {} element(s), {} media file(s)",
        report.page_count, report.element_count, report.media_count
    );
    if report.losses.is_empty() {
        println!("  loss report: no lossy mappings");
    } else {
        println!(
            "  loss report: {loss_total} item(s) in {} category(ies)",
            report.losses.len()
        );
        for loss in &report.losses {
            println!(
                "    - {} ({}): {}× (first: {} `{}`)",
                loss.kind.label(),
                loss.kind.code(),
                loss.count,
                loss_location(loss.page),
                loss.element
            );
            println!("      {}", loss.kind.detail());
        }
    }
    // Skips already aggregated into a loss category are printed once above;
    // only unclassified unsupported constructs need their own section.
    let detailed: Vec<_> = report
        .skipped
        .iter()
        .filter(|skip| skip.loss.is_none())
        .collect();
    if detailed.is_empty() {
        println!("  unsupported constructs: none");
    } else {
        let mut by_reason = BTreeMap::<String, usize>::new();
        for skip in &detailed {
            *by_reason.entry(skip.reason.clone()).or_default() += 1;
            eprintln!(
                "  skipped: {} `{}` — {}",
                loss_location(skip.page),
                skip.name,
                skip.reason
            );
        }
        println!("  {} unsupported construct(s):", detailed.len());
        for (reason, count) in &by_reason {
            println!("    - {count}x {reason}");
        }
    }
    if let Some(hash) = &report.src_hash {
        println!("  sync point recorded: {} = {hash}", input.display());
    }
    if report.migrated_sidecars {
        println!("  note: legacy .src.hash/.build.hash records migrated to .sync.hash");
    }
    if fail_on_loss && loss_total > 0 {
        eprintln!(
            "error: conversion lost source information ({loss_total} item(s)); see the loss report"
        );
        return 2;
    }
    0
}

fn run_check(entry: &Path) -> i32 {
    let project = match load(entry) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };

    let issues = pptd::validate::validate_project(&project);
    let (warnings, errors): (Vec<_>, Vec<_>) = issues.iter().partition(|i| i.is_warning());
    if !errors.is_empty() {
        for issue in &issues {
            eprintln!("- {issue}");
        }
        eprintln!("check failed: {} error(s)", errors.len());
        2
    } else if !warnings.is_empty() {
        for warning in &warnings {
            eprintln!("- {warning}");
        }
        println!("check passed with {} warning(s)", warnings.len());
        0
    } else {
        println!(
            "{}: {} page(s) validated — no issues",
            entry.display(),
            project.pages.len()
        );
        0
    }
}

fn run_dump(entry: &Path) -> i32 {
    let project = match load(entry) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };

    let presentation = &project.presentation;
    println!("entry:   {}", entry.display());
    println!("version: {}", presentation.version);
    println!("title:   {:?}", presentation.title);
    println!(
        "size:    {} x {}",
        presentation.size.width, presentation.size.height
    );
    match &presentation.theme {
        Some(theme) => println!(
            "theme:   {} color(s), {} textStyle(s), {} tableStyle(s)",
            theme.colors.len(),
            theme.text_styles.len(),
            theme.table_styles.len()
        ),
        None => println!("theme:   (none)"),
    }
    println!("pages:   {}", project.pages.len());
    for (i, page) in project.pages.iter().enumerate() {
        let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
        for element in &page.elements {
            *counts.entry(element.type_name()).or_default() += 1;
        }
        let summary = counts
            .iter()
            .map(|(kind, count)| format!("{count}x {kind}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {}: {:?} — {}", i + 1, page.page_type, summary);
    }
    0
}

fn run_build(entry: &Path, output: Option<&Path>, force: bool) -> i32 {
    let project = match load(entry) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };

    let issues = pptd::validate::validate_project(&project);
    let (warnings, errors): (Vec<_>, Vec<_>) = issues.iter().partition(|i| i.is_warning());
    for warning in &warnings {
        eprintln!("- {warning}");
    }
    if !errors.is_empty() {
        eprintln!("refusing to build: {} error(s)", errors.len());
        for error in &errors {
            eprintln!("- {error}");
        }
        return 2;
    }

    let output_path = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| entry.with_extension("pptx"));

    // Overwrite guard: an existing output file may only be clobbered with
    // positive evidence that its current bytes are accounted for: its live
    // hash must match the value recorded when this project last legitimately
    // wrote it (convert or build). Anything else — no record, a record for a
    // different file, or a hash mismatch — refuses: overwriting a file the
    // PPTD cannot vouch for could silently destroy external edits.
    if output_path.exists() && !force {
        let verdict = match hash::classify_output(&output_path, &project.root_dir) {
            Ok(verdict) => verdict,
            Err(err) => {
                eprintln!("error: {err}");
                return 1;
            }
        };
        use hash::OutputGuard::{InSync, Stale, Uncovered};
        match verdict {
            InSync => {}
            Stale => {
                eprintln!(
                    "refusing to build: {} changed after its last recorded sync point (external edit?)",
                    output_path.display()
                );
                eprintln!("  building now would permanently overwrite those changes.");
                eprintln!(
                    "  fix: pass --force (or -f) to overwrite anyway:\n    pptd build {} -o {} --force\n  or delete the file:\n    rm {}",
                    entry.display(),
                    output_path.display(),
                    output_path.display()
                );
                return 2;
            }
            Uncovered => {
                eprintln!(
                    "refusing to build: {} already exists but no sync point covers it — its provenance is unknown",
                    output_path.display()
                );
                eprintln!(
                    "  overwriting it could destroy external edits that are not in the PPTD."
                );
                eprintln!(
                    "  fix: pass --force (or -f) to overwrite anyway:\n    pptd build {} -o {} --force\n  or delete the file:\n    rm {}",
                    entry.display(),
                    output_path.display(),
                    output_path.display()
                );
                return 2;
            }
        }
    }

    match pptx::writer::PptxWriter::new(&project).build(&output_path) {
        Ok(()) => {
            println!("wrote {}", output_path.display());
            // Bookkeeping: the freshly written output becomes the recorded
            // sync state for this file. That single write both authorizes
            // the next build over the same path (the A/C-mode iterate loop)
            // and makes a later `convert` of this file skip as unchanged —
            // no flag needed, every legitimate write records itself.
            match hash::sha256_of(&output_path) {
                Ok(h) => {
                    let mut records = hash::Records::load(&project.root_dir);
                    records.set(&output_path, &h);
                    match records.save(&project.root_dir) {
                        Ok(()) => {
                            println!("  sync point recorded: {} = {h}", output_path.display())
                        }
                        Err(err) => eprintln!("warning: could not record sync point: {err}"),
                    }
                }
                Err(err) => eprintln!("warning: could not hash the output for bookkeeping: {err}"),
            }
            0
        }
        Err(err) => {
            eprintln!("error: {err}");
            1
        }
    }
}

fn load(entry: &Path) -> Result<pptd::Project> {
    pptd::load_project(entry)
}
