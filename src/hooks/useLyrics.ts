import { useState, useEffect, useMemo, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { LyricsState, NowPlaying } from "../lib/types";
import { findActiveLyricIndex } from "../lib/lyricsIndex";

export function useLyrics(
  nowPlaying: NowPlaying | null,
  positionMs: number,
  offsetMs: number = 0
) {
  const [backendLyrics, setBackendLyrics] = useState<LyricsState | null>(null);

  // 1. Initial snapshot fetch and event listener
  useEffect(() => {
    invoke<{ lyrics: LyricsState | null }>("get_snapshot")
      .then((snapshot) => {
        if (snapshot?.lyrics) {
          setBackendLyrics(snapshot.lyrics);
        }
      })
      .catch(console.debug);

    let unlisten: (() => void) | undefined;

    listen<LyricsState>("lumaq://lyrics", (event) => {
      setBackendLyrics(event.payload);
    }).then((un) => {
      unlisten = un;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const trackKey = nowPlaying?.trackKey ?? null;

  // 2. Fetch lyrics if we have an active track but no lyrics matching it yet
  useEffect(() => {
    if (trackKey && (!backendLyrics || backendLyrics.trackKey !== trackKey)) {
      invoke("lyrics_refetch").catch(console.debug);
    }
  }, [trackKey, backendLyrics]);

  // 3. Derive lyricsState
  const lyricsState: LyricsState | null = useMemo(() => {
    if (!trackKey) {
      return null;
    }
    if (backendLyrics && backendLyrics.trackKey === trackKey) {
      return backendLyrics;
    }
    return { kind: "loading", trackKey };
  }, [trackKey, backendLyrics]);

  // Compute active lyric index via binary search
  const activeLyricIndex = useMemo(() => {
    if (lyricsState?.kind === "synced") {
      return findActiveLyricIndex(lyricsState.lines, positionMs, offsetMs);
    }
    return -1;
  }, [lyricsState, positionMs, offsetMs]);

  const refetchLyrics = useCallback(() => {
    invoke("lyrics_refetch").catch(console.debug);
  }, []);

  return {
    lyricsState,
    activeLyricIndex,
    refetchLyrics,
  };
}
