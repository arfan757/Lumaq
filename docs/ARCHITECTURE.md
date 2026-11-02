# Lumaq Architecture

This document details the internal design and data flow of Lumaq.

```
+-----------------------------------------------------------------------------------------+
|                                    WINDOWS OS (WinRT)                                   |
|                                                                                         |
|       +-------------------------------------------------------------------------+       |
|       |     GlobalSystemMediaTransportControlsSessionManager (GSMTC)            |       |
|       +-------------------------------------------------------------------------+       |
+--------------------------------------------|--------------------------------------------+
                                             |
                                    Thread-safe WinRT Polling / Events
                                             |
+--------------------------------------------v--------------------------------------------+
|                               LUMAQ RUST BACKEND (src-tauri)                            |
|                                                                                         |
|  +--------------------------------+           +--------------------------------------+  |
|  |   Dedicated GSMTC Thread       |           |          Lyrics Manager              |  |
|  |  (Single-threaded apartment)   |           |  - Request cancellation (request_id) |  |
|  |  - Debounces events (~100ms)   |           |  - Debounce 300ms on track change    |  |
|  |  - Extrapolates position       |           |  - LRCLIB API GET /api/get           |  |
|  |  - Extracts thumbnail -> base64|           |  - LRCLIB API GET /api/search fallbk |  |
|  |  - Ad detection & track hashes |           |  - On-disk LRU cache (24h neg/30d pos|  |
|  +--------------------------------+           +--------------------------------------+  |
|                   |                                                |                    |
|                   +-----------------------+------------------------+                    |
|                                           |                                             |
|                               +-----------v-----------+                                 |
|                               |  Tauri IPC Event Bus  |                                 |
|                               |  - lumaq://now-playing|                                 |
|                               |  - lumaq://playback   |                                 |
|                               |  - lumaq://lyrics     |                                 |
|                               +-----------+-----------+                                 |
+-------------------------------------------|---------------------------------------------+
                                            |
                                  Tauri Events & Commands
                                            |
+-------------------------------------------v---------------------------------------------+
|                                REACT FRONTEND (WebView2)                                |
|                                                                                         |
|   +-------------------+  +--------------------+  +-----------------------------------+  |
|   |  usePlayback Hook |  |   useLyrics Hook   |  |        useSettings Hook           |  |
|   |  - rAF position   |  |  - Binary search   |  |  - In-card & tray synchronization |  |
|   |    extrapolation  |  |    active line     |  |  - Persistence                    |  |
|   +-------------------+  +--------------------+  +-----------------------------------+  |
|                                           |                                             |
|   +---------------------------------------v-----------------------------------------+   |
|   |                             Liquid-Glass Presentation Layer                     |   |
|   |  - Specular 1px rim highlight + inset illumination                              |   |
|   |  - Blurred dynamic album-art backdrop crossfade                                 |   |
|   |  - Fluid 2-line lyrics motion (current bright/italic, next dimmed)              |   |
|   |  - Interactive seeking progress bar & playback controls                         |   |
|   +---------------------------------------------------------------------------------+   |
+-----------------------------------------------------------------------------------------+
```

---

## 1. Concurrency & WinRT COM Safety

WinRT objects such as `GlobalSystemMediaTransportControlsSessionManager` contain non-thread-safe pointers (`*mut c_void`). Passing or awaiting them across multi-threaded Tokio worker threads causes compilation errors or undefined behavior.

To solve this cleanly:
- Lumaq spawns a dedicated native OS thread named `lumaq-gsmtc`.
- The GSMTC thread maintains exclusive ownership of the WinRT session manager and event handlers.
- Communication with the Tokio runtime and Tauri commands occurs via standard thread-safe crossbeam/mpsc message channels (`MediaCommand` and `MediaEvent`).
- Control commands (`play_pause`, `seek`, `next`, `previous`, `shuffle`, `repeat`) are processed synchronously on the GSMTC thread and never block Tokio worker threads.

---

## 2. Media Observation & Extrapolation

1. **Debouncing:** When media tracks switch or seek events occur, Windows can trigger multiple rapid updates with transient empty strings. Lumaq debounces updates (~100ms) and checks track stability (~250ms) before committing track changes.
2. **Timeline Extrapolation:** Windows GSMTC rarely sends continuous timeline updates. Lumaq calculates:
   $$\text{position} = \text{lastPosition} + (\text{now} - \text{lastUpdated}) \times \text{playbackRate}$$
   The Rust backend re-emits a resync event every 2 seconds while playing to prevent clock drift.
3. **Frontend Interpolation:** The frontend runs a `requestAnimationFrame` interpolation loop, advancing the progress bar and timestamp with sub-frame smoothness without overloading the IPC bridge.

---

## 3. Lyrics Architecture

1. **Orchestration:** When a new `trackKey` is identified, previous in-flight HTTP requests are cancelled via an atomic `request_id`.
2. **LRCLIB Client:**
   - Primary: `GET /api/get` with track name, artist name, album name, and duration.
   - Fallback: `GET /api/search` with normalized track name (stripping "Remastered", "feat.", "Deluxe Edition", etc.).
3. **On-Disk Cache:**
   - Stored in `%LOCALAPPDATA%/com.lumaq.desktop/lyrics_cache.json`.
   - Successful lyrics cached for 30 days.
   - Missing/404 lyrics cached for 24 hours to prevent repeated API calls.
   - Total cache size capped at 1,000 entries with LRU eviction.

---

## 4. Frontend & Security Model

- **Content Security Policy:** The frontend is strictly locked down:
  `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'none'; font-src 'self';`
- **Zero Frontend Network Access:** All HTTP requests are performed by the Rust backend. The frontend communicates exclusively via Tauri IPC.
- **Font & Asset Isolation:** All typography (Inter) is bundled locally via `@fontsource/inter`.
