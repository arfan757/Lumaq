import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { NowPlaying, PlaybackState, Snapshot } from "../lib/types";
import { extrapolatePosition } from "../lib/interpolation";

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

  const [playingPositionMs, setPlayingPositionMs] = useState(0);

  const lastReceivedAtRef = useRef<number>(0);
  const playbackRef = useRef<PlaybackState>(playback);
  const durationMsRef = useRef<number>(0);
  const animFrameRef = useRef<number | null>(null);

  useEffect(() => {
    playbackRef.current = playback;
  }, [playback]);

  useEffect(() => {
    durationMsRef.current = nowPlaying?.durationMs ?? 0;
  }, [nowPlaying?.durationMs]);

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
        const currentMs = extrapolatePosition(
          playbackRef.current,
          lastReceivedAtRef.current,
          performance.now(),
          durationMsRef.current
        );
        setPlayingPositionMs(currentMs);
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
  }, [playback.status]);

  // Load initial snapshot and subscribe to events
  useEffect(() => {
    invoke<Snapshot>("get_snapshot")
      .then((snapshot) => {
        if (snapshot.nowPlaying) setNowPlaying(snapshot.nowPlaying);
        if (snapshot.playback) {
          setPlayback(snapshot.playback);
          lastReceivedAtRef.current = performance.now();
        }
      })
      .catch((err) => console.debug("Initial snapshot fetch:", err));

    let unlistenNp: (() => void) | undefined;
    let unlistenPb: (() => void) | undefined;

    listen<NowPlaying | null>("lumaq://now-playing", (event) => {
      setNowPlaying(event.payload);
    }).then((un) => {
      unlistenNp = un;
    });

    listen<PlaybackState>("lumaq://playback", (event) => {
      lastReceivedAtRef.current = performance.now();
      setPlayback(event.payload);
    }).then((un) => {
      unlistenPb = un;
    });

    return () => {
      unlistenNp?.();
      unlistenPb?.();
    };
  }, []);

  const playPause = useCallback(() => {
    invoke("media_play_pause").catch(console.debug);
  }, []);

  const next = useCallback(() => {
    invoke("media_next").catch(console.debug);
  }, []);

  const prev = useCallback(() => {
    invoke("media_prev").catch(console.debug);
  }, []);

  const seek = useCallback((positionMs: number) => {
    setPlayingPositionMs(positionMs);
    lastReceivedAtRef.current = performance.now();
    invoke("media_seek", { positionMs }).catch(console.debug);
  }, []);

  const toggleShuffle = useCallback(() => {
    invoke("media_toggle_shuffle").catch(console.debug);
  }, []);

  const cycleRepeat = useCallback(() => {
    invoke("media_cycle_repeat").catch(console.debug);
  }, []);

  const currentPosition =
    playback.status === "playing" ? playingPositionMs : playback.positionMs;

  return {
    nowPlaying,
    playback,
    interpolatedPositionMs: currentPosition,
    playPause,
    next,
    prev,
    seek,
    toggleShuffle,
    cycleRepeat,
  };
}
