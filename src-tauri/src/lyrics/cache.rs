use super::lrc::LyricLine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const NOT_FOUND_TTL_SECS: u64 = 24 * 3600; // 24 hours
const SUCCESS_TTL_SECS: u64 = 30 * 24 * 3600; // 30 days
const MAX_CACHE_ENTRIES: usize = 2000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CachedLyrics {
    Synced { lines: Vec<LyricLine> },
    Plain { text: String },
    Instrumental,
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub cached: CachedLyrics,
    pub fetched_at_secs: u64,
}

pub struct LyricsDiskCache {
    cache_dir: PathBuf,
}

impl LyricsDiskCache {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let cache_dir = app_data_dir.join("lyrics_cache");
        let _ = fs::create_dir_all(&cache_dir);
        Self { cache_dir }
    }

    fn entry_path(&self, track_key: &str) -> PathBuf {
        // Sanitize track_key for safe filesystem naming
        let safe_key: String = track_key
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        self.cache_dir.join(format!("{}.json", safe_key))
    }

    pub fn get(&self, track_key: &str) -> Option<CachedLyrics> {
        let path = self.entry_path(track_key);
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let ttl = match entry.cached {
            CachedLyrics::NotFound => NOT_FOUND_TTL_SECS,
            _ => SUCCESS_TTL_SECS,
        };

        if now_secs.saturating_sub(entry.fetched_at_secs) > ttl {
            let _ = fs::remove_file(&path);
            return None;
        }

        Some(entry.cached)
    }

    pub fn put(&self, track_key: &str, cached: CachedLyrics) {
        let path = self.entry_path(track_key);
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = CacheEntry {
            cached,
            fetched_at_secs: now_secs,
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = fs::write(&path, json);
        }

        self.evict_if_needed();
    }

    fn evict_if_needed(&self) {
        let entries = match fs::read_dir(&self.cache_dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let mtime = meta.modified().unwrap_or(UNIX_EPOCH);
                    files.push((entry.path(), mtime));
                }
            }
        }

        if files.len() > MAX_CACHE_ENTRIES {
            // Sort oldest first
            files.sort_by_key(|(_, mtime)| *mtime);
            let to_remove = files.len() - MAX_CACHE_ENTRIES;
            for (path, _) in files.into_iter().take(to_remove) {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_get_and_expiration() {
        let tmp = std::env::temp_dir().join(format!("lumaq_test_cache_{}", std::process::id()));
        let cache = LyricsDiskCache::new(tmp.clone());

        let lines = vec![LyricLine {
            time_ms: 1000,
            text: "Hello".to_string(),
        }];

        cache.put(
            "track_123",
            CachedLyrics::Synced {
                lines: lines.clone(),
            },
        );
        let retrieved = cache.get("track_123");

        assert!(matches!(retrieved, Some(CachedLyrics::Synced { .. })));

        // Test non-existent track
        assert!(cache.get("non_existent").is_none());

        let _ = fs::remove_dir_all(tmp);
    }
}
