import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import App from '../../src/App';

const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
    invoke: mockInvoke,
}));

describe('App', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    it('renders search tab by default', () => {
        render(<App />);
        expect(screen.getByText('Search')).toBeInTheDocument();
        expect(screen.getByText('Settings')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('Search files...')).toBeInTheDocument();
    });

    it('switches to settings tab when clicked', () => {
        render(<App />);
        fireEvent.click(screen.getByText('Settings'));
        expect(screen.getByText('Indexed Folders')).toBeInTheDocument();
    });

    it('switches back to search tab when clicked', () => {
        render(<App />);
        fireEvent.click(screen.getByText('Settings'));
        fireEvent.click(screen.getByText('Search'));
        expect(screen.getByPlaceholderText('Search files...')).toBeInTheDocument();
    });

    it('displays empty state when no results', async () => {
        mockInvoke.mockResolvedValue([]);

        render(<App />);
        const input = screen.getByPlaceholderText('Search files...');
        fireEvent.change(input, { target: { value: 'test' } });

        await waitFor(() => {
            expect(screen.getByText('No results found')).toBeInTheDocument();
        });
    });

    it('displays loading state during search', async () => {
        mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(resolve, 100)));

        render(<App />);
        const input = screen.getByPlaceholderText('Search files...');
        fireEvent.change(input, { target: { value: 'test' } });

        expect(screen.getByText('Indexing...')).toBeInTheDocument();
    });

    it('displays search results', async () => {
        mockInvoke.mockResolvedValue([
            { path: '/test/file1.txt', file_name: 'file1.txt', score: 1.0 },
            { path: '/test/file2.txt', file_name: 'file2.txt', score: 0.9 },
        ]);

        render(<App />);
        const input = screen.getByPlaceholderText('Search files...');
        fireEvent.change(input, { target: { value: 'test' } });

        await waitFor(() => {
            expect(screen.getByText('file1.txt')).toBeInTheDocument();
            expect(screen.getByText('file2.txt')).toBeInTheDocument();
        });
    });
});
