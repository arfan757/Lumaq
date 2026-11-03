import { useState, useEffect, useMemo, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { LyricsState, NowPlaying } from "../lib/types";
import { findActiveLyricIndex } from "../lib/lyricsIndex";

function extractKey(obj: any): string | null {
  if (!obj) return null;
  return obj.trackKey ?? obj.track_key ?? null;
}

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
          const raw = snapshot.lyrics as any;
          const key = extractKey(raw) || "";
          setBackendLyrics({ ...raw, trackKey: key });
        }
      })
      .catch(console.debug);

    let unlisten: (() => void) | undefined;

    listen<LyricsState>("lumaq://lyrics", (event) => {
      const raw = event.payload as any;
      if (raw) {
        const key = extractKey(raw) || "";
        setBackendLyrics({ ...raw, trackKey: key });
      }
    }).then((un) => {
      unlisten = un;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const trackKey = extractKey(nowPlaying);

  // 2. Fetch lyrics if we have an active track but no lyrics matching it yet
  useEffect(() => {
    const backendKey = extractKey(backendLyrics);
    if (trackKey && backendKey !== trackKey) {
      invoke("lyrics_refetch").catch(console.debug);
    }
  }, [trackKey, backendLyrics]);

  // 3. Derive lyricsState
  const lyricsState: LyricsState | null = useMemo(() => {
    if (!trackKey) {
      return null;
    }
    const backendKey = extractKey(backendLyrics);
    if (backendLyrics && backendKey === trackKey) {
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
