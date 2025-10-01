import React from 'react';
import { useStealthMode } from '../hooks/useStealthMode';

interface StealthToggleProps {
  compact?: boolean;
  showLabel?: boolean;
  className?: string;
}

const StealthToggle: React.FC<StealthToggleProps> = ({ 
  compact = false, 
  showLabel = true,
  className = '' 
}) => {
  const { 
    isStealthActive, 
    toggleStealth, 
    isLoading, 
    error, 
    lastHotkeyAction 
  } = useStealthMode();

  const buttonStyle: React.CSSProperties = {
    padding: compact ? '8px 12px' : '12px 16px',
    background: isStealthActive 
      ? 'linear-gradient(135deg, #ff4757, #ff3838)' 
      : 'linear-gradient(135deg, #00c896, #00b894)',
    color: 'white',
    border: 'none',
    borderRadius: compact ? '6px' : '8px',
    cursor: isLoading ? 'not-allowed' : 'pointer',
    fontSize: compact ? '12px' : '14px',
    fontWeight: 'bold',
    fontFamily: 'monospace',
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    boxShadow: isStealthActive 
      ? '0 2px 8px rgba(255, 71, 87, 0.3)' 
      : '0 2px 8px rgba(0, 200, 150, 0.3)',
    transition: 'all 0.2s ease',
    opacity: isLoading ? 0.7 : 1,
    position: 'relative',
    overflow: 'hidden'
  };

  const iconStyle: React.CSSProperties = {
    fontSize: compact ? '14px' : '16px',
    animation: isStealthActive ? 'pulse 2s infinite' : 'none'
  };

  return (
    <div className={className}>
      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.6; }
        }
        .stealth-button:hover {
          transform: translateY(-1px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        }
        .stealth-mode .stealth-button:hover {
          transform: none !important;
          box-shadow: none !important;
        }
      `}</style>
      
      <button
        onClick={toggleStealth}
        disabled={isLoading}
        style={buttonStyle}
        className="stealth-button"
      >
        <span style={iconStyle}>
          {isLoading ? '⏳' : isStealthActive ? '🕵️' : '🔓'}
        </span>
        
        {showLabel && (
          <span>
            {isLoading 
              ? (isStealthActive ? 'Deactivating...' : 'Activating...') 
              : (isStealthActive ? 'STEALTH ON' : 'STEALTH OFF')
            }
          </span>
        )}
      </button>

      {/* Status indicators */}
      {!compact && (
        <div style={{ marginTop: '8px', fontSize: '11px', fontFamily: 'monospace' }}>
          {lastHotkeyAction && (
            <div style={{ 
              color: '#ff6b35', 
              background: 'rgba(255, 107, 53, 0.1)', 
              padding: '4px 8px', 
              borderRadius: '4px',
              marginBottom: '4px',
              border: '1px solid rgba(255, 107, 53, 0.3)'
            }}>
              🎯 {lastHotkeyAction}
            </div>
          )}
          
          {error && (
            <div style={{ 
              color: '#ff4757', 
              background: 'rgba(255, 71, 87, 0.1)', 
              padding: '4px 8px', 
              borderRadius: '4px',
              border: '1px solid rgba(255, 71, 87, 0.3)'
            }}>
              ❌ {error}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default StealthToggle;
