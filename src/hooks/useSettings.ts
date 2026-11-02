import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../lib/types";

const DEFAULT_SETTINGS: AppSettings = {
  scale: 1.0,
  opacity: 1.0,
  alwaysOnTop: true,
  lockPosition: false,
  clickThrough: false,
  autostart: false,
  sourcePreference: "spotify-first",
  lyricsEnabled: true,
  lyricsOffsetMs: 0,
  snapToEdges: true,
  realBlur: false,
  schemaVersion: 1,
};

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);

  useEffect(() => {
    invoke<AppSettings>("get_settings")
      .then((loaded) => {
        if (loaded) setSettings(loaded);
      })
      .catch((err) => console.debug("Failed to load settings:", err));
  }, []);

  const updateSettings = useCallback(
    (updatedPartial: Partial<AppSettings>) => {
      setSettings((prev) => {
        const nextSettings: AppSettings = { ...prev, ...updatedPartial };
        invoke("set_settings", { newSettings: nextSettings }).catch(console.debug);
        return nextSettings;
      });
    },
    []
  );

  return {
    settings,
    updateSettings,
  };
}

