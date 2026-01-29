import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ResultsList from '../../src/components/ResultsList';

describe('ResultsList', () => {
    const mockResults = [
        { path: '/test/file1.txt', file_name: 'file1.txt', score: 1.0 },
        { path: '/test/file2.txt', file_name: 'file2.txt', score: 0.9 },
        { path: '/test/file3.md', file_name: 'file3.md', score: 0.8 },
    ];

    it('displays empty state when no results', () => {
        render(<ResultsList results={[]} selectedIndex={0} isLoading={false} onSelect={vi.fn()} />);

        expect(screen.getByText('No results found')).toBeInTheDocument();
    });

    it('displays loading state', () => {
        render(<ResultsList results={[]} selectedIndex={0} isLoading={true} onSelect={vi.fn()} />);

        expect(screen.getByText('Indexing...')).toBeInTheDocument();
    });

    it('displays search results', () => {
        const onSelect = vi.fn();
        render(<ResultsList results={mockResults} selectedIndex={0} isLoading={false} onSelect={onSelect} />);

        expect(screen.getByText('file1.txt')).toBeInTheDocument();
        expect(screen.getByText('file2.txt')).toBeInTheDocument();
        expect(screen.getByText('file3.md')).toBeInTheDocument();
    });

    it('highlights selected item', () => {
        const onSelect = vi.fn();
        const { container } = render(
            <ResultsList results={mockResults} selectedIndex={1} isLoading={false} onSelect={onSelect} />
        );

        const items = container.querySelectorAll('.result-item');
        expect(items[1]).toHaveClass('selected');
    });

    it('calls onSelect when clicking result', () => {
        const onSelect = vi.fn();
        render(<ResultsList results={mockResults} selectedIndex={0} isLoading={false} onSelect={onSelect} />);

        fireEvent.click(screen.getByText('file2.txt'));
        expect(onSelect).toHaveBeenCalledWith(mockResults[1]);
    });

    it('displays file paths', () => {
        render(<ResultsList results={mockResults} selectedIndex={0} isLoading={false} onSelect={vi.fn()} />);

        expect(screen.getByText('/test/file1.txt')).toBeInTheDocument();
    });
});
