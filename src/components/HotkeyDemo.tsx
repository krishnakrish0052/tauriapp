import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface StealthHotkeyEvent {
  action: string;
  hotkey_id: number;
  timestamp: string;
}

interface StealthStatus {
  active: boolean;
  registered_hotkeys: string[];
  hotkey_mappings: { [key: string]: string };
}

const HotkeyDemo: React.FC = () => {
  const [stealthStatus, setStealthStatus] = useState<StealthStatus | null>(null);
  const [lastHotkeyEvent, setLastHotkeyEvent] = useState<StealthHotkeyEvent | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Listen for hotkey events
  useEffect(() => {
    const unsubscribe = listen<StealthHotkeyEvent>('stealth-hotkey', (event) => {
      console.log('Received hotkey event:', event.payload);
      setLastHotkeyEvent(event.payload);
    });

    return () => {
      unsubscribe.then(fn => fn());
    };
  }, []);

  // Get initial status
  useEffect(() => {
    refreshStatus();
  }, []);

  const refreshStatus = async () => {
    try {
      const status = await invoke<StealthStatus>('get_stealth_status');
      setStealthStatus(status);
    } catch (error) {
      console.error('Failed to get stealth status:', error);
    }
  };

  const activateStealth = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<string>('activate_stealth_mode');
      console.log('Stealth mode activated:', result);
      await refreshStatus();
    } catch (error) {
      console.error('Failed to activate stealth mode:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const deactivateStealth = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<string>('deactivate_stealth_mode');
      console.log('Stealth mode deactivated:', result);
      await refreshStatus();
    } catch (error) {
      console.error('Failed to deactivate stealth mode:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const testHotkey = async (hotkeyName: string) => {
    try {
      const result = await invoke<string>('test_stealth_hotkey', { hotkeyName });
      console.log('Hotkey test result:', result);
    } catch (error) {
      console.error('Failed to test hotkey:', error);
    }
  };

  const hotkeyMappings = [
    { key: 'Ctrl+Shift+S', action: 'system_sound_toggle', description: 'Toggle System Sound' },
    { key: 'Ctrl+Shift+Z', action: 'ai_answer_trigger', description: 'Trigger AI Answer' },
    { key: 'Ctrl+Shift+X', action: 'window_toggle', description: 'Hide/Show Window' },
    { key: 'Ctrl+Shift+M', action: 'mic_toggle', description: 'Toggle Microphone' },
    { key: 'Ctrl+Shift+A', action: 'analyze_screen', description: 'Analyze Screen' },
    { key: 'Ctrl+Shift+I', action: 'manual_input', description: 'Manual Question Entry' },
    { key: 'Ctrl+Shift+Enter', action: 'submit_question', description: 'Submit Question' },
    { key: 'Ctrl+Shift+C', action: 'clear_area', description: 'Clear Listening Area' },
  ];

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-6 text-center">MockMate Global Hotkey System</h1>
      
      <div className="mb-6 p-4 bg-blue-50 rounded-lg">
        <h2 className="text-lg font-semibold mb-2">Status</h2>
        <p className="text-sm">
          <strong>Active:</strong> {stealthStatus?.active ? '✅ Yes' : '❌ No'}
        </p>
        <p className="text-sm">
          <strong>Registered Hotkeys:</strong> {stealthStatus?.registered_hotkeys?.length || 0}
        </p>
      </div>

      <div className="mb-6 flex gap-4">
        <button
          onClick={activateStealth}
          disabled={isLoading || stealthStatus?.active}
          className="px-4 py-2 bg-green-600 text-white rounded disabled:bg-gray-400 disabled:cursor-not-allowed hover:bg-green-700"
        >
          {isLoading ? 'Loading...' : 'Activate Global Hotkeys'}
        </button>
        
        <button
          onClick={deactivateStealth}
          disabled={isLoading || !stealthStatus?.active}
          className="px-4 py-2 bg-red-600 text-white rounded disabled:bg-gray-400 disabled:cursor-not-allowed hover:bg-red-700"
        >
          {isLoading ? 'Loading...' : 'Deactivate Global Hotkeys'}
        </button>
        
        <button
          onClick={refreshStatus}
          disabled={isLoading}
          className="px-4 py-2 bg-blue-600 text-white rounded disabled:bg-gray-400 disabled:cursor-not-allowed hover:bg-blue-700"
        >
          Refresh Status
        </button>
      </div>

      <div className="mb-6">
        <h2 className="text-lg font-semibold mb-4">Available Hotkeys</h2>
        <div className="grid gap-3">
          {hotkeyMappings.map(({ key, description }) => (
            <div key={key} className="flex items-center justify-between p-3 bg-gray-50 rounded">
              <div className="flex-1">
                <code className="text-sm font-mono bg-gray-200 px-2 py-1 rounded">{key}</code>
                <span className="ml-3 text-sm">{description}</span>
              </div>
              <button
                onClick={() => testHotkey(key)}
                disabled={!stealthStatus?.active}
                className="px-3 py-1 bg-indigo-600 text-white text-sm rounded disabled:bg-gray-400 disabled:cursor-not-allowed hover:bg-indigo-700"
              >
                Test
              </button>
            </div>
          ))}
        </div>
      </div>

      <div className="mb-6 p-4 bg-yellow-50 rounded-lg">
        <h3 className="text-md font-semibold mb-2">ℹ️ Important Notes</h3>
        <ul className="text-sm space-y-1">
          <li>• <strong>Fn Key Issue:</strong> The Fn key cannot be reliably detected by software. We use <strong>Ctrl+Shift</strong> combinations instead.</li>
          <li>• <strong>Global Hotkeys:</strong> These work system-wide, even when MockMate is not focused.</li>
          <li>• <strong>Windows Only:</strong> Real global hotkeys currently only work on Windows. Other platforms use simulation.</li>
          <li>• <strong>Stealth Mode:</strong> When active, hotkeys are registered but won't show system notifications.</li>
        </ul>
      </div>

      {lastHotkeyEvent && (
        <div className="p-4 bg-green-50 rounded-lg">
          <h3 className="text-md font-semibold mb-2">🎯 Last Hotkey Triggered</h3>
          <div className="text-sm">
            <p><strong>Action:</strong> {lastHotkeyEvent.action}</p>
            <p><strong>Hotkey ID:</strong> {lastHotkeyEvent.hotkey_id}</p>
            <p><strong>Timestamp:</strong> {new Date(lastHotkeyEvent.timestamp).toLocaleString()}</p>
          </div>
        </div>
      )}

      <div className="mt-6 p-4 bg-gray-50 rounded-lg">
        <h3 className="text-md font-semibold mb-2">🧪 Testing Instructions</h3>
        <ol className="text-sm space-y-1 list-decimal list-inside">
          <li>Click "Activate Global Hotkeys" to register the hotkey system</li>
          <li>Try pressing any of the <code>Ctrl+Shift+Key</code> combinations shown above</li>
          <li>You should see the "Last Hotkey Triggered" section update when a hotkey is pressed</li>
          <li>Use the "Test" buttons to simulate hotkey presses for testing</li>
          <li>Check the console (F12) for detailed logs</li>
        </ol>
      </div>
    </div>
  );
};

export default HotkeyDemo;
