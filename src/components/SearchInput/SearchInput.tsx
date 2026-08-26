import React, { useRef, useEffect } from 'react';
import { Search } from 'lucide-react';
import './SearchInput.css';

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  placeholder?: string;
}

export const SearchInput: React.FC<SearchInputProps> = React.memo(
  ({ value, onChange, onKeyDown, placeholder = 'Search applications, files, commands...' }) => {
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
      // Focus on mount and whenever window becomes visible
      inputRef.current?.focus();
    }, []);

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange(e.target.value);
    };

    return (
      <div className="search-input-container">
        <Search className="search-icon" size={22} strokeWidth={2} />
        <input
          ref={inputRef}
          type="text"
          className="search-input"
          value={value}
          onChange={handleChange}
          onKeyDown={onKeyDown}
          placeholder={placeholder}
          spellCheck={false}
          autoComplete="off"
          autoCorrect="off"
          aria-label="Search"
        />
      </div>
    );
  },
);

SearchInput.displayName = 'SearchInput';
