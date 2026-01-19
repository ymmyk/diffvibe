/**
 * Simple undo/redo history manager
 */

export interface History<T> {
  past: T[];
  present: T;
  future: T[];
}

export function createHistory<T>(initial: T): History<T> {
  return {
    past: [],
    present: initial,
    future: []
  };
}

export function push<T>(history: History<T>, value: T): History<T> {
  // Don't push if same as present
  if (value === history.present) return history;

  return {
    past: [...history.past, history.present],
    present: value,
    future: [] // Clear future on new edit
  };
}

export function undo<T>(history: History<T>): History<T> {
  if (history.past.length === 0) return history;

  const newPast = [...history.past];
  const previous = newPast.pop()!;

  return {
    past: newPast,
    present: previous,
    future: [history.present, ...history.future]
  };
}

export function redo<T>(history: History<T>): History<T> {
  if (history.future.length === 0) return history;

  const newFuture = [...history.future];
  const next = newFuture.shift()!;

  return {
    past: [...history.past, history.present],
    present: next,
    future: newFuture
  };
}

export function canUndo<T>(history: History<T>): boolean {
  return history.past.length > 0;
}

export function canRedo<T>(history: History<T>): boolean {
  return history.future.length > 0;
}

export function reset<T>(history: History<T>, value: T): History<T> {
  return createHistory(value);
}
