use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![read_file, write_file, compute_diff, compute_diff_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
