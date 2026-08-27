import React, { useEffect, useState, Suspense, lazy } from 'react';
import { Launcher } from './features/launcher/Launcher';
import { useSettingsStore } from './stores/settingsStore';
import { listen } from '@tauri-apps/api/event';

const Settings = lazy(() => import('./features/settings/Settings'));
const Onboarding = lazy(() => import('./features/onboarding/Onboarding'));

export const App: React.FC = () => {
  const loadSettings = useSettingsStore((state) => state.loadSettings);
  const [showSettings, setShowSettings] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(false);

  useEffect(() => {
    void loadSettings();
    const completed = localStorage.getItem('spotlight_onboarding_completed');
    if (!completed) {
      setShowOnboarding(true);
    }

    let unlisten: (() => void) | null = null;
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      void listen('open-settings', () => {
        setShowSettings(true);
      }).then((un) => {
        unlisten = un;
      });
    }

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [loadSettings]);

  if (showOnboarding) {
    return (
      <div className="launcher-window">
        <Suspense fallback={null}>
          <Onboarding
            onComplete={() => {
              setShowOnboarding(false);
            }}
          />
        </Suspense>
      </div>
    );
  }

  if (showSettings) {
    return (
      <div className="launcher-window">
        <Suspense fallback={null}>
          <Settings
            onClose={() => {
              setShowSettings(false);
            }}
          />
        </Suspense>
      </div>
    );
  }

  return (
    <Launcher
      onOpenSettings={() => {
        setShowSettings(true);
      }}
    />
  );
};

export default App;
