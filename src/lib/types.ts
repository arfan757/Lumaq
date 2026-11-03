export type PlaybackStatus = "playing" | "paused" | "stopped" | "none";
export type RepeatMode = "off" | "track" | "list";

export interface NowPlaying {
  trackKey: string; // hash of title|artist|album|durationMs
  title: string;
  artist: string;
  album: string;
  durationMs: number;
  artwork: string | null; // data URL
  isAd: boolean;
  sourceApp: string; // e.g. "Spotify.exe"
}

export interface PlaybackState {
  status: PlaybackStatus;
  positionMs: number; // already extrapolated at emit time
  rate: number; // playback rate, default 1
  shuffle: boolean | null; // null = unsupported
  repeat: RepeatMode | null;
  canPlayPause: boolean;
  canNext: boolean;
  canPrev: boolean;
  canSeek: boolean;
  canShuffle: boolean;
  canRepeat: boolean;
}

export interface LyricLine {
  timeMs: number;
  text: string;
}

export type LyricsState =
  | { kind: "loading"; trackKey: string; track_key?: string }
  | { kind: "synced"; trackKey: string; track_key?: string; lines: LyricLine[] }
  | { kind: "plain"; trackKey: string; track_key?: string; text: string }
  | { kind: "instrumental"; trackKey: string; track_key?: string }
  | { kind: "notFound"; trackKey: string; track_key?: string }
  | { kind: "error"; trackKey: string; track_key?: string; message: string };

export type SourcePreference = "spotify-first" | "any";

export interface AppSettings {
  scale: number; // 0.75 to 1.5
  opacity: number; // 0.5 to 1.0
  alwaysOnTop: boolean;
  lockPosition: boolean;
  clickThrough: boolean;
  autostart: boolean;
  sourcePreference: SourcePreference;
  lyricsEnabled: boolean;
  lyricsOffsetMs: number; // -2000 to +2000
  snapToEdges: boolean;
  realBlur: boolean; // experimental
  schemaVersion: number;
}

export interface Snapshot {
  nowPlaying: NowPlaying | null;
  playback: PlaybackState;
  lyrics: LyricsState | null;
  settings: AppSettings;
}

