/**
 * Types for diff operations - matches Rust backend types
 */

export type DiffTag = 'equal' | 'insert' | 'delete';

export interface DiffLine {
  tag: DiffTag;
  old_index: number | null;
  new_index: number | null;
  value: string;
}

export interface DiffStats {
  additions: number;
  deletions: number;
  unchanged: number;
}

export interface DiffResult {
  lines: DiffLine[];
  stats: DiffStats;
}

export interface FileContent {
  path: string;
  content: string;
  encoding: string;
  size: number;
  line_count: number;
  is_binary: boolean;
  exists: boolean;
}

export interface FileDiffResult {
  left: FileContent;
  right: FileContent;
  diff: DiffResult;
}

// Three-way merge types
export type ChunkType = 'Equal' | 'LocalOnly' | 'RemoteOnly' | 'Conflict';

export interface MergeChunk {
  chunk_type: ChunkType;
  base_start: number;
  base_count: number;
  local_lines: string[];
  remote_lines: string[];
}

export interface MergeResult {
  chunks: MergeChunk[];
  conflict_count: number;
  merged_content: string;
}

export type CompareMode = 'file' | 'directory' | 'merge';

export interface RecentComparison {
  id: string;
  mode: CompareMode;
  leftPath: string;
  rightPath: string;
  basePath?: string;
  timestamp: number;
}

// CLI args from backend
export type CliMode =
  | { mode: 'None' }
  | { mode: 'Diff'; left: string; right: string }
  | { mode: 'DirDiff'; left: string; right: string }
  | { mode: 'Merge'; local: string; base: string; remote: string; output: string | null };

// Directory comparison types
export type FileStatus = 'Identical' | 'Modified' | 'LeftOnly' | 'RightOnly';

export interface CompareEntry {
  name: string;
  rel_path: string;
  is_dir: boolean;
  status: FileStatus;
  left_size: number | null;
  right_size: number | null;
  children: CompareEntry[];
}

export interface CompareStats {
  identical: number;
  modified: number;
  left_only: number;
  right_only: number;
  total_files: number;
}

export interface DirectoryCompareResult {
  left_path: string;
  right_path: string;
  entries: CompareEntry[];
  stats: CompareStats;
}

// Single directory scan types
export interface DirEntry {
  name: string;
  rel_path: string;
  is_dir: boolean;
  size: number;
  children: DirEntry[];
}

export interface ScanResult {
  root_path: string;
  entries: DirEntry[];
  file_count: number;
}

// Aligned directory comparison types (new backend)
export type EntryStatus = 'match' | 'modified' | 'leftonly' | 'rightonly';

export interface AlignedEntry {
  name: string;
  rel_path: string;
  is_dir: boolean;
  left_size: number | null;
  right_size: number | null;
  status: EntryStatus;
  children: AlignedEntry[];
}

export interface AlignedScanResult {
  root_left: string;
  root_right: string;
  entries: AlignedEntry[];
  stats: CompareStats;
}

export interface ScanProgress {
  phase: string;
  files: number;
  message: string;
}
