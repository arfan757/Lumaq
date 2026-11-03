import React, { useRef, useCallback, useState, useEffect } from "react";
import { formatTime } from "../lib/interpolation";

interface ProgressBarProps {
  currentMs: number;
  durationMs: number;
  canSeek: boolean;
  onSeek: (ms: number) => void;
}

const clamp = (value: number, min: number, max: number): number =>
  Math.min(Math.max(value, min), max);

export const ProgressBar: React.FC<ProgressBarProps> = ({
  currentMs,
  durationMs,
  canSeek,
  onSeek,
}) => {
  const trackRef = useRef<HTMLDivElement>(null);
  const draggingRef = useRef(false);
  const cleanupRef = useRef<(() => void) | null>(null);
  const dragMsRef = useRef<number | null>(null);
  const [dragMs, setDragMs] = useState<number | null>(null);

  const updateDrag = useCallback((ms: number | null) => {
    dragMsRef.current = ms;
    setDragMs(ms);
  }, []);

  useEffect(() => {
    return () => {
      cleanupRef.current?.();
    };
  }, []);

  const calculateSeekTime = useCallback(
    (clientX: number): number => {
      if (!trackRef.current || durationMs <= 0) return 0;
      const rect = trackRef.current.getBoundingClientRect();
      const clickX = clamp(clientX - rect.left, 0, rect.width);
      const percentage = clickX / rect.width;
      return Math.round(percentage * durationMs);
    },
    [durationMs]
  );

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!canSeek || durationMs <= 0) return;
    e.preventDefault();
    e.stopPropagation();

    const target = e.currentTarget;
    const pointerId = e.pointerId;
    try {
      target.setPointerCapture(pointerId);
    } catch {
      // Pointer capture is best-effort; window listeners still work.
    }
    target.focus?.();

    const startMs = calculateSeekTime(e.clientX);
    draggingRef.current = true;
    updateDrag(startMs);

    const onPointerMove = (moveEvent: PointerEvent) => {
      if (!draggingRef.current) return;
      updateDrag(calculateSeekTime(moveEvent.clientX));
    };

    const releaseCapture = () => {
      try {
        target.releasePointerCapture(pointerId);
      } catch {
        // Already released.
      }
    };

    const cleanup = () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", onPointerCancel);
      releaseCapture();
      cleanupRef.current = null;
    };

    const finish = (upEvent: PointerEvent | null) => {
      if (!draggingRef.current) return;
      draggingRef.current = false;
      const finalMs = upEvent
        ? calculateSeekTime(upEvent.clientX)
        : dragMsRef.current ?? startMs;
      updateDrag(null);
      cleanup();
      onSeek(finalMs);
    };

    const onPointerUp = (upEvent: PointerEvent) => finish(upEvent);
    const onPointerCancel = () => finish(null);

    cleanupRef.current?.();
    cleanupRef.current = cleanup;
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerCancel);
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    // Stop the native event before it reaches Tauri's document-level
    // drag-region listener, which would otherwise move the whole widget.
    e.stopPropagation();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (!canSeek || durationMs <= 0) return;
    const step = e.shiftKey ? 10_000 : 5_000;
    if (e.key === "ArrowRight") {
      e.preventDefault();
      onSeek(clamp(currentMs + step, 0, durationMs));
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      onSeek(clamp(currentMs - step, 0, durationMs));
    } else if (e.key === "Home") {
      e.preventDefault();
      onSeek(0);
    } else if (e.key === "End") {
      e.preventDefault();
      onSeek(durationMs);
    }
  };

  const displayMs = dragMs ?? currentMs;
  const progressPercent =
    durationMs > 0 ? clamp((displayMs / durationMs) * 100, 0, 100) : 0;

  return (
    <div
      className="progress-section"
      data-tauri-drag-region="false"
      onMouseDown={handleMouseDown}
    >
      <div
        className={`progress-hit-area${dragMs !== null ? " dragging" : ""}`}
        data-tauri-drag-region="false"
        onPointerDown={handlePointerDown}
        onMouseDown={handleMouseDown}
        onKeyDown={handleKeyDown}
        role="slider"
        aria-label="Playback progress"
        aria-valuemin={0}
        aria-valuemax={durationMs}
        aria-valuenow={Math.round(displayMs)}
        tabIndex={canSeek ? 0 : -1}
      >
        <div ref={trackRef} className="progress-bar-track">
          <div className="progress-bar-fill" style={{ width: `${progressPercent}%` }} />
          <div className="progress-bar-thumb" style={{ left: `${progressPercent}%` }} />
        </div>
      </div>
      <div className="progress-time-row">
        <span>{formatTime(displayMs)}</span>
        <span>{formatTime(durationMs)}</span>
      </div>
    </div>
  );
};
