import React, { useState } from "react";
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

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setSettingsOpen((prev) => !prev);
  };

  const bgStyle = nowPlaying?.artwork
    ? { backgroundImage: `url(${nowPlaying.artwork})` }
    : undefined;

  return (
    <div
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
