import { LyricLine } from "./types";

/**
 * Finds the index of the currently active lyric line using binary search.
 *
 * @param lines Array of parsed lyric lines sorted by timeMs ascending.
 * @param positionMs Current playback position in milliseconds.
 * @param offsetMs Optional user-defined offset in milliseconds (default 0).
 * @returns Index of the active line in `lines`, or -1 if position is before the first line or lines are empty.
 */
export function findActiveLyricIndex(
  lines: LyricLine[],
  positionMs: number,
  offsetMs: number = 0
): number {
  if (!lines || lines.length === 0) {
    return -1;
  }

  const effectiveTime = positionMs + offsetMs;

  // Before the first line
  if (effectiveTime < lines[0].timeMs) {
    return -1;
  }

  // After or at the last line
  if (effectiveTime >= lines[lines.length - 1].timeMs) {
    return lines.length - 1;
  }

  let low = 0;
  let high = lines.length - 1;
  let result = 0;

  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (lines[mid].timeMs <= effectiveTime) {
      result = mid;
      low = mid + 1; // Try to find a later line that also qualifies
    } else {
      high = mid - 1;
    }
  }

  return result;
}

/**
 * Gets the current and next lyric lines given an active index.
 * Gaps or empty text are handled gracefully.
 */
export function getLyricDisplay(
  lines: LyricLine[],
  activeIndex: number
): { currentText: string; nextText: string } {
  if (!lines || lines.length === 0 || activeIndex < 0 || activeIndex >= lines.length) {
    return { currentText: "", nextText: lines?.[0]?.text ?? "" };
  }

  const current = lines[activeIndex]?.text?.trim() ?? "";
  const next = lines[activeIndex + 1]?.text?.trim() ?? "";

  return {
    currentText: current.length > 0 ? current : "...",
    nextText: next.length > 0 ? next : "",
  };
}

