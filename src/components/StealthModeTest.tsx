import React from 'react';
import { useStealthMode } from '../hooks/useStealthMode';
import StealthToggle from './StealthToggle';

const StealthModeTest: React.FC = () => {
  const { isStealthActive, lastHotkeyAction } = useStealthMode();

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
        🧪 Stealth Mode Test Panel
      </h2>

      {/* Status Indicator */}
      <div style={{ 
        marginBottom: '20px',
        padding: '15px',
        background: isStealthActive ? 'rgba(255, 71, 87, 0.2)' : 'rgba(0, 200, 150, 0.2)',
        border: `1px solid ${isStealthActive ? '#ff4757' : '#00c896'}`,
        borderRadius: '4px',
        textAlign: 'center',
        fontWeight: 'bold',
        fontSize: '16px'
      }}>
        {isStealthActive ? '🕵️ STEALTH MODE ACTIVE' : '🔓 NORMAL MODE'}
        {lastHotkeyAction && (
          <div style={{ marginTop: '10px', color: '#ff6b35' }}>
            Last hotkey: {lastHotkeyAction}
          </div>
        )}
      </div>

      {/* Stealth Toggle */}
      <div style={{ marginBottom: '30px', textAlign: 'center' }}>
        <StealthToggle compact={false} showLabel={true} />
      </div>

      {/* Test Elements with Tooltips */}
      <div style={{ 
        marginBottom: '20px',
        padding: '15px',
        background: 'rgba(255, 255, 255, 0.05)',
        borderRadius: '4px',
        border: '1px solid rgba(255, 255, 255, 0.1)'
      }}>
        <h3 style={{ color: '#00d4ff', marginBottom: '15px' }}>
          🎯 Test Elements (Hover to see tooltips when stealth is OFF)
        </h3>
        
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '15px' }}>
          
          {/* Button with title attribute */}
          <button
            style={{
              padding: '10px 15px',
              background: '#0096ff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '14px'
            }}
          >
            Button with Title
          </button>

          {/* Button with aria-label */}
          <button
            aria-label="This is a tooltip via aria-label"
            style={{
              padding: '10px 15px',
              background: '#00c896',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '14px'
            }}
          >
            Button with Aria-Label
          </button>

          {/* Input with placeholder */}
          <input
            type="text"
            placeholder="This placeholder reveals functionality"
            style={{
              padding: '10px',
              background: 'rgba(255, 255, 255, 0.1)',
              border: '1px solid rgba(255, 255, 255, 0.2)',
              borderRadius: '4px',
              color: 'white',
              fontSize: '14px'
            }}
          />

          {/* Image with alt text */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <img 
              src="data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTEyIDJMMTMuMDkgOC4yNkwyMCA5TDEzLjA5IDE1Ljc0TDEyIDIyTDEwLjkxIDE1Ljc0TDQgOUwxMC45MSA4LjI2TDEyIDJaIiBmaWxsPSIjZmY2YjM1Ii8+Cjwvc3ZnPgo="
              alt="This alt text shows on hover in some browsers"
              style={{ width: '24px', height: '24px' }}
            />
            <span style={{ fontSize: '14px' }}>Image with Alt Text</span>
          </div>

          {/* Element with data-tooltip */}
          <div
            data-tooltip="This is a custom data-tooltip"
            data-tip="Another tooltip library format"
            style={{
              padding: '10px',
              background: '#ff6b35',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '14px',
              textAlign: 'center'
            }}
          >
            Custom Tooltip Element
          </div>

          {/* Link with title */}
          <a
            href="#"
            onClick={(e) => e.preventDefault()}
            style={{
              color: '#00d4ff',
              textDecoration: 'none',
              fontSize: '14px',
              padding: '10px',
              display: 'block',
              border: '1px solid #00d4ff',
              borderRadius: '4px',
              textAlign: 'center'
            }}
          >
            Link with Title
          </a>
        </div>
      </div>

      {/* Instructions */}
      <div style={{
        padding: '15px',
        background: 'rgba(255, 107, 53, 0.1)',
        border: '1px solid rgba(255, 107, 53, 0.3)',
        borderRadius: '4px',
        fontSize: '13px',
        lineHeight: '1.6'
      }}>
        <h4 style={{ color: '#ff6b35', marginTop: 0 }}>🔬 Testing Instructions:</h4>
        <ol style={{ marginBottom: 0, paddingLeft: '20px' }}>
          <li><strong>Normal Mode:</strong> Hover over the test elements above to see tooltips</li>
          <li><strong>Activate Stealth:</strong> Click "ACTIVATE STEALTH" button</li>
          <li><strong>Test Stealth:</strong> Hover over the same elements - NO tooltips should appear</li>
          <li><strong>Check Developer Tools:</strong> Inspect elements to verify attributes are removed</li>
          <li><strong>Deactivate:</strong> Click "DEACTIVATE STEALTH" to restore tooltips</li>
        </ol>
      </div>

      {/* Current Status */}
      <div style={{
        marginTop: '20px',
        padding: '15px',
        background: 'rgba(0, 212, 255, 0.1)',
        border: '1px solid rgba(0, 212, 255, 0.3)',
        borderRadius: '4px',
        fontSize: '13px'
      }}>
        <h4 style={{ color: '#00d4ff', marginTop: 0 }}>📊 Current Status:</h4>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px' }}>
          <div><strong>Stealth Active:</strong> {isStealthActive ? 'YES ✅' : 'NO ❌'}</div>
          <div><strong>CSS Class Applied:</strong> {document.body.classList.contains('stealth-mode') ? 'YES ✅' : 'NO ❌'}</div>
          <div><strong>Title Attributes:</strong> {document.querySelectorAll('[title]').length} found</div>
          <div><strong>Aria-Label Attributes:</strong> {document.querySelectorAll('[aria-label]').length} found</div>
          <div><strong>Observer Active:</strong> {(window as any).stealthObserver ? 'YES ✅' : 'NO ❌'}</div>
          <div><strong>Last Hotkey:</strong> {lastHotkeyAction || 'None'}</div>
        </div>
      </div>
    </div>
  );
};

export default StealthModeTest;
