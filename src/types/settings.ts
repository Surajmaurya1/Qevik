export type ThemeMode = 'dark' | 'light' | 'system';

export interface AppSettings {
  hotkey: string;
  theme: ThemeMode;
  start_with_windows: boolean;
  max_results: number;
  enable_calculator: boolean;
  enable_web_search: boolean;
  indexed_directories: string[];
  ignored_extensions: string[];
}

export const DEFAULT_SETTINGS: AppSettings = {
  hotkey: 'Alt+Space',
  theme: 'dark',
  start_with_windows: false,
  max_results: 12,
  enable_calculator: true,
  enable_web_search: false,
  indexed_directories: [],
  ignored_extensions: ['tmp', 'log', 'bak'],
};
