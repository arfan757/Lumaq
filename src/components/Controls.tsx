import React from "react";
import { PlaybackState } from "../lib/types";

interface ControlsProps {
  playback: PlaybackState;
  onPlayPause: () => void;
  onNext: () => void;
  onPrev: () => void;
  onToggleShuffle: () => void;
  onCycleRepeat: () => void;
}

const ControlsComponent: React.FC<ControlsProps> = ({
  playback,
  onPlayPause,
  onNext,
  onPrev,
  onToggleShuffle,
  onCycleRepeat,
}) => {
  const isPlaying = playback.status === "playing";
  const shuffleActive = playback.shuffle === true;
  const repeatActive = playback.repeat === "track" || playback.repeat === "list";

  return (
    <div
      className="controls-row"
      data-tauri-drag-region="false"
      onMouseDown={(e) => e.stopPropagation()}
    >
      {/* Shuffle */}
      <button
        type="button"
        className={`control-btn ${shuffleActive ? "active" : ""}`}
        disabled={!playback.canShuffle}
        onClick={onToggleShuffle}
        aria-label="Toggle shuffle"
        title={playback.canShuffle ? `Shuffle ${shuffleActive ? "on" : "off"}` : "Shuffle unsupported"}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="16 3 21 3 21 8" />
          <line x1="4" y1="20" x2="21" y2="3" />
          <polyline points="21 16 21 21 16 21" />
          <line x1="15" y1="15" x2="21" y2="21" />
          <line x1="4" y1="4" x2="9" y2="9" />
        </svg>
      </button>

      {/* Previous */}
      <button
        type="button"
        className="control-btn"
        disabled={!playback.canPrev}
        onClick={onPrev}
        aria-label="Previous track"
        title="Previous track"
      >
        <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="19 20 9 12 19 4 19 20" />
          <line x1="5" y1="19" x2="5" y2="5" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
        </svg>
      </button>

      {/* Play / Pause */}
      <button
        type="button"
        className="control-btn control-btn-play-pause"
        disabled={!playback.canPlayPause}
        onClick={onPlayPause}
        aria-label={isPlaying ? "Pause" : "Play"}
        title={isPlaying ? "Pause" : "Play"}
      >
        {isPlaying ? (
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="4" width="4" height="16" rx="1" />
            <rect x="14" y="4" width="4" height="16" rx="1" />
          </svg>
        ) : (
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" style={{ marginLeft: "2px" }}>
            <polygon points="6 3 20 12 6 21 6 3" />
          </svg>
        )}
      </button>

      {/* Next */}
      <button
        type="button"
        className="control-btn"
        disabled={!playback.canNext}
        onClick={onNext}
        aria-label="Next track"
        title="Next track"
      >
        <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="5 4 15 12 5 20 5 4" />
          <line x1="19" y1="5" x2="19" y2="19" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
        </svg>
      </button>

      {/* Repeat */}
      <button
        type="button"
        className={`control-btn ${repeatActive ? "active" : ""}`}
        disabled={!playback.canRepeat}
        onClick={onCycleRepeat}
        aria-label={`Cycle repeat: ${playback.repeat || "off"}`}
        title={playback.canRepeat ? `Repeat: ${playback.repeat || "off"}` : "Repeat unsupported"}
      >
        {playback.repeat === "track" ? (
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="17 1 21 5 17 9" />
            <path d="M3 11V9a4 4 0 0 1 4-4h14" />
            <polyline points="7 23 3 19 7 15" />
            <path d="M21 13v2a4 4 0 0 1-4 4H3" />
            <text x="10" y="15" fontSize="8" fill="currentColor" fontWeight="bold">1</text>
          </svg>
        ) : (
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="17 1 21 5 17 9" />
            <path d="M3 11V9a4 4 0 0 1 4-4h14" />
            <polyline points="7 23 3 19 7 15" />
            <path d="M21 13v2a4 4 0 0 1-4 4H3" />
          </svg>
        )}
      </button>
    </div>
  );
};

export const Controls = React.memo(ControlsComponent);
