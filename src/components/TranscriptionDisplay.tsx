import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';

interface TranscriptionDisplayProps {
  isActive: boolean;
  className?: string;
}

interface TranscriptionResult {
  text: string;
  is_final: boolean;
  confidence: number;
  timestamp: string;
  source: string;
  model: string;
}

// interface VADEvent {
//   timestamp: string;
//   source: string;
// }

interface AudioLevel {
  rms: number;
  peak: number;
  speech: boolean;
  timestamp: number;
}

export default function TranscriptionDisplay({ isActive, className = '' }: TranscriptionDisplayProps) {
  const [finalTranscripts, setFinalTranscripts] = useState<string[]>([]);
  const [interimText, setInterimText] = useState<string>('');
  const [isListening, setIsListening] = useState<boolean>(false);
  const [confidence, setConfidence] = useState<number>(0);
  const [audioLevel, setAudioLevel] = useState<AudioLevel>({ rms: 0, peak: 0, speech: false, timestamp: 0 });
  const [connectionStatus, setConnectionStatus] = useState<string>('disconnected');
  const [model, setModel] = useState<string>('');
  
  const transcriptionRef = useRef<HTMLDivElement>(null);
  const unlistenersRef = useRef<(() => void)[]>([]);

  // Set up event listeners for transcription events
  useEffect(() => {
    if (!isActive) {
      // Clean up listeners when not active
      unlistenersRef.current.forEach(unlisten => unlisten());
      unlistenersRef.current = [];
      setConnectionStatus('disconnected');
      return;
    }

    const setupListeners = async () => {
      try {
        // Listen for transcription results
        const unlistenTranscription = await listen('transcription-result', (event: any) => {
          const result = event.payload as TranscriptionResult;
          console.log('📝 Transcription result:', result);

          if (result.is_final && result.text.trim()) {
            // Add final transcript to history
            setFinalTranscripts(prev => [...prev.slice(-4), result.text]); // Keep last 5 final results
            setInterimText(''); // Clear interim text
            setConfidence(result.confidence);
            setModel(result.model);
          } else if (!result.is_final && result.text.trim()) {
            // Update interim text
            setInterimText(result.text);
            setConfidence(result.confidence);
            setModel(result.model);
          }
        });

        // Listen for VAD (Voice Activity Detection) events
        const unlistenVADStart = await listen('vad-speech-start', (_event: any) => {
          console.log('🔊 Speech started');
          setIsListening(true);
        });

        const unlistenVADEnd = await listen('vad-speech-end', (_event: any) => {
          console.log('🔇 Speech ended');
          setIsListening(false);
        });

        // Listen for transcription status updates
        const unlistenStatus = await listen('transcription-status', (event: any) => {
          const status = event.payload;
          console.log('📡 Transcription status:', status);
          setConnectionStatus(status.status || 'unknown');
          if (status.model) {
            setModel(status.model);
          }
        });

        // Listen for audio level updates (if available)
        const unlistenAudioLevel = await listen('audio-level', (event: any) => {
          const level = event.payload as AudioLevel;
          setAudioLevel(level);
        });

        // Store unlisteners for cleanup
        unlistenersRef.current = [
          unlistenTranscription,
          unlistenVADStart,
          unlistenVADEnd,
          unlistenStatus,
          unlistenAudioLevel
        ];

        setConnectionStatus('connecting');
        console.log('✅ Transcription listeners set up successfully');
      } catch (error) {
        console.error('❌ Failed to set up transcription listeners:', error);
        setConnectionStatus('error');
      }
    };

    setupListeners();

    // Cleanup on unmount or when isActive changes
    return () => {
      unlistenersRef.current.forEach(unlisten => unlisten());
      unlistenersRef.current = [];
    };
  }, [isActive]);

  // Auto-scroll to bottom when new transcription arrives
  useEffect(() => {
    if (transcriptionRef.current) {
      transcriptionRef.current.scrollTop = transcriptionRef.current.scrollHeight;
    }
  }, [finalTranscripts, interimText]);

  // Audio level visualization
  const getAudioLevelColor = () => {
    if (!audioLevel.speech) return '#64748b'; // Gray when no speech
    if (audioLevel.peak > 0.5) return '#ef4444'; // Red for loud
    if (audioLevel.peak > 0.2) return '#eab308'; // Yellow for medium
    return '#22c55e'; // Green for quiet speech
  };

  const getConnectionStatusColor = () => {
    switch (connectionStatus) {
      case 'streaming': return '#22c55e';
      case 'connecting': return '#eab308';
      case 'error': return '#ef4444';
      default: return '#64748b';
    }
  };

  if (!isActive) {
    return (
      <div className={`p-4 text-center text-gray-500 ${className}`}>
        <div className="text-sm">Transcription is not active</div>
        <div className="text-xs mt-1">Click "Start Transcription" to begin</div>
      </div>
    );
  }

  return (
    <div className={`bg-gray-900 text-white rounded-lg border border-gray-700 ${className}`}>
      {/* Header with status indicators */}
      <div className="flex items-center justify-between p-3 border-b border-gray-700">
        <div className="flex items-center space-x-3">
          {/* Connection status indicator */}
          <div className="flex items-center space-x-2">
            <div 
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: getConnectionStatusColor() }}
            />
            <span className="text-xs font-medium">
              {connectionStatus === 'streaming' ? 'Live' : connectionStatus}
            </span>
          </div>

          {/* Audio level indicator */}
          <div className="flex items-center space-x-2">
            <div className="flex space-x-0.5">
              {[...Array(5)].map((_, i) => (
                <div
                  key={i}
                  className="w-1 h-3 bg-gray-600 rounded-full"
                  style={{
                    backgroundColor: audioLevel.peak > (i + 1) * 0.2 ? getAudioLevelColor() : '#374151',
                    opacity: isListening ? 1 : 0.5
                  }}
                />
              ))}
            </div>
            <span className="text-xs text-gray-400">
              {isListening ? 'Listening' : 'Quiet'}
            </span>
          </div>
        </div>

        {/* Model and confidence info */}
        <div className="text-xs text-gray-400">
          {model && <span>{model}</span>}
          {confidence > 0 && (
            <span className="ml-2">
              {Math.round(confidence * 100)}%
            </span>
          )}
        </div>
      </div>

      {/* Transcription content */}
      <div 
        ref={transcriptionRef}
        className="p-4 h-32 overflow-y-auto text-sm leading-relaxed"
        style={{ minHeight: '8rem' }}
      >
        {/* Final transcripts */}
        {finalTranscripts.map((transcript, index) => (
          <div key={index} className="mb-2 text-gray-100">
            {transcript}
          </div>
        ))}

        {/* Interim transcript */}
        {interimText && (
          <div className="text-gray-400 italic">
            {interimText}
            <span className="animate-pulse ml-1">|</span>
          </div>
        )}

        {/* Placeholder when no transcription */}
        {finalTranscripts.length === 0 && !interimText && (
          <div className="text-gray-500 text-center py-8">
            {connectionStatus === 'streaming' ? (
              <div>
                <div className="text-lg mb-2">🎤</div>
                <div>Ready to transcribe...</div>
                <div className="text-xs mt-1">Speak to see live transcription</div>
              </div>
            ) : (
              <div>
                <div className="text-lg mb-2">⏳</div>
                <div>Connecting to transcription service...</div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Quick actions */}
      <div className="px-4 py-2 border-t border-gray-700 text-xs text-gray-400">
        <div className="flex justify-between items-center">
          <span>
            {finalTranscripts.length > 0 && `${finalTranscripts.length} completed`}
          </span>
          <button
            onClick={() => {
              setFinalTranscripts([]);
              setInterimText('');
            }}
            className="text-gray-400 hover:text-white px-2 py-1 rounded hover:bg-gray-700"
          >
            Clear
          </button>
        </div>
      </div>
    </div>
  );
}