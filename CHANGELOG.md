# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-03

### Added
- Liquid-glass desktop widget UI for Windows with frosted glass backdrop, specular rim highlight, and album art crossfade.
- Windows Global System Media Transport Controls (GSMTC) integration for zero-login, zero-API-key playback tracking.
- Spotify-first media session selection with dynamic fallback to any active media player.
- Two-line time-synced lyrics display powered by LRCLIB API with automatic title/artist normalization and fallback search.
- In-memory and on-disk LRU lyrics caching with 24-hour negative caching and 30-day positive caching.
- Concurrent lyrics request cancellation using monotonic atomic request IDs and debounce timer.
- Interactive playback controls: play/pause, next, previous, seek slider, shuffle, and repeat modes.
- Frame-rate synchronized (`requestAnimationFrame`) position interpolation with 2-second backend resync.
- Spotify advertisement detection (`isAd = true`) skipping lyrics lookup.
- Smart edge snapping (within 12px) and multi-monitor window bounds recovery on startup.
- Recoverable click-through mode with System Tray management.
- Unit tests for LRC parser, normalizer, cache, settings, and frontend interpolation logic.
- GitHub Actions CI workflow for frontend and backend validation.
