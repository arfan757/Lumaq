import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../lib/types";

const DEFAULT_SETTINGS: AppSettings = {
  scale: 1.0,
  opacity: 1.0,
  alwaysOnTop: false,
  lockPosition: false,
  clickThrough: false,
  autostart: false,
  sourcePreference: "spotify-first",
  lyricsEnabled: true,
  lyricsOffsetMs: 0,
  snapToEdges: true,
  realBlur: false,
  schemaVersion: 2,
};

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const settingsRef = useRef<AppSettings>(settings);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  useEffect(() => {
    invoke<AppSettings>("get_settings")
      .then((loaded) => {
        if (loaded) setSettings(loaded);
      })
      .catch((err) => console.debug("Failed to load settings:", err));
  }, []);

  const updateSettings = useCallback(
    (updatedPartial: Partial<AppSettings>) => {
      const nextSettings: AppSettings = { ...settingsRef.current, ...updatedPartial };
      setSettings(nextSettings);
      invoke("set_settings", { newSettings: nextSettings }).catch(console.debug);
    },
    []
  );

  return {
    settings,
    updateSettings,
  };
}

