import React, { useRef, useEffect } from 'react';
import { SearchResult } from '../../types/results';
import { ResultItem } from '../ResultItem/ResultItem';
import './ResultList.css';

interface ResultListProps {
  results: SearchResult[];
  selectedIndex: number;
  onSelect: (result: SearchResult) => void;
  onExecute: (result: SearchResult) => void;
  query: string;
}

export const ResultList: React.FC<ResultListProps> = React.memo(
  ({ results, selectedIndex, onSelect, onExecute, query }) => {
    const listRef = useRef<HTMLDivElement>(null);

    // Ensure selected item is scrolled into view
    useEffect(() => {
      if (listRef.current && selectedIndex >= 0) {
        const items = listRef.current.querySelectorAll('.result-item');
        const selectedItem = items[selectedIndex] as HTMLElement | undefined;
        if (selectedItem) {
          selectedItem.scrollIntoView({ block: 'nearest' });
        }
      }
    }, [selectedIndex]);

    if (results.length === 0) {
      if (query.trim().length > 0) {
        return (
          <div className="result-list-empty">
            <span>No results found for &ldquo;{query}&rdquo;</span>
          </div>
        );
      }
      return null;
    }

    return (
      <div className="result-list" ref={listRef} role="listbox">
        {results.map((result, idx) => (
          <ResultItem
            key={result.id}
            result={result}
            isSelected={idx === selectedIndex}
            onSelect={onSelect}
            onExecute={onExecute}
          />
        ))}
      </div>
    );
  },
);

ResultList.displayName = 'ResultList';
