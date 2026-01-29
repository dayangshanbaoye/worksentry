import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import SearchBar from './components/SearchBar';
import ResultsList from './components/ResultsList';
import Settings from './components/Settings';

interface SearchResult {
  path: string;
  file_name: string;
  score: number;
}

type TabType = 'search' | 'settings';

function App() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [activeTab, setActiveTab] = useState<TabType>('search');
  const [isLoading, setIsLoading] = useState(false);

  const handleSearch = useCallback(async (searchQuery: string) => {
    setQuery(searchQuery);
    if (searchQuery.trim() === '') {
      setResults([]);
      return;
    }
    setIsLoading(true);
    try {
      console.log('Searching for:', searchQuery);
      const searchResults = await invoke<SearchResult[]>('search', {
        query: searchQuery,
        limit: 20,
      });
      console.log('Search results:', searchResults);
      setResults(searchResults);
      setSelectedIndex(0);
    } catch (error) {
      console.error('Search failed:', error);
      alert('Search failed: ' + error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (activeTab !== 'search') return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        if (results[selectedIndex]) {
          openFile(results[selectedIndex].path);
        }
        break;
    }
  }, [results, selectedIndex, activeTab]);

  const openFile = async (path: string) => {
    try {
      await invoke('open_file', { path });
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  return (
    <div className="container">
      <div className="search-container">
        <div className="tabs">
          <button
            className={`tab ${activeTab === 'search' ? 'active' : ''}`}
            onClick={() => setActiveTab('search')}
          >
            Search
          </button>
          <button
            className={`tab ${activeTab === 'settings' ? 'active' : ''}`}
            onClick={() => setActiveTab('settings')}
          >
            Settings
          </button>
        </div>

        {activeTab === 'search' && (
          <>
            <SearchBar
              query={query}
              onChange={handleSearch}
              onKeyDown={handleKeyDown}
            />
            <ResultsList
              results={results}
              selectedIndex={selectedIndex}
              isLoading={isLoading}
              onSelect={(result) => openFile(result.path)}
            />
          </>
        )}

        {activeTab === 'settings' && <Settings />}
      </div>
    </div>
  );
}

export default App;
