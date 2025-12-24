import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import './App.css';

// --- Icons (Feather Icons style) ---
const SunIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="5" /><line x1="12" y1="1" x2="12" y2="3" /><line x1="12" y1="21" x2="12" y2="23" /><line x1="4.22" y1="4.22" x2="5.64" y2="5.64" /><line x1="18.36" y1="18.36" x2="19.78" y2="19.78" /><line x1="1" y1="12" x2="3" y2="12" /><line x1="21" y1="12" x2="23" y2="12" /><line x1="4.22" y1="19.78" x2="5.64" y2="18.36" /><line x1="18.36" y1="5.64" x2="19.78" y2="4.22" /></svg>
);
const MoonIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" /></svg>
);
const FolderIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" /></svg>
);
const FileIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" /><polyline points="13 2 13 9 20 9" /></svg>
);
const PlayIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="5 3 19 12 5 21 5 3" /></svg>
);
const CpuIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="4" y="4" width="16" height="16" rx="2" ry="2" /><rect x="9" y="9" width="6" height="6" /><line x1="9" y1="1" x2="9" y2="4" /><line x1="15" y1="1" x2="15" y2="4" /><line x1="9" y1="20" x2="9" y2="23" /><line x1="15" y1="20" x2="15" y2="23" /><line x1="20" y1="9" x2="23" y2="9" /><line x1="20" y1="14" x2="23" y2="14" /><line x1="1" y1="9" x2="4" y2="9" /><line x1="1" y1="14" x2="4" y2="14" /></svg>
);
const SettingsIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
);


function App() {
  const [theme, setTheme] = useState<'light' | 'dark'>('dark');
  const [gameName, setGameName] = useState('');
  const [romPath, setRomPath] = useState('');
  const [emulatorPath, setEmulatorPath] = useState('');
  const [biosPath, setBiosPath] = useState('');
  const [status, setStatus] = useState<string | null>(null);
  const [isSuccess, setIsSuccess] = useState(false);
  const [isForging, setIsForging] = useState(false);

  // Initialize theme
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme(prev => prev === 'light' ? 'dark' : 'light');
  };

  async function selectRom() {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Game ROM',
        extensions: ['iso', 'cso', 'bin', 'nsp', 'xci', 'rvz', 'wbfs', 'chd']
      }]
    });
    if (selected) {
      // Just take the first selection if somehow multiple return (though multiple: false)
      const pathStr = Array.isArray(selected) ? selected[0] : (selected as string);
      if (!pathStr) return;

      setRomPath(pathStr);
      if (!gameName) {
        const filename = pathStr.split(/[\\/]/).pop();
        if (filename) {
          const name = filename.replace(/\.[^/.]+$/, "").replace(/_/g, " ");
          setGameName(name);
        }
      }
    }
  }

  async function selectEmulator() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Executable', extensions: ['exe', ''] }]
    });
    if (selected) setEmulatorPath(selected as string);
  }

  async function selectBios() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'BIOS File', extensions: ['bin', 'rom', 'img', 'bios'] }]
    });
    if (selected) setBiosPath(selected as string);
  }

  async function handleForge() {
    if (!gameName || !romPath || !emulatorPath) {
      setStatus('Please complete all required fields.');
      setIsSuccess(false);
      return;
    }

    setIsForging(true);
    setStatus(null);

    try {
      const result = await invoke('forge_executable', {
        gameName,
        emulatorPath,
        romPath,
        biosPath: biosPath || null,
        outputDir: 'output',
        args: []
      });
      setStatus(`Success! Executable ready at: ${result}`);
      setIsSuccess(true);
    } catch (error) {
      setStatus(`Error: ${error}`);
      setIsSuccess(false);
    } finally {
      setIsForging(false);
    }
  }

  return (
    <div className="app-container">
      {/* Top Bar */}
      <div className="top-bar">
        <div className="brand">
          <span className="logo-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z" /></svg>
          </span>
          <span className="brand-text">EmuForge</span>
        </div>
        <button className="theme-toggle" onClick={toggleTheme} title="Toggle Theme">
          {theme === 'light' ? <MoonIcon /> : <SunIcon />}
        </button>
      </div>

      {/* Main Content */}
      <div className="main-card">
        <div className="header-section">
          <h1 className="app-title">Create Standalone Game</h1>
          <p className="app-subtitle">Bundle your emulator and ROM into a single executable.</p>
        </div>

        <div className="form-grid">

          {/* Game Title */}
          <div className="input-wrapper">
            <div className="label-row"><label>Game Title</label></div>
            <div className="input-container">
              <input
                type="text"
                className="text-input"
                placeholder="e.g. God of War"
                value={gameName}
                onChange={(e) => setGameName(e.target.value)}
              />
              {/* No icon needed for simple text input, looks cleaner */}
            </div>
          </div>

          {/* ROM File */}
          <div className="input-wrapper">
            <div className="label-row"><label>ROM Location</label></div>
            <div className="input-container">
              <input
                type="text"
                className="text-input"
                placeholder="/path/to/game.iso"
                value={romPath}
                onChange={(e) => setRomPath(e.target.value)}
              />
              <button className="input-action-btn" onClick={selectRom} title="Browse File">
                <FolderIcon />
              </button>
            </div>
          </div>

          {/* Emulator & BIOS Row */}
          <div className="row">
            <div className="col input-wrapper">
              <div className="label-row"><label>Emulator Binary</label></div>
              <div className="input-container">
                <input
                  type="text"
                  className="text-input"
                  placeholder="Select exe..."
                  value={emulatorPath}
                  onChange={(e) => setEmulatorPath(e.target.value)}
                />
                <button className="input-action-btn" onClick={selectEmulator} title="Browse Executable">
                  <SettingsIcon />
                </button>
              </div>
            </div>

            <div className="col input-wrapper">
              <div className="label-row"><label>BIOS (Optional)</label></div>
              <div className="input-container">
                <input
                  type="text"
                  className="text-input"
                  placeholder="Select BIOS..."
                  value={biosPath}
                  onChange={(e) => setBiosPath(e.target.value)}
                />
                <button className="input-action-btn" onClick={selectBios} title="Browse BIOS">
                  <CpuIcon />
                </button>
              </div>
            </div>
          </div>
        </div>

        <hr className="divider" />

        <button
          className="forge-btn"
          onClick={handleForge}
          disabled={isForging}
        >
          {isForging ? <div className="spinner"></div> : <PlayIcon />}
          <span>{isForging ? 'Forging Executable...' : 'Forge Executable'}</span>
        </button>

        {status && (
          <div className={`status-bar ${isSuccess ? 'status-success' : 'status-error'}`}>
            {isSuccess ? '✔ ' : '✖ '} {status}
          </div>
        )}

      </div>
    </div>
  );
}

export default App;
