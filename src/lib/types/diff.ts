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
}

export interface FileDiffResult {
  left: FileContent;
  right: FileContent;
  diff: DiffResult;
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
