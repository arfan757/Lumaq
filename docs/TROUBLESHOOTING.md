# Troubleshooting Lumaq

Common issues and their solutions.

---

### 1. "Nothing playing" shown while Spotify is playing music

- **Cause:** Spotify's desktop overlay / media keys integration may be disabled, or Windows GSMTC hasn't registered the session.
- **Solution:**
  1. In the Spotify desktop app, go to **Settings** (`Ctrl + P`).
  2. Scroll to **Display Options**.
  3. Ensure **"Show desktop overlay when using media keys"** is enabled.
  4. Try pressing a keyboard media key (e.g. Play/Pause) once to wake the Windows media transport session.
  5. If running Spotify via Microsoft Store vs standalone installer, both work, but restarting Spotify once can force GSMTC registration.

---

### 2. Click-Through mode is active and I can't click on Lumaq

- **Cause:** In click-through mode, mouse events pass straight through the window so you can interact with background apps unimpeded.
- **Solution:**
  1. Look at the Windows **System Tray** (bottom-right of your taskbar, inside the `^` notification area if hidden).
  2. Right-click the **Lumaq** tray icon.
  3. Click **"Click-through (Off)"** to restore normal mouse interaction.

---

### 3. Widget disappeared or is off-screen after unplugging a monitor

- **Cause:** The saved window coordinates may reside on a disconnected display.
- **Solution:**
  1. Restart Lumaq. On startup, Lumaq automatically checks display coordinates against active monitors. If the window coordinates fall outside all current monitor viewports, Lumaq resets the position to the bottom-left of your primary screen.
  2. Alternatively, right-click the tray icon and toggle **"Show Widget"**.

---

### 4. Lyrics are not showing or show "Lyrics not found"

- **Cause:** LRCLIB does not have user-submitted lyrics for this specific track or recording.
- **Solution:**
  1. Right-click the card and click **"Retry lyrics"** to refresh.
  2. Instrumental tracks will automatically display *"Instrumental track"*.
  3. For newly released or niche tracks, lyrics may not yet be available in the public LRCLIB database.

---

### 5. High CPU usage or stuttering animations

- **Solution:**
  1. In the Lumaq Settings drawer, ensure **"Enable Real Blur"** is disabled if your GPU does not handle translucent acrylic blur smoothly.
  2. Check **Windows Settings > Accessibility > Visual Effects** and verify **Animation effects** and **Transparency effects** are turned on.

