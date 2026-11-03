import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { NowPlaying, PlaybackState, Snapshot } from "../lib/types";
import { extrapolatePosition } from "../lib/interpolation";

const SEEK_OVERRIDE_TTL_MS = 3000;
const SEEK_MATCH_TOLERANCE_MS = 1500;

interface SeekOverride {
  positionMs: number;
  at: number;
  expiresAt: number;
}

function clampPosition(value: number, durationMs: number): number {
  const max = durationMs > 0 ? durationMs : Number.MAX_SAFE_INTEGER;
  return Math.min(Math.max(0, Math.round(value)), max);
}

export function usePlayback() {
  const [nowPlaying, setNowPlaying] = useState<NowPlaying | null>(null);
  const [playback, setPlayback] = useState<PlaybackState>({
    status: "none",
    positionMs: 0,
    rate: 1.0,
    shuffle: null,
    repeat: null,
    canPlayPause: false,
    canNext: false,
    canPrev: false,
    canSeek: false,
    canShuffle: false,
    canRepeat: false,
  });

  const [positionMs, setPositionMs] = useState(0);

  const lastReceivedAtRef = useRef<number>(0);
  const playbackRef = useRef<PlaybackState>(playback);
  const durationMsRef = useRef<number>(0);
  const animFrameRef = useRef<number | null>(null);
  const seekOverrideRef = useRef<SeekOverride | null>(null);

  useEffect(() => {
    playbackRef.current = playback;
  }, [playback]);

  useEffect(() => {
    durationMsRef.current = nowPlaying?.durationMs ?? 0;
  }, [nowPlaying?.durationMs]);

  // Compute the position to display, honouring a recent seek before the
  // backend has reported the new position.
  const computeDisplayPosition = useCallback((now: number): number => {
    const override = seekOverrideRef.current;
    if (override) {
      if (now < override.expiresAt) {
        if (playbackRef.current.status === "playing") {
          const rate = playbackRef.current.rate || 1.0;
          return clampPosition(
            override.positionMs + (now - override.at) * rate,
            durationMsRef.current
          );
        }
        return clampPosition(override.positionMs, durationMsRef.current);
      }
      seekOverrideRef.current = null;
    }

    return extrapolatePosition(
      playbackRef.current,
      lastReceivedAtRef.current,
      now,
      durationMsRef.current
    );
  }, []);

  // Animation frame interpolation loop for active playback
  useEffect(() => {
    if (playback.status !== "playing") {
      return;
    }

    let lastRenderTime = 0;
    const targetInterval = 1000 / 30; // ~30 fps cap

    const loop = (time: number) => {
      if (time - lastRenderTime >= targetInterval) {
        lastRenderTime = time;
        setPositionMs(computeDisplayPosition(time));
      }

      if (playbackRef.current.status === "playing") {
        animFrameRef.current = requestAnimationFrame(loop);
      }
    };

    animFrameRef.current = requestAnimationFrame(loop);

    return () => {
      if (animFrameRef.current !== null) {
        cancelAnimationFrame(animFrameRef.current);
      }
    };
  }, [playback.status, computeDisplayPosition]);

  const applyPlaybackState = useCallback((next: PlaybackState) => {
    const now = performance.now();
    lastReceivedAtRef.current = now;
    playbackRef.current = next;

    const override = seekOverrideRef.current;
    if (override) {
      const matches =
        Math.abs(next.positionMs - override.positionMs) <= SEEK_MATCH_TOLERANCE_MS;
      const expired = now >= override.expiresAt;
      const stopped = next.status === "stopped" || next.status === "none";
      if (matches || expired || stopped) {
        seekOverrideRef.current = null;
      }
    }

    setPlayback(next);

    if (!seekOverrideRef.current) {
      setPositionMs(clampPosition(next.positionMs, durationMsRef.current));
    }
  }, []);

  // Load initial snapshot and subscribe to events
  useEffect(() => {
    invoke<Snapshot>("get_snapshot")
      .then((snapshot) => {
        if (snapshot.nowPlaying) setNowPlaying(snapshot.nowPlaying);
        if (snapshot.playback) applyPlaybackState(snapshot.playback);
      })
      .catch((err) => console.debug("Initial snapshot fetch:", err));

    let unlistenNp: (() => void) | undefined;
    let unlistenPb: (() => void) | undefined;

    listen<NowPlaying | null>("lumaq://now-playing", (event) => {
      // A new track invalidates any pending seek target.
      seekOverrideRef.current = null;
      setNowPlaying(event.payload);
    }).then((un) => {
      unlistenNp = un;
    });

    listen<PlaybackState>("lumaq://playback", (event) => {
      applyPlaybackState(event.payload);
    }).then((un) => {
      unlistenPb = un;
    });

    return () => {
      unlistenNp?.();
      unlistenPb?.();
    };
  }, [applyPlaybackState]);

  const playPause = useCallback(() => {
    invoke("media_play_pause").catch(console.debug);
  }, []);

  const next = useCallback(() => {
    invoke("media_next").catch(console.debug);
  }, []);

  const prev = useCallback(() => {
    invoke("media_prev").catch(console.debug);
  }, []);

  const seek = useCallback((targetMs: number) => {
    const now = performance.now();
    const clamped = clampPosition(targetMs, durationMsRef.current);

    seekOverrideRef.current = {
      positionMs: clamped,
      at: now,
      expiresAt: now + SEEK_OVERRIDE_TTL_MS,
    };
    lastReceivedAtRef.current = now;
    setPositionMs(clamped);

    invoke("media_seek", { positionMs: clamped }).catch(console.debug);
  }, []);

  const toggleShuffle = useCallback(() => {
    invoke("media_toggle_shuffle").catch(console.debug);
  }, []);

  const cycleRepeat = useCallback(() => {
    invoke("media_cycle_repeat").catch(console.debug);
  }, []);

  return {
    nowPlaying,
    playback,
    interpolatedPositionMs: positionMs,
    playPause,
    next,
    prev,
    seek,
    toggleShuffle,
    cycleRepeat,
  };
}
