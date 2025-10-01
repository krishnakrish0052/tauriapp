import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useStealthMode } from '../hooks/useStealthMode';

interface HotkeyEvent {
  action: string;
  hotkey_id: number;
  timestamp: string;
}

const StealthModeDemo: React.FC = () => {
  const { isStealthActive, activateStealth, deactivateStealth, isLoading } = useStealthMode();
  const [hotkeyEvents, setHotkeyEvents] = useState<HotkeyEvent[]>([]);
  const [lastAction, setLastAction] = useState<string>('');

  // Listen for hotkey events
  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen<HotkeyEvent>('stealth-hotkey', (event) => {
        console.log('🎯 Hotkey received:', event.payload);
        setHotkeyEvents(prev => [event.payload, ...prev.slice(0, 4)]);
        setLastAction(event.payload.action);
        
        // Clear after 3 seconds
        setTimeout(() => setLastAction(''), 3000);
      });
      return unlisten;
    };

    let unlisten: (() => void) | null = null;
    setupListener().then(fn => { unlisten = fn; });
    
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleManualTrigger = async (action: string) => {
    try {
      await invoke('test_stealth_hotkey', { hotkeyName: getHotkeyName(action) });
    } catch (error) {
      console.error('Failed to trigger hotkey:', error);
    }
  };

  const getHotkeyName = (action: string): string => {
    const mapping: { [key: string]: string } = {
      'system_sound_toggle': 'Shift+Ctrl+S',
      'ai_answer_trigger': 'Shift+Ctrl+Z',
      'window_toggle': 'Shift+Ctrl+X',
      'mic_toggle': 'Shift+Ctrl+M',
      'analyze_screen': 'Shift+Ctrl+A',
      'manual_input': 'Shift+Ctrl+I',
      'submit_question': 'Shift+Ctrl+Enter',
      'clear_area': 'Shift+Ctrl+C'
    };
    return mapping[action] || action;
  };

  const getActionDescription = (action: string): string => {
    const descriptions: { [key: string]: string } = {
      'system_sound_toggle': 'Toggle System Audio Recording',
      'ai_answer_trigger': 'Generate AI Answer',
      'window_toggle': 'Hide/Show Window',
      'mic_toggle': 'Toggle Microphone',
      'analyze_screen': 'Analyze Current Screen',
      'manual_input': 'Manual Question Entry Mode',
      'submit_question': 'Submit Current Question',
      'clear_area': 'Clear Listening Area'
    };
    return descriptions[action] || action;
  };

  const testTooltips = () => {
    const titleElements = document.querySelectorAll('[title]');
    const ariaElements = document.querySelectorAll('[aria-label]');
    return {
      titleCount: titleElements.length,
      ariaCount: ariaElements.length,
      stealthClass: document.body.classList.contains('stealth-mode')
    };
  };

  const tooltipStatus = testTooltips();

  return (
    <div style={{
      padding: '20px',
      fontFamily: 'monospace',
      background: 'linear-gradient(135deg, rgba(0, 0, 0, 0.95), rgba(30, 30, 30, 0.95))',
      color: 'white',
      borderRadius: '12px',
      border: '2px solid rgba(255, 255, 255, 0.1)',
      boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
      maxWidth: '900px',
      margin: '0 auto'
    }}>
      {/* Header */}
      <div style={{ textAlign: 'center', marginBottom: '30px' }}>
        <h1 style={{ 
          color: '#ff6b35', 
          margin: '0 0 10px 0', 
          fontSize: '32px',
          textShadow: '0 0 10px rgba(255, 107, 53, 0.5)' 
        }}>
          🕵️ MockMate Stealth Mode Demo
        </h1>
        <p style={{ color: 'rgba(255, 255, 255, 0.8)', margin: 0, fontSize: '16px' }}>
          Complete Interview Stealth System - Hide tooltips, use hotkeys, avoid detection
        </p>
      </div>

      {/* Status Panel */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '20px',
        marginBottom: '30px'
      }}>
        {/* Stealth Status */}
        <div style={{
          padding: '20px',
          background: isStealthActive 
            ? 'linear-gradient(135deg, rgba(255, 71, 87, 0.2), rgba(200, 50, 70, 0.1))'
            : 'linear-gradient(135deg, rgba(0, 200, 150, 0.2), rgba(0, 150, 100, 0.1))',
          border: `2px solid ${isStealthActive ? '#ff4757' : '#00c896'}`,
          borderRadius: '8px',
          textAlign: 'center'
        }}>
          <div style={{ fontSize: '48px', marginBottom: '10px' }}>
            {isStealthActive ? '🕵️' : '🔓'}
          </div>
          <h3 style={{ 
            margin: '0 0 10px 0', 
            color: isStealthActive ? '#ff4757' : '#00c896',
            fontSize: '18px' 
          }}>
            {isStealthActive ? 'STEALTH ACTIVE' : 'NORMAL MODE'}
          </h3>
          <button
            onClick={isStealthActive ? deactivateStealth : activateStealth}
            disabled={isLoading}
            style={{
              padding: '12px 24px',
              background: isStealthActive 
                ? 'linear-gradient(135deg, #00c896, #00b894)' 
                : 'linear-gradient(135deg, #ff4757, #ff3838)',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              fontWeight: 'bold',
              opacity: isLoading ? 0.7 : 1,
              transform: isLoading ? 'scale(0.98)' : 'scale(1)',
              transition: 'all 0.2s ease'
            }}
          >
            {isLoading ? '⏳ Working...' : isStealthActive ? '🔓 DEACTIVATE' : '🕵️ ACTIVATE'}
          </button>
        </div>

        {/* System Status */}
        <div style={{
          padding: '20px',
          background: 'rgba(0, 212, 255, 0.1)',
          border: '2px solid rgba(0, 212, 255, 0.3)',
          borderRadius: '8px'
        }}>
          <h3 style={{ margin: '0 0 15px 0', color: '#00d4ff' }}>📊 System Status</h3>
          <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
            <div><strong>Tooltip Elements:</strong> {tooltipStatus.titleCount} title, {tooltipStatus.ariaCount} aria-label</div>
            <div><strong>Stealth CSS:</strong> {tooltipStatus.stealthClass ? '✅ Applied' : '❌ Not applied'}</div>
            <div><strong>Hotkey Events:</strong> {hotkeyEvents.length} received</div>
            <div><strong>Last Action:</strong> {lastAction || 'None'}</div>
          </div>
        </div>
      </div>

      {/* Hotkey Controls */}
      <div style={{
        marginBottom: '30px',
        padding: '20px',
        background: 'rgba(255, 255, 255, 0.05)',
        borderRadius: '8px',
        border: '1px solid rgba(255, 255, 255, 0.1)'
      }}>
        <h3 style={{ color: '#00d4ff', margin: '0 0 20px 0' }}>🎮 Hotkey Controls</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '10px' }}>
          {[
            { action: 'system_sound_toggle', icon: '🔊' },
            { action: 'ai_answer_trigger', icon: '🤖' },
            { action: 'window_toggle', icon: '🪟' },
            { action: 'mic_toggle', icon: '🎤' },
            { action: 'analyze_screen', icon: '📋' },
            { action: 'manual_input', icon: '✍️' },
            { action: 'submit_question', icon: '📤' },
            { action: 'clear_area', icon: '🧹' }
          ].map(({ action, icon }) => (
            <button
              key={action}
              onClick={() => handleManualTrigger(action)}
              disabled={!isStealthActive}
              style={{
                padding: '10px 12px',
                background: lastAction === action 
                  ? 'linear-gradient(135deg, #ff6b35, #ff5722)'
                  : 'linear-gradient(135deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.05))',
                color: 'white',
                border: lastAction === action ? '2px solid #ff6b35' : '1px solid rgba(255, 255, 255, 0.2)',
                borderRadius: '6px',
                cursor: !isStealthActive ? 'not-allowed' : 'pointer',
                fontSize: '12px',
                textAlign: 'left',
                opacity: !isStealthActive ? 0.5 : 1,
                transform: lastAction === action ? 'scale(1.05)' : 'scale(1)',
                transition: 'all 0.2s ease',
                boxShadow: lastAction === action ? '0 4px 12px rgba(255, 107, 53, 0.3)' : 'none'
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span style={{ fontSize: '16px' }}>{icon}</span>
                <div>
                  <div style={{ fontWeight: 'bold', fontSize: '11px', color: '#00d4ff' }}>
                    {getHotkeyName(action)}
                  </div>
                  <div style={{ fontSize: '10px', opacity: 0.8 }}>
                    {action.split('_').join(' ').toUpperCase()}
                  </div>
                </div>
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Event Log */}
      <div style={{
        marginBottom: '30px',
        padding: '20px',
        background: 'rgba(0, 0, 0, 0.3)',
        borderRadius: '8px',
        border: '1px solid rgba(255, 255, 255, 0.1)'
      }}>
        <h3 style={{ color: '#00d4ff', margin: '0 0 15px 0' }}>📋 Recent Hotkey Events</h3>
        {hotkeyEvents.length === 0 ? (
          <div style={{ 
            color: 'rgba(255, 255, 255, 0.5)', 
            fontStyle: 'italic', 
            textAlign: 'center',
            padding: '20px'
          }}>
            No hotkey events yet. Activate stealth mode and try the buttons above!
          </div>
        ) : (
          <div style={{ maxHeight: '150px', overflowY: 'auto' }}>
            {hotkeyEvents.map((event, index) => (
              <div
                key={`${event.timestamp}-${index}`}
                style={{
                  padding: '8px 12px',
                  background: 'rgba(255, 255, 255, 0.05)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '4px',
                  marginBottom: '8px',
                  fontSize: '12px',
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center'
                }}
              >
                <div>
                  <strong style={{ color: '#ff6b35' }}>{getActionDescription(event.action)}</strong>
                  <span style={{ color: 'rgba(255, 255, 255, 0.7)', marginLeft: '10px' }}>
                    ({getHotkeyName(event.action)})
                  </span>
                </div>
                <div style={{ color: 'rgba(255, 255, 255, 0.5)', fontSize: '10px' }}>
                  ID: {event.hotkey_id}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Test Elements */}
      <div style={{
        padding: '20px',
        background: 'rgba(255, 107, 53, 0.1)',
        border: '2px solid rgba(255, 107, 53, 0.3)',
        borderRadius: '8px'
      }}>
        <h3 style={{ color: '#ff6b35', margin: '0 0 15px 0' }}>🧪 Tooltip Test Elements</h3>
        <p style={{ fontSize: '13px', marginBottom: '15px', color: 'rgba(255, 255, 255, 0.8)' }}>
          Hover over these elements. When stealth mode is ACTIVE, no tooltips should appear:
        </p>
        
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', gap: '10px' }}>
          <button
            style={{
              padding: '10px',
              background: '#0096ff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '12px'
            }}
          >
            Hover Test Button
          </button>
          
          <input
            type="text"
            placeholder="This placeholder should be hidden"
            style={{
              padding: '8px',
              background: 'rgba(255, 255, 255, 0.1)',
              border: '1px solid rgba(255, 255, 255, 0.2)',
              borderRadius: '4px',
              color: 'white',
              fontSize: '12px'
            }}
          />
          
          <div
            aria-label="Aria label tooltip test"
            data-tooltip="Custom tooltip test"
            style={{
              padding: '10px',
              background: '#00c896',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '12px',
              textAlign: 'center'
            }}
          >
            Custom Tooltips
          </div>
        </div>
      </div>
    </div>
  );
};

export default StealthModeDemo;
