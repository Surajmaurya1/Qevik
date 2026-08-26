import React, { useState, useEffect, useCallback, useRef } from 'react';
import { SearchInput, SearchInputRef } from '../../components/SearchInput/SearchInput';
import { ResultList } from '../../components/ResultList/ResultList';
import { SearchResult } from '../../types/results';
import { search, launch, hideLauncher, getRecentResults } from '../../lib/ipc';
import { listen } from '@tauri-apps/api/event';
import './Launcher.css';

interface LauncherProps {
  onOpenSettings?: () => void;
}

export const Launcher: React.FC<LauncherProps> = ({ onOpenSettings }) => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState<number>(0);
  const searchInputRef = useRef<SearchInputRef>(null);
  const debounceTimerRef = useRef<number | null>(null);

  // Load recent launches on initial mount or when query becomes empty
  const loadRecentOrEmpty = useCallback(async () => {
    try {
      const recent = await getRecentResults();
      setResults(recent);
      setSelectedIndex(0);
    } catch {
      setResults([]);
    }
  }, []);

  // Debounced search logic (50-80ms per Section 8 & Section 19)
  const performSearch = useCallback(
    (searchQuery: string) => {
      if (debounceTimerRef.current) {
        window.clearTimeout(debounceTimerRef.current);
      }

      if (!searchQuery.trim()) {
        void loadRecentOrEmpty();
        return;
      }

      debounceTimerRef.current = window.setTimeout(() => {
        void (async () => {
          try {
            const response = await search(searchQuery);
            setResults(response.results);
            setSelectedIndex(0);
          } catch {
            // In case of cancellation or error, leave current state
          }
        })();
      }, 60);
    },
    [loadRecentOrEmpty],
  );

  useEffect(() => {
    void loadRecentOrEmpty();

    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        void hideLauncher();
      } else if (
        e.key.length === 1 &&
        !e.ctrlKey &&
        !e.altKey &&
        !e.metaKey &&
        document.activeElement?.tagName !== 'INPUT'
      ) {
        // If user typed a printable character while focus was elsewhere, focus input
        searchInputRef.current?.focus();
      }
    };

    window.addEventListener('keydown', handleGlobalKeyDown);

    let unlisten: (() => void) | null = null;
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      void listen('launcher-shown', () => {
        // When launcher window is re-revealed, reset query and focus
        setQuery('');
        void loadRecentOrEmpty();
        searchInputRef.current?.focus();
      }).then((un) => {
        unlisten = un;
      });
    }

    return () => {
      window.removeEventListener('keydown', handleGlobalKeyDown);
      if (unlisten) {
        unlisten();
      }
      if (debounceTimerRef.current) {
        window.clearTimeout(debounceTimerRef.current);
      }
    };
  }, [loadRecentOrEmpty]);

  const handleQueryChange = (newQuery: string) => {
    setQuery(newQuery);
    performSearch(newQuery);
  };

  const handleExecute = useCallback(
    async (targetResult?: SearchResult) => {
      const itemToLaunch = targetResult ?? results[selectedIndex];
      if (!itemToLaunch) return;

      try {
        await launch(itemToLaunch.id, itemToLaunch.result_type);
        await hideLauncher();
      } catch (err) {
        console.error('Launch failed:', err);
      }
    },
    [results, selectedIndex],
  );

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowDown' || (e.key === 'Tab' && !e.shiftKey)) {
      e.preventDefault();
      if (results.length > 0) {
        setSelectedIndex((prev) => (prev + 1) % results.length);
      }
    } else if (e.key === 'ArrowUp' || (e.key === 'Tab' && e.shiftKey)) {
      e.preventDefault();
      if (results.length > 0) {
        setSelectedIndex((prev) => (prev - 1 + results.length) % results.length);
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      void handleExecute();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      void hideLauncher();
    } else if (e.ctrlKey && e.key.toLowerCase() === 'l') {
      e.preventDefault();
      setQuery('');
      void loadRecentOrEmpty();
    } else if (e.ctrlKey && e.key === ',') {
      e.preventDefault();
      if (onOpenSettings) {
        onOpenSettings();
      }
    }
  };

  const handleSelect = useCallback((item: SearchResult) => {
    setResults((prev) => {
      const index = prev.findIndex((r) => r.id === item.id);
      if (index !== -1) {
        setSelectedIndex(index);
      }
      return prev;
    });
  }, []);

  return (
    <div className="launcher-window anim-fade-in">
      <div className="launcher-card">
        <SearchInput
          ref={searchInputRef}
          value={query}
          onChange={handleQueryChange}
          onKeyDown={handleKeyDown}
          onClose={() => {
            void hideLauncher();
          }}
        />

        <ResultList
          results={results}
          selectedIndex={selectedIndex}
          onSelect={handleSelect}
          onExecute={(res) => {
            void handleExecute(res);
          }}
          query={query}
        />
      </div>
    </div>
  );
};
