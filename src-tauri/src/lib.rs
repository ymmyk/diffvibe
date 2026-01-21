use clap::Parser;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use xxhash_rust::xxh3::xxh3_64;
use rayon::prelude::*;

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
    /// Two directories - directory diff mode
    DirDiff { left: String, right: String },
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
        2 => {
            let left = args.files[0].clone();
            let right = args.files[1].clone();
            
            // Check if both are directories
            let left_is_dir = Path::new(&left).is_dir();
            let right_is_dir = Path::new(&right).is_dir();
            
            if left_is_dir && right_is_dir {
                CliMode::DirDiff { left, right }
            } else {
                CliMode::Diff { left, right }
            }
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
    pub exists: bool,
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

// Directory comparison types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    Identical,
    Modified,
    LeftOnly,
    RightOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareEntry {
    pub name: String,
    pub rel_path: String,  // Relative path from root
    pub is_dir: bool,
    pub status: FileStatus,
    pub left_size: Option<u64>,
    pub right_size: Option<u64>,
    pub children: Vec<CompareEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryCompareResult {
    pub left_path: String,
    pub right_path: String,
    pub entries: Vec<CompareEntry>,
    pub stats: CompareStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompareStats {
    pub identical: usize,
    pub modified: usize,
    pub left_only: usize,
    pub right_only: usize,
    pub total_files: usize,
}

// Simple directory entry for single-dir scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub rel_path: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: Vec<DirEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub root_path: String,
    pub entries: Vec<DirEntry>,
    pub file_count: usize,
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

    // Check if file exists
    if !file_path.exists() {
        return Ok(FileContent {
            path: path.to_string(),
            content: String::new(),
            encoding: "utf-8".to_string(),
            size: 0,
            line_count: 0,
            is_binary: false,
            exists: false,
        });
    }

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
            exists: true,
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
        exists: true,
    })
}

#[tauri::command]
fn write_file(path: &str, content: &str, encoding: &str) -> Result<(), String> {
    let file_path = Path::new(path);

    // Create parent directories if needed
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }

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
fn copy_file(from_path: &str, to_path: &str) -> Result<(), String> {
    let from = Path::new(from_path);
    let to = Path::new(to_path);

    // Create parent directories if needed
    if let Some(parent) = to.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }

    fs::copy(from, to).map_err(|e| format!("Failed to copy file: {}", e))?;
    Ok(())
}

#[tauri::command]
fn copy_dir(from_path: &str, to_path: &str) -> Result<(), String> {
    let from = Path::new(from_path);
    let to = Path::new(to_path);

    // Use fs_extra for recursive directory copy
    let options = fs_extra::dir::CopyOptions::new();
    fs_extra::dir::copy(from, to, &options).map_err(|e| format!("Failed to copy directory: {}", e))?;
    Ok(())
}

#[tauri::command]
fn file_exists(path: &str) -> Result<bool, String> {
    Ok(Path::new(path).exists())
}

#[tauri::command]
fn is_directory(path: &str) -> Result<bool, String> {
    Ok(Path::new(path).is_dir())
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

/// Compute file hash for comparison (first 64KB for speed)
fn file_hash(path: &Path) -> Option<u64> {
    let bytes = fs::read(path).ok()?;
    let check_len = bytes.len().min(65536);
    Some(xxh3_64(&bytes[..check_len]))
}

/// Scan a directory and build a map of rel_path -> (size, hash)
fn scan_dir_entries(root: &Path) -> Result<HashMap<String, (u64, Option<u64>)>, String> {
    let mut entries = HashMap::new();
    scan_dir_recursive(root, root, &mut entries)?;
    Ok(entries)
}

fn scan_dir_recursive(
    root: &Path,
    current: &Path,
    entries: &mut HashMap<String, (u64, Option<u64>)>,
) -> Result<(), String> {
    let read_dir = fs::read_dir(current).map_err(|e| format!("Failed to read {:?}: {}", current, e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let rel_path = path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();

        // Skip hidden files/dirs
        if rel_path.starts_with('.') || rel_path.contains("/.") {
            continue;
        }

        if path.is_dir() {
            scan_dir_recursive(root, &path, entries)?;
        } else if path.is_file() {
            let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
            let size = meta.len();
            let hash = file_hash(&path);
            entries.insert(rel_path, (size, hash));
        }
    }

    Ok(())
}

/// Build tree structure from flat comparison results
fn build_compare_tree(
    left_entries: &HashMap<String, (u64, Option<u64>)>,
    right_entries: &HashMap<String, (u64, Option<u64>)>,
    stats: &mut CompareStats,
) -> Vec<CompareEntry> {
    // Collect all unique paths and sort them
    let mut all_paths: Vec<&String> = left_entries.keys().chain(right_entries.keys()).collect();
    all_paths.sort();
    all_paths.dedup();

    // Group by directory structure
    let mut root_entries: Vec<CompareEntry> = Vec::new();
    let mut dir_map: HashMap<String, Vec<CompareEntry>> = HashMap::new();

    for rel_path in all_paths {
        let left = left_entries.get(rel_path);
        let right = right_entries.get(rel_path);

        let status = match (left, right) {
            (Some((_, lh)), Some((_, rh))) => {
                if lh == rh {
                    stats.identical += 1;
                    FileStatus::Identical
                } else {
                    stats.modified += 1;
                    FileStatus::Modified
                }
            }
            (Some(_), None) => {
                stats.left_only += 1;
                FileStatus::LeftOnly
            }
            (None, Some(_)) => {
                stats.right_only += 1;
                FileStatus::RightOnly
            }
            (None, None) => continue,
        };

        stats.total_files += 1;

        let name = Path::new(rel_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| rel_path.clone());

        let entry = CompareEntry {
            name,
            rel_path: rel_path.clone(),
            is_dir: false,
            status,
            left_size: left.map(|(s, _)| *s),
            right_size: right.map(|(s, _)| *s),
            children: Vec::new(),
        };

        // Get parent directory
        let parent = Path::new(rel_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if parent.is_empty() {
            root_entries.push(entry);
        } else {
            dir_map.entry(parent).or_default().push(entry);
        }
    }

    // Build directory entries and nest children
    let mut all_dirs: Vec<String> = dir_map.keys().cloned().collect();
    all_dirs.sort_by_key(|b| Reverse(b.matches('/').count())); // Deepest first

    for dir_path in all_dirs {
        if let Some(children) = dir_map.remove(&dir_path) {
            let name = Path::new(&dir_path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| dir_path.clone());

            // Compute dir status from children
            let status = compute_dir_status(&children);

            let dir_entry = CompareEntry {
                name,
                rel_path: dir_path.clone(),
                is_dir: true,
                status,
                left_size: None,
                right_size: None,
                children,
            };

            let parent = Path::new(&dir_path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            if parent.is_empty() {
                root_entries.push(dir_entry);
            } else {
                dir_map.entry(parent).or_default().push(dir_entry);
            }
        }
    }

    // Sort entries: directories first, then by name
    root_entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    root_entries
}

fn compute_dir_status(children: &[CompareEntry]) -> FileStatus {
    let mut has_modified = false;
    let mut has_left_only = false;
    let mut has_right_only = false;

    for child in children {
        match child.status {
            FileStatus::Modified => has_modified = true,
            FileStatus::LeftOnly => has_left_only = true,
            FileStatus::RightOnly => has_right_only = true,
            FileStatus::Identical => {}
        }
    }

    if has_modified {
        FileStatus::Modified
    } else if has_left_only && !has_right_only {
        FileStatus::LeftOnly
    } else if has_right_only && !has_left_only {
        FileStatus::RightOnly
    } else if has_left_only || has_right_only {
        FileStatus::Modified // Mixed
    } else {
        FileStatus::Identical
    }
}

#[tauri::command]
fn compare_directories(left_path: &str, right_path: &str) -> Result<DirectoryCompareResult, String> {
    let left = Path::new(left_path);
    let right = Path::new(right_path);

    if !left.is_dir() {
        return Err(format!("{} is not a directory", left_path));
    }
    if !right.is_dir() {
        return Err(format!("{} is not a directory", right_path));
    }

    let left_entries = scan_dir_entries(left)?;
    let right_entries = scan_dir_entries(right)?;

    let mut stats = CompareStats {
        identical: 0,
        modified: 0,
        left_only: 0,
        right_only: 0,
        total_files: 0,
    };

    let entries = build_compare_tree(&left_entries, &right_entries, &mut stats);

    Ok(DirectoryCompareResult {
        left_path: left_path.to_string(),
        right_path: right_path.to_string(),
        entries,
        stats,
    })
}

/// Scan a single directory and return tree structure
fn matches_ignore_pattern(rel_path: &str, patterns: &Vec<String>) -> bool {
    for pattern in patterns {
        if pattern.ends_with('/') {
            // Directory pattern
            if rel_path.starts_with(pattern) || rel_path == &pattern[..pattern.len()-1] {
                return true;
            }
        } else if pattern.contains('*') {
            // Simple glob matching
            let regex_pattern = pattern.replace('.', "\\.").replace('*', ".*");
            if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                if regex.is_match(rel_path) {
                    return true;
                }
            }
        } else {
            // Exact match
            if rel_path == pattern {
                return true;
            }
        }
    }
    false
}

fn build_dir_tree(root: &Path, current: &Path, ignore_patterns: &Vec<String>) -> Result<Vec<DirEntry>, String> {

    // First, collect all directory entries
    let dir_entries: Result<Vec<_>, _> = fs::read_dir(current)
        .map_err(|e| format!("Failed to read {:?}: {}", current, e))?
        .collect();

    let dir_entries = dir_entries.map_err(|e| e.to_string())?;

    // Process entries in parallel
    let mut entries: Vec<DirEntry> = dir_entries
        .into_par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => return None,
            };

            // Skip if matches ignore patterns
            if matches_ignore_pattern(&rel_path, ignore_patterns) {
                return None;
            }

            // Skip hidden files (but allow .git if not ignored)
            if name.starts_with('.') && !rel_path.starts_with(".git/") {
                return None;
            }

            if path.is_dir() {
                // Recursively build tree for directories
                match build_dir_tree(root, &path, ignore_patterns) {
                    Ok(children) => {
                        // Only include directories that have children or are not empty
                        if !children.is_empty() {
                            Some(DirEntry {
                                name,
                                rel_path,
                                is_dir: true,
                                size: 0,
                                children,
                            })
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            } else if path.is_file() {
                let size = fs::metadata(&path).ok().map(|m| m.len()).unwrap_or(0);
                Some(DirEntry {
                    name,
                    rel_path,
                    is_dir: false,
                    size,
                    children: Vec::new(),
                })
            } else {
                None
            }
        })
        .collect();

    // Sort: dirs first, then by name
    entries.par_sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return a.is_dir.cmp(&b.is_dir).reverse(); // dirs first
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });

    Ok(entries)
}

fn count_files(entries: &[DirEntry]) -> usize {
    entries.iter().map(|e| {
        if e.is_dir { count_files(&e.children) } else { 1 }
    }).sum()
}

/// Get just diff stats for two files (lightweight, no full diff)
#[tauri::command]
fn get_diff_stats(left_path: &str, right_path: &str) -> Result<DiffStats, String> {
    let left = read_file(left_path)?;
    let right = read_file(right_path)?;

    // If either doesn't exist or is binary, return simple stats
    if !left.exists || !right.exists {
        return Ok(DiffStats {
            additions: if right.exists { right.line_count } else { 0 },
            deletions: if left.exists { left.line_count } else { 0 },
            unchanged: 0,
        });
    }

    if left.is_binary || right.is_binary {
        return Ok(DiffStats {
            additions: 0,
            deletions: 0,
            unchanged: 0,
        });
    }

    let diff = compute_diff(&left.content, &right.content);
    Ok(diff.stats)
}

#[tauri::command]
async fn scan_directory_lazy(path: &str, ignore_patterns: Vec<String>, max_depth: usize) -> Result<ScanResult, String> {
    let root = Path::new(path);
    if !root.is_dir() {
        return Err(format!("{} is not a directory", path));
    }

    // Only scan top-level entries, don't recurse deeply
    let entries = build_dir_tree_lazy(root, root, &ignore_patterns, 0, max_depth)?;
    let file_count = count_files(&entries);

    Ok(ScanResult {
        root_path: path.to_string(),
        entries,
        file_count,
    })
}

#[tauri::command]
async fn expand_directory(path: &str, rel_path: &str, ignore_patterns: Vec<String>) -> Result<Vec<DirEntry>, String> {
    let root = Path::new(path);
    let target = root.join(rel_path);
    
    if !target.is_dir() {
        return Err(format!("{} is not a directory", target.display()));
    }

    // Scan just this directory's children (one level)
    let entries = build_dir_tree_lazy(root, &target, &ignore_patterns, 0, 1)?;
    
    Ok(entries)
}

fn build_dir_tree_lazy(
    root: &Path,
    current: &Path,
    ignore_patterns: &Vec<String>,
    current_depth: usize,
    max_depth: usize,
) -> Result<Vec<DirEntry>, String> {
    // Stop recursion at max depth
    if current_depth >= max_depth {
        return Ok(Vec::new());
    }

    // First, collect all directory entries
    let dir_entries: Result<Vec<_>, _> = fs::read_dir(current)
        .map_err(|e| format!("Failed to read {:?}: {}", current, e))?
        .collect();

    let dir_entries = dir_entries.map_err(|e| e.to_string())?;

    // Process entries in parallel
    let mut entries: Vec<DirEntry> = dir_entries
        .into_par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => return None,
            };

            // Skip if matches ignore patterns
            if matches_ignore_pattern(&rel_path, ignore_patterns) {
                return None;
            }

            // Skip hidden files (but allow .git if not ignored)
            if name.starts_with('.') && !rel_path.starts_with(".git/") {
                return None;
            }

            if path.is_dir() {
                // For directories, only recurse if within depth limit
                let children = if current_depth + 1 < max_depth {
                    match build_dir_tree_lazy(root, &path, ignore_patterns, current_depth + 1, max_depth) {
                        Ok(ch) => ch,
                        Err(_) => Vec::new(),
                    }
                } else {
                    // Return empty children but mark that we have a directory
                    Vec::new()
                };

                Some(DirEntry {
                    name,
                    rel_path,
                    is_dir: true,
                    size: 0,
                    children,
                })
            } else if path.is_file() {
                let size = fs::metadata(&path).ok().map(|m| m.len()).unwrap_or(0);
                Some(DirEntry {
                    name,
                    rel_path,
                    is_dir: false,
                    size,
                    children: Vec::new(),
                })
            } else {
                None
            }
        })
        .collect();

    // Sort: dirs first, then by name
    entries.par_sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return a.is_dir.cmp(&b.is_dir).reverse(); // dirs first
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });

    Ok(entries)
}

#[tauri::command]
async fn scan_directory(path: &str, ignore_patterns: Vec<String>) -> Result<ScanResult, String> {
    let root = Path::new(path);
    if !root.is_dir() {
        return Err(format!("{} is not a directory", path));
    }

    let path_owned = path.to_string();
    let ignore_patterns = ignore_patterns.clone();

    // Run in blocking thread pool to avoid blocking async runtime
    let result = tokio::task::spawn_blocking(move || {
        let root = Path::new(&path_owned);
        let entries = build_dir_tree(root, root, &ignore_patterns)?;
        let file_count = count_files(&entries);

        Ok::<ScanResult, String>(ScanResult {
            root_path: path_owned,
            entries,
            file_count,
        })
    })
    .await
    .map_err(|e| format!("Scan task failed: {}", e))??;

    Ok(result)
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
        .invoke_handler(tauri::generate_handler![read_file, write_file, copy_file, copy_dir, file_exists, is_directory, compute_diff, compute_diff_files, compute_three_way_diff, get_cli_args, exit_app, compare_directories, scan_directory, scan_directory_lazy, expand_directory, get_diff_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_directories() {
        use std::fs;

        // Create temp test directories
        let temp = std::env::temp_dir().join("diffvibe_test");
        let left = temp.join("left");
        let right = temp.join("right");

        // Clean up any previous test
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&left).unwrap();
        fs::create_dir_all(&right).unwrap();

        // Same file
        fs::write(left.join("same.txt"), "same content").unwrap();
        fs::write(right.join("same.txt"), "same content").unwrap();

        // Modified file
        fs::write(left.join("modified.txt"), "left version").unwrap();
        fs::write(right.join("modified.txt"), "right version").unwrap();

        // Left only
        fs::write(left.join("left-only.txt"), "only left").unwrap();

        // Right only
        fs::write(right.join("right-only.txt"), "only right").unwrap();

        let result = compare_directories(
            left.to_str().unwrap(),
            right.to_str().unwrap()
        ).unwrap();

        assert_eq!(result.stats.identical, 1);
        assert_eq!(result.stats.modified, 1);
        assert_eq!(result.stats.left_only, 1);
        assert_eq!(result.stats.right_only, 1);
        assert_eq!(result.stats.total_files, 4);

        // Clean up
        let _ = fs::remove_dir_all(&temp);
    }

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
