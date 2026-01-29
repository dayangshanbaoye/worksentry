interface SearchBarProps {
  query: string;
  onChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
}

function SearchBar({ query, onChange, onKeyDown }: SearchBarProps) {
  return (
    <div className="search-input-wrapper">
      <span className="search-icon">⌕</span>
      <input
        type="text"
        className="search-input"
        placeholder="Search files..."
        value={query}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={onKeyDown}
        autoFocus
      />
    </div>
  );
}

export default SearchBar;
