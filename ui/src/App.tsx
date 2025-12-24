import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

function App() {
  const [gameName, setGameName] = useState('');
  const [romPath, setRomPath] = useState('');
  const [emulatorPath, setEmulatorPath] = useState('');
  const [status, setStatus] = useState('');
  const [isForging, setIsForging] = useState(false);

  // Manual file path input for MVP since we don't have dialog configured yet
  // or use basic input type="file" if browser allows (it won't give full path in browser, but Tauri might polyfill?)
  // Actually input type="file" in Tauri usually just gives the name, not full path, unless we use the dialog plugin.
  // For MVP, we will use text inputs for paths.
  
  async function handleForge() {
    if (!gameName || !romPath || !emulatorPath) {
      setStatus('Please fill all fields');
      return;
    }

    setIsForging(true);
    setStatus('Forging...');

    try {
      const result = await invoke('forge_executable', {
        gameName,
        emulatorPath,
        romPath,
        outputDir: 'output', // Hardcoded for now
        args: []
      });
      setStatus(`✅ Success! Executable created at: ${result}`);
    } catch (error) {
      setStatus(`❌ Error: ${error}`);
    } finally {
      setIsForging(false);
    }
  }

  return (
    <div className="container">
      <header>
        <h1>🔥 EmuForge</h1>
        <p>Transform your ROMs into standalone executables.</p>
      </header>
      
      <main className="forge-form">
        <div className="input-group">
          <label>Game Name</label>
          <input 
            type="text" 
            placeholder="e.g. God of War Ghost of Sparta" 
            value={gameName}
            onChange={(e) => setGameName(e.target.value)}
          />
        </div>

        <div className="input-group">
          <label>ROM Path (Absolute)</label>
          <input 
            type="text" 
            placeholder="/home/user/games/god_of_war.iso" 
            value={romPath}
            onChange={(e) => setRomPath(e.target.value)}
          />
        </div>

        <div className="input-group">
          <label>Emulator Path (Absolute)</label>
          <input 
            type="text" 
            placeholder="/usr/bin/ppsspp" 
            value={emulatorPath}
            onChange={(e) => setEmulatorPath(e.target.value)}
          />
        </div>

        <button 
          className="forge-btn"
          onClick={handleForge}
          disabled={isForging}
        >
          {isForging ? 'Forging...' : '🔨 Forge Now'}
        </button>

        {status && <div className="status-message">{status}</div>}
      </main>
    </div>
  );
}

export default App;
