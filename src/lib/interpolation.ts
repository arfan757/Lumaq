import { PlaybackState } from "./types";

/**
 * Extrapolates playback position in milliseconds.
 *
 * @param playback Playback state from the backend.
 * @param lastReceivedAt Timestamp (ms from performance.now() or Date.now()) when the state was received.
 * @param currentTimestamp Current timestamp (ms).
 * @param durationMs Total duration in milliseconds.
 * @returns Extrapolated position clamped to [0, durationMs].
 */
export function extrapolatePosition(
  playback: PlaybackState,
  lastReceivedAt: number,
  currentTimestamp: number,
  durationMs: number
): number {
  if (playback.status !== "playing") {
    return Math.min(Math.max(0, playback.positionMs), durationMs);
  }

  const elapsed = Math.max(0, currentTimestamp - lastReceivedAt);
  const extrapolated = playback.positionMs + elapsed * (playback.rate || 1.0);

  return Math.min(Math.max(0, Math.round(extrapolated)), durationMs);
}

/**
 * Formats a duration in milliseconds to "m:ss" format with tabular alignment.
 */
export function formatTime(ms: number): string {
  if (isNaN(ms) || ms < 0) {
    return "0:00";
  }

  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
