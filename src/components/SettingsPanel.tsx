import React from "react";
import { AppSettings, SourcePreference } from "../lib/types";

interface SettingsPanelProps {
  settings: AppSettings;
  isOpen: boolean;
  onClose: () => void;
  onUpdate: (updated: Partial<AppSettings>) => void;
}

export const SettingsPanel: React.FC<SettingsPanelProps> = ({
  settings,
  isOpen,
  onClose,
  onUpdate,
}) => {
  if (!isOpen) return null;

  return (
    <div
      className="settings-drawer"
      data-tauri-drag-region="false"
      onMouseDown={(e) => e.stopPropagation()}
      onClick={(e) => e.stopPropagation()}
    >
      <div className="settings-header">
        <span className="settings-title">Lumaq Settings</span>
        <button
          type="button"
          className="settings-close-btn"
          onClick={onClose}
          aria-label="Close settings"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <div className="settings-items">
        {/* Source preference */}
        <div className="settings-row">
          <label htmlFor="source-pref">Media Source</label>
          <select
            id="source-pref"
            value={settings.sourcePreference}
            onChange={(e) => onUpdate({ sourcePreference: e.target.value as SourcePreference })}
            style={{
              background: "rgba(255,255,255,0.08)",
              color: "#fff",
              border: "1px solid rgba(255,255,255,0.2)",
              borderRadius: "4px",
              padding: "2px 6px",
              fontSize: "12px",
            }}
          >
            <option value="spotify-first" style={{ background: "#222" }}>Spotify First</option>
            <option value="any" style={{ background: "#222" }}>Any Media Player</option>
          </select>
        </div>

        {/* Lyrics offset */}
        <div className="settings-row">
          <label htmlFor="lyrics-offset">Lyrics Offset ({settings.lyricsOffsetMs}ms)</label>
          <input
            id="lyrics-offset"
            type="range"
            min="-2000"
            max="2000"
            step="50"
            value={settings.lyricsOffsetMs}
            onChange={(e) => onUpdate({ lyricsOffsetMs: parseInt(e.target.value, 10) })}
            style={{ width: "110px", accentColor: "var(--color-accent)" }}
          />
        </div>

        {/* Scale */}
        <div className="settings-row">
          <label htmlFor="widget-scale">Card Scale ({Math.round(settings.scale * 100)}%)</label>
          <input
            id="widget-scale"
            type="range"
            min="0.75"
            max="1.5"
            step="0.05"
            value={settings.scale}
            onChange={(e) => onUpdate({ scale: parseFloat(e.target.value) })}
            style={{ width: "110px", accentColor: "var(--color-accent)" }}
          />
        </div>

        {/* Opacity */}
        <div className="settings-row">
          <label htmlFor="widget-opacity">Opacity ({Math.round(settings.opacity * 100)}%)</label>
          <input
            id="widget-opacity"
            type="range"
            min="0.5"
            max="1.0"
            step="0.05"
            value={settings.opacity}
            onChange={(e) => onUpdate({ opacity: parseFloat(e.target.value) })}
            style={{ width: "110px", accentColor: "var(--color-accent)" }}
          />
        </div>

        {/* Always on top toggle */}
        <div className="settings-row">
          <label htmlFor="always-top">Always on Top</label>
          <input
            id="always-top"
            type="checkbox"
            checked={settings.alwaysOnTop}
            onChange={(e) => onUpdate({ alwaysOnTop: e.target.checked })}
            style={{ accentColor: "var(--color-accent)", transform: "scale(1.2)" }}
          />
        </div>

        {/* Lock Position */}
        <div className="settings-row">
          <label htmlFor="lock-pos">Lock Position</label>
          <input
            id="lock-pos"
            type="checkbox"
            checked={settings.lockPosition}
            onChange={(e) => onUpdate({ lockPosition: e.target.checked })}
            style={{ accentColor: "var(--color-accent)", transform: "scale(1.2)" }}
          />
        </div>

        {/* Click-through mode */}
        <div className="settings-row">
          <label htmlFor="click-thru" title="Recoverable from the tray icon menu">
            Click-through (Tray to undo)
          </label>
          <input
            id="click-thru"
            type="checkbox"
            checked={settings.clickThrough}
            onChange={(e) => onUpdate({ clickThrough: e.target.checked })}
            style={{ accentColor: "var(--color-accent)", transform: "scale(1.2)" }}
          />
        </div>

        {/* Edge Snapping */}
        <div className="settings-row">
          <label htmlFor="edge-snap">Snap to Screen Edges</label>
          <input
            id="edge-snap"
            type="checkbox"
            checked={settings.snapToEdges}
            onChange={(e) => onUpdate({ snapToEdges: e.target.checked })}
            style={{ accentColor: "var(--color-accent)", transform: "scale(1.2)" }}
          />
        </div>
      </div>
    </div>
  );
};

