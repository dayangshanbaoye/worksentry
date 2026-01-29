import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Config {
  indexed_folders: string[];
  hotkey: {
    modifiers: string[];
    key: string;
  };
  enable_history: boolean;
  enable_bookmarks: boolean;
}


interface IndexStats {
  document_count: number;
  size_bytes: number;
  index_path: string;
  file_count?: number;
  bookmark_count?: number;
  history_count?: number;
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
    enable_history: false,
    enable_bookmarks: false,
  });
  const [stats, setStats] = useState<IndexStats | null>(null);
  const [newFolder, setNewFolder] = useState('');
  const [isReindexing, setIsReindexing] = useState(false);

  const [browserStatus, setBrowserStatus] = useState<{ installed_browsers: string[] } | null>(null);

  useEffect(() => {
    loadConfig();
    loadStats();
    loadBrowserStatus();
  }, []);

  const loadBrowserStatus = async () => {
    try {
      const status = await invoke<{ installed_browsers: string[] }>('get_browser_status');
      setBrowserStatus(status);
    } catch (error) {
      console.error('Failed to load browser status:', error);
    }
  };



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
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '16px', marginBottom: '16px' }}>
            <div>
              <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Total Items</div>
              <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{stats?.document_count || 0}</div>
            </div>
            <div>
              <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Index Size</div>
              <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{formatBytes(stats?.size_bytes || 0)}</div>
            </div>
          </div>

          {/* Breakdown */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px', borderTop: '1px solid var(--border)', paddingTop: '12px' }}>
            <div>
              <div style={{ color: 'var(--text-secondary)', fontSize: '11px' }}>Files</div>
              <div style={{ fontWeight: 'bold' }}>{stats?.file_count ?? '-'}</div>
            </div>
            <div>
              <div style={{ color: 'var(--text-secondary)', fontSize: '11px' }}>History</div>
              <div style={{ fontWeight: 'bold' }}>{stats?.history_count ?? '-'}</div>
            </div>
            <div>
              <div style={{ color: 'var(--text-secondary)', fontSize: '11px' }}>Bookmarks</div>
              <div style={{ fontWeight: 'bold' }}>{stats?.bookmark_count ?? '-'}</div>
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

      <div style={{ marginTop: '24px' }}>
        <h3 style={{ marginBottom: '12px' }}>Browser Integration</h3>

        {browserStatus && (
          <div style={{
            background: 'var(--bg-secondary)',
            padding: '16px',
            borderRadius: '8px',
            marginBottom: '16px'
          }}>
            <div style={{ marginBottom: '12px', fontSize: '14px' }}>
              <div style={{ fontWeight: 'bold', marginBottom: '8px' }}>Detected Browsers:</div>
              {browserStatus.installed_browsers.length > 0 ? (
                <div style={{ display: 'flex', gap: '8px' }}>
                  {browserStatus.installed_browsers.map(b => (
                    <span key={b} style={{
                      background: 'var(--accent)',
                      color: 'white',
                      padding: '2px 8px',
                      borderRadius: '4px',
                      fontSize: '12px'
                    }}>
                      {b}
                    </span>
                  ))}
                </div>
              ) : (
                <span style={{ color: 'var(--text-secondary)' }}>None detected</span>
              )}
            </div>

            <div style={{ marginTop: '16px', display: 'flex', flexDirection: 'column', gap: '16px' }}>

              {/* Bookmarks Toggle */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div>
                  <div style={{ fontWeight: 'bold' }}>Index Bookmarks</div>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                    Include bookmarks from detected browsers.
                  </div>
                </div>
                <label className="switch" style={{ position: 'relative', display: 'inline-block', width: '40px', height: '24px' }}>
                  <input
                    type="checkbox"
                    checked={config.enable_bookmarks}
                    onChange={async (e) => {
                      const enabled = e.target.checked;
                      setConfig(prev => ({ ...prev, enable_bookmarks: enabled }));
                      try { await invoke('set_bookmarks_enabled', { enabled }); await loadConfig(); }
                      catch (err) { console.error(err); setConfig(prev => ({ ...prev, enable_bookmarks: !enabled })); }
                    }}
                    style={{ opacity: 0, width: 0, height: 0 }}
                  />
                  <span style={{
                    position: 'absolute', cursor: 'pointer', top: 0, left: 0, right: 0, bottom: 0,
                    backgroundColor: config.enable_bookmarks ? 'var(--accent)' : '#ccc',
                    transition: '.4s', borderRadius: '34px'
                  }}>
                    <span style={{
                      position: 'absolute', content: "", height: '16px', width: '16px', left: '4px', bottom: '4px',
                      backgroundColor: 'white', transition: '.4s', borderRadius: '50%',
                      transform: config.enable_bookmarks ? 'translateX(16px)' : 'translateX(0)'
                    }}></span>
                  </span>
                </label>
              </div>

              {/* History Toggle */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div>
                  <div style={{ fontWeight: 'bold' }}>Index Browsing History</div>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                    Include full browsing history (may be large).
                  </div>
                </div>
                <label className="switch" style={{ position: 'relative', display: 'inline-block', width: '40px', height: '24px' }}>
                  <input
                    type="checkbox"
                    checked={config.enable_history}
                    onChange={async (e) => {
                      const enabled = e.target.checked;
                      setConfig(prev => ({ ...prev, enable_history: enabled }));
                      try { await invoke('set_history_enabled', { enabled }); await loadConfig(); }
                      catch (err) { console.error(err); setConfig(prev => ({ ...prev, enable_history: !enabled })); }
                    }}
                    style={{ opacity: 0, width: 0, height: 0 }}
                  />
                  <span style={{
                    position: 'absolute', cursor: 'pointer', top: 0, left: 0, right: 0, bottom: 0,
                    backgroundColor: config.enable_history ? 'var(--accent)' : '#ccc',
                    transition: '.4s', borderRadius: '34px'
                  }}>
                    <span style={{
                      position: 'absolute', content: "", height: '16px', width: '16px', left: '4px', bottom: '4px',
                      backgroundColor: 'white', transition: '.4s', borderRadius: '50%',
                      transform: config.enable_history ? 'translateX(16px)' : 'translateX(0)'
                    }}></span>
                  </span>
                </label>
              </div>

            </div>
          </div>
        )}
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
