import React from "react";
import { LyricsState, NowPlaying } from "../lib/types";
import { getLyricDisplay } from "../lib/lyricsIndex";

interface LyricsViewProps {
  lyricsState: LyricsState | null;
  nowPlaying: NowPlaying | null;
  activeIndex: number;
}

export const LyricsView: React.FC<LyricsViewProps> = ({
  lyricsState,
  nowPlaying,
  activeIndex,
}) => {
  // Idle state: nothing playing
  if (!nowPlaying) {
    return (
      <div className="lyrics-container">
        <div className="lyrics-status-message">
          <span>Nothing playing</span>
          <span className="lyrics-subtext">Play a song on Spotify</span>
        </div>
      </div>
    );
  }

  // Ad playing state
  if (nowPlaying.isAd) {
    return (
      <div className="lyrics-container">
        <div className="lyrics-status-message">
          <span>Ad playing</span>
          <span className="lyrics-subtext">Waiting for song to resume</span>
        </div>
      </div>
    );
  }

  // Loading state
  if (!lyricsState || lyricsState.kind === "loading") {
    return (
      <div className="lyrics-container">
        <div className="lyrics-status-message lyrics-loading-shimmer">
          <span>{nowPlaying.title}</span>
          <span className="lyrics-subtext">{nowPlaying.artist} • Loading lyrics...</span>
        </div>
      </div>
    );
  }

  // Synced lyrics state
  if (lyricsState.kind === "synced") {
    const { currentText, nextText } = getLyricDisplay(lyricsState.lines, activeIndex);
    return (
      <div className="lyrics-container">
        <div className="lyric-line-current" title={currentText}>
          {currentText || "..."}
        </div>
        <div className="lyric-line-next" title={nextText}>
          {nextText}
        </div>
      </div>
    );
  }

  // Plain lyrics state
  if (lyricsState.kind === "plain") {
    return (
      <div className="lyrics-container">
        <div className="lyrics-status-message">
          <span style={{ fontWeight: 600 }}>{nowPlaying.title}</span>
          <span className="lyrics-subtext">{nowPlaying.artist}</span>
          <span className="lyrics-subtext" style={{ fontStyle: "italic", opacity: 0.6 }}>
            Lyrics not synced
          </span>
        </div>
      </div>
    );
  }

  // Instrumental state
  if (lyricsState.kind === "instrumental") {
    return (
      <div className="lyrics-container">
        <div className="lyrics-status-message">
          <span>Instrumental</span>
          <span className="lyrics-subtext">{nowPlaying.title} — {nowPlaying.artist}</span>
        </div>
      </div>
    );
  }

  // Not found state
  if (lyricsState.kind === "notFound") {
    return (
      <div className="lyrics-container">
        <div className="lyrics-status-message">
          <span>{nowPlaying.title}</span>
          <span className="lyrics-subtext">{nowPlaying.artist}</span>
          <span className="lyrics-subtext" style={{ opacity: 0.6 }}>No lyrics found</span>
        </div>
      </div>
    );
  }

  // Error state
  if (lyricsState.kind === "error") {
    return (
      <div className="lyrics-container">
        <div className="lyrics-status-message">
          <span>{nowPlaying.title}</span>
          <span className="lyrics-subtext">{nowPlaying.artist}</span>
          <span className="lyrics-subtext" style={{ opacity: 0.6 }}>Lyrics unavailable</span>
        </div>
      </div>
    );
  }

  return null;
};
