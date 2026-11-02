import { useState, useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { LyricsState, NowPlaying } from "../lib/types";
import { findActiveLyricIndex } from "../lib/lyricsIndex";

export function useLyrics(
  nowPlaying: NowPlaying | null,
  positionMs: number,
  offsetMs: number = 0
) {
  const [currentTrackKey, setCurrentTrackKey] = useState<string | null>(null);
  const [lyricsState, setLyricsState] = useState<LyricsState | null>(null);

  // Sync state when nowPlaying track changes
  if (nowPlaying?.trackKey !== currentTrackKey) {
    setCurrentTrackKey(nowPlaying?.trackKey ?? null);
    setLyricsState(
      nowPlaying ? { kind: "loading", trackKey: nowPlaying.trackKey } : null
    );
  }

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<LyricsState>("lumaq://lyrics", (event) => {
      setLyricsState(event.payload);
    }).then((un) => {
      unlisten = un;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  // Compute active lyric index via binary search
  const activeLyricIndex = useMemo(() => {
    if (lyricsState?.kind === "synced") {
      return findActiveLyricIndex(lyricsState.lines, positionMs, offsetMs);
    }
    return -1;
  }, [lyricsState, positionMs, offsetMs]);

  const refetchLyrics = () => {
    invoke("lyrics_refetch").catch(console.debug);
  };

  return {
    lyricsState,
    activeLyricIndex,
    refetchLyrics,
  };
}
