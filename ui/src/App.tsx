import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import './App.css';

// Simple SVG Icons
const FolderIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" /></svg>
);

const SettingsIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.38a2 2 0 0 0-.73-2.73l-.15-.1a2 2 0 0 1-1-1.72v-.51a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" /><circle cx="12" cy="12" r="3" /></svg>
);

const ChipIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" /><path d="M12 12v.01" /></svg>
);

function App() {
  const [gameName, setGameName] = useState('');
  const [romPath, setRomPath] = useState('');
  const [emulatorPath, setEmulatorPath] = useState('');
  const [biosPath, setBiosPath] = useState('');
  const [status, setStatus] = useState('');
  const [isForging, setIsForging] = useState(false);

  async function selectRom() {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Game ROM',
        extensions: ['iso', 'cso', 'bin', 'nsp', 'xci', 'rvz', 'wbfs', 'chd']
      }]
    });
    if (selected) {
      setRomPath(selected as string);
      if (!gameName) {
        const filename = (selected as string).split(/[\\/]/).pop();
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
      filters: [{
        name: 'Executable',
        extensions: ['exe', '']
      }]
    });
    if (selected) setEmulatorPath(selected as string);
  }

  async function selectBios() {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'BIOS File',
        extensions: ['bin', 'rom', 'img', 'bios']
      }]
    });
    if (selected) setBiosPath(selected as string);
  }

  async function handleForge() {
    if (!gameName || !romPath || !emulatorPath) {
      setStatus('⚠️ Please fill required fields (Name, ROM, Emu)');
      return;
    }

    setIsForging(true);
    setStatus('🔨 Forging your game...');

    try {
      const result = await invoke('forge_executable', {
        gameName,
        emulatorPath,
        romPath,
        biosPath: biosPath || null, // Pass null if empty
        outputDir: 'output',
        args: []
      });
      setStatus(`✅ Success! Created at: ${result}`);
    } catch (error) {
      setStatus(`❌ Error: ${error}`);
    } finally {
      setIsForging(false);
    }
  }

  return (
    <div className="container">
      <div className="glass-panel">
        <h2 className="panel-title">EmuForge</h2>
        <p className="panel-subtitle">Standalone Game Creator</p>

        <div className="forge-form">
          <div className="input-group">
            <label>Game Title</label>
            <input
              type="text"
              placeholder="e.g. God of War"
              value={gameName}
              onChange={(e) => setGameName(e.target.value)}
              className="modern-input"
            />
          </div>

          <div className="input-group">
            <label>ROM Location</label>
            <div className="input-with-btn">
              <input
                type="text"
                placeholder="Select a ROM file..."
                value={romPath}
                onChange={(e) => setRomPath(e.target.value)}
                className="modern-input"
              />
              <button className="icon-btn" onClick={selectRom}>
                <FolderIcon />
              </button>
            </div>
          </div>

          <div className="input-with-btn-group-row">
            <div className="input-group half-width">
              <label>Emulator Binary</label>
              <div className="input-with-btn">
                <input
                  type="text"
                  placeholder="Select executable..."
                  value={emulatorPath}
                  onChange={(e) => setEmulatorPath(e.target.value)}
                  className="modern-input"
                />
                <button className="icon-btn" onClick={selectEmulator}>
                  <SettingsIcon />
                </button>
              </div>
            </div>

            <div className="input-group half-width">
              <label>BIOS (Optional)</label>
              <div className="input-with-btn">
                <input
                  type="text"
                  placeholder="Select BIOS..."
                  value={biosPath}
                  onChange={(e) => setBiosPath(e.target.value)}
                  className="modern-input"
                />
                <button className="icon-btn" onClick={selectBios}>
                  <ChipIcon />
                </button>
              </div>
            </div>
          </div>

          <div className="action-area">
            <button
              className={`forge-btn ${isForging ? 'loading' : ''}`}
              onClick={handleForge}
              disabled={isForging}
            >
              {isForging ? 'Forging...' : '🔥 Forge Executable'}
            </button>
          </div>

          {status && (
            <div className={`status-message ${status.includes('Error') ? 'error' : 'success'}`}>
              {status}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
