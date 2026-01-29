import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Config {
  indexed_folders: string[];
  hotkey: {
    modifiers: string[];
    key: string;
  };
}


interface IndexStats {
  document_count: number;
  size_bytes: number;
  index_path: string;
}

function formatBytes(bytes: number, decimals = 2) {
  if (!+bytes) return '0 Bytes';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

function Settings() {
  const [config, setConfig] = useState<Config>({
    indexed_folders: [],
    hotkey: { modifiers: ['Alt'], key: 'Space' },
  });
  const [stats, setStats] = useState<IndexStats | null>(null);
  const [newFolder, setNewFolder] = useState('');
  const [isReindexing, setIsReindexing] = useState(false);

  useEffect(() => {
    loadConfig();
    loadStats();
  }, []);

  const loadConfig = async () => {
    try {
      const cfg = await invoke<Config>('get_config');
      setConfig(cfg);
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  };

  const loadStats = async () => {
    try {
      const s = await invoke<IndexStats>('get_index_stats');
      setStats(s);
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  };

  const handleAddFolder = async () => {
    if (!newFolder.trim()) return;
    try {
      console.log('Adding folder:', newFolder);
      await invoke('add_folder', { path: newFolder });
      setNewFolder('');
      await loadConfig();
      await loadStats();
      alert('Folder added successfully!');
    } catch (error) {
      console.error('Failed to add folder:', error);
      alert('Failed to add folder: ' + error);
    }
  };

  const handleRemoveFolder = async (path: string) => {
    try {
      console.log('Removing folder:', path);
      await invoke('remove_folder', { path });
      await loadConfig();
      await loadStats();
    } catch (error) {
      console.error('Failed to remove folder:', error);
      alert('Failed to remove folder: ' + error);
    }
  };

  const handleReindex = async () => {
    setIsReindexing(true);
    try {
      console.log('Starting reindex...');
      await invoke('reindex');
      await loadStats();
      alert('Reindex complete!');
    } catch (error) {
      console.error('Failed to reindex:', error);
      alert('Failed to reindex: ' + error);
    } finally {
      setIsReindexing(false);
    }
  };

  return (
    <div className="settings-panel">
      <h3>Indexed Folders</h3>
      <div style={{ display: 'flex', gap: '8px', marginTop: '12px' }}>
        <input
          type="text"
          value={newFolder}
          onChange={(e) => setNewFolder(e.target.value)}
          placeholder="C:\\path\\to\\folder"
          style={{
            flex: 1,
            padding: '8px 12px',
            borderRadius: '6px',
            border: '1px solid var(--border)',
            background: 'var(--bg-primary)',
            color: 'var(--text-primary)',
          }}
        />
        <button className="btn btn-primary" onClick={handleAddFolder}>
          Add
        </button>
      </div>

      <div className="folder-list">
        {config.indexed_folders.map((folder) => (
          <div key={folder} className="folder-item">
            <span className="folder-path">{folder}</span>
            <button
              className="btn btn-danger"
              onClick={() => handleRemoveFolder(folder)}
            >
              Remove
            </button>
          </div>
        ))}
        {config.indexed_folders.length === 0 && (
          <p style={{ color: 'var(--text-secondary)', marginTop: '16px' }}>
            No folders indexed yet
          </p>
        )}
      </div>

      <div style={{ marginTop: '24px' }}>
        <div className="stats-container" style={{ 
          background: 'var(--bg-secondary)', 
          padding: '16px', 
          borderRadius: '8px',
          marginBottom: '16px'
        }}>
          <h4 style={{ marginTop: 0, marginBottom: '12px' }}>Index Statistics</h4>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
            <div>
              <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Documents</div>
              <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{stats?.document_count || 0}</div>
            </div>
            <div>
              <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Index Size</div>
              <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{formatBytes(stats?.size_bytes || 0)}</div>
            </div>
          </div>
          {stats?.index_path && (
            <div style={{ marginTop: '12px', fontSize: '11px', color: 'var(--text-secondary)', wordBreak: 'break-all' }}>
              Location: {stats.index_path}
            </div>
          )}
        </div>

        <button
          className="btn btn-primary"
          onClick={handleReindex}
          disabled={isReindexing || config.indexed_folders.length === 0}
        >
          {isReindexing ? 'Reindexing...' : 'Reindex All Files'}
        </button>
      </div>

      <div className="hotkey-config" style={{ marginTop: '24px' }}>
        <h3>Hotkey</h3>
        <p style={{ color: 'var(--text-secondary)', marginTop: '8px', fontSize: '14px' }}>
          Current: {config.hotkey.modifiers.join('+')} + {config.hotkey.key}
        </p>
        <p style={{ color: 'var(--text-secondary)', marginTop: '8px', fontSize: '12px' }}>
          Hotkey configuration coming soon
        </p>
      </div>
    </div>
  );
}

export default Settings;
