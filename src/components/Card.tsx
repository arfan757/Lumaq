import React, { useState, useRef, useEffect } from "react";

// Extend window to carry the liquidGlass function injected by liquid-glass.js
declare global {
  interface Window {
    liquidGlass?: (
      el: HTMLElement,
      opts?: {
        scale?: number;
        chroma?: number;
        border?: number;
        mapBlur?: number;
        blur?: number;
        saturate?: number;
        radius?: number | string;
        fallbackBlur?: number;
      }
    ) => { supported: boolean; refresh: () => void; destroy: () => void };
  }
}
import { NowPlaying, PlaybackState, LyricsState, AppSettings } from "../lib/types";
import { Artwork } from "./Artwork";
import { LyricsView } from "./LyricsView";
import { ProgressBar } from "./ProgressBar";
import { Controls } from "./Controls";
import { SettingsPanel } from "./SettingsPanel";
interface CardProps {
  nowPlaying: NowPlaying | null;
  playback: PlaybackState;
  lyricsState: LyricsState | null;
  activeLyricIndex: number;
  interpolatedPositionMs: number;
  settings: AppSettings;
  onPlayPause: () => void;
  onNext: () => void;
  onPrev: () => void;
  onSeek: (ms: number) => void;
  onToggleShuffle: () => void;
  onCycleRepeat: () => void;
  onUpdateSettings: (updated: Partial<AppSettings>) => void;
}

export const Card: React.FC<CardProps> = ({
  nowPlaying,
  playback,
  lyricsState,
  activeLyricIndex,
  interpolatedPositionMs,
  settings,
  onPlayPause,
  onNext,
  onPrev,
  onSeek,
  onToggleShuffle,
  onCycleRepeat,
  onUpdateSettings,
}) => {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const glassRef = useRef<{ destroy: () => void } | null>(null);

  // Wire liquid-glass.js refraction effect onto the card element.
  // The script is loaded via index.html <script> so window.liquidGlass is available.
  useEffect(() => {
    const el = containerRef.current;
    if (!el || !window.liquidGlass) return;

    glassRef.current = window.liquidGlass(el, {
      scale: -90,      // refraction strength — subtle so text stays legible
      chroma: 5,       // chromatic fringe width
      border: 0.12,    // fraction of element size for rim band
      mapBlur: 18,     // softness of the refraction edge
      blur: 14,        // frosted blur behind the glass
      saturate: 1.5,   // colour pop
      fallbackBlur: 20, // Safari/Firefox fallback
    });

    return () => {
      glassRef.current?.destroy();
      glassRef.current = null;
    };
  }, []); // run once on mount

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setSettingsOpen((prev) => !prev);
  };

  const bgStyle = nowPlaying?.artwork
    ? { backgroundImage: `url(${nowPlaying.artwork})` }
    : undefined;

  return (
    <div
      ref={containerRef}
      className="glass-container card-root"
      data-tauri-drag-region={!settings.lockPosition}
      onContextMenu={handleContextMenu}
      style={{
        transform: `scale(${settings.scale})`,
        opacity: settings.opacity,
        transformOrigin: "center center",
      }}
    >
      {/* Blurred artwork background layer */}
      <div
        className={`glass-artwork-bg ${!nowPlaying?.artwork ? "fallback" : ""}`}
        style={bgStyle}
        data-tauri-drag-region={!settings.lockPosition}
      />

      {/* Dark base tint layer */}
      <div className="glass-tint" data-tauri-drag-region={!settings.lockPosition} />

      {/* Diagonal sheen */}
      <div className="glass-sheen" data-tauri-drag-region={!settings.lockPosition} />

      {/* 1px Specular rim light */}
      <div className="glass-rim" data-tauri-drag-region={!settings.lockPosition} />

      {/* Inset highlight and glow */}
      <div className="glass-highlights" data-tauri-drag-region={!settings.lockPosition} />

      {/* Glass content surface */}
      <div className="glass-content" data-tauri-drag-region={!settings.lockPosition}>
        {/* Top row: Artwork + Lyrics */}
        <div className="card-top-row" data-tauri-drag-region={!settings.lockPosition}>
          <Artwork artwork={nowPlaying?.artwork ?? null} title={nowPlaying?.title} />
          <LyricsView
            lyricsState={lyricsState}
            nowPlaying={nowPlaying}
            activeIndex={activeLyricIndex}
          />
        </div>

        {/* Middle row: Progress bar */}
        <ProgressBar
          currentMs={interpolatedPositionMs}
          durationMs={nowPlaying?.durationMs ?? 0}
          canSeek={playback.canSeek}
          onSeek={onSeek}
        />

        {/* Bottom row: Playback controls */}
        <Controls
          playback={playback}
          onPlayPause={onPlayPause}
          onNext={onNext}
          onPrev={onPrev}
          onToggleShuffle={onToggleShuffle}
          onCycleRepeat={onCycleRepeat}
        />
      </div>

      {/* Settings Overlay Drawer */}
      <SettingsPanel
        settings={settings}
        isOpen={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        onUpdate={onUpdateSettings}
      />
    </div>
  );
};

