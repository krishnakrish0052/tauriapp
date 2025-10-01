// useDeepgram.ts - Direct JavaScript SDK integration for Nova-3 real-time transcription
// Following official Deepgram documentation: https://developers.deepgram.com/docs/live-streaming-audio

import { useState, useEffect, useRef, useCallback } from 'react';
import { createClient, LiveTranscriptionEvents } from '@deepgram/sdk';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface DeepgramConfig {
  model: string;
  language: string;
  smart_format: boolean;
  interim_results: boolean;
  endpointing: boolean;
  vad_events: boolean;
  punctuate: boolean;
  numerals: boolean;
  channels: number;
  sample_rate: number;
  encoding: string;
}

interface TranscriptionResult {
  text: string;
  is_final: boolean;
  confidence: number;
  timestamp: string;
  source: string;
}

interface UseDeepgramReturn {
  isConnected: boolean;
  isTranscribing: boolean;
  transcriptionResult: TranscriptionResult | null;
  error: string | null;
  startTranscription: () => Promise<void>;
  stopTranscription: () => Promise<void>;
  connectionStatus: string;
}

export const useDeepgram = (): UseDeepgramReturn => {
  // State
  const [isConnected, setIsConnected] = useState(false);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [transcriptionResult, setTranscriptionResult] = useState<TranscriptionResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [connectionStatus, setConnectionStatus] = useState('disconnected');

  // Refs
  const deepgramRef = useRef<any | null>(null);
  const connectionRef = useRef<any | null>(null);
  const isStartingRef = useRef(false);

  // Configuration following official Deepgram docs for Nova-3
  const config: DeepgramConfig = {
    model: 'nova-3', // Latest, fastest, most accurate model
    language: 'en-US',
    smart_format: true,
    interim_results: true,
    endpointing: true, 
    vad_events: true,
    punctuate: true,
    numerals: true,
    channels: 1,
    sample_rate: 44100,
    encoding: 'linear16'
  };

  // Initialize Deepgram client
  useEffect(() => {
    const initDeepgram = async () => {
      try {
        // Get API key from environment (embedded at build time)
        const apiKey = import.meta.env.VITE_DEEPGRAM_API_KEY || 
                      process.env.DEEPGRAM_API_KEY || 
                      '8178bb79a552d96705f111b6e460e11198d2fa41'; // Fallback

        console.log('🚀 Initializing Deepgram client with Nova-3...');
        const deepgram = createClient(apiKey);
        deepgramRef.current = deepgram;
        
        console.log('✅ Deepgram client initialized successfully');
      } catch (err) {
        console.error('❌ Failed to initialize Deepgram client:', err);
        setError(`Failed to initialize Deepgram: ${err}`);
      }
    };

    initDeepgram();
  }, []);

  // Set up audio event listeners for Pluely audio
  useEffect(() => {
    let unlistenSpeechDetected: (() => void) | null = null;
    let unlistenAudioChunk: (() => void) | null = null;

    const setupAudioListener = async () => {
      try {
        // Listen for real-time audio chunks for streaming transcription
        unlistenAudioChunk = await listen('audio-chunk', (event: any) => {
          const b64AudioData = event.payload as string;
          
          if (connectionRef.current && isTranscribing) {
            try {
              // Decode base64 WAV data
              const wavData = atob(b64AudioData);
              const wavBytes = new Uint8Array(wavData.length);
              for (let i = 0; i < wavData.length; i++) {
                wavBytes[i] = wavData.charCodeAt(i);
              }

              // Skip WAV header (44 bytes) to get raw PCM audio
              const pcmData = wavBytes.slice(44);
              
              // Send raw PCM audio to Deepgram WebSocket for real-time processing
              connectionRef.current.send(pcmData);
            } catch (err) {
              console.error('❌ Error processing real-time audio chunk for Deepgram:', err);
            }
          }
        });
        
        // Still listen for speech-detected events for compatibility/debugging
        unlistenSpeechDetected = await listen('speech-detected', (event: any) => {
          const b64AudioData = event.payload as string;
          console.log('🎤 Received speech-detected event:', b64AudioData.length, 'bytes (base64)');
          // Note: We now primarily use audio-chunk events for real-time transcription
        });

        console.log('✅ Audio event listeners set up for Pluely integration (streaming + speech detection)');
      } catch (err) {
        console.error('❌ Failed to set up audio listeners:', err);
        setError(`Failed to set up audio listeners: ${err}`);
      }
    };

    setupAudioListener();

    return () => {
      if (unlistenAudioChunk) {
        unlistenAudioChunk();
      }
      if (unlistenSpeechDetected) {
        unlistenSpeechDetected();
      }
    };
  }, [isTranscribing]);

  const startTranscription = useCallback(async () => {
    if (isStartingRef.current || !deepgramRef.current) {
      console.log('⚠️ Transcription already starting or Deepgram not initialized');
      return;
    }

    isStartingRef.current = true;
    
    try {
      console.log('🌊 Starting Nova-3 live transcription...');
      setConnectionStatus('connecting');
      setError(null);

      // Create live transcription connection
      const connection = deepgramRef.current.listen.live({
        model: config.model,
        language: config.language,
        smart_format: config.smart_format,
        interim_results: config.interim_results,
        endpointing: config.endpointing,
        vad_events: config.vad_events,
        punctuate: config.punctuate,
        numerals: config.numerals,
        channels: config.channels,
        sample_rate: config.sample_rate,
        encoding: config.encoding as any
      });

      connectionRef.current = connection;

      // Set up event listeners following official docs
      connection.on(LiveTranscriptionEvents.Open, () => {
        console.log('✅ Deepgram WebSocket connection opened');
        setIsConnected(true);
        setConnectionStatus('connected');
      });

      connection.on(LiveTranscriptionEvents.Close, () => {
        console.log('🔌 Deepgram WebSocket connection closed');
        setIsConnected(false);
        setIsTranscribing(false);
        setConnectionStatus('disconnected');
        connectionRef.current = null;
      });

      connection.on(LiveTranscriptionEvents.Transcript, (data: any) => {
        console.log('📝 Received transcription from Nova-3:', data);
        
        // Process transcription result following official response format
        const alternatives = data.channel?.alternatives;
        if (alternatives && alternatives.length > 0) {
          const transcript = alternatives[0].transcript;
          const confidence = alternatives[0].confidence || 0;
          
          if (transcript && transcript.trim()) {
            const result: TranscriptionResult = {
              text: transcript,
              is_final: data.is_final || false,
              confidence: confidence,
              timestamp: new Date().toISOString(),
              source: 'deepgram_js_nova3'
            };

            setTranscriptionResult(result);
            
            // Log for debugging
            const resultType = result.is_final ? 'FINAL' : 'INTERIM';
            console.log(`📝 ${resultType}: "${result.text}" (${(result.confidence * 100).toFixed(1)}%)`);
          }
        }
      });

      connection.on(LiveTranscriptionEvents.Metadata, (data: any) => {
        console.log('📊 Deepgram metadata:', data);
      });

      connection.on(LiveTranscriptionEvents.Error, (err: any) => {
        console.error('❌ Deepgram WebSocket error:', err);
        setError(`Deepgram error: ${err.message || err}`);
        setConnectionStatus('error');
      });

      // Start Pluely system audio capture
      console.log('🎵 Starting Pluely system audio capture...');
      await invoke('start_pluely_system_audio_capture');
      
      setIsTranscribing(true);
      console.log('✅ Nova-3 transcription started successfully');

    } catch (err) {
      console.error('❌ Failed to start transcription:', err);
      setError(`Failed to start transcription: ${err}`);
      setConnectionStatus('error');
    } finally {
      isStartingRef.current = false;
    }
  }, []);

  const stopTranscription = useCallback(async () => {
    try {
      console.log('🛑 Stopping Nova-3 transcription...');
      
      // Close Deepgram connection
      if (connectionRef.current) {
        connectionRef.current.finish();
        connectionRef.current = null;
      }

      // Stop Pluely system audio capture
      await invoke('stop_pluely_system_audio_capture');
      
      setIsConnected(false);
      setIsTranscribing(false);
      setConnectionStatus('disconnected');
      setTranscriptionResult(null);
      
      console.log('✅ Nova-3 transcription stopped');
    } catch (err) {
      console.error('❌ Failed to stop transcription:', err);
      setError(`Failed to stop transcription: ${err}`);
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (connectionRef.current) {
        connectionRef.current.finish();
      }
    };
  }, []);

  return {
    isConnected,
    isTranscribing,
    transcriptionResult,
    error,
    startTranscription,
    stopTranscription,
    connectionStatus
  };
};