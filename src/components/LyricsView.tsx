import React from "react";
import { LyricsState, NowPlaying } from "../lib/types";
import { getLyricDisplay } from "../lib/lyricsIndex";

interface LyricsViewProps {
  lyricsState: LyricsState | null;
  nowPlaying: NowPlaying | null;
  activeIndex: number;
}

function lineSizeClass(text: string | undefined): string {
  const length = text?.length ?? 0;
  if (length > 64) return "lyric-line--dense";
  if (length > 40) return "lyric-line--compact";
  return "";
}

const LyricsViewComponent: React.FC<LyricsViewProps> = ({
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
    const currentDisplay = currentText || (nextText ? "♪ Intro" : nowPlaying.title);
    return (
      <div className="lyrics-container">
        <div
          className={`lyric-line-current ${lineSizeClass(currentDisplay)}`}
          title={currentDisplay}
        >
          {currentDisplay}
        </div>
        <div className={`lyric-line-next ${lineSizeClass(nextText)}`} title={nextText}>
          {nextText}
        </div>
      </div>
    );
  }

  // Plain lyrics state — show the first lines rather than hiding them
  if (lyricsState.kind === "plain") {
    const plainLines = lyricsState.text
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0);
    const line1 = plainLines[0] || nowPlaying.title;
    const line2 = plainLines[1] || nowPlaying.artist;
    return (
      <div className="lyrics-container">
        <div className={`lyric-line-current ${lineSizeClass(line1)}`} title={line1}>
          {line1}
        </div>
        <div className={`lyric-line-next ${lineSizeClass(line2)}`} title={line2}>
          {line2}
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
          <span className="lyrics-subtext">
            {nowPlaying.title} — {nowPlaying.artist}
          </span>
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
          <span className="lyrics-subtext" style={{ opacity: 0.6 }}>
            No lyrics found
          </span>
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
          <span className="lyrics-subtext" style={{ opacity: 0.6 }}>
            Lyrics unavailable
          </span>
        </div>
      </div>
    );
  }

  return null;
};

export const LyricsView = React.memo(LyricsViewComponent);
