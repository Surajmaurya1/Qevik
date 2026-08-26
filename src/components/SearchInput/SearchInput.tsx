import React, { useRef, useEffect, useCallback, useImperativeHandle, forwardRef } from 'react';
import { Search, X } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';
import './SearchInput.css';

export interface SearchInputRef {
  focus: () => void;
  select: () => void;
}

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  onClose?: () => void;
  placeholder?: string;
}

export const SearchInput = React.memo(
  forwardRef<SearchInputRef, SearchInputProps>(
    (
      {
        value,
        onChange,
        onKeyDown,
        onClose,
        placeholder = 'Search applications, files, commands...',
      },
      ref,
    ) => {
      const inputRef = useRef<HTMLInputElement>(null);

      const focusAndSelect = useCallback(() => {
        if (inputRef.current) {
          inputRef.current.focus();
          inputRef.current.select();
        }
      }, []);

      useImperativeHandle(
        ref,
        () => ({
          focus: () => inputRef.current?.focus(),
          select: () => inputRef.current?.select(),
        }),
        [],
      );

      useEffect(() => {
        // Immediately focus on mount
        focusAndSelect();

        const timer = setTimeout(focusAndSelect, 30);

        const handleWindowFocus = () => {
          focusAndSelect();
        };

        const handleVisibilityChange = () => {
          if (document.visibilityState === 'visible') {
            focusAndSelect();
          }
        };

        window.addEventListener('focus', handleWindowFocus);
        document.addEventListener('visibilitychange', handleVisibilityChange);

        let unlisten: (() => void) | null = null;
        if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
          void listen('launcher-shown', () => {
            focusAndSelect();
          }).then((un) => {
            unlisten = un;
          });
        }

        return () => {
          clearTimeout(timer);
          window.removeEventListener('focus', handleWindowFocus);
          document.removeEventListener('visibilitychange', handleVisibilityChange);
          if (unlisten) {
            unlisten();
          }
        };
      }, [focusAndSelect]);

      const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        onChange(e.target.value);
      };

      return (
        <div className="search-input-container" onClick={focusAndSelect}>
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
            autoFocus
          />
          {onClose && (
            <button
              type="button"
              className="search-close-button"
              onClick={(e) => {
                e.stopPropagation();
                onClose();
              }}
              aria-label="Close Spotlight"
              title="Close Spotlight (Esc)"
            >
              <X size={18} strokeWidth={2} />
            </button>
          )}
        </div>
      );
    },
  ),
);

SearchInput.displayName = 'SearchInput';
