use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SourcePreference {
    #[default]
    SpotifyFirst,
    Any,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub scale: f64,   // 0.75 to 1.5
    pub opacity: f64, // 0.5 to 1.0
    pub always_on_top: bool,
    pub lock_position: bool,
    pub click_through: bool,
    pub autostart: bool,
    pub source_preference: SourcePreference,
    pub lyrics_enabled: bool,
    pub lyrics_offset_ms: i64, // -2000 to +2000
    pub snap_to_edges: bool,
    pub real_blur: bool, // experimental acrylic
    pub schema_version: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            opacity: 1.0,
            always_on_top: false,
            lock_position: false,
            click_through: false,
            autostart: false,
            source_preference: SourcePreference::SpotifyFirst,
            lyrics_enabled: true,
            lyrics_offset_ms: 0,
            snap_to_edges: true,
            real_blur: false,
            schema_version: 2,
        }
    }
}

impl AppSettings {
    pub fn validate_and_clamp(&mut self) {
        self.scale = self.scale.clamp(0.75, 1.5);
        self.opacity = self.opacity.clamp(0.5, 1.0);
        self.lyrics_offset_ms = self.lyrics_offset_ms.clamp(-2000, 2000);
    }

    pub fn load_or_default(path: &PathBuf) -> Self {
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
                Ok(mut settings) => {
                    // v1 defaulted to always-on-top. Widgets live on the desktop
                    // now, so move existing installs to the new default.
                    if settings.schema_version < 2 {
                        settings.always_on_top = false;
                        settings.schema_version = 2;
                    }
                    settings.validate_and_clamp();
                    settings
                }
                Err(err) => {
                    warn!("Corrupt settings file, falling back to defaults: {}", err);
                    Self::default()
                }
            },
            Err(err) => {
                warn!("Failed to read settings file: {}", err);
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(path, json).map_err(|e| format!("Failed to write settings to disk: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_validation_and_clamping() {
        let mut settings = AppSettings {
            scale: 2.5,
            opacity: 0.1,
            lyrics_offset_ms: 9000,
            ..Default::default()
        };

        settings.validate_and_clamp();
        assert_eq!(settings.scale, 1.5);
        assert_eq!(settings.opacity, 0.5);
        assert_eq!(settings.lyrics_offset_ms, 2000);
    }

    #[test]
    fn test_corrupt_settings_fallback() {
        let tmp = std::env::temp_dir().join(format!(
            "lumaq_corrupt_settings_{}.json",
            std::process::id()
        ));
        fs::write(&tmp, "{ invalid json content").unwrap();

        let loaded = AppSettings::load_or_default(&tmp);
        assert_eq!(loaded, AppSettings::default());

        let _ = fs::remove_file(tmp);
    }
}
