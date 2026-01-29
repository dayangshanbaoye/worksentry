import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import SearchBar from '../../src/components/SearchBar';

describe('SearchBar', () => {
    it('renders input field', () => {
        const onChange = vi.fn();
        const onKeyDown = vi.fn();

        render(<SearchBar query="" onChange={onChange} onKeyDown={onKeyDown} />);

        expect(screen.getByPlaceholderText('Search files...')).toBeInTheDocument();
    });

    it('calls onChange when typing', () => {
        const onChange = vi.fn();
        const onKeyDown = vi.fn();

        render(<SearchBar query="" onChange={onChange} onKeyDown={onKeyDown} />);

        const input = screen.getByPlaceholderText('Search files...');
        fireEvent.change(input, { target: { value: 'test' } });

        expect(onChange).toHaveBeenCalledWith('test');
    });

    it('calls onKeyDown when pressing keys', () => {
        const onChange = vi.fn();
        const onKeyDown = vi.fn();

        render(<SearchBar query="" onChange={onChange} onKeyDown={onKeyDown} />);

        const input = screen.getByPlaceholderText('Search files...');
        fireEvent.keyDown(input, { key: 'ArrowDown' });

        expect(onKeyDown).toHaveBeenCalled();
    });

    it('displays current query value', () => {
        const onChange = vi.fn();
        const onKeyDown = vi.fn();

        render(<SearchBar query="hello world" onChange={onChange} onKeyDown={onKeyDown} />);

        const input = screen.getByPlaceholderText('Search files...');
        expect(input).toHaveValue('hello world');
    });
});
