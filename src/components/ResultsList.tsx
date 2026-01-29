interface SearchResult {
  path: string;
  file_name: string;
  score: number;
  record_type?: string; // "file", "history", "bookmark"
}

interface ResultsListProps {
  results: SearchResult[];
  selectedIndex: number;
  isLoading: boolean;
  onSelect: (result: SearchResult) => void;
}

function ResultsList({ results, selectedIndex, isLoading, onSelect }: ResultsListProps) {
  if (isLoading) {
    return (
      <div className="empty-state">
        <p>Indexing...</p>
      </div>
    );
  }

  if (results.length === 0) {
    return (
      <div className="empty-state">
        <p>No results found</p>
      </div>
    );
  }

  return (
    <div className="results-list">
      {results.map((result, index) => {
        const isUrl = result.record_type === 'history' || result.record_type === 'bookmark';
        return (
          <div
            key={result.path}
            className={`result-item ${index === selectedIndex ? 'selected' : ''}`}
            onClick={() => onSelect(result)}
          >
            <div className="result-name">
              {result.file_name}
              {isUrl && (
                <span style={{
                  fontSize: '10px',
                  marginLeft: '8px',
                  padding: '2px 6px',
                  borderRadius: '4px',
                  backgroundColor: result.record_type === 'bookmark' ? '#FFD700' : '#87CEEB',
                  color: '#333'
                }}>
                  {result.record_type === 'bookmark' ? 'BOOKMARK' : 'HISTORY'}
                </span>
              )}
            </div>
            <div className="result-path" style={{ color: isUrl ? '#4a9eff' : 'inherit' }}>
              {result.path}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default ResultsList;
