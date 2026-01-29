interface SearchResult {
  path: string;
  file_name: string;
  score: number;
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
      {results.map((result, index) => (
        <div
          key={result.path}
          className={`result-item ${index === selectedIndex ? 'selected' : ''}`}
          onClick={() => onSelect(result)}
        >
          <div className="result-name">{result.file_name}</div>
          <div className="result-path">{result.path}</div>
        </div>
      ))}
    </div>
  );
}

export default ResultsList;
