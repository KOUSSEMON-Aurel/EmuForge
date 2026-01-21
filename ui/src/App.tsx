import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { downloadDir, join } from '@tauri-apps/api/path';

import { listen } from '@tauri-apps/api/event';
import './App.css';

// --- Icons (Feather Icons style) ---
const SunIcon = () => (
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="12" cy="12" r="5" />
    <line x1="12" y1="1" x2="12" y2="3" />
    <line x1="12" y1="21" x2="12" y2="23" />
    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
    <line x1="1" y1="12" x2="3" y2="12" />
    <line x1="21" y1="12" x2="23" y2="12" />
    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
  </svg>
);

const MoonIcon = () => (
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
  </svg>
);

const FolderIcon = () => (
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
  </svg>
);

const PlayIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="none">
    <polygon points="5 3 19 12 5 21 5 3" />
  </svg>
);

const CpuIcon = () => (
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <rect x="4" y="4" width="16" height="16" rx="2" ry="2" />
    <rect x="9" y="9" width="6" height="6" />
    <line x1="9" y1="1" x2="9" y2="4" />
    <line x1="15" y1="1" x2="15" y2="4" />
    <line x1="9" y1="20" x2="9" y2="23" />
    <line x1="15" y1="20" x2="15" y2="23" />
    <line x1="20" y1="9" x2="23" y2="9" />
    <line x1="20" y1="14" x2="23" y2="14" />
    <line x1="1" y1="9" x2="4" y2="9" />
    <line x1="1" y1="14" x2="4" y2="14" />
  </svg>
);

const SettingsIcon = () => (
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
  </svg>
);



function App() {
  const [theme, setTheme] = useState<'light' | 'dark'>('dark');
  const [gameName, setGameName] = useState('');
  const [romPath, setRomPath] = useState('');
  const [emulatorPath, setEmulatorPath] = useState('');
  const [biosPath, setBiosPath] = useState('');
  const [selectedPlugin, setSelectedPlugin] = useState('auto');
  const [targetOs, setTargetOs] = useState('auto');
  const [fullscreen, setFullscreen] = useState(true);
  const [portableMode, setPortableMode] = useState(false);
  const [outputDir, setOutputDir] = useState('output'); // Folder for generated files

  const [status, setStatus] = useState<string | null>(null);
  const [isSuccess, setIsSuccess] = useState(false);
  const [isForging, setIsForging] = useState(false);
  const [progressMessage, setProgressMessage] = useState("");

  // Modal State
  const [showEmulatorModal, setShowEmulatorModal] = useState(false);
  // Download State
  const [downloadingEmu, setDownloadingEmu] = useState<string | null>(null);
  const [installedEmulators, setInstalledEmulators] = useState<string[]>([]);
  // Platform Detection State
  const [detectedPlatform, setDetectedPlatform] = useState<string | null>(null);

  // Initialize theme
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  // Set default output directory to Downloads/EmuForge_Output to avoid dev-mode reloads
  useEffect(() => {
    const initDir = async () => {
      try {
        const docDir = await downloadDir();
        const safeDir = await join(docDir, 'EmuForge_Output');
        setOutputDir(safeDir);
        console.log("Default Output Dir set to:", safeDir);
      } catch (e) {
        console.error("Failed to resolve download dir:", e);
      }
    };
    initDir();
  }, []);

  // Fetch installed list when modal opens
  useEffect(() => {
    if (showEmulatorModal) {
      invoke('get_installed_emulators').then((res) => {
        console.log("Installed Emulators:", res);
        setInstalledEmulators(res as string[]);
      }).catch((e) => console.error("Failed to fetch installed:", e));
    }
  }, [showEmulatorModal]);

  const toggleTheme = () => {

    setTheme(prev => prev === 'light' ? 'dark' : 'light');
  };

  // RECOMMENDATION ENGINE
  // RECOMMENDATION ENGINE (Async Intelligent Detection)
  useEffect(() => {
    if (romPath) {
      // 1. D'abord reset pour éviter de garder l'ancienne valeur
      setDetectedPlatform(null);

      // 2. Appel au backend pour analyser le fichier
      invoke('detect_platform', { path: romPath })
        .then((platform) => {
          console.log("Detected Platform:", platform);
          setDetectedPlatform(platform as string);
        })
        .catch((e) => console.error("Detection failed:", e));
    } else {
      setDetectedPlatform(null);
    }
  }, [romPath]);

  const getRecommendedEmulator = () => {
    if (!detectedPlatform) return [];

    switch (detectedPlatform) {
      // Consoles Nintendo
      case 'wii': return ['dolphin'];
      case 'gamecube': return ['dolphin'];
      case 'switch': return ['ryujinx'];
      case 'wiiu': return ['cemu'];
      case '3ds': return ['lime3ds'];
      case 'nds': return ['melonds'];

      // Consoles Sony
      case 'ps1': return ['duckstation']; // Meilleur choix, PCSX2 peut potentiellement jouer PS1 mais DuckStation est dédié
      case 'ps2': return ['pcsx2'];
      case 'ps3': return ['rpcs3']; // NOUVEAU
      case 'ps4': return []; // Pas d'émulateur supporté pour l'instant
      case 'psp': return ['ppsspp'];

      // Consoles Microsoft
      case 'xbox': return ['xemu'];

      // Sega
      case 'dreamcast': return ['redream'];

      // Fallback si "unknown" mais extension connue (géré par analyzer.rs maintenant)
      default: return [];
    }
  };

  const recommendations = getRecommendedEmulator() || [];

  async function selectRom() {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Game ROM',
        extensions: ['iso', 'cso', 'bin', 'cue', 'm3u', 'img', 'nsp', 'xci', 'rvz', 'wbfs', 'chd', 'nds', '3ds', 'cia', 'wua', 'wux', 'wud', 'gdi', 'cdi']
      }]
    });
    if (selected) {
      const pathStr = Array.isArray(selected) ? selected[0] : (selected as string);
      if (!pathStr) return;

      setRomPath(pathStr);
      setRomPath(pathStr);
      // Always update game name from filename when a new file is selected
      const filename = pathStr.split(/[\\/]/).pop();
      if (filename) {
        const name = filename.replace(/\.[^/.]+$/, "").replace(/_/g, " ");
        setGameName(name);
      }
      // Auto-open modal if emulator is empty? No, maybe intrusive.
    }
  }

  // Modified: Opens Modal instead of direct file picker
  function openEmulatorModal() {
    setShowEmulatorModal(true);
  }

  async function handleBrowseLocal() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Executable', extensions: ['exe', ''] }]
    });
    if (selected) {
      setEmulatorPath(selected as string);
      setShowEmulatorModal(false);
    }
  }

  async function handleDownload(emuId: string) {
    setDownloadingEmu(emuId);
    try {
      const path = await invoke('download_emulator', { emuId: emuId });

      // Update local state to reflect installation immediately
      setInstalledEmulators(prev => [...prev, emuId]);

      // Auto-select the new path
      setEmulatorPath(path as string);
      setShowEmulatorModal(false);
    } catch (e) {
      setStatus(`Download failed: ${e}`);
      console.error("Download error:", e);
    } finally {
      setDownloadingEmu(null);
    }

  }

  async function selectBios() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'BIOS File', extensions: ['bin', 'rom', 'img', 'bios', '*'] }]
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
    setProgressMessage("");

    // Setup progress listener
    const unlisten = await listen('forge-progress', (event: any) => {
      setProgressMessage(event.payload.message);
    });

    try {
      const result = await invoke('forge_executable', {
        gameName,
        emulatorPath,
        romPath,
        biosPath: biosPath || null,
        outputDir: outputDir,
        plugin: selectedPlugin,
        targetOs,
        fullscreen,
        args: [],
        portableMode: portableMode

      });
      unlisten();
      setStatus(`Success! Executable ready at: ${result}`);
      setIsSuccess(true);
    } catch (error) {
      unlisten();
      setStatus(`Error: ${error}`);
      setIsSuccess(false);
    } finally {
      setIsForging(false);
      setProgressMessage("");
    }
  }

  const emuList = [
    { id: 'ppsspp', name: 'PPSSPP', desc: 'Best for PSP' },
    { id: 'pcsx2', name: 'PCSX2', desc: 'Best for PS2' },
    { id: 'duckstation', name: 'DuckStation', desc: 'Best for PS1' },
    { id: 'dolphin', name: 'Dolphin', desc: 'GameCube / Wii' },
    { id: 'ryujinx', name: 'Ryujinx', desc: 'Nintendo Switch' },
    { id: 'cemu', name: 'Cemu', desc: 'Wii U' },
    { id: 'melonds', name: 'melonDS', desc: 'Nintendo DS' },
    { id: 'lime3ds', name: 'Lime3DS', desc: 'Nintendo 3DS' },
    { id: 'redream', name: 'Redream', desc: 'Dreamcast' },
    { id: 'rpcs3', name: 'RPCS3', desc: 'PlayStation 3' },
    { id: 'xemu', name: 'xemu', desc: 'Original Xbox' },
  ];

  return (
    <div className="app-container">
      {/* Top Bar */}
      {/* Top Bar with Window Controls */}
      <div className="top-bar" data-tauri-drag-region>
        <div className="brand">
          <span className="logo-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z" /></svg>
          </span>
          <span className="brand-text">EmuForge</span>
        </div>
        <div className="window-controls">
          <button className="theme-toggle" onClick={toggleTheme} title="Toggle Theme" style={{ marginRight: '8px' }}>
            {theme === 'light' ? <MoonIcon /> : <SunIcon />}
          </button>
          <button
            className="win-btn close-btn-win"
            onClick={() => invoke('quit_app')}
            title="Close"
            style={{ zIndex: 9999, position: 'relative', cursor: 'pointer' }}
          >
            &times;
          </button>
        </div>
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
                <button className="input-action-btn" onClick={openEmulatorModal} title="Select or Download Emulator">
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

          {/* Plugin Selection & Target OS */}
          <div className="row">
            <div className="col input-wrapper">
              <div className="label-row"><label>Emulator Plugin</label></div>
              <div className="input-container">
                <select
                  className="text-input"
                  value={selectedPlugin}
                  onChange={(e) => setSelectedPlugin(e.target.value)}
                >
                  <option value="auto">Auto-Detect</option>
                  <option value="ppsspp">PPSSPP (PSP)</option>
                  <option value="pcsx2">PCSX2 (PS2)</option>
                  <option value="duckstation">DuckStation (PS1)</option>
                  <option value="dolphin">Dolphin (GC/Wii)</option>
                  <option value="rpcs3">RPCS3 (PS3)</option>
                  <option value="cemu">Cemu (Wii U)</option>
                  <option value="ryujinx">Ryujinx (Switch)</option>
                  <option value="xemu">xemu (Xbox)</option>
                  <option value="redream">Redream (Dreamcast)</option>
                  <option value="lime3ds">Lime3DS (3DS)</option>
                  <option value="melonds">melonDS (DS)</option>
                </select>
              </div>
            </div>

            <div className="col input-wrapper">
              <div className="label-row"><label>Target Platform</label></div>
              <div className="input-container">
                <select
                  className="text-input"
                  value={targetOs}
                  onChange={(e) => setTargetOs(e.target.value)}
                >
                  <option value="auto">Auto (Match Emulator)</option>
                  <option value="windows">Windows (.exe)</option>
                  <option value="linux">Linux (Binary)</option>
                </select>
              </div>
            </div>
          </div>


          {/* Options Row */}
          <div className="options-row">
            <label className="checkbox-container">
              <input
                type="checkbox"
                checked={fullscreen}
                onChange={(e) => setFullscreen(e.target.checked)}
              />
              <span className="checkbox-text">Start in Fullscreen</span>
            </label>

            <label className="checkbox-container">
              <input
                type="checkbox"
                checked={portableMode}
                onChange={(e) => setPortableMode(e.target.checked)}
              />
              <span className="checkbox-text">📦 Mode Portable (All-in-One)</span>
            </label>
          </div>

          {portableMode && (
            <div className="portable-info">
              <small>⚠️ L'exécutable contiendra l'émulateur, la ROM et le BIOS. Taille finale pouvant aller de 200 Mo à plusieurs Go.</small>
            </div>
          )}

        </div>

        <hr className="divider" />

        <button
          className="forge-btn"
          onClick={handleForge}
          disabled={isForging}
        >
          {isForging ? <div className="spinner"></div> : <PlayIcon />}
          <span>
            {isForging
              ? (progressMessage || 'Forging Executable...')
              : 'Forge Executable'}
          </span>
        </button>



      </div>

      {/* MODAL */}
      {showEmulatorModal && (
        <div className="modal-overlay" onClick={() => setShowEmulatorModal(false)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2 className="modal-title">Emulator Selection</h2>
              <button className="close-btn" onClick={() => setShowEmulatorModal(false)}>&times;</button>
            </div>

            <div className="modal-body">
              <div style={{ marginBottom: '1rem' }}>
                <button className="forge-btn" style={{ background: 'var(--surface-hover)', border: '1px solid var(--border-color)', color: 'var(--text-main)' }} onClick={handleBrowseLocal}>
                  <FolderIcon /> Choose from Computer
                </button>
              </div>

              <div className="divider" style={{ margin: '0.5rem 0' }}></div>
              <label style={{ display: 'block', marginBottom: '0.5rem' }}>Download & Install</label>

              <div style={{ maxHeight: '300px', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '8px', paddingRight: '4px' }}>
                {emuList.map(emu => {
                  const isRecommended = recommendations.includes(emu.id);
                  const isInstalled = installedEmulators.includes(emu.id);

                  return (
                    <div key={emu.id} className="emu-option-btn" onClick={() => handleDownload(emu.id)}>
                      <div className="emu-info">
                        <div style={{ display: 'flex', alignItems: 'center' }}>
                          <span className="emu-name">{emu.name}</span>
                          {isRecommended && <span className="recommendation-badge">Recommended</span>}
                        </div>
                        <span className="emu-desc">{emu.desc}</span>
                      </div>
                      <div style={{ display: 'flex', alignItems: 'center', color: isInstalled ? 'var(--status-success, #2da44e)' : 'inherit' }}>
                        {downloadingEmu === emu.id ?
                          <div className="spinner" style={{ borderColor: 'var(--text-secondary)', borderTopColor: 'transparent', width: 14, height: 14 }}></div>
                          : isInstalled ?
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="20 6 9 17 4 12"></polyline></svg>
                            :
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                        }
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* STATUS MODAL */}
      {status && !isForging && (
        <div className="modal-overlay" onClick={() => setStatus(null)}>
          <div className="modal-content status-popup-content" onClick={e => e.stopPropagation()}>

            {isSuccess ? (
              <svg className="status-icon-large" viewBox="0 0 24 24" fill="none" stroke="#2da44e" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                <polyline points="22 4 12 14.01 9 11.01"></polyline>
              </svg>
            ) : (
              <svg className="status-icon-large" viewBox="0 0 24 24" fill="none" stroke="#cf222e" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="15" y1="9" x2="9" y2="15"></line>
                <line x1="9" y1="9" x2="15" y2="15"></line>
              </svg>
            )}

            <h2 className="status-title" style={{ color: isSuccess ? 'var(--text-main)' : '#cf222e' }}>
              {isSuccess ? 'Success!' : 'Error'}
            </h2>

            <p className="status-message">{status}</p>

            <button className="forge-btn" onClick={() => setStatus(null)} style={{ maxWidth: '200px' }}>
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;