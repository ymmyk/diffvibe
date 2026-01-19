/**
 * Types for diff operations
 */

export type DiffTag = 'equal' | 'insert' | 'delete';

export interface DiffLine {
  tag: DiffTag;
  oldIndex: number | null;
  newIndex: number | null;
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

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  encoding: string;
  lineCount: number;
}

export interface FileContent {
  info: FileInfo;
  content: string;
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
