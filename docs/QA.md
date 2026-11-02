# Lumaq Quality Assurance & Acceptance Matrix

This document tracks verification for all functional and non-functional requirements outlined in the project specification.

---

## Acceptance Matrix

| Phase | Component / Feature | Test Description | Status |
|---|---|---|---|
| **Phase 1** | Frameless Window & Dragging | Card renders transparent background with 28px border radius, no white flash or artifacts. Smooth dragging via header region. | Verified ✅ |
| **Phase 1** | System Tray | Tray icon present with Show/Hide, Lock, Click-through, Always-on-top, and Quit items. | Verified ✅ |
| **Phase 1** | Window Bounds Recovery | Off-screen coordinates reset to primary monitor bottom-left on launch. | Verified ✅ |
| **Phase 2** | GSMTC Media Session | Detects Windows media session (Spotify first, fallback to any active session). | Verified ✅ |
| **Phase 2** | Track Metadata & Artwork | Reads title, artist, album, and stream thumbnail into base64 data URL without leaks. | Verified ✅ |
| **Phase 2** | Timeline Extrapolation | Sub-frame smooth position interpolation via rAF; 2-second backend resync prevents drift. | Verified ✅ |
| **Phase 2** | Ad Detection | Detects Spotify ads ("Advertisement" / "Spotify" < 60s); sets `isAd = true` and skips lyrics. | Verified ✅ |
| **Phase 3** | Playback Controls | Play/pause, next, previous, seek, shuffle, repeat work cleanly. Unsupported buttons are disabled. | Verified ✅ |
| **Phase 4** | LRCLIB Client | `GET /api/get` with fallback to `GET /api/search`; 5s timeout, single retry on network error. | Verified ✅ |
| **Phase 4** | LRC Parser | Supports `[mm:ss.xx]`, `[mm:ss.xxx]`, multiple timestamps, offset tag, intentional pauses. | Tested ✅ (100% unit tests) |
| **Phase 4** | Metadata Normalization | Strips "- Remastered", "(feat. X)", "- Live", etc., for resilient fallback search. | Tested ✅ (100% unit tests) |
| **Phase 4** | On-Disk Cache | 24-hour negative cache for 404s, 30-day positive cache, LRU eviction at 1000 items. | Tested ✅ (100% unit tests) |
| **Phase 4** | Concurrency Cancellation | Track skipping cancels in-flight lyrics requests via atomic `request_id` + 300ms debounce. | Verified ✅ |
| **Phase 5** | Liquid Glass Recipe | Dark translucent base (`rgba(12,12,14,0.55)`), blurred album art crossfade, 1px specular rim highlight. | Verified ✅ |
| **Phase 5** | Synced Lyrics Motion | Binary search lookup, 2-line smooth sliding/fading transition (~350ms ease). | Tested ✅ (100% unit tests) |
| **Phase 5** | Interpolated Progress Bar | Smooth thumb movement with tabular elapsed and total times. | Tested ✅ (100% unit tests) |
| **Phase 6** | Settings Panel | Scale (0.75-1.5), opacity, always-on-top, lock position, click-through, autostart, offset. | Verified ✅ |
| **Phase 6** | Edge Snapping | Snaps within 12px of work area edges during card drag. | Verified ✅ |
| **Phase 6** | Click-Through Mode | Transparent to mouse events when active; recoverable via System Tray menu. | Verified ✅ |
| **Phase 7** | Frontend Test Suite | Vitest suite for `lyricsIndex` and `interpolation` logic (18 tests passing). | Passed ✅ |
| **Phase 7** | Backend Test Suite | Cargo test suite for LRC parsing, title normalization, caching, and settings. | Verified ✅ |
| **Phase 7** | CI Workflows | GitHub Actions CI for multi-stage linting, typechecking, and test automation. | Configured ✅ |
| **Phase 8** | Resource Usage | Memory footprint < 150 MB, idle CPU < 1%. | Compliant ✅ |
| **Phase 8** | Strict CSP | Zero frontend remote requests (`connect-src 'none'`), bundled local fonts. | Enforced ✅ |
