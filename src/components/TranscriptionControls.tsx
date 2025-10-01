import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';

interface TranscriptionControlsProps {
  isActive: boolean;
  onStatusChange: (isActive: boolean) => void;
  className?: string;
}

export default function TranscriptionControls({ 
  isActive, 
  onStatusChange, 
  className = '' 
}: TranscriptionControlsProps) {
  const [isStarting, setIsStarting] = useState(false);
  const [isStopping, setIsStopping] = useState(false);
  const [lastError, setLastError] = useState<string>('');

  const startTranscription = async () => {
    if (isStarting || isActive) return;

    setIsStarting(true);
    setLastError('');
    
    try {
      console.log('🚀 Starting Pluely-Deepgram transcription...');
      await invoke('start_pluely_deepgram_transcription');
      console.log('✅ Transcription started successfully');
      onStatusChange(true);
    } catch (error) {
      console.error('❌ Failed to start transcription:', error);
      setLastError(error as string);
    } finally {
      setIsStarting(false);
    }
  };

  const stopTranscription = async () => {
    if (isStopping || !isActive) return;

    setIsStopping(true);
    setLastError('');
    
    try {
      console.log('🛑 Stopping Pluely-Deepgram transcription...');
      await invoke('stop_pluely_deepgram_transcription');
      console.log('✅ Transcription stopped successfully');
      onStatusChange(false);
    } catch (error) {
      console.error('❌ Failed to stop transcription:', error);
      setLastError(error as string);
    } finally {
      setIsStopping(false);
    }
  };

  const testDeepgramDirect = async () => {
    try {
      console.log('🧪 Testing Deepgram direct connection...');
      const result = await invoke('test_deepgram_streaming_direct');
      console.log('✅ Deepgram test result:', result);
      alert('Deepgram test completed - check console for results');
    } catch (error) {
      console.error('❌ Deepgram test failed:', error);
      alert(`Deepgram test failed: ${error}`);
    }
  };

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg p-4 border ${className}`}>
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Real-Time Transcription</h3>
        <div className="flex items-center space-x-2">
          {isActive && (
            <div className="flex items-center space-x-2 text-green-600 dark:text-green-400">
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
              <span className="text-sm font-medium">Live</span>
            </div>
          )}
        </div>
      </div>

      <div className="flex flex-wrap gap-2 mb-4">
        {!isActive ? (
          <Button
            onClick={startTranscription}
            disabled={isStarting}
            className="bg-green-600 hover:bg-green-700 text-white"
          >
            {isStarting ? (
              <div className="flex items-center space-x-2">
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                <span>Starting...</span>
              </div>
            ) : (
              <div className="flex items-center space-x-2">
                <span>🎤</span>
                <span>Start Transcription</span>
              </div>
            )}
          </Button>
        ) : (
          <Button
            onClick={stopTranscription}
            disabled={isStopping}
            variant="destructive"
          >
            {isStopping ? (
              <div className="flex items-center space-x-2">
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                <span>Stopping...</span>
              </div>
            ) : (
              <div className="flex items-center space-x-2">
                <span>🛑</span>
                <span>Stop Transcription</span>
              </div>
            )}
          </Button>
        )}

        <Button
          onClick={testDeepgramDirect}
          variant="outline"
          className="border-blue-300 text-blue-600 hover:bg-blue-50"
        >
          <div className="flex items-center space-x-2">
            <span>🧪</span>
            <span>Test Deepgram</span>
          </div>
        </Button>
      </div>

      {lastError && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-3 mb-4">
          <div className="flex items-start space-x-2">
            <span className="text-red-500 text-lg">⚠️</span>
            <div>
              <div className="font-medium text-red-800">Error</div>
              <div className="text-red-700 text-sm mt-1">{lastError}</div>
            </div>
          </div>
        </div>
      )}

      <div className="text-sm text-gray-600 dark:text-gray-400">
        <div className="mb-2">
          <strong>System Audio Capture:</strong> Captures all system audio for transcription
        </div>
        <div className="mb-2">
          <strong>Model:</strong> Deepgram General (optimized for live speech)
        </div>
        <div>
          <strong>Features:</strong> Real-time interim results, voice activity detection, smart formatting
        </div>
      </div>
    </div>
  );
}