use super::lrc::{parse_lrc, LyricLine};
use super::normalize::{normalize_album, normalize_artist, normalize_title};
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, warn};

const LRCLIB_BASE_URL: &str = "https://lrclib.net/api";
const USER_AGENT: &str = "Lumaq/0.1.0 (https://github.com/lumaq/lumaq)";
const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LrclibResponse {
    pub id: Option<u64>,
    pub track_name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    pub instrumental: Option<bool>,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchResult {
    Synced(Vec<LyricLine>),
    Plain(String),
    Instrumental,
    NotFound,
    Error(String),
}

pub struct LrclibClient {
    client: reqwest::Client,
}

impl Default for LrclibClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LrclibClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_default();

        Self { client }
    }

    pub async fn fetch_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        duration_ms: u64,
    ) -> FetchResult {
        let duration_secs = (duration_ms / 1000) as i64;

        // 1. Primary lookup: GET /api/get
        let primary_res = self.request_get(title, artist, album, duration_secs).await;

        match primary_res {
            Ok(Some(response)) => {
                return Self::process_response(response);
            }
            Ok(None) => {
                debug!("Primary LRCLIB /api/get returned 404; falling back to /api/search");
            }
            Err(e) => {
                warn!("Primary LRCLIB request error: {}", e);
            }
        }

        // 2. Fallback lookup: GET /api/search with normalized title/artist
        let norm_title = normalize_title(title);
        let norm_artist = normalize_artist(artist);
        let norm_album = normalize_album(album);

        let search_res = self
            .request_search(&norm_title, &norm_artist, &norm_album, duration_secs)
            .await;

        match search_res {
            Ok(Some(response)) => Self::process_response(response),
            Ok(None) => FetchResult::NotFound,
            Err(e) => FetchResult::Error(e),
        }
    }

    async fn request_get(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        duration_secs: i64,
    ) -> Result<Option<LrclibResponse>, String> {
        let mut url = format!(
            "{}/get?track_name={}&artist_name={}",
            LRCLIB_BASE_URL,
            urlencoding(title),
            urlencoding(artist)
        );

        if !album.is_empty() {
            url.push_str(&format!("&album_name={}", urlencoding(album)));
        }
        if duration_secs > 0 {
            url.push_str(&format!("&duration={}", duration_secs));
        }

        self.send_with_retry(&url).await
    }

    async fn request_search(
        &self,
        title: &str,
        artist: &str,
        _album: &str,
        duration_secs: i64,
    ) -> Result<Option<LrclibResponse>, String> {
        let url = format!(
            "{}/search?track_name={}&artist_name={}",
            LRCLIB_BASE_URL,
            urlencoding(title),
            urlencoding(artist)
        );

        let res = self.send_with_retry_raw(&url).await?;
        if res.status().as_u16() == 404 {
            return Ok(None);
        }

        if !res.status().is_success() {
            return Err(format!("LRCLIB search returned HTTP {}", res.status()));
        }

        let items: Vec<LrclibResponse> = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse LRCLIB search JSON: {}", e))?;

        if items.is_empty() {
            return Ok(None);
        }

        // Pick best match within +/- 3s duration tolerance if duration is known
        if duration_secs > 0 {
            for item in &items {
                if let Some(d) = item.duration {
                    let diff = (d.round() as i64 - duration_secs).abs();
                    if diff <= 3 {
                        return Ok(Some(item.clone()));
                    }
                }
            }
        }

        // Otherwise pick the first result that has synced or plain lyrics
        for item in &items {
            if item.synced_lyrics.is_some() || item.plain_lyrics.is_some() {
                return Ok(Some(item.clone()));
            }
        }

        Ok(items.into_iter().next())
    }

    async fn send_with_retry(&self, url: &str) -> Result<Option<LrclibResponse>, String> {
        let res = self.send_with_retry_raw(url).await?;

        if res.status().as_u16() == 404 {
            return Ok(None);
        }

        if !res.status().is_success() {
            return Err(format!("LRCLIB API returned HTTP {}", res.status()));
        }

        let body: LrclibResponse = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse LRCLIB JSON: {}", e))?;

        Ok(Some(body))
    }

    async fn send_with_retry_raw(&self, url: &str) -> Result<reqwest::Response, String> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.client.get(url).send().await {
                Ok(res) => {
                    let status = res.status();
                    // Do not retry 404 or other client errors (4xx)
                    if status.is_client_error() {
                        return Ok(res);
                    }
                    // If 5xx server error and attempt == 1, retry once after backoff
                    if status.is_server_error() && attempt == 1 {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        continue;
                    }
                    return Ok(res);
                }
                Err(e) => {
                    if attempt == 1 {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        continue;
                    }
                    return Err(format!("Network error contacting LRCLIB: {}", e));
                }
            }
        }
    }

    fn process_response(resp: LrclibResponse) -> FetchResult {
        if resp.instrumental.unwrap_or(false) {
            return FetchResult::Instrumental;
        }

        if let Some(synced) = resp.synced_lyrics {
            let lines = parse_lrc(&synced);
            if !lines.is_empty() {
                return FetchResult::Synced(lines);
            }
        }

        if let Some(plain) = resp.plain_lyrics {
            let trimmed = plain.trim();
            if !trimmed.is_empty() {
                return FetchResult::Plain(trimmed.to_string());
            }
        }

        FetchResult::NotFound
    }
}

fn urlencoding(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.as_bytes() {
        match *b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}
