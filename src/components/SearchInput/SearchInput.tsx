import React, { useRef, useEffect } from 'react';
import { Search, X } from 'lucide-react';
import './SearchInput.css';

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  onClose?: () => void;
  placeholder?: string;
}

export const SearchInput: React.FC<SearchInputProps> = React.memo(
  ({
    value,
    onChange,
    onKeyDown,
    onClose,
    placeholder = 'Search applications, files, commands...',
  }) => {
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
        {onClose && (
          <button
            type="button"
            className="search-close-button"
            onClick={onClose}
            aria-label="Close Spotlight"
            title="Close Spotlight (Esc)"
          >
            <X size={18} strokeWidth={2} />
          </button>
        )}
      </div>
    );
  },
);

SearchInput.displayName = 'SearchInput';
