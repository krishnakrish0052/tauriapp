import { useState, useEffect } from 'react';
import { StereoMixManager } from './StereoMixManager';
import { PermissionManager } from './PermissionManager';

interface AppWithStereoMixProps {
  children: React.ReactNode;
}

export function AppWithStereoMix({ children }: AppWithStereoMixProps) {
  const [permissionsGranted, setPermissionsGranted] = useState(false);
  const [stereoMixReady, setStereoMixReady] = useState(false);
  const [showStereoMixManager, setShowStereoMixManager] = useState(false);
  const [audioSystemReady, setAudioSystemReady] = useState(false);

  useEffect(() => {
    // When permissions are granted, check if we need to set up Stereo Mix
    if (permissionsGranted && !stereoMixReady && !showStereoMixManager) {
      console.log('✅ Permissions granted, checking Stereo Mix setup...');
      setShowStereoMixManager(true);
    }

    // When both permissions and Stereo Mix are ready, the audio system is ready
    if (permissionsGranted && stereoMixReady && !audioSystemReady) {
      console.log('🎵 Audio system fully initialized!');
      setAudioSystemReady(true);
    }
  }, [permissionsGranted, stereoMixReady, showStereoMixManager, audioSystemReady]);

  const handlePermissionsGranted = () => {
    console.log('✅ Permissions granted by user');
    setPermissionsGranted(true);
  };

  const handleStereoMixReady = (enabled: boolean) => {
    console.log(`🎵 Stereo Mix ready: ${enabled}`);
    setStereoMixReady(enabled);
    setShowStereoMixManager(false);
  };

  return (
    <div className="app-with-stereo-mix">
      {/* Show permission manager first */}
      {!permissionsGranted && (
        <PermissionManager onPermissionsGranted={handlePermissionsGranted} />
      )}
      
      {/* Show Stereo Mix manager after permissions are granted */}
      {permissionsGranted && showStereoMixManager && (
        <StereoMixManager 
          onStereoMixReady={handleStereoMixReady}
          autoEnableOnFirstRun={true}
          showInstructions={true}
        />
      )}

      {/* Show main app content */}
      <div className={`main-app-content ${audioSystemReady ? 'ready' : 'initializing'}`}>
        {children}
        
        {/* Optional audio system status indicator */}
        {!audioSystemReady && permissionsGranted && (
          <div className="audio-status-indicator">
            <div className="status-badge">
              {!stereoMixReady ? '🔧 Setting up audio system...' : '✅ Audio system ready!'}
            </div>
          </div>
        )}
      </div>

      <style>{`
        .app-with-stereo-mix {
          position: relative;
          width: 100%;
          height: 100vh;
        }

        .main-app-content {
          width: 100%;
          height: 100%;
          transition: opacity 0.3s ease;
        }

        .main-app-content.initializing {
          opacity: 0.8;
        }

        .main-app-content.ready {
          opacity: 1;
        }

        .audio-status-indicator {
          position: fixed;
          top: 20px;
          right: 20px;
          z-index: 999;
        }

        .status-badge {
          background: rgba(0, 0, 0, 0.8);
          color: white;
          padding: 0.5rem 1rem;
          border-radius: 20px;
          font-size: 0.8rem;
          animation: pulse 2s infinite;
        }

        @keyframes pulse {
          0%, 100% { opacity: 0.8; }
          50% { opacity: 1; }
        }
      `}</style>
    </div>
  );
}

// Example usage in your main App.tsx:
/*
import { AppWithStereoMix } from './components/AppWithStereoMix';
import { YourMainAppComponent } from './components/YourMainApp';

function App() {
  return (
    <AppWithStereoMix>
      <YourMainAppComponent />
    </AppWithStereoMix>
  );
}

export default App;
*/
