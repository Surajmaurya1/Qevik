import { create } from 'zustand';
import { AppSettings, DEFAULT_SETTINGS, ThemeMode } from '../types/settings';
import { getSettings, updateSettings } from '../lib/ipc';

interface SettingsState {
  settings: AppSettings;
  isLoading: boolean;
  loadSettings: () => Promise<void>;
  setTheme: (theme: ThemeMode) => Promise<void>;
  saveSettings: (updated: AppSettings) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  isLoading: true,

  loadSettings: async () => {
    try {
      const settings = await getSettings();
      set({ settings, isLoading: false });
      document.documentElement.setAttribute('data-theme', settings.theme);
    } catch {
      set({ isLoading: false });
    }
  },

  setTheme: async (theme: ThemeMode) => {
    const current = get().settings;
    const updated = { ...current, theme };
    set({ settings: updated });
    document.documentElement.setAttribute('data-theme', theme);
    await updateSettings(updated);
  },

  saveSettings: async (updated: AppSettings) => {
    set({ settings: updated });
    document.documentElement.setAttribute('data-theme', updated.theme);
    await updateSettings(updated);
  },
}));
