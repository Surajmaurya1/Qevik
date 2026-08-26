import React from 'react';
import { SearchResult } from '../../types/results';
import { Icon } from '../Icon/Icon';
import './ResultItem.css';

interface ResultItemProps {
  result: SearchResult;
  isSelected: boolean;
  onSelect: (result: SearchResult) => void;
  onExecute: (result: SearchResult) => void;
}

export const ResultItem: React.FC<ResultItemProps> = React.memo(
  ({ result, isSelected, onSelect, onExecute }) => {
    const formatTypeLabel = (type: SearchResult['result_type']) => {
      switch (type) {
        case 'app':
          return 'App';
        case 'file':
          return 'File';
        case 'folder':
          return 'Folder';
        case 'calculator':
          return 'Calc';
        case 'command':
          return 'Command';
        case 'web':
          return 'Web';
        default:
          return '';
      }
    };

    return (
      <div
        className={`result-item ${isSelected ? 'selected' : ''}`}
        onClick={() => {
          onExecute(result);
        }}
        onMouseEnter={() => {
          onSelect(result);
        }}
        role="option"
        aria-selected={isSelected}
      >
        <div className="result-icon-wrapper">
          <Icon iconId={result.icon_id} resultType={result.result_type} className="result-icon" />
        </div>
        <div className="result-content">
          <div className="result-name">{result.display_name}</div>
          <div className="result-subtitle" title={result.subtitle}>
            {result.subtitle}
          </div>
        </div>
        <div className="result-badge">{formatTypeLabel(result.result_type)}</div>
      </div>
    );
  },
);

ResultItem.displayName = 'ResultItem';
