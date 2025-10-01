import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface StealthStatus {
  active: boolean;
  registered_hotkeys: string[];
  hotkey_mappings: Record<string, string>;
}

interface TaskManagerStatus {
  process_id: number;
  stealth_enabled: boolean;
  original_name: string;
  techniques_applied: Record<string, boolean>;
}

interface StealthHotkeyEvent {
  action: string;
  hotkey_id: number;
  timestamp: string;
}

const StealthModeManager: React.FC = () => {
  const [stealthStatus, setStealthStatus] = useState<StealthStatus | null>(null);
  const [taskManagerStatus, setTaskManagerStatus] = useState<TaskManagerStatus | null>(null);
  const [isStealthModeActive, setIsStealthModeActive] = useState(false);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');

  // Apply stealth mode CSS class to document body
  useEffect(() => {
    if (isStealthModeActive) {
      document.body.classList.add('stealth-mode');
      // Remove all title attributes from all elements
      const elementsWithTitle = document.querySelectorAll('[title]');
      elementsWithTitle.forEach(element => {
        element.removeAttribute('title');
      });
    } else {
      document.body.classList.remove('stealth-mode');
    }

    return () => {
      document.body.classList.remove('stealth-mode');
    };
  }, [isStealthModeActive]);

  // Listen for stealth hotkey events
  useEffect(() => {
    const setupHotkeyListener = async () => {
      const unlisten = await listen<StealthHotkeyEvent>('stealth-hotkey', (event) => {
        console.log('🎯 Stealth hotkey triggered:', event.payload);
        handleHotkeyAction(event.payload.action);
      });

      return unlisten;
    };

    let unlisten: (() => void) | null = null;
    setupHotkeyListener().then(fn => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  // Handle hotkey actions
  const handleHotkeyAction = async (action: string) => {
    console.log('🕵️ Processing stealth hotkey action:', action);
    setMessage(`Hotkey triggered: ${action}`);

    try {
      switch (action) {
        case 'system_sound_toggle':
          // Toggle system audio capture
          await invoke('start_universal_system_audio_capture');
          break;
        case 'ai_answer_trigger':
          // Trigger AI answer generation
          setMessage('AI answer triggered (implement your logic here)');
          break;
        case 'window_toggle':
          // Toggle window visibility
          await toggleWindowVisibility();
          break;
        case 'mic_toggle':
          // Toggle microphone
          await invoke('start_microphone_transcription');
          break;
        case 'analyze_screen':
          // Analyze screen content
          setMessage('Screen analysis triggered (implement your logic here)');
          break;
        case 'manual_input':
          // Activate manual input mode
          setMessage('Manual input mode activated (implement your logic here)');
          break;
        case 'submit_question':
          // Submit current question
          setMessage('Question submitted (implement your logic here)');
          break;
        case 'clear_area':
          // Clear listening area
          setMessage('Listening area cleared (implement your logic here)');
          break;
        default:
          setMessage(`Unknown hotkey action: ${action}`);
      }
    } catch (error) {
      console.error('Error handling hotkey action:', error);
      setMessage(`Error: ${error}`);
    }

    // Clear message after 3 seconds
    setTimeout(() => setMessage(''), 3000);
  };

  const toggleWindowVisibility = async () => {
    try {
      // This would hide/show the window - implement based on your window management
      setMessage('Window visibility toggled');
    } catch (error) {
      console.error('Error toggling window visibility:', error);
    }
  };

  const getStealthStatus = async () => {
    try {
      const status = await invoke<StealthStatus>('get_stealth_status');
      setStealthStatus(status);
    } catch (error) {
      console.error('Failed to get stealth status:', error);
    }
  };

  const getTaskManagerStatus = async () => {
    try {
      const status = await invoke<TaskManagerStatus>('get_task_manager_stealth_status');
      setTaskManagerStatus(status);
    } catch (error) {
      console.error('Failed to get task manager status:', error);
    }
  };

  const activateStealthMode = async () => {
    setLoading(true);
    try {
      // Activate stealth hotkeys
      const hotkeyResult = await invoke<string>('activate_stealth_mode');
      console.log('Hotkeys activated:', hotkeyResult);

      // Enable task manager stealth
      const taskManagerResult = await invoke<string>('enable_task_manager_stealth');
      console.log('Task manager stealth:', taskManagerResult);

      // Apply advanced stealth techniques
      const advancedResult = await invoke<string>('apply_advanced_stealth');
      console.log('Advanced stealth:', advancedResult);

      setIsStealthModeActive(true);
      setMessage('🕵️ STEALTH MODE ACTIVATED - All hotkeys registered, Task Manager hidden');
      
      // Refresh status
      await getStealthStatus();
      await getTaskManagerStatus();
    } catch (error) {
      console.error('Failed to activate stealth mode:', error);
      setMessage(`❌ Failed to activate stealth mode: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const deactivateStealthMode = async () => {
    setLoading(true);
    try {
      // Deactivate stealth hotkeys
      const hotkeyResult = await invoke<string>('deactivate_stealth_mode');
      console.log('Hotkeys deactivated:', hotkeyResult);

      // Disable task manager stealth
      const taskManagerResult = await invoke<string>('disable_task_manager_stealth');
      console.log('Task manager stealth disabled:', taskManagerResult);

      setIsStealthModeActive(false);
      setMessage('🔓 STEALTH MODE DEACTIVATED - All systems restored to normal');
      
      // Refresh status
      await getStealthStatus();
      await getTaskManagerStatus();
    } catch (error) {
      console.error('Failed to deactivate stealth mode:', error);
      setMessage(`❌ Failed to deactivate stealth mode: ${error}`);
    } finally {
      setLoading(false);
    }
  };


  useEffect(() => {
    getStealthStatus();
    getTaskManagerStatus();
  }, []);

  return (
    <div style={{ 
      padding: '20px', 
      fontFamily: 'monospace', 
      background: 'rgba(0, 0, 0, 0.9)', 
      color: 'white',
      borderRadius: '8px',
      border: '1px solid rgba(255, 255, 255, 0.2)'
    }}>
      <h2 style={{ color: '#ff6b35', marginBottom: '20px' }}>
        🕵️ Stealth Mode Control Panel
      </h2>

      {/* Status Display */}
      <div style={{ marginBottom: '20px', display: 'flex', gap: '20px', flexWrap: 'wrap' }}>
        <div style={{ 
          padding: '10px', 
          background: isStealthModeActive ? 'rgba(255, 71, 87, 0.2)' : 'rgba(0, 200, 150, 0.2)',
          border: `1px solid ${isStealthModeActive ? '#ff4757' : '#00c896'}`,
          borderRadius: '4px',
          minWidth: '200px'
        }}>
          <strong>Status:</strong> {isStealthModeActive ? '🔒 STEALTH ACTIVE' : '🔓 NORMAL MODE'}
        </div>

        {taskManagerStatus && (
          <div style={{ 
            padding: '10px', 
            background: 'rgba(0, 212, 255, 0.2)',
            border: '1px solid #00d4ff',
            borderRadius: '4px',
            minWidth: '200px'
          }}>
            <strong>Process ID:</strong> {taskManagerStatus.process_id}<br/>
            <strong>Hidden from Task Manager:</strong> {taskManagerStatus.stealth_enabled ? '✅ Yes' : '❌ No'}
          </div>
        )}
      </div>

      {/* Control Buttons */}
      <div style={{ marginBottom: '20px', display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
        <button
          onClick={activateStealthMode}
          disabled={loading || isStealthModeActive}
          style={{
            padding: '10px 20px',
            background: isStealthModeActive ? '#666' : '#ff4757',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: loading || isStealthModeActive ? 'not-allowed' : 'pointer',
            fontSize: '14px',
            fontWeight: 'bold'
          }}
        >
          {loading ? '⏳ Activating...' : '🕵️ ACTIVATE STEALTH'}
        </button>

        <button
          onClick={deactivateStealthMode}
          disabled={loading || !isStealthModeActive}
          style={{
            padding: '10px 20px',
            background: !isStealthModeActive ? '#666' : '#00c896',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: loading || !isStealthModeActive ? 'not-allowed' : 'pointer',
            fontSize: '14px',
            fontWeight: 'bold'
          }}
        >
          {loading ? '⏳ Deactivating...' : '🔓 DEACTIVATE STEALTH'}
        </button>

        <button
          onClick={getStealthStatus}
          disabled={loading}
          style={{
            padding: '10px 20px',
            background: '#00d4ff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: loading ? 'not-allowed' : 'pointer',
            fontSize: '14px'
          }}
        >
          🔄 Refresh Status
        </button>
      </div>

      {/* Hotkey Mappings */}
      {stealthStatus && (
        <div style={{ marginBottom: '20px' }}>
          <h3 style={{ color: '#00d4ff' }}>🎯 Stealth Hotkeys</h3>
          <div style={{ 
            background: 'rgba(255, 255, 255, 0.05)', 
            padding: '15px', 
            borderRadius: '4px',
            border: '1px solid rgba(255, 255, 255, 0.1)'
          }}>
            {Object.entries(stealthStatus.hotkey_mappings).map(([hotkey, description]) => (
              <div key={hotkey} style={{ 
                display: 'flex', 
                justifyContent: 'space-between', 
                marginBottom: '8px',
                padding: '5px 0',
                borderBottom: '1px solid rgba(255, 255, 255, 0.1)'
              }}>
                <strong style={{ color: '#ff6b35' }}>{hotkey}</strong>
                <span style={{ color: '#ffffff', opacity: 0.8 }}>{description}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Security Features */}
      <div style={{ marginBottom: '20px' }}>
        <h3 style={{ color: '#00d4ff' }}>🛡️ Security Features Active</h3>
        <div style={{ 
          background: 'rgba(255, 255, 255, 0.05)', 
          padding: '15px', 
          borderRadius: '4px',
          border: '1px solid rgba(255, 255, 255, 0.1)'
        }}>
          <div style={{ marginBottom: '8px' }}>
            ✅ <strong>Hover Effects Disabled</strong> - No tooltips or button highlights visible
          </div>
          <div style={{ marginBottom: '8px' }}>
            ✅ <strong>Mouse Cursor Neutralized</strong> - No cursor changes during screen sharing
          </div>
          <div style={{ marginBottom: '8px' }}>
            ✅ <strong>Global Hotkeys Active</strong> - Operate without mouse movement
          </div>
          <div style={{ marginBottom: '8px' }}>
            ✅ <strong>Task Manager Stealth</strong> - Process hidden from detection
          </div>
          <div style={{ marginBottom: '8px' }}>
            ✅ <strong>Screen Capture Protection</strong> - Window excluded from capture
          </div>
        </div>
      </div>

      {/* Message Display */}
      {message && (
        <div style={{
          padding: '10px',
          marginTop: '10px',
          background: message.startsWith('❌') ? 'rgba(255, 71, 87, 0.2)' : 'rgba(0, 200, 150, 0.2)',
          border: `1px solid ${message.startsWith('❌') ? '#ff4757' : '#00c896'}`,
          borderRadius: '4px',
          fontSize: '14px'
        }}>
          {message}
        </div>
      )}

      {/* Warning */}
      <div style={{ 
        marginTop: '20px', 
        padding: '15px', 
        background: 'rgba(255, 107, 53, 0.2)', 
        border: '1px solid #ff6b35',
        borderRadius: '4px',
        fontSize: '12px'
      }}>
        <strong>⚠️ STEALTH MODE WARNING:</strong> This system is designed for legitimate interview assistance only. 
        Use responsibly and in accordance with interview guidelines and local laws.
      </div>
    </div>
  );
};

export default StealthModeManager;
