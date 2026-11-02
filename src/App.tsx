import React from "react";
import { usePlayback } from "./hooks/usePlayback";
import { useLyrics } from "./hooks/useLyrics";
import { useSettings } from "./hooks/useSettings";
import { Card } from "./components/Card";

export const App: React.FC = () => {
  const {
    nowPlaying,
    playback,
    interpolatedPositionMs,
    playPause,
    next,
    prev,
    seek,
    toggleShuffle,
    cycleRepeat,
  } = usePlayback();

  const { settings, updateSettings } = useSettings();

  const { lyricsState, activeLyricIndex } = useLyrics(
    nowPlaying,
    interpolatedPositionMs,
    settings.lyricsOffsetMs
  );

  return (
    <div className="app-window" data-tauri-drag-region={!settings.lockPosition}>
      <Card
        nowPlaying={nowPlaying}
        playback={playback}
        lyricsState={lyricsState}
        activeLyricIndex={activeLyricIndex}
        interpolatedPositionMs={interpolatedPositionMs}
        settings={settings}
        onPlayPause={playPause}
        onNext={next}
        onPrev={prev}
        onSeek={seek}
        onToggleShuffle={toggleShuffle}
        onCycleRepeat={cycleRepeat}
        onUpdateSettings={updateSettings}
      />
    </div>
  );
};

export default App;

