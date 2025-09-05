import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface PermissionManagerProps {
  onPermissionsGranted?: () => void;
}

export function PermissionManager({ onPermissionsGranted }: PermissionManagerProps) {
  const [isChecking, setIsChecking] = useState(true);
  const [needsPermissions, setNeedsPermissions] = useState(false);
  const [isRequestingPermissions, setIsRequestingPermissions] = useState(false);
  const [permissionError, setPermissionError] = useState<string | null>(null);

  useEffect(() => {
    checkPermissions();
  }, []);

  const checkPermissions = async () => {
    try {
      setIsChecking(true);
      const hasPermissions = await invoke<boolean>('check_permissions');
      
      if (hasPermissions) {
        console.log('✅ Audio permissions already granted');
        onPermissionsGranted?.();
      } else {
        console.log('⚠️ Audio permissions needed');
        setNeedsPermissions(true);
      }
    } catch (error) {
      console.error('❌ Failed to check permissions:', error);
      setPermissionError(`Failed to check permissions: ${error}`);
    } finally {
      setIsChecking(false);
    }
  };

  const requestPermissions = async () => {
    try {
      setIsRequestingPermissions(true);
      setPermissionError(null);
      
      // Initialize first run permissions
      await invoke('initialize_first_run');
      
      // Request microphone permissions
      await invoke('request_permissions');
      
      // Wait a moment for user to grant permissions
      setTimeout(async () => {
        const hasPermissions = await invoke<boolean>('check_permissions');
        if (hasPermissions) {
          console.log('✅ Permissions granted successfully');
          setNeedsPermissions(false);
          onPermissionsGranted?.();
        } else {
          console.log('⚠️ Permissions still not granted');
          setPermissionError('Please enable microphone access in Windows Settings');
        }
        setIsRequestingPermissions(false);
      }, 3000);
      
    } catch (error) {
      console.error('❌ Failed to request permissions:', error);
      setPermissionError(`Failed to request permissions: ${error}`);
      setIsRequestingPermissions(false);
    }
  };

  if (isChecking) {
    return (
      <div className="permission-manager checking">
        <div className="permission-content">
          <div className="spinner"></div>
          <p>🔍 Checking audio permissions...</p>
        </div>
      </div>
    );
  }

  if (!needsPermissions) {
    return null; // Permissions already granted
  }

  return (
    <div className="permission-manager needs-permission">
      <div className="permission-content">
        <div className="permission-icon">🎤</div>
        <h3>Audio Permissions Required</h3>
        <p>
          MockMate needs access to your microphone and system audio to provide 
          real-time interview assistance. This is a one-time setup.
        </p>
        
        {permissionError && (
          <div className="permission-error">
            <p>⚠️ {permissionError}</p>
            <p>Please manually enable microphone access in:</p>
            <p><strong>Settings → Privacy & Security → Microphone</strong></p>
          </div>
        )}
        
        <div className="permission-actions">
          <button 
            onClick={requestPermissions}
            disabled={isRequestingPermissions}
            className="permission-btn primary"
          >
            {isRequestingPermissions ? '🔄 Opening Settings...' : '🎤 Grant Audio Access'}
          </button>
          
          <button 
            onClick={checkPermissions}
            className="permission-btn secondary"
          >
            ✅ I've Granted Permissions
          </button>
        </div>
        
        <div className="permission-help">
          <p>
            <strong>What this enables:</strong><br/>
            • Microphone recording for voice analysis<br/>
            • System audio capture for interview context<br/>
            • Real-time transcription services
          </p>
        </div>
      </div>
      
      <style jsx>{`
        .permission-manager {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.8);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }
        
        .permission-content {
          background: white;
          border-radius: 12px;
          padding: 2rem;
          max-width: 400px;
          text-align: center;
          box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }
        
        .permission-icon {
          font-size: 3rem;
          margin-bottom: 1rem;
        }
        
        .permission-error {
          background: #fee;
          border: 1px solid #fcc;
          border-radius: 8px;
          padding: 1rem;
          margin: 1rem 0;
          font-size: 0.9rem;
        }
        
        .permission-actions {
          display: flex;
          gap: 1rem;
          margin: 1.5rem 0;
        }
        
        .permission-btn {
          flex: 1;
          padding: 0.75rem 1rem;
          border: none;
          border-radius: 8px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }
        
        .permission-btn.primary {
          background: #007acc;
          color: white;
        }
        
        .permission-btn.primary:hover {
          background: #005f99;
        }
        
        .permission-btn.secondary {
          background: #f5f5f5;
          color: #333;
        }
        
        .permission-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }
        
        .permission-help {
          font-size: 0.8rem;
          color: #666;
          margin-top: 1rem;
          text-align: left;
        }
        
        .spinner {
          width: 2rem;
          height: 2rem;
          border: 3px solid #f3f3f3;
          border-top: 3px solid #007acc;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin: 0 auto 1rem;
        }
        
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
