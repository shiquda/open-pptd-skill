//! Sync-point bookkeeping (`.sync.hash` sidecar).
//!
//! One sidecar in the PPTD work directory records, for every file this
//! project has legitimately written, the SHA-256 the file had at that
//! moment. Format: a `<hash>` line followed by its `<path>` line, repeated:
//!
//! ```text
//! <sha256-hex>
//! <canonical path>
//! ```
//!
//! `convert` records the source it converted; `build` records the output it
//! wrote. Guards then compare live: when a later `build` wants to overwrite
//! an existing file, that file's *current* hash is computed on the spot and
//! checked against the recorded value. A mismatch means an external edit the
//! PPTD cannot vouch for, and the build refuses. Doing this inside the CLI —
//! instead of leaving it to shell snippets in the skill — means the agent
//! physically cannot forget to check.
//!
//! Older builds kept the convert source in `.src.hash` (optionally without a
//! path line) and, briefly, the last build output in `.build.hash`. Those
//! sidecars use the same pair format; they are absorbed into the map on
//! load and removed on the next save.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::Result;

/// Sidecar file names, all inside the work directory.
const SYNC_SIDECAR: &str = ".sync.hash";
const LEGACY_SIDECARS: [&str; 2] = [".src.hash", ".build.hash"];

/// Compute the lowercase hex SHA-256 of a file.
pub fn sha256_of(path: &Path) -> Result<String> {
    let data = fs::read(path).map_err(|source| crate::Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(hex(&Sha256::digest(&data)))
}

fn sidecar_path(work_dir: &Path, name: &str) -> PathBuf {
    work_dir.join(name)
}

/// Canonical form of `path`, falling back to resolving relative to parent
/// when the target file does not yet exist on disk (e.g. before `build`).
pub fn canonical(path: &Path) -> PathBuf {
    let p = if path.as_os_str().is_empty() {
        Path::new(".")
    } else {
        path
    };
    if let Ok(c) = p.canonicalize() {
        return c;
    }
    // If the path does not exist, canonicalize its parent and join the file name.
    if let Some(parent) = p.parent() {
        let parent_canon = canonical(parent);
        if let Some(file_name) = p.file_name() {
            return parent_canon.join(file_name);
        }
    }
    p.to_path_buf()
}

/// True for a 64-character lowercase hex string.
fn is_hash(text: &str) -> bool {
    text.len() == 64 && text.chars().all(|c| c.is_ascii_hexdigit())
}

/// Sync records for a work directory: canonical file path → hash at the
/// last legitimate write of that file by this project.
#[derive(Debug, Default, Clone)]
pub struct Records {
    entries: BTreeMap<PathBuf, String>,
    /// Hash from a legacy path-less `.src.hash`. It names no file, so it can
    /// only be re-bound to the file being converted (see `legacy_hash`).
    legacy_hash: Option<String>,
    /// Whether any legacy sidecar was found on load (removed on `save`).
    legacy_present: bool,
}

impl Records {
    /// Load the work directory's records, absorbing legacy sidecars.
    /// Legacy entries only fill keys the current format does not already
    /// know; the current format is authoritative.
    pub fn load(work_dir: &Path) -> Records {
        let mut records = Records::default();
        if let Some(entries) = parse_pairs(&sidecar_path(work_dir, SYNC_SIDECAR), work_dir) {
            records.entries = entries;
        }
        for name in LEGACY_SIDECARS {
            let Some(sp) = parse_single(&sidecar_path(work_dir, name)) else {
                continue;
            };
            records.legacy_present = true;
            match sp.path {
                Some(path) => {
                    records.entries.entry(canonical(&path)).or_insert(sp.hash);
                }
                None => {
                    if records.legacy_hash.is_none() {
                        records.legacy_hash = Some(sp.hash);
                    }
                }
            }
        }
        records.entries.retain(|path, _| path.exists());
        records
    }

    /// The recorded hash for `path`, if this project ever wrote it.
    pub fn get(&self, path: &Path) -> Option<&String> {
        self.entries.get(&canonical(path))
    }

    /// Record `hash` as the last-known state of `path`.
    pub fn set(&mut self, path: &Path, hash: &str) {
        self.entries.insert(canonical(path), hash.to_string());
    }

    /// Hash from a legacy path-less sidecar, if any.
    pub fn legacy_hash(&self) -> Option<&String> {
        self.legacy_hash.as_ref()
    }

    /// Whether legacy sidecars were found (they are removed on `save`).
    pub fn has_legacy(&self) -> bool {
        self.legacy_present
    }

    /// Write the records and remove absorbed legacy sidecars.
    pub fn save(&self, work_dir: &Path) -> Result<()> {
        let mut text = String::new();
        let canon_work_dir = canonical(work_dir);
        let mut entries = self.entries.clone();
        entries.retain(|path, _| path.exists());
        for (path, hash) in &entries {
            text.push_str(hash);
            text.push('\n');
            let stored = if !canon_work_dir.as_os_str().is_empty() {
                match path.strip_prefix(&canon_work_dir) {
                    Ok(rel) if rel.is_relative() && !rel.to_string_lossy().starts_with('/') => {
                        format!("./{}", rel.display())
                    }
                    _ => path.to_string_lossy().to_string(),
                }
            } else {
                path.to_string_lossy().to_string()
            };
            text.push_str(&stored);
            text.push('\n');
        }
        fs::write(sidecar_path(work_dir, SYNC_SIDECAR), text).map_err(|source| {
            crate::Error::Io {
                path: sidecar_path(work_dir, SYNC_SIDECAR),
                source,
            }
        })?;
        for name in LEGACY_SIDECARS {
            let _ = fs::remove_file(sidecar_path(work_dir, name));
        }
        Ok(())
    }
}

/// Parse the multi-entry pair format. Stops at the first malformed pair,
/// keeping the entries parsed so far.
fn parse_pairs(path: &Path, work_dir: &Path) -> Option<BTreeMap<PathBuf, String>> {
    let text = fs::read_to_string(path).ok()?;
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let mut entries = BTreeMap::new();
    let canon_work_dir = canonical(work_dir);
    while let Some(hash) = lines.next() {
        if !is_hash(hash) {
            break;
        }
        let Some(path_str) = lines.next() else {
            break;
        };
        // Normalize path_str: strip malformed relative prefixes like ".//" from prior bugs.
        let (is_rel, norm_p) = if let Some(stripped) = path_str.strip_prefix(".//") {
            if stripped.starts_with('/') {
                (false, PathBuf::from(stripped))
            } else {
                (true, PathBuf::from(stripped))
            }
        } else if let Some(stripped) = path_str.strip_prefix("./") {
            if stripped.starts_with('/') {
                (false, PathBuf::from(stripped))
            } else {
                (true, PathBuf::from(stripped))
            }
        } else {
            let p = Path::new(path_str);
            (p.is_relative(), p.to_path_buf())
        };

        let resolved = if is_rel {
            canonical(&canon_work_dir.join(norm_p))
        } else {
            canonical(&norm_p)
        };
        entries.insert(resolved, hash.to_string());
    }
    Some(entries)
}

/// Parse a legacy single-entry sidecar (`hash` line, optional `path` line).
fn parse_single(path: &Path) -> Option<SyncPoint> {
    let text = fs::read_to_string(path).ok()?;
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let hash = lines.next()?;
    if !is_hash(hash) {
        return None;
    }
    let path = lines.next().map(PathBuf::from);
    Some(SyncPoint {
        hash: hash.to_string(),
        path,
    })
}

/// A legacy sidecar record: hash plus, when known, the file it refers to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPoint {
    pub hash: String,
    pub path: Option<PathBuf>,
}

/// Verdict for "may this `build` overwrite an existing output file?"
///
/// The overwrite guard requires positive evidence that the bytes about to
/// be clobbered are accounted for: the file's current hash (computed live)
/// must match the value recorded when this project last legitimately wrote
/// it. Anything else refuses, per the skill protocol: **a build must never
/// silently destroy a file it cannot vouch for.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputGuard {
    /// The file's current hash matches its record — safe to overwrite, and
    /// this build refreshes the record on completion.
    InSync,
    /// The file changed after its last recorded write: external edits not
    /// present in the PPTD would be destroyed.
    Stale,
    /// No record covers this file (never written by this project, or the
    /// sidecar is missing). Its provenance is unknown, so overwriting is
    /// refused until `convert` establishes a baseline.
    Uncovered,
}

/// Classify an output file against the work directory's records. Callers
/// should skip the guard entirely when the output does not exist.
pub fn classify_output(output: &Path, work_dir: &Path) -> Result<OutputGuard> {
    match Records::load(work_dir).get(output) {
        Some(recorded) => {
            if *recorded == sha256_of(output)? {
                Ok(OutputGuard::InSync)
            } else {
                Ok(OutputGuard::Stale)
            }
        }
        None => Ok(OutputGuard::Uncovered),
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_roundtrip_and_guard() {
        let dir = std::env::temp_dir().join(format!("sf-hash-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.pptx");
        fs::write(&a, b"hello").unwrap();

        // Nothing recorded yet → uncovered.
        let records = Records::load(&dir);
        assert!(records.get(&a).is_none());
        assert_eq!(classify_output(&a, &dir).unwrap(), OutputGuard::Uncovered);

        // Record a, save, reload → InSync.
        let h = sha256_of(&a).unwrap();
        assert_eq!(h.len(), 64);
        let mut records = Records::load(&dir);
        records.set(&a, &h);
        records.save(&dir).unwrap();
        assert_eq!(classify_output(&a, &dir).unwrap(), OutputGuard::InSync);

        // Content changed → Stale (hash computed live vs recorded).
        fs::write(&a, b"world").unwrap();
        assert_eq!(classify_output(&a, &dir).unwrap(), OutputGuard::Stale);

        // Same content but a different file → still uncovered: records are
        // per file, not per content.
        let b = dir.join("b.pptx");
        fs::write(&b, b"world").unwrap();
        assert_eq!(classify_output(&b, &dir).unwrap(), OutputGuard::Uncovered);

        // Multiple outputs coexist in one map.
        let mut records = Records::load(&dir);
        records.set(&b, &sha256_of(&b).unwrap());
        records.save(&dir).unwrap();
        assert_eq!(classify_output(&a, &dir).unwrap(), OutputGuard::Stale);
        assert_eq!(classify_output(&b, &dir).unwrap(), OutputGuard::InSync);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_sidecars_are_absorbed_and_removed() {
        let dir = std::env::temp_dir().join(format!("sf-hash-legacy-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.pptx");
        fs::write(&a, b"hello").unwrap();
        let h = sha256_of(&a).unwrap();

        // Legacy `.src.hash` with a path line feeds the map on load.
        fs::write(dir.join(".src.hash"), format!("{h}\n{}\n", a.display())).unwrap();
        fs::write(dir.join(".build.hash"), format!("{h}\n{}\n", a.display())).unwrap();
        let records = Records::load(&dir);
        assert!(records.has_legacy());
        assert_eq!(records.get(&a), Some(&h));
        assert_eq!(classify_output(&a, &dir).unwrap(), OutputGuard::InSync);
        records.save(&dir).unwrap();
        assert!(!dir.join(".src.hash").exists());
        assert!(!dir.join(".build.hash").exists());
        assert!(dir.join(".sync.hash").exists());

        // Legacy path-less `.src.hash` exposes its hash for re-binding.
        let dir2 = std::env::temp_dir().join(format!("sf-hash-legacy2-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir2);
        fs::create_dir_all(&dir2).unwrap();
        fs::write(dir2.join(".src.hash"), format!("{h}\n")).unwrap();
        let records = Records::load(&dir2);
        assert_eq!(records.legacy_hash(), Some(&h));
        assert!(records.get(&a).is_none());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&dir2);
    }

    #[test]
    fn path_normalization_and_deduplication() {
        let dir = std::env::temp_dir().join(format!("sf-hash-norm-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Inside project: relative output => stored as ./out.pptx
        let out = dir.join("out.pptx");
        fs::write(&out, b"inside").unwrap();
        let h_out = sha256_of(&out).unwrap();

        // Outside project: absolute file in temp_dir
        let ext = std::env::temp_dir().join(format!("sf-hash-ext-{}.pptx", std::process::id()));
        fs::write(&ext, b"external").unwrap();
        let h_ext = sha256_of(&ext).unwrap();

        let mut records = Records::load(&dir);
        records.set(&out, &h_out);
        records.set(&ext, &h_ext);
        // Setting the same external file twice updates in place
        records.set(&ext, &h_ext);
        records.save(&dir).unwrap();

        let sidecar_content = fs::read_to_string(dir.join(".sync.hash")).unwrap();
        assert!(sidecar_content.contains("./out.pptx"), "must store ./out.pptx");
        assert!(!sidecar_content.contains(".//"), "must not contain .//");

        // External path must appear exactly once in the sidecar
        let ext_canon = canonical(&ext).to_string_lossy().to_string();
        let ext_count = sidecar_content.lines().filter(|l| l.contains(&ext_canon)).count();
        assert_eq!(ext_count, 1, "external path must appear exactly once");

        // Moving the project directory preserves relative sync points
        let dir_moved = std::env::temp_dir().join(format!("sf-hash-moved-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir_moved);
        fs::rename(&dir, &dir_moved).unwrap();

        let out_moved = dir_moved.join("out.pptx");
        assert_eq!(
            classify_output(&out_moved, &dir_moved).unwrap(),
            OutputGuard::InSync,
            "relative output must remain InSync after project directory rename"
        );

        // Modifying the file triggers Stale
        fs::write(&out_moved, b"modified").unwrap();
        assert_eq!(
            classify_output(&out_moved, &dir_moved).unwrap(),
            OutputGuard::Stale,
            "modified file must be Stale"
        );

        let _ = fs::remove_dir_all(&dir_moved);
        let _ = fs::remove_file(&ext);
    }

    #[test]
    fn malformed_dot_slash_slash_is_recovered_and_stale_pruned() {
        let dir = std::env::temp_dir().join(format!("sf-hash-malform-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let real_ext = std::env::temp_dir().join(format!("sf-hash-real-{}.pptx", std::process::id()));
        fs::write(&real_ext, b"content").unwrap();
        let h_real = sha256_of(&real_ext).unwrap();

        // Simulate a sidecar created by the bug: .//<abs_path>
        let malformed_content = format!(
            "{h_real}\n.//{}\n0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n/nonexistent/stale.pptx\n",
            real_ext.display()
        );
        fs::write(dir.join(".sync.hash"), malformed_content).unwrap();

        let records = Records::load(&dir);
        // Stale non-existent path was pruned on load
        assert_eq!(records.entries.len(), 1);
        assert_eq!(records.get(&real_ext), Some(&h_real));
        assert_eq!(classify_output(&real_ext, &dir).unwrap(), OutputGuard::InSync);

        // Resaving cleans up the format completely
        records.save(&dir).unwrap();
        let saved = fs::read_to_string(dir.join(".sync.hash")).unwrap();
        assert!(!saved.contains(".//"));
        assert!(!saved.contains("/nonexistent/stale.pptx"));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(&real_ext);
    }
}
