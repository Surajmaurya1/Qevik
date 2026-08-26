import { invoke } from '@tauri-apps/api/core';
import { SearchResponse, SearchResult, LaunchResponse } from '../types/results';
import { AppSettings, DEFAULT_SETTINGS } from '../types/settings';
import { IndexStatus, AppInfo } from '../types/ipc';

/**
 * Check if the application is running in a Tauri webview environment.
 */
export function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Execute search query via Rust backend.
 */
export async function search(query: string): Promise<SearchResponse> {
  if (!isTauriEnvironment()) {
    // Development browser preview mock
    return mockSearch(query);
  }
  return await invoke<SearchResponse>('search', { query });
}

/**
 * Launch a search result item.
 */
export async function launch(id: string, resultType: string): Promise<LaunchResponse> {
  if (!isTauriEnvironment()) {
    return { success: true };
  }
  return await invoke<LaunchResponse>('launch', {
    id,
    resultType,
    result_type: resultType,
  });
}

/**
 * Request application or file icon as base64 data.
 */
export async function getIcon(id: string): Promise<string | null> {
  if (!isTauriEnvironment()) {
    return null;
  }
  const response = await invoke<{ data: string | null }>('get_icon', { id });
  return response.data;
}

/**
 * Hide the launcher window.
 */
export async function hideLauncher(): Promise<void> {
  if (!isTauriEnvironment()) {
    return;
  }
  await invoke('hide_launcher');
}

/**
 * Retrieve user settings.
 */
export async function getSettings(): Promise<AppSettings> {
  if (!isTauriEnvironment()) {
    return DEFAULT_SETTINGS;
  }
  return await invoke<AppSettings>('get_settings');
}

/**
 * Update user settings.
 */
export async function updateSettings(settings: AppSettings): Promise<boolean> {
  if (!isTauriEnvironment()) {
    return true;
  }
  return await invoke<boolean>('update_settings', { settings });
}

/**
 * Retrieve recent launches on empty query.
 */
export async function getRecentResults(): Promise<SearchResult[]> {
  if (!isTauriEnvironment()) {
    return [];
  }
  return await invoke<SearchResult[]>('get_recent_results');
}

/**
 * Retrieve index status statistics.
 */
export async function getIndexStatus(): Promise<IndexStatus> {
  if (!isTauriEnvironment()) {
    return {
      is_indexing: false,
      total_applications: 0,
      total_files: 0,
      total_folders: 0,
      last_indexed_at: Date.now(),
    };
  }
  return await invoke<IndexStatus>('get_index_status');
}

/**
 * Request manual rebuild of the index.
 */
export async function rebuildIndex(): Promise<void> {
  if (!isTauriEnvironment()) {
    return;
  }
  await invoke('rebuild_index');
}

/**
 * Get basic application metadata.
 */
export async function getAppInfo(): Promise<AppInfo> {
  if (!isTauriEnvironment()) {
    return {
      name: 'Spotlight for Windows',
      version: '1.0.0',
      tauri_version: '2.0.0',
    };
  }
  return await invoke<AppInfo>('get_app_info');
}

// Development mock data for browser preview testing
function mockSearch(query: string): SearchResponse {
  const q = query.trim().toLowerCase();
  if (!q) {
    return {
      results: [
        {
          id: 'app-terminal',
          result_type: 'app',
          display_name: 'Windows Terminal',
          subtitle: 'C:\\Program Files\\WindowsApps\\wt.exe',
          score: 1.0,
          icon_id: null,
        },
        {
          id: 'app-vscode',
          result_type: 'app',
          display_name: 'Visual Studio Code',
          subtitle: 'C:\\Users\\User\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe',
          score: 0.95,
          icon_id: null,
        },
      ],
      duration_ms: 2,
    };
  }

  // Calculator preview mock
  if (q.startsWith('=') || /^[\d\s+\-*/%().]+$/.test(q)) {
    const expr = q.startsWith('=') ? q.slice(1).trim() : q;
    const evaluated = safeEvaluateMath(expr);
    if (evaluated !== null) {
      return {
        results: [
          {
            id: 'calc-result',
            result_type: 'calculator',
            display_name: String(evaluated),
            subtitle: `= ${expr}`,
            score: 2.0,
            icon_id: null,
          },
        ],
        duration_ms: 1,
      };
    }
  }

  return {
    results: [
      {
        id: 'app-mock-1',
        result_type: 'app',
        display_name: `${query} Application`,
        subtitle: `C:\\Program Files\\${query}.exe`,
        score: 0.9,
        icon_id: null,
      },
    ],
    duration_ms: 5,
  };
}

function safeEvaluateMath(expr: string): number | null {
  const sanitized = expr.replace(/\s+/g, '');
  if (!/^[\d+\-*/().]+$/.test(sanitized)) {
    return null;
  }

  try {
    // Simple recursive descent parser for basic math without eval
    let pos = 0;
    const parseNumber = (): number => {
      const start = pos;
      if (sanitized.charAt(pos) === '-' || sanitized.charAt(pos) === '+') pos++;
      while (
        pos < sanitized.length &&
        (sanitized.charAt(pos) === '.' ||
          (sanitized.charAt(pos) >= '0' && sanitized.charAt(pos) <= '9'))
      ) {
        pos++;
      }
      return parseFloat(sanitized.slice(start, pos));
    };

    const parseFactor = (): number => {
      if (sanitized[pos] === '(') {
        pos++;
        const val = parseExpr();
        if (sanitized[pos] === ')') pos++;
        return val;
      }
      return parseNumber();
    };

    const parseTerm = (): number => {
      let val = parseFactor();
      while (pos < sanitized.length && (sanitized[pos] === '*' || sanitized[pos] === '/')) {
        const op = sanitized[pos];
        pos++;
        const next = parseFactor();
        if (op === '*') val *= next;
        else if (op === '/') val /= next;
      }
      return val;
    };

    const parseExpr = (): number => {
      let val = parseTerm();
      while (pos < sanitized.length && (sanitized[pos] === '+' || sanitized[pos] === '-')) {
        const op = sanitized[pos];
        pos++;
        const next = parseTerm();
        if (op === '+') val += next;
        else if (op === '-') val -= next;
      }
      return val;
    };

    const result = parseExpr();
    return isNaN(result) ? null : result;
  } catch {
    return null;
  }
}
