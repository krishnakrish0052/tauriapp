import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface StereoMixCapabilities {
  stereo_mix_available: boolean;
  alternative_devices: string[];
  requires_manual_enable: boolean;
  system_info: {
    os: string;
    arch: string;
  };
}

interface StereoMixManagerProps {
  onStereoMixReady?: (enabled: boolean) => void;
  autoEnableOnFirstRun?: boolean;
  showInstructions?: boolean;
}

export function StereoMixManager({ 
  onStereoMixReady, 
  autoEnableOnFirstRun = true,
  showInstructions = true 
}: StereoMixManagerProps) {
  const [isChecking, setIsChecking] = useState(true);
  const [stereoMixEnabled, setStereoMixEnabled] = useState(false);
  const [capabilities, setCapabilities] = useState<StereoMixCapabilities | null>(null);
  const [isEnabling, setIsEnabling] = useState(false);
  const [enableMessage, setEnableMessage] = useState<string | null>(null);
  const [showManualInstructions, setShowManualInstructions] = useState(false);
  const [instructions, setInstructions] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initializeStereoMix();
  }, []);

  const initializeStereoMix = async () => {
    try {
      setIsChecking(true);
      setError(null);

      // Check current system capabilities
      console.log('🔍 Checking Stereo Mix capabilities...');
      const caps = await invoke<StereoMixCapabilities>('get_stereo_mix_capabilities');
      setCapabilities(caps);
      
      console.log('📊 Stereo Mix capabilities:', caps);

      // Check if Stereo Mix is currently enabled
      const isEnabled = await invoke<boolean>('check_stereo_mix_enabled');
      setStereoMixEnabled(isEnabled);
      
      console.log('🎵 Stereo Mix currently enabled:', isEnabled);

      if (isEnabled) {
        console.log('✅ Stereo Mix is already enabled');
        onStereoMixReady?.(true);
      } else if (autoEnableOnFirstRun) {
        console.log('🚀 Attempting automatic Stereo Mix enablement...');
        await attemptAutoEnable();
      } else {
        console.log('⚠️ Stereo Mix disabled, auto-enable disabled');
        onStereoMixReady?.(false);
      }
    } catch (error) {
      console.error('❌ Failed to initialize Stereo Mix:', error);
      setError(`Failed to check Stereo Mix: ${error}`);
      onStereoMixReady?.(false);
    } finally {
      setIsChecking(false);
    }
  };

  const attemptAutoEnable = async () => {
    try {
      setIsEnabling(true);
      setEnableMessage('Attempting to enable Stereo Mix automatically...');
      
      console.log('🔧 Starting automatic Stereo Mix enablement...');
      const result = await invoke<string>('enable_stereo_mix');
      
      console.log('📝 Auto-enable result:', result);
      setEnableMessage(result);
      
      // Wait a moment and check if it worked
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      const isEnabled = await invoke<boolean>('check_stereo_mix_enabled');
      setStereoMixEnabled(isEnabled);
      
      if (isEnabled) {
        console.log('✅ Stereo Mix enabled successfully');
        setEnableMessage('✅ Stereo Mix enabled successfully!');
        setTimeout(() => {
          onStereoMixReady?.(true);
          setEnableMessage(null);
        }, 2000);
      } else {
        console.log('⚠️ Automatic enablement may have failed, showing manual instructions');
        if (showInstructions) {
          await loadManualInstructions();
          setShowManualInstructions(true);
        }
        onStereoMixReady?.(false);
      }
    } catch (error) {
      console.error('❌ Failed to auto-enable Stereo Mix:', error);
      setEnableMessage(`Auto-enable failed: ${error}`);
      
      if (showInstructions) {
        await loadManualInstructions();
        setShowManualInstructions(true);
      }
      onStereoMixReady?.(false);
    } finally {
      setIsEnabling(false);
    }
  };

  const loadManualInstructions = async () => {
    try {
      const instructionList = await invoke<string[]>('get_stereo_mix_instructions');
      setInstructions(instructionList);
    } catch (error) {
      console.error('Failed to load manual instructions:', error);
    }
  };

  const openRecordingDevices = async () => {
    try {
      await invoke('open_recording_devices');
      console.log('📂 Opened Windows Recording devices');
    } catch (error) {
      console.error('❌ Failed to open Recording devices:', error);
      setError(`Failed to open Recording devices: ${error}`);
    }
  };

  const recheckStereoMix = async () => {
    try {
      setIsChecking(true);
      const isEnabled = await invoke<boolean>('check_stereo_mix_enabled');
      setStereoMixEnabled(isEnabled);
      
      if (isEnabled) {
        console.log('✅ Stereo Mix now enabled after manual setup');
        setShowManualInstructions(false);
        setEnableMessage('✅ Stereo Mix is now enabled!');
        setTimeout(() => {
          onStereoMixReady?.(true);
          setEnableMessage(null);
        }, 2000);
      } else {
        setError('Stereo Mix still not detected. Please make sure it is enabled and restart the app.');
      }
    } catch (error) {
      console.error('❌ Failed to recheck Stereo Mix:', error);
      setError(`Failed to recheck: ${error}`);
    } finally {
      setIsChecking(false);
    }
  };

  if (isChecking) {
    return (
      <div className="stereo-mix-manager checking">
        <div className="stereo-mix-content">
          <div className="spinner"></div>
          <p>🔍 Checking Stereo Mix availability...</p>
        </div>
      </div>
    );
  }

  if (stereoMixEnabled && !enableMessage) {
    return null; // Everything is working, don't show UI
  }

  if (enableMessage && !showManualInstructions) {
    return (
      <div className="stereo-mix-manager status">
        <div className="stereo-mix-content">
          <div className="status-icon">
            {isEnabling ? '🔄' : enableMessage.startsWith('✅') ? '✅' : '⚠️'}
          </div>
          <p>{enableMessage}</p>
          {isEnabling && <div className="spinner small"></div>}
        </div>
      </div>
    );
  }

  if (showManualInstructions) {
    return (
      <div className="stereo-mix-manager manual">
        <div className="stereo-mix-content">
          <div className="stereo-mix-icon">🎵</div>
          <h3>Enable Stereo Mix for System Audio</h3>
          
          <p>
            To capture system audio, we need to enable "Stereo Mix" on your system. 
            This allows us to record what you hear during interviews.
          </p>

          {capabilities && capabilities.alternative_devices.length > 0 && (
            <div className="alternative-devices">
              <p><strong>Alternative devices found:</strong></p>
              <ul>
                {capabilities.alternative_devices.map((device, index) => (
                  <li key={index}>{device}</li>
                ))}
              </ul>
            </div>
          )}

          <div className="manual-instructions">
            <h4>Manual Setup Steps:</h4>
            <ol>
              {instructions.map((instruction, index) => (
                <li key={index}>{instruction}</li>
              ))}
            </ol>
          </div>

          {error && (
            <div className="stereo-mix-error">
              <p>⚠️ {error}</p>
            </div>
          )}

          <div className="stereo-mix-actions">
            <button 
              onClick={openRecordingDevices}
              className="stereo-mix-btn primary"
            >
              🎛️ Open Recording Settings
            </button>
            
            <button 
              onClick={recheckStereoMix}
              className="stereo-mix-btn secondary"
              disabled={isChecking}
            >
              {isChecking ? '🔄 Checking...' : '✅ I\'ve Enabled It'}
            </button>
            
            <button 
              onClick={() => {
                setShowManualInstructions(false);
                onStereoMixReady?.(false);
              }}
              className="stereo-mix-btn tertiary"
            >
              Skip for Now
            </button>
          </div>

          <div className="stereo-mix-help">
            <p>
              <strong>Why Stereo Mix?</strong><br/>
              • Captures system audio for complete interview context<br/>
              • Works with any video call application<br/>
              • Required for "System Sound + Mic" recording mode<br/>
              • Can be disabled after interviews if preferred
            </p>
          </div>
        </div>
        
        <style>{`
          .stereo-mix-manager {
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
          
          .stereo-mix-content {
            background: white;
            border-radius: 12px;
            padding: 2rem;
            max-width: 500px;
            max-height: 80vh;
            overflow-y: auto;
            text-align: center;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
          }
          
          .stereo-mix-icon, .status-icon {
            font-size: 3rem;
            margin-bottom: 1rem;
          }
          
          .alternative-devices {
            background: #f0f8ff;
            border: 1px solid #b0d4ff;
            border-radius: 8px;
            padding: 1rem;
            margin: 1rem 0;
            text-align: left;
          }
          
          .alternative-devices ul {
            margin: 0.5rem 0;
            padding-left: 1.5rem;
          }
          
          .manual-instructions {
            background: #f9f9f9;
            border: 1px solid #ddd;
            border-radius: 8px;
            padding: 1rem;
            margin: 1rem 0;
            text-align: left;
          }
          
          .manual-instructions ol {
            margin: 0.5rem 0;
            padding-left: 1.5rem;
          }
          
          .manual-instructions li {
            margin: 0.5rem 0;
          }
          
          .stereo-mix-error {
            background: #fee;
            border: 1px solid #fcc;
            border-radius: 8px;
            padding: 1rem;
            margin: 1rem 0;
            font-size: 0.9rem;
          }
          
          .stereo-mix-actions {
            display: flex;
            flex-wrap: wrap;
            gap: 1rem;
            margin: 1.5rem 0;
            justify-content: center;
          }
          
          .stereo-mix-btn {
            padding: 0.75rem 1rem;
            border: none;
            border-radius: 8px;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
            flex: 1;
            min-width: 140px;
          }
          
          .stereo-mix-btn.primary {
            background: #007acc;
            color: white;
          }
          
          .stereo-mix-btn.primary:hover {
            background: #005f99;
          }
          
          .stereo-mix-btn.secondary {
            background: #28a745;
            color: white;
          }
          
          .stereo-mix-btn.secondary:hover {
            background: #218838;
          }
          
          .stereo-mix-btn.tertiary {
            background: #6c757d;
            color: white;
          }
          
          .stereo-mix-btn.tertiary:hover {
            background: #545b62;
          }
          
          .stereo-mix-btn:disabled {
            opacity: 0.6;
            cursor: not-allowed;
          }
          
          .stereo-mix-help {
            font-size: 0.85rem;
            color: #666;
            margin-top: 1rem;
            text-align: left;
            background: #f8f9fa;
            border-radius: 8px;
            padding: 1rem;
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
          
          .spinner.small {
            width: 1rem;
            height: 1rem;
            border-width: 2px;
            display: inline-block;
            margin: 0 0.5rem;
          }
          
          @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
          }
          
          .stereo-mix-manager.checking,
          .stereo-mix-manager.status {
            background: rgba(0, 0, 0, 0.3);
          }
          
          .stereo-mix-manager.status .stereo-mix-content {
            max-width: 300px;
            padding: 1.5rem;
          }
        `}</style>
      </div>
    );
  }

  return null;
}
