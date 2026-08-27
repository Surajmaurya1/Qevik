import React, { useState } from 'react';
import { useSettingsStore } from '../../stores/settingsStore';
import { ArrowLeft, Moon, Sun, Monitor, HardDrive, Keyboard } from 'lucide-react';
import './Settings.css';

interface SettingsProps {
  onClose: () => void;
}

export const Settings: React.FC<SettingsProps> = ({ onClose }) => {
  const { settings, setTheme, saveSettings } = useSettingsStore();
  const [hotkey, setHotkey] = useState(settings.hotkey);
  const [startWithWindows, setStartWithWindows] = useState(settings.start_with_windows);
  const [maxResults, setMaxResults] = useState(settings.max_results);
  const [enableCalc, setEnableCalc] = useState(settings.enable_calculator);
  const [enableWeb, setEnableWeb] = useState(settings.enable_web_search);
  const [isSaved, setIsSaved] = useState(false);

  const handleSave = async () => {
    await saveSettings({
      ...settings,
      hotkey,
      start_with_windows: startWithWindows,
      max_results: maxResults,
      enable_calculator: enableCalc,
      enable_web_search: enableWeb,
    });
    setIsSaved(true);
    setTimeout(() => {
      setIsSaved(false);
    }, 2000);
  };

  return (
    <div className="settings-container anim-fade-in">
      <div className="settings-header">
        <button className="settings-back-button" onClick={onClose} aria-label="Back to search">
          <ArrowLeft size={18} />
        </button>
        <h2>Preferences</h2>
      </div>

      <div className="settings-content">
        {/* Appearance */}
        <section className="settings-section">
          <h3>
            <Moon size={16} /> Appearance & Theme
          </h3>
          <div className="theme-options">
            <button
              className={`theme-button ${settings.theme === 'dark' ? 'active' : ''}`}
              onClick={() => {
                void setTheme('dark');
              }}
            >
              <Moon size={16} /> Dark
            </button>
            <button
              className={`theme-button ${settings.theme === 'light' ? 'active' : ''}`}
              onClick={() => {
                void setTheme('light');
              }}
            >
              <Sun size={16} /> Light
            </button>
            <button
              className={`theme-button ${settings.theme === 'system' ? 'active' : ''}`}
              onClick={() => {
                void setTheme('system');
              }}
            >
              <Monitor size={16} /> System
            </button>
          </div>
        </section>

        {/* Global Hotkey & Startup */}
        <section className="settings-section">
          <h3>
            <Keyboard size={16} /> Activation & System
          </h3>
          <div className="setting-row">
            <label htmlFor="hotkey-input">Global Hotkey</label>
            <input
              id="hotkey-input"
              type="text"
              className="settings-input"
              value={hotkey}
              onChange={(e) => {
                setHotkey(e.target.value);
              }}
              placeholder="e.g. Alt+Space"
            />
          </div>
          <div className="setting-toggle-row">
            <label htmlFor="autostart-toggle">Start with Windows (Boot in background)</label>
            <input
              id="autostart-toggle"
              type="checkbox"
              checked={startWithWindows}
              onChange={(e) => {
                setStartWithWindows(e.target.checked);
              }}
            />
          </div>
        </section>

        {/* Search & Indexing */}
        <section className="settings-section">
          <h3>
            <HardDrive size={16} /> Search Providers
          </h3>
          <div className="setting-toggle-row">
            <label htmlFor="calc-toggle">Calculator Provider (= 2 + 2)</label>
            <input
              id="calc-toggle"
              type="checkbox"
              checked={enableCalc}
              onChange={(e) => {
                setEnableCalc(e.target.checked);
              }}
            />
          </div>
          <div className="setting-toggle-row">
            <label htmlFor="web-toggle">Web Search Fallback</label>
            <input
              id="web-toggle"
              type="checkbox"
              checked={enableWeb}
              onChange={(e) => {
                setEnableWeb(e.target.checked);
              }}
            />
          </div>
          <div className="setting-row">
            <label htmlFor="max-results-input">Max Results Displayed</label>
            <input
              id="max-results-input"
              type="number"
              min={5}
              max={20}
              className="settings-input number-input"
              value={maxResults}
              onChange={(e) => {
                setMaxResults(Number(e.target.value));
              }}
            />
          </div>
        </section>

        <div className="settings-actions">
          <button
            className="settings-save-button"
            onClick={() => {
              void handleSave();
            }}
          >
            {isSaved ? 'Saved ✓' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default Settings;
