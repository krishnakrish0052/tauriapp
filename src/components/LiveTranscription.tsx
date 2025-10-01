// LiveTranscription.tsx - Real-time transcription using Deepgram JavaScript SDK
// Direct integration with Nova-3 model for fastest, most accurate transcription

import React, { useState, useRef, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { useDeepgram } from '@/hooks/useDeepgram';

interface LiveTranscriptionProps {
  className?: string;
  onTranscriptionUpdate?: (text: string, isFinal: boolean) => void;
}

export const LiveTranscription: React.FC<LiveTranscriptionProps> = ({ 
  className = '',
  onTranscriptionUpdate 
}) => {
  const {
    isConnected,
    isTranscribing,
    transcriptionResult,
    error,
    startTranscription,
    stopTranscription,
    connectionStatus
  } = useDeepgram();

  const [fullTranscript, setFullTranscript] = useState('');
  const [interimTranscript, setInterimTranscript] = useState('');
  const [sessionTranscripts, setSessionTranscripts] = useState<Array<{
    text: string;
    timestamp: string;
    confidence: number;
  }>>([]);

  const transcriptRef = useRef<HTMLDivElement>(null);

  // Handle transcription results
  useEffect(() => {
    if (transcriptionResult) {
      const { text, is_final, confidence, timestamp } = transcriptionResult;
      
      if (is_final) {
        // Final transcription - add to full transcript
        const newFullTranscript = fullTranscript ? `${fullTranscript} ${text}` : text;
        setFullTranscript(newFullTranscript);
        setInterimTranscript(''); // Clear interim

        // Add to session history
        setSessionTranscripts(prev => [...prev, {
          text,
          timestamp,
          confidence
        }]);

        // Notify parent component
        if (onTranscriptionUpdate) {
          onTranscriptionUpdate(newFullTranscript, true);
        }

        console.log(`📝 FINAL: "${text}" (${(confidence * 100).toFixed(1)}%)`);
      } else {
        // Interim transcription - update interim only
        setInterimTranscript(text);
        
        // Notify parent component
        if (onTranscriptionUpdate) {
          onTranscriptionUpdate(fullTranscript + ' ' + text, false);
        }

        console.log(`⏳ INTERIM: "${text}"`);
      }
    }
  }, [transcriptionResult, fullTranscript, onTranscriptionUpdate]);

  // Auto-scroll to bottom when new content is added
  useEffect(() => {
    if (transcriptRef.current) {
      transcriptRef.current.scrollTop = transcriptRef.current.scrollHeight;
    }
  }, [fullTranscript, interimTranscript]);

  const handleToggleTranscription = async () => {
    if (isTranscribing) {
      await stopTranscription();
    } else {
      await startTranscription();
    }
  };

  const clearTranscript = () => {
    setFullTranscript('');
    setInterimTranscript('');
    setSessionTranscripts([]);
    
    if (onTranscriptionUpdate) {
      onTranscriptionUpdate('', true);
    }
  };

  const getConnectionStatusColor = () => {
    switch (connectionStatus) {
      case 'connected': return 'text-green-400';
      case 'connecting': return 'text-yellow-400';
      case 'error': return 'text-red-400';
      default: return 'text-gray-400';
    }
  };

  const getConnectionStatusIcon = () => {
    switch (connectionStatus) {
      case 'connected': return 'radio_button_checked';
      case 'connecting': return 'pending';
      case 'error': return 'error';
      default: return 'radio_button_unchecked';
    }
  };

  return (
    <div className={`bg-white/10 backdrop-blur-sm rounded-lg border border-white/20 p-4 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <span className="material-icons text-blue-400">record_voice_over</span>
          <h3 className="text-white font-medium text-sm">Live Transcription (Nova-3)</h3>
        </div>
        
        <div className="flex items-center gap-2">
          {/* Connection Status */}
          <div className="flex items-center gap-1">
            <span className={`material-icons text-xs ${getConnectionStatusColor()}`}>
              {getConnectionStatusIcon()}
            </span>
            <span className={`text-xs ${getConnectionStatusColor()}`}>
              {connectionStatus}
            </span>
          </div>
          
          {/* Clear Button */}
          <Button
            onClick={clearTranscript}
            disabled={!fullTranscript && !interimTranscript}
            className="w-6 h-6 bg-white/10 hover:bg-white/20 border-0 rounded p-0 flex items-center justify-center"
          >
            <span className="material-icons text-gray-400 text-sm">clear</span>
          </Button>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="mb-3 p-2 bg-red-500/20 border border-red-500/30 rounded text-red-300 text-xs">
          <div className="flex items-center gap-1">
            <span className="material-icons text-sm">error</span>
            <span>Error: {error}</span>
          </div>
        </div>
      )}

      {/* Transcription Display */}
      <div 
        ref={transcriptRef}
        className="bg-black/20 rounded p-3 min-h-[120px] max-h-[200px] overflow-y-auto mb-3 border border-white/10"
      >
        {fullTranscript || interimTranscript ? (
          <div className="text-white text-sm leading-relaxed">
            {/* Final transcript */}
            <span>{fullTranscript}</span>
            {/* Interim transcript in different color */}
            {interimTranscript && (
              <span className="text-blue-300 italic opacity-75">
                {fullTranscript ? ' ' : ''}{interimTranscript}
              </span>
            )}
          </div>
        ) : (
          <div className="text-gray-400 text-sm italic">
            {isTranscribing 
              ? "Listening for speech..." 
              : "Press Start to begin live transcription with Nova-3"
            }
          </div>
        )}
      </div>

      {/* Controls */}
      <div className="flex items-center gap-2">
        <Button
          onClick={handleToggleTranscription}
          disabled={connectionStatus === 'connecting'}
          className={`flex items-center gap-2 px-4 py-2 text-sm font-medium text-white border-0 rounded transition-all ${
            isTranscribing
              ? 'bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700'
              : 'bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700'
          } ${connectionStatus === 'connecting' ? 'opacity-50' : ''}`}
        >
          {connectionStatus === 'connecting' ? (
            <>
              <span className="material-icons animate-spin text-sm">hourglass_empty</span>
              <span>Connecting...</span>
            </>
          ) : isTranscribing ? (
            <>
              <span className="material-icons text-sm">stop</span>
              <span>Stop</span>
            </>
          ) : (
            <>
              <span className="material-icons text-sm">play_arrow</span>
              <span>Start Nova-3</span>
            </>
          )}
        </Button>

        {/* Session Info */}
        <div className="flex items-center gap-4 text-xs text-gray-300 ml-auto">
          {sessionTranscripts.length > 0 && (
            <span>{sessionTranscripts.length} segments</span>
          )}
          {isConnected && (
            <div className="flex items-center gap-1">
              <span className="material-icons text-green-400 text-xs">wifi</span>
              <span>Nova-3 Connected</span>
            </div>
          )}
        </div>
      </div>

      {/* Session Transcripts (collapsed by default) */}
      {sessionTranscripts.length > 0 && (
        <details className="mt-3">
          <summary className="text-xs text-gray-400 cursor-pointer hover:text-gray-300">
            View Session History ({sessionTranscripts.length} segments)
          </summary>
          <div className="mt-2 space-y-1 max-h-32 overflow-y-auto">
            {sessionTranscripts.map((transcript, index) => (
              <div key={index} className="text-xs bg-white/5 p-2 rounded border-l-2 border-blue-400/50">
                <div className="text-white">{transcript.text}</div>
                <div className="text-gray-400 text-xs mt-1">
                  {new Date(transcript.timestamp).toLocaleTimeString()} • 
                  Confidence: {(transcript.confidence * 100).toFixed(1)}%
                </div>
              </div>
            ))}
          </div>
        </details>
      )}
    </div>
  );
};

export default LiveTranscription;