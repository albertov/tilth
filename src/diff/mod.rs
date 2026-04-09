pub mod format;
pub mod matching;
pub mod overlay;
pub mod parse;

use std::path::PathBuf;

use crate::types::OutlineKind;

#[derive(Debug)]
pub enum DiffSource {
    GitUncommitted,
    GitStaged,
    GitRef(String),
    Files(PathBuf, PathBuf),
    Patch(PathBuf),
    Log(String),
}

#[derive(Debug)]
pub struct FileDiff {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub status: FileStatus,
    pub hunks: Vec<Hunk>,
    pub is_generated: bool,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug)]
pub struct Hunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug)]
pub struct DiffSymbol {
    pub entry: crate::types::OutlineEntry,
    pub identity: SymbolIdentity,
    pub content_hash: u64,
    pub structural_hash: u64,
    pub source_text: String,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SymbolIdentity {
    pub kind: OutlineKind,
    pub parent_path: String,
    pub name: String,
}

#[derive(Debug)]
pub struct SymbolChange {
    pub name: String,
    pub kind: OutlineKind,
    pub change: ChangeType,
    pub match_confidence: MatchConfidence,
    pub line: u32,
    pub old_sig: Option<String>,
    pub new_sig: Option<String>,
    pub size_delta: Option<(u32, u32)>,
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    Added,
    Deleted,
    BodyChanged,
    SignatureChanged,
    Renamed { old_name: String },
    Moved { old_path: PathBuf },
    RenamedAndMoved { old_name: String, old_path: PathBuf },
    Unchanged,
}

#[derive(Debug, Clone)]
pub enum MatchConfidence {
    Exact,
    Structural,
    Fuzzy(f32),
    Ambiguous(u32),
}

#[derive(Debug)]
pub struct FileOverlay {
    pub path: PathBuf,
    pub symbol_changes: Vec<SymbolChange>,
    pub attributed_hunks: Vec<(String, Vec<DiffLine>)>,
    pub conflicts: Vec<Conflict>,
    pub new_content: Option<String>,
}

#[derive(Debug)]
pub struct Conflict {
    pub line: u32,
    pub ours: String,
    pub theirs: String,
    pub enclosing_fn: Option<String>,
}

#[derive(Debug)]
pub struct CommitSummary {
    pub hash: String,
    pub timestamp: i64,
    pub message: String,
    pub author: String,
    pub overlays: Vec<FileOverlay>,
}

/// Resolve the diff source from CLI/MCP parameters.
///
/// Priority: patch > log > a+b > source > default (uncommitted).
/// Returns an error if only one of `a` or `b` is provided.
pub fn resolve_source(
    source: Option<&str>,
    a: Option<&str>,
    b: Option<&str>,
    patch: Option<&str>,
    log: Option<&str>,
) -> Result<DiffSource, String> {
    if let Some(p) = patch {
        return Ok(DiffSource::Patch(PathBuf::from(p)));
    }
    if let Some(l) = log {
        return Ok(DiffSource::Log(l.to_string()));
    }
    match (a, b) {
        (Some(fa), Some(fb)) => return Ok(DiffSource::Files(PathBuf::from(fa), PathBuf::from(fb))),
        (Some(_), None) | (None, Some(_)) => {
            return Err("both --a and --b must be provided together".to_string());
        }
        (None, None) => {}
    }
    if let Some(s) = source {
        let ds = match s {
            "staged" => DiffSource::GitStaged,
            "uncommitted" | "working" => DiffSource::GitUncommitted,
            r => DiffSource::GitRef(r.to_string()),
        };
        return Ok(ds);
    }
    Ok(DiffSource::GitUncommitted)
}

/// Execute a git diff command and return raw unified diff output.
fn run_git_diff(source: &DiffSource) -> Result<String, String> {
    use std::process::Command;

    match source {
        DiffSource::Log(_) => {
            return Err("log mode should not call run_git_diff directly".to_string());
        }
        DiffSource::Patch(path) => {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("failed to read patch file: {e}"))?;
            return Ok(content);
        }
        _ => {}
    }

    let mut cmd = Command::new("git");
    cmd.arg("diff");

    match source {
        DiffSource::GitUncommitted => {
            // working tree vs HEAD (unstaged + staged)
            cmd.arg("HEAD");
        }
        DiffSource::GitStaged => {
            cmd.arg("--staged");
        }
        DiffSource::GitRef(r) => {
            cmd.arg(r);
        }
        DiffSource::Files(fa, fb) => {
            cmd.arg("--no-index").arg("--").arg(fa).arg(fb);
        }
        // Patch and Log are handled above
        DiffSource::Patch(_) | DiffSource::Log(_) => unreachable!(),
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run git diff: {e}"))?;

    // git diff --no-index exits 1 when there are differences; that is normal.
    // For all other variants, a non-zero exit is unexpected but we still return
    // whatever stdout was produced so the caller can decide.
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Orchestrator stub — connects parse → overlay → format pipeline.
/// Full implementation in task #198.
pub fn diff(
    source: &DiffSource,
    _scope: Option<&str>,
    _budget: Option<u64>,
) -> Result<String, String> {
    let raw = run_git_diff(source)?;
    if raw.is_empty() {
        return Ok("No changes.".to_string());
    }
    let file_diffs = parse::parse_unified_diff(&raw);
    Ok(format!("# Diff: {} files changed\n(pipeline pending)", file_diffs.len()))
}
