import { describe, it, expect } from "vitest";
import { formatTime, extrapolatePosition } from "../lib/interpolation";
import { PlaybackState } from "../lib/types";

describe("formatTime", () => {
  it("formats zero and small durations", () => {
    expect(formatTime(0)).toBe("0:00");
    expect(formatTime(500)).toBe("0:00");
    expect(formatTime(1000)).toBe("0:01");
    expect(formatTime(9000)).toBe("0:09");
    expect(formatTime(10000)).toBe("0:10");
  });

  it("formats minutes and seconds", () => {
    expect(formatTime(65000)).toBe("1:05");
    expect(formatTime(120000)).toBe("2:00");
    expect(formatTime(215000)).toBe("3:35");
  });

  it("handles negative and NaN values gracefully", () => {
    expect(formatTime(-100)).toBe("0:00");
    expect(formatTime(NaN)).toBe("0:00");
  });
});

describe("extrapolatePosition", () => {
  const basePlayback: PlaybackState = {
    status: "playing",
    positionMs: 10000,
    rate: 1.0,
    shuffle: false,
    repeat: "off",
    canPlayPause: true,
    canNext: true,
    canPrev: true,
    canSeek: true,
    canShuffle: true,
    canRepeat: true,
  };

  it("does not extrapolate if paused", () => {
    const pausedPlayback = { ...basePlayback, status: "paused" as const };
    const pos = extrapolatePosition(pausedPlayback, 1000, 3000, 180000);
    expect(pos).toBe(10000);
  });

  it("extrapolates forward when playing with rate 1.0", () => {
    // 2000ms elapsed since lastReceivedAt
    const pos = extrapolatePosition(basePlayback, 1000, 3000, 180000);
    expect(pos).toBe(12000);
  });

  it("takes playback rate into account", () => {
    const fastPlayback = { ...basePlayback, rate: 1.5 };
    // 2000ms elapsed * 1.5 = 3000ms added
    const pos = extrapolatePosition(fastPlayback, 1000, 3000, 180000);
    expect(pos).toBe(13000);
  });

  it("clamps to duration bounds", () => {
    // Exceeds duration of 15000ms
    const pos = extrapolatePosition(basePlayback, 1000, 10000, 15000);
    expect(pos).toBe(15000);
  });
});

