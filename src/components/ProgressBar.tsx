import React, { useRef, useCallback } from "react";
import { formatTime } from "../lib/interpolation";

interface ProgressBarProps {
  currentMs: number;
  durationMs: number;
  canSeek: boolean;
  onSeek: (ms: number) => void;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  currentMs,
  durationMs,
  canSeek,
  onSeek,
}) => {
  const trackRef = useRef<HTMLDivElement>(null);

  const calculateSeekTime = useCallback(
    (clientX: number): number => {
      if (!trackRef.current || durationMs <= 0) return 0;
      const rect = trackRef.current.getBoundingClientRect();
      const clickX = Math.max(0, Math.min(clientX - rect.left, rect.width));
      const percentage = clickX / rect.width;
      return Math.round(percentage * durationMs);
    },
    [durationMs]
  );

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!canSeek || durationMs <= 0) return;
    e.preventDefault();
    e.stopPropagation();

    const targetMs = calculateSeekTime(e.clientX);
    onSeek(targetMs);

    const onPointerMove = (moveEvent: PointerEvent) => {
      const ms = calculateSeekTime(moveEvent.clientX);
      onSeek(ms);
    };

    const onPointerUp = () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  };

  const progressPercent = durationMs > 0 ? Math.min(100, Math.max(0, (currentMs / durationMs) * 100)) : 0;

  return (
    <div className="progress-section" data-tauri-drag-region="false">
      <div
        ref={trackRef}
        className="progress-bar-track"
        onPointerDown={handlePointerDown}
        role="slider"
        aria-label="Playback progress"
        aria-valuemin={0}
        aria-valuemax={durationMs}
        aria-valuenow={currentMs}
        tabIndex={canSeek ? 0 : -1}
      >
        <div className="progress-bar-fill" style={{ width: `${progressPercent}%` }} />
        <div className="progress-bar-thumb" style={{ left: `${progressPercent}%` }} />
      </div>
      <div className="progress-time-row">
        <span>{formatTime(currentMs)}</span>
        <span>{formatTime(durationMs)}</span>
      </div>
    </div>
  );
};

