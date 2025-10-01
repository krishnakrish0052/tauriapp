import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface RealStealthStatus {
  process_id: number;
  original_name: string;
  disguised_name: string;
  stealth_active: boolean;
  techniques_active: { [key: string]: boolean };
  injected_process?: number;
  windows_hidden: number;
}

const RealStealthTest: React.FC = () => {
  const [status, setStatus] = useState<RealStealthStatus | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState('');

  // Fetch status on component mount
  useEffect(() => {
    fetchStatus();
    // Auto-refresh status every 5 seconds
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchStatus = async () => {
    try {
      const result = await invoke<RealStealthStatus>('get_real_stealth_status');
      setStatus(result);
    } catch (error) {
      console.error('Failed to get real stealth status:', error);
    }
  };

  const activateRealStealth = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<string>('activate_real_stealth');
      setMessage(result);
      await fetchStatus();
    } catch (error) {
      setMessage(`❌ Failed to activate real stealth: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const deactivateRealStealth = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<string>('deactivate_real_stealth');
      setMessage(result);
      await fetchStatus();
    } catch (error) {
      setMessage(`❌ Failed to deactivate real stealth: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div style={{
      padding: '20px',
      fontFamily: 'monospace',
      background: 'linear-gradient(135deg, #1a1a1a, #2d2d2d)',
      color: 'white',
      borderRadius: '12px',
      border: '2px solid #ff4757',
      boxShadow: '0 0 20px rgba(255, 71, 87, 0.3)',
    }}>
      <h2 style={{ color: '#ff4757', marginBottom: '20px', textAlign: 'center' }}>
        🔥 REAL TASK MANAGER STEALTH
      </h2>
      <p style={{ color: '#ffa502', marginBottom: '20px', textAlign: 'center' }}>
        ⚠️ This will ACTUALLY hide the process from Task Manager
      </p>

      {/* Status Display */}
      <div style={{ marginBottom: '20px' }}>
        <h3 style={{ color: '#00d4ff' }}>📊 Current Status</h3>
        {status ? (
          <div style={{
            background: 'rgba(0, 212, 255, 0.1)',
            border: '1px solid #00d4ff',
            borderRadius: '8px',
            padding: '15px',
            marginTop: '10px'
          }}>
            <p><strong>Process ID:</strong> {status.process_id}</p>
            <p><strong>Original Name:</strong> {status.original_name}</p>
            <p><strong>Disguised As:</strong> {status.disguised_name}</p>
            <p><strong>Stealth Active:</strong> {status.stealth_active ? '🔒 YES' : '🔓 NO'}</p>
            <p><strong>Windows Hidden:</strong> {status.windows_hidden}</p>
            {status.injected_process && (
              <p><strong>Injected Process PID:</strong> {status.injected_process}</p>
            )}
          </div>
        ) : (
          <p>Loading status...</p>
        )}
      </div>

      {/* Techniques Status */}
      {status && (
        <div style={{ marginBottom: '20px' }}>
          <h3 style={{ color: '#00d4ff' }}>🛡️ Active Techniques</h3>
          <div style={{
            background: 'rgba(0, 212, 255, 0.1)',
            border: '1px solid #00d4ff',
            borderRadius: '8px',
            padding: '15px',
            marginTop: '10px'
          }}>
            {Object.entries(status.techniques_active).map(([technique, active]) => (
              <div key={technique} style={{ marginBottom: '8px' }}>
                <span style={{ color: active ? '#00c896' : '#666' }}>
                  {active ? '✅' : '❌'} {technique.replace(/_/g, ' ').toUpperCase()}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Control Buttons */}
      <div style={{ marginBottom: '20px', display: 'flex', gap: '15px', justifyContent: 'center' }}>
        <button
          onClick={activateRealStealth}
          disabled={isLoading || (status?.stealth_active ?? false)}
          style={{
            padding: '12px 24px',
            background: (status?.stealth_active ?? false) ? '#666' : '#ff4757',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            cursor: isLoading || (status?.stealth_active ?? false) ? 'not-allowed' : 'pointer',
            fontSize: '16px',
            fontWeight: 'bold',
            boxShadow: '0 4px 12px rgba(255, 71, 87, 0.3)',
          }}
        >
          {isLoading ? '⏳ Processing...' : '🔥 ACTIVATE REAL STEALTH'}
        </button>

        <button
          onClick={deactivateRealStealth}
          disabled={isLoading || !(status?.stealth_active ?? true)}
          style={{
            padding: '12px 24px',
            background: !(status?.stealth_active ?? true) ? '#666' : '#00c896',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            cursor: isLoading || !(status?.stealth_active ?? true) ? 'not-allowed' : 'pointer',
            fontSize: '16px',
            fontWeight: 'bold',
            boxShadow: '0 4px 12px rgba(0, 200, 150, 0.3)',
          }}
        >
          {isLoading ? '⏳ Processing...' : '🔓 DEACTIVATE STEALTH'}
        </button>

        <button
          onClick={fetchStatus}
          disabled={isLoading}
          style={{
            padding: '12px 24px',
            background: '#00d4ff',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            cursor: isLoading ? 'not-allowed' : 'pointer',
            fontSize: '16px',
            fontWeight: 'bold',
            boxShadow: '0 4px 12px rgba(0, 212, 255, 0.3)',
          }}
        >
          🔄 REFRESH
        </button>
      </div>

      {/* Message Display */}
      {message && (
        <div style={{
          padding: '15px',
          marginTop: '20px',
          background: message.startsWith('❌') ? 'rgba(255, 71, 87, 0.2)' : 'rgba(0, 200, 150, 0.2)',
          border: `1px solid ${message.startsWith('❌') ? '#ff4757' : '#00c896'}`,
          borderRadius: '8px',
          fontSize: '14px',
          whiteSpace: 'pre-line'
        }}>
          {message}
        </div>
      )}

      {/* Instructions */}
      <div style={{
        marginTop: '20px',
        padding: '15px',
        background: 'rgba(255, 107, 53, 0.2)',
        border: '1px solid #ff6b35',
        borderRadius: '8px',
        fontSize: '12px'
      }}>
        <h4 style={{ color: '#ff6b35', marginBottom: '10px' }}>🧪 Testing Instructions:</h4>
        <ol style={{ paddingLeft: '20px', lineHeight: '1.6' }}>
          <li>Open Task Manager (Ctrl+Shift+Esc)</li>
          <li>Note the current "MockMate" process</li>
          <li>Click "ACTIVATE REAL STEALTH"</li>
          <li>Check Task Manager - process should be disguised/hidden</li>
          <li>Windows should be hidden from taskbar and Alt+Tab</li>
          <li>Use "DEACTIVATE STEALTH" to restore visibility</li>
        </ol>
        <p style={{ marginTop: '10px', color: '#ffa502' }}>
          ⚠️ <strong>Warning:</strong> This implements real hiding techniques for legitimate interview assistance only.
        </p>
      </div>
    </div>
  );
};

export default RealStealthTest;