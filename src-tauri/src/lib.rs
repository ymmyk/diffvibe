use clap::Parser;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

/// CLI arguments for DiffVibe
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(name = "diffvibe")]
#[command(about = "A visual diff and merge tool")]
pub struct CliArgs {
    /// Files to compare or merge (2 for diff, 3 for merge: local base remote)
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,

    /// Output file for merged result (merge mode only)
    #[arg(short, long)]
    pub output: Option<String>,
}

/// Parsed CLI mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum CliMode {
    /// No files specified - show home
    None,
    /// Two files - diff mode
    Diff { left: String, right: String },
    /// Three files - merge mode (local, base, remote)
    Merge {
        local: String,
        base: String,
        remote: String,
        output: Option<String>,
    },
}

// Global storage for CLI args (parsed once at startup)
static CLI_ARGS: OnceLock<CliMode> = OnceLock::new();

/// Parse CLI args and store globally
pub fn parse_cli_args() {
    let args = CliArgs::parse();
    let mode = match args.files.len() {
        0 => CliMode::None,
        2 => CliMode::Diff {
            left: args.files[0].clone(),
            right: args.files[1].clone(),
        },
        3 => CliMode::Merge {
            local: args.files[0].clone(),
            base: args.files[1].clone(),
            remote: args.files[2].clone(),
            output: args.output,
        },
        _ => {
            eprintln!("Usage: diffvibe <left> <right>           # diff mode");
            eprintln!("       diffvibe <local> <base> <remote>  # merge mode");
            std::process::exit(1);
        }
    };
    CLI_ARGS.set(mode).ok();
}

#[tauri::command]
fn get_cli_args() -> CliMode {
    CLI_ARGS.get().cloned().unwrap_or(CliMode::None)
}

#[tauri::command]
fn exit_app(code: i32) {
    std::process::exit(code);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub encoding: String,
    pub size: u64,
    pub line_count: usize,
    pub is_binary: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffLine {
    pub tag: String,       // "equal", "insert", "delete"
    pub old_index: Option<usize>,
    pub new_index: Option<usize>,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
    pub stats: DiffStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub unchanged: usize,
}

// Three-way merge types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkType {
    Equal,      // Same in all three
    LocalOnly,  // Changed only in local
    RemoteOnly, // Changed only in remote
    Conflict,   // Changed in both (differently)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeChunk {
    pub chunk_type: ChunkType,
    pub base_start: usize,
    pub base_count: usize,
    pub local_lines: Vec<String>,
    pub remote_lines: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeResult {
    pub chunks: Vec<MergeChunk>,
    pub conflict_count: usize,
    pub merged_content: String,  // Auto-merged with conflict markers
}

/// Check if content is binary by looking for null bytes in first 8KB
fn is_binary(bytes: &[u8]) -> bool {
    let check_len = bytes.len().min(8192);
    bytes[..check_len].contains(&0)
}

/// Try to decode bytes as UTF-8, fallback to Latin-1
fn decode_content(bytes: &[u8]) -> (String, &'static str) {
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), "utf-8"),
        Err(_) => {
            // Fallback to Latin-1 (ISO-8859-1) - every byte is valid
            let content: String = bytes.iter().map(|&b| b as char).collect();
            (content, "latin-1")
        }
    }
}

/// Encode string to bytes using specified encoding
fn encode_content(content: &str, encoding: &str) -> Vec<u8> {
    match encoding {
        "latin-1" => content.chars().map(|c| c as u8).collect(),
        _ => content.as_bytes().to_vec(), // Default to UTF-8
    }
}

#[tauri::command]
fn read_file(path: &str) -> Result<FileContent, String> {
    let file_path = Path::new(path);

    // Get file metadata for size
    let metadata = fs::metadata(file_path).map_err(|e| e.to_string())?;
    let size = metadata.len();

    // Read raw bytes
    let bytes = fs::read(file_path).map_err(|e| e.to_string())?;

    // Check for binary
    let is_binary = is_binary(&bytes);

    if is_binary {
        return Ok(FileContent {
            path: path.to_string(),
            content: String::new(),
            encoding: "binary".to_string(),
            size,
            line_count: 0,
            is_binary: true,
        });
    }

    // Decode text content
    let (content, encoding) = decode_content(&bytes);
    let line_count = content.lines().count();

    Ok(FileContent {
        path: path.to_string(),
        content,
        encoding: encoding.to_string(),
        size,
        line_count,
        is_binary: false,
    })
}

#[tauri::command]
fn write_file(path: &str, content: &str, encoding: &str) -> Result<(), String> {
    let file_path = Path::new(path);

    // Create .bak backup if file exists
    if file_path.exists() {
        let backup_path = format!("{}.bak", path);
        fs::copy(file_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;
    }

    // Encode content and write
    let bytes = encode_content(content, encoding);
    fs::write(file_path, bytes).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn compute_diff(left: &str, right: &str) -> DiffResult {
    let diff = TextDiff::from_lines(left, right);
    let mut lines = Vec::new();
    let mut additions = 0;
    let mut deletions = 0;
    let mut unchanged = 0;

    for change in diff.iter_all_changes() {
        let (tag, old_index, new_index) = match change.tag() {
            ChangeTag::Equal => {
                unchanged += 1;
                ("equal", change.old_index(), change.new_index())
            }
            ChangeTag::Insert => {
                additions += 1;
                ("insert", None, change.new_index())
            }
            ChangeTag::Delete => {
                deletions += 1;
                ("delete", change.old_index(), None)
            }
        };

        lines.push(DiffLine {
            tag: tag.to_string(),
            old_index,
            new_index,
            value: change.value().to_string(),
        });
    }

    DiffResult {
        lines,
        stats: DiffStats {
            additions,
            deletions,
            unchanged,
        },
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDiffResult {
    pub left: FileContent,
    pub right: FileContent,
    pub diff: DiffResult,
}

#[tauri::command]
fn compute_diff_files(left_path: &str, right_path: &str) -> Result<FileDiffResult, String> {
    let left = read_file(left_path)?;
    let right = read_file(right_path)?;

    // If either is binary, return empty diff
    if left.is_binary || right.is_binary {
        return Ok(FileDiffResult {
            left,
            right,
            diff: DiffResult {
                lines: vec![],
                stats: DiffStats {
                    additions: 0,
                    deletions: 0,
                    unchanged: 0,
                },
            },
        });
    }

    let diff = compute_diff(&left.content, &right.content);

    Ok(FileDiffResult { left, right, diff })
}

/// Three-way merge using diff operations from the similar crate
/// Walks through diff ops for base→local and base→remote in parallel
#[tauri::command]
fn compute_three_way_diff(base: &str, local: &str, remote: &str) -> MergeResult {
    use similar::{ChangeTag, TextDiff};

    // Get changes from base to each branch
    let local_diff = TextDiff::from_lines(base, local);
    let remote_diff = TextDiff::from_lines(base, remote);

    // Collect all changes with their base line indices
    let local_changes: Vec<_> = local_diff.iter_all_changes().collect();
    let remote_changes: Vec<_> = remote_diff.iter_all_changes().collect();

    let chunks: Vec<MergeChunk> = Vec::new();
    let mut conflict_count = 0;
    let mut merged_lines: Vec<String> = Vec::new();

    let base_lines: Vec<&str> = base.lines().collect();

    // Track insertions AFTER each base line index
    // Key = base_idx means "insert these lines after processing base line base_idx"
    let mut local_inserts: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
    let mut remote_inserts: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
    let mut local_deletes: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut remote_deletes: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // Process local changes - track where inserts go
    let mut base_idx = 0usize;
    for change in &local_changes {
        match change.tag() {
            ChangeTag::Equal => {
                base_idx += 1;
            }
            ChangeTag::Delete => {
                local_deletes.insert(base_idx);
                base_idx += 1;
            }
            ChangeTag::Insert => {
                // Insert goes at current base position (before the next base line)
                local_inserts.entry(base_idx).or_default().push(change.value().to_string());
            }
        }
    }

    // Process remote changes
    base_idx = 0;
    for change in &remote_changes {
        match change.tag() {
            ChangeTag::Equal => {
                base_idx += 1;
            }
            ChangeTag::Delete => {
                remote_deletes.insert(base_idx);
                base_idx += 1;
            }
            ChangeTag::Insert => {
                remote_inserts.entry(base_idx).or_default().push(change.value().to_string());
            }
        }
    }

    // Helper to add insertions at a given position
    let add_insertions = |pos: usize, merged: &mut Vec<String>, conflicts: &mut usize| {
        let local_ins = local_inserts.get(&pos);
        let remote_ins = remote_inserts.get(&pos);

        match (local_ins, remote_ins) {
            (Some(l_lines), Some(r_lines)) => {
                if l_lines == r_lines {
                    for line in l_lines {
                        merged.push(line.trim_end().to_string());
                    }
                } else {
                    *conflicts += 1;
                    merged.push("<<<<<<< LOCAL".to_string());
                    for line in l_lines {
                        merged.push(line.trim_end().to_string());
                    }
                    merged.push("=======".to_string());
                    for line in r_lines {
                        merged.push(line.trim_end().to_string());
                    }
                    merged.push(">>>>>>> REMOTE".to_string());
                }
            }
            (Some(l_lines), None) => {
                for line in l_lines {
                    merged.push(line.trim_end().to_string());
                }
            }
            (None, Some(r_lines)) => {
                for line in r_lines {
                    merged.push(line.trim_end().to_string());
                }
            }
            (None, None) => {}
        }
    };

    // Walk through base lines
    for (i, base_line) in base_lines.iter().enumerate() {
        // First, add any insertions BEFORE this base line
        add_insertions(i, &mut merged_lines, &mut conflict_count);

        let local_deleted = local_deletes.contains(&i);
        let remote_deleted = remote_deletes.contains(&i);

        // Handle the base line itself
        match (local_deleted, remote_deleted) {
            (true, true) => {
                // Both deleted - don't include
            }
            (true, false) => {
                // Local deleted, remote kept - take local's deletion (skip line)
            }
            (false, true) => {
                // Remote deleted, local kept - take remote's deletion (skip line)
            }
            (false, false) => {
                // Both kept - include the line
                merged_lines.push(base_line.to_string());
            }
        }
    }

    // Add any insertions after the last base line
    add_insertions(base_lines.len(), &mut merged_lines, &mut conflict_count);

    MergeResult {
        chunks,
        conflict_count,
        merged_content: merged_lines.join("\n"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Parse CLI args before starting Tauri
    parse_cli_args();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![read_file, write_file, compute_diff, compute_diff_files, compute_three_way_diff, get_cli_args, exit_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_way_all_same() {
        let base = "line 1\nline 2";
        let local = "line 1\nline 2";
        let remote = "line 1\nline 2";

        let result = compute_three_way_diff(base, local, remote);

        assert_eq!(result.conflict_count, 0);
        assert_eq!(result.merged_content, "line 1\nline 2");
    }

    #[test]
    fn test_three_way_with_conflict() {
        let base = "original line";
        let local = "local change";
        let remote = "remote change";

        let result = compute_three_way_diff(base, local, remote);

        assert_eq!(result.conflict_count, 1);
        assert!(result.merged_content.contains("<<<<<<< LOCAL"));
        assert!(result.merged_content.contains("======="));
        assert!(result.merged_content.contains(">>>>>>> REMOTE"));
    }

    #[test]
    fn test_three_way_same_change() {
        let base = "original";
        let local = "changed";
        let remote = "changed";

        let result = compute_three_way_diff(base, local, remote);

        assert_eq!(result.conflict_count, 0);
        assert_eq!(result.merged_content, "changed");
    }

    #[test]
    fn test_three_way_local_only() {
        let base = "line 1\nline 2";
        let local = "line 1\nmodified";  // Local changed line 2
        let remote = "line 1\nline 2";   // Remote unchanged

        let result = compute_three_way_diff(base, local, remote);

        assert_eq!(result.conflict_count, 0);
        assert!(result.merged_content.contains("modified"));
        assert!(!result.merged_content.contains("line 2")); // Base line should be replaced
    }

    #[test]
    fn test_three_way_remote_only() {
        let base = "line 1\nline 2";
        let local = "line 1\nline 2";    // Local unchanged
        let remote = "line 1\nchanged";  // Remote changed line 2

        let result = compute_three_way_diff(base, local, remote);

        assert_eq!(result.conflict_count, 0);
        assert!(result.merged_content.contains("changed"));
        assert!(!result.merged_content.contains("line 2")); // Base line should be replaced
    }

    #[test]
    fn test_three_way_realistic() {
        // Simulate a more realistic scenario
        let base = "// Header
function foo() {
    return 1;
}

function bar() {
    return 2;
}";

        // Local adds logging to foo
        let local = "// Header
function foo() {
    console.log('foo called');
    return 1;
}

function bar() {
    return 2;
}";

        // Remote changes bar's return value
        let remote = "// Header
function foo() {
    return 1;
}

function bar() {
    return 42;
}";

        let result = compute_three_way_diff(base, local, remote);

        println!("=== MERGED OUTPUT ===");
        println!("{}", result.merged_content);
        println!("=== CONFLICTS: {} ===", result.conflict_count);

        // Should have both changes merged without conflict
        assert_eq!(result.conflict_count, 0, "Should have no conflicts");
        assert!(result.merged_content.contains("console.log"), "Should have local's logging");
        assert!(result.merged_content.contains("return 42"), "Should have remote's return value");
    }
}
