use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![compute_diff])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
