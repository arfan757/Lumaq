import { describe, it, expect } from "vitest";
import { findActiveLyricIndex, getLyricDisplay } from "../lib/lyricsIndex";
import { LyricLine } from "../lib/types";

describe("findActiveLyricIndex", () => {
  const sampleLines: LyricLine[] = [
    { timeMs: 5000, text: "First line of the song" },
    { timeMs: 12000, text: "Second line arrives" },
    { timeMs: 20000, text: "Third line here" },
    { timeMs: 35000, text: "Final closing line" },
  ];

  it("returns -1 for empty lines array", () => {
    expect(findActiveLyricIndex([], 10000)).toBe(-1);
  });

  it("returns -1 when playback position is before the first line", () => {
    expect(findActiveLyricIndex(sampleLines, 0)).toBe(-1);
    expect(findActiveLyricIndex(sampleLines, 4999)).toBe(-1);
  });

  it("returns 0 at exactly the first line timestamp", () => {
    expect(findActiveLyricIndex(sampleLines, 5000)).toBe(0);
  });

  it("returns correct index between lines", () => {
    expect(findActiveLyricIndex(sampleLines, 6000)).toBe(0);
    expect(findActiveLyricIndex(sampleLines, 11999)).toBe(0);
    expect(findActiveLyricIndex(sampleLines, 12000)).toBe(1);
    expect(findActiveLyricIndex(sampleLines, 15000)).toBe(1);
    expect(findActiveLyricIndex(sampleLines, 19999)).toBe(1);
    expect(findActiveLyricIndex(sampleLines, 20000)).toBe(2);
  });

  it("returns last index when position is at or past the last line", () => {
    expect(findActiveLyricIndex(sampleLines, 35000)).toBe(3);
    expect(findActiveLyricIndex(sampleLines, 50000)).toBe(3);
  });

  it("respects positive offset (advances lyrics)", () => {
    // Song is at 4500ms, but user set +1000ms offset -> effective 5500ms -> should match line 0
    expect(findActiveLyricIndex(sampleLines, 4500, 1000)).toBe(0);
  });

  it("respects negative offset (delays lyrics)", () => {
    // Song is at 5500ms, user set -1000ms offset -> effective 4500ms -> should return -1
    expect(findActiveLyricIndex(sampleLines, 5500, -1000)).toBe(-1);
  });
});

describe("getLyricDisplay", () => {
  const sampleLines: LyricLine[] = [
    { timeMs: 5000, text: "Line 1" },
    { timeMs: 10000, text: "Line 2" },
    { timeMs: 15000, text: "" }, // Instrumental gap
    { timeMs: 20000, text: "Line 4" },
  ];

  it("handles negative activeIndex (intro/before lyrics)", () => {
    const res = getLyricDisplay(sampleLines, -1);
    expect(res.currentText).toBe("");
    expect(res.nextText).toBe("Line 1");
  });

  it("handles regular line display", () => {
    const res = getLyricDisplay(sampleLines, 0);
    expect(res.currentText).toBe("Line 1");
    expect(res.nextText).toBe("Line 2");
  });

  it("handles empty lines (instrumental break)", () => {
    const res = getLyricDisplay(sampleLines, 2);
    expect(res.currentText).toBe("...");
    expect(res.nextText).toBe("Line 4");
  });

  it("handles last line with no next line", () => {
    const res = getLyricDisplay(sampleLines, 3);
    expect(res.currentText).toBe("Line 4");
    expect(res.nextText).toBe("");
  });
});
