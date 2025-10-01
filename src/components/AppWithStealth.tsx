import React, { useState } from 'react';
import StealthModeManager from './StealthModeManager';
import StealthToggle from './StealthToggle';
import { useStealthMode } from '../hooks/useStealthMode';

// Mock components representing your existing app
const MockMicrophoneCapture = () => {
  const { lastHotkeyAction } = useStealthMode();
  
  return (
    <div style={{ 
      padding: '20px', 
      background: 'rgba(0, 150, 255, 0.1)', 
      borderRadius: '8px',
      border: '1px solid rgba(0, 150, 255, 0.3)',
      marginBottom: '20px'
    }}>
      <h3 style={{ color: '#0096ff' }}>🎤 Microphone Capture</h3>
      <p>Real-time transcription active...</p>
      <button 
        style={{ 
          padding: '8px 16px', 
          background: '#0096ff', 
          color: 'white', 
          border: 'none', 
          borderRadius: '4px',
          cursor: 'pointer'
        }}
      >
        Toggle Mic (Shift+Ctrl+M)
      </button>
      
      {lastHotkeyAction === 'mic_toggle' && (
        <div style={{ 
          marginTop: '10px', 
          color: '#ff6b35', 
          fontWeight: 'bold',
          background: 'rgba(255, 107, 53, 0.1)',
          padding: '5px 10px',
          borderRadius: '4px'
        }}>
          🎯 Microphone toggled via hotkey!
        </div>
      )}
    </div>
  );
};

const MockSystemAudio = () => {
  const { lastHotkeyAction } = useStealthMode();
  
  return (
    <div style={{ 
      padding: '20px', 
      background: 'rgba(0, 200, 150, 0.1)', 
      borderRadius: '8px',
      border: '1px solid rgba(0, 200, 150, 0.3)',
      marginBottom: '20px'
    }}>
      <h3 style={{ color: '#00c896' }}>🔊 System Audio Capture</h3>
      <p>Monitoring system audio for questions...</p>
      <button 
        style={{ 
          padding: '8px 16px', 
          background: '#00c896', 
          color: 'white', 
          border: 'none', 
          borderRadius: '4px',
          cursor: 'pointer'
        }}
      >
        Toggle System Audio (Shift+Ctrl+S)
      </button>
      
      {lastHotkeyAction === 'system_sound_toggle' && (
        <div style={{ 
          marginTop: '10px', 
          color: '#ff6b35', 
          fontWeight: 'bold',
          background: 'rgba(255, 107, 53, 0.1)',
          padding: '5px 10px',
          borderRadius: '4px'
        }}>
          🎯 System audio toggled via hotkey!
        </div>
      )}
    </div>
  );
};

const MockAIResponse = () => {
  const { lastHotkeyAction } = useStealthMode();
  
  return (
    <div style={{ 
      padding: '20px', 
      background: 'rgba(255, 107, 53, 0.1)', 
      borderRadius: '8px',
      border: '1px solid rgba(255, 107, 53, 0.3)',
      marginBottom: '20px'
    }}>
      <h3 style={{ color: '#ff6b35' }}>🤖 AI Response</h3>
      <p>Ready to generate answers...</p>
      <textarea 
        placeholder="AI-generated answer will appear here..."
        style={{ 
          width: '100%', 
          height: '80px', 
          padding: '10px',
          background: 'rgba(255, 255, 255, 0.1)',
          border: '1px solid rgba(255, 255, 255, 0.2)',
          borderRadius: '4px',
          color: 'white',
          resize: 'none'
        }}
      />
      <button 
        style={{ 
          padding: '8px 16px', 
          background: '#ff6b35', 
          color: 'white', 
          border: 'none', 
          borderRadius: '4px',
          cursor: 'pointer',
          marginTop: '10px'
        }}
      >
        Generate Answer (Shift+Ctrl+Z)
      </button>
      
      {lastHotkeyAction === 'ai_answer_trigger' && (
        <div style={{ 
          marginTop: '10px', 
          color: '#00c896', 
          fontWeight: 'bold',
          background: 'rgba(0, 200, 150, 0.1)',
          padding: '5px 10px',
          borderRadius: '4px'
        }}>
          🎯 AI answer generation triggered via hotkey!
        </div>
      )}
    </div>
  );
};

const AppWithStealth: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'dashboard' | 'stealth'>('dashboard');
  const { isStealthActive } = useStealthMode();

  const tabStyle = (isActive: boolean): React.CSSProperties => ({
    padding: '10px 20px',
    background: isActive ? '#ff6b35' : 'rgba(255, 255, 255, 0.1)',
    color: 'white',
    border: 'none',
    borderRadius: '8px 8px 0 0',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 'bold',
    marginRight: '2px'
  });

  return (
    <div style={{ 
      minHeight: '100vh', 
      background: 'linear-gradient(135deg, #1e3c72, #2a5298)',
      padding: '20px',
      fontFamily: 'Arial, sans-serif'
    }}>
      {/* Header with stealth status */}
      <div style={{ 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center', 
        marginBottom: '30px',
        background: 'rgba(0, 0, 0, 0.3)',
        padding: '15px 20px',
        borderRadius: '10px'
      }}>
        <div>
          <h1 style={{ color: 'white', margin: 0, fontSize: '24px' }}>
            🎯 MockMate Pro
          </h1>
          <p style={{ color: 'rgba(255, 255, 255, 0.7)', margin: '5px 0 0 0' }}>
            Interview Assistant with Stealth Capabilities
          </p>
        </div>
        
        <StealthToggle compact={false} showLabel={true} />
      </div>

      {/* Tab Navigation */}
      <div style={{ marginBottom: '0' }}>
        <button 
          onClick={() => setActiveTab('dashboard')}
          style={tabStyle(activeTab === 'dashboard')}
        >
          📊 Dashboard
        </button>
        <button 
          onClick={() => setActiveTab('stealth')}
          style={tabStyle(activeTab === 'stealth')}
        >
          🕵️ Stealth Control
        </button>
      </div>

      {/* Content Area */}
      <div style={{ 
        background: 'rgba(0, 0, 0, 0.4)', 
        borderRadius: '0 10px 10px 10px',
        padding: '30px',
        minHeight: '600px'
      }}>
        {activeTab === 'dashboard' ? (
          <div>
            <h2 style={{ color: 'white', marginBottom: '20px' }}>
              Main Dashboard
            </h2>
            
            {/* Status Banner */}
            {isStealthActive && (
              <div style={{
                background: 'rgba(255, 71, 87, 0.2)',
                border: '2px solid #ff4757',
                borderRadius: '8px',
                padding: '15px',
                marginBottom: '20px',
                color: '#ff4757',
                fontWeight: 'bold',
                textAlign: 'center'
              }}>
                🕵️ STEALTH MODE ACTIVE - No hover effects, hotkeys enabled, hidden from Task Manager
              </div>
            )}
            
            {/* Mock App Components */}
            <MockMicrophoneCapture />
            <MockSystemAudio />
            <MockAIResponse />
            
            {/* Hotkey Reference */}
            <div style={{
              background: 'rgba(255, 255, 255, 0.05)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              borderRadius: '8px',
              padding: '20px',
              color: 'white'
            }}>
              <h3 style={{ color: '#00d4ff', marginBottom: '15px' }}>
                🎯 Stealth Hotkeys Reference
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px', fontSize: '14px' }}>
                <div><strong>Shift+Ctrl+S</strong> - Toggle System Audio</div>
                <div><strong>Shift+Ctrl+M</strong> - Toggle Microphone</div>
                <div><strong>Shift+Ctrl+Z</strong> - Generate AI Answer</div>
                <div><strong>Shift+Ctrl+X</strong> - Hide/Show Window</div>
                <div><strong>Shift+Ctrl+A</strong> - Analyze Screen</div>
                <div><strong>Shift+Ctrl+I</strong> - Manual Input Mode</div>
                <div><strong>Shift+Ctrl+Enter</strong> - Submit Question</div>
                <div><strong>Shift+Ctrl+C</strong> - Clear Listening Area</div>
              </div>
            </div>
          </div>
        ) : (
          <StealthModeManager />
        )}
      </div>
    </div>
  );
};

export default AppWithStealth;
