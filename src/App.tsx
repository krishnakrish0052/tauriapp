import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useResponsiveWindow } from '@/hooks/useResponsiveWindow';
// Using Material Icons instead of lucide-react to avoid antivirus issues

type AppScreen = 'session_connection' | 'main' | 'confirmation';

interface SessionState {
  sessionId: string;
  userId: string;
  isActive: boolean;
  timer: string;
  credits: number;
  startTime: number;
  // Additional session details from database
  sessionName?: string;
  companyName?: string;
  jobTitle?: string;
  userEmail?: string;
  status?: string;
}

interface AppState {
  // Screen management
  currentScreen: AppScreen;
  
  // Audio states
  isMicOn: boolean;
  isSystemSoundOn: boolean;
  isTranscriptionActive: boolean;
  
  // Transcription
  transcriptionText: string;
  fullTranscription: string;
  interimTranscription: string;
  
  // Input and AI
  inputText: string;
  isLoading: boolean;
  isStreaming: boolean;
  streamingText: string;
  accumulatedAIResponse: string;
  
  // Session management
  session: SessionState;
  isStartingSession: boolean;
  
  // Provider and model
  selectedModel: string;
  selectedProvider: string;
  
  // UI states
  isModelDropdownOpen: boolean;
  availableModels: Array<{id: string, name: string, provider: string}>;
  
  // QA tracking
  currentQuestionId: string | null;
  questionCounter: number;
  
  // Embedded AI Response
  aiResponseVisible: boolean;
  aiResponseContent: string;
  aiResponseStreaming: boolean;
}

function App() {
  const { autoResize } = useResponsiveWindow();
  
  const [state, setState] = useState<AppState>({
    currentScreen: 'session_connection',
    isMicOn: false,
    isSystemSoundOn: false,
    isTranscriptionActive: false,
    transcriptionText: 'Ready to assist you...',
    fullTranscription: '',
    interimTranscription: '',
    inputText: '',
    isLoading: false,
    isStreaming: false,
    streamingText: '',
    accumulatedAIResponse: '',
    session: {
      sessionId: '',
      userId: '',
      isActive: false,
      timer: '00:00',
      credits: 0,
      startTime: 0,
    },
    isStartingSession: false,
    selectedModel: 'llama', // Use fast Llama model as default
    selectedProvider: 'pollinations',
    isModelDropdownOpen: false,
    availableModels: [], // Start with empty array, will be populated from backend
    // QA tracking
    currentQuestionId: null,
    questionCounter: 0,
    // Embedded AI Response
    aiResponseVisible: false,
    aiResponseContent: '',
    aiResponseStreaming: false,
  });

  
  // Refs for accessing current state in event handlers
  const stateRef = useRef(state);
  
  // Timer ref for cleanup
  const timerIntervalRef = useRef<NodeJS.Timeout | null>(null);
  
  // Update state ref whenever state changes
  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  // Timer utilities
  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const updateTimerDisplay = (startTime: number) => {
    const currentTime = Date.now();
    const elapsedSeconds = Math.floor((currentTime - startTime) / 1000);
    const formattedTime = formatTime(elapsedSeconds);
    
    setState(prev => ({ 
      ...prev, 
      session: { ...prev.session, timer: formattedTime } 
    }));
    
    return elapsedSeconds;
  };

  // Timer management effect
  useEffect(() => {
    // Clear any existing timer
    if (timerIntervalRef.current) {
      clearInterval(timerIntervalRef.current);
      timerIntervalRef.current = null;
    }
    
    // Start timer if session is active and has a start time
    if (state.session.isActive && state.session.startTime > 0) {
      console.log('⏱️ Starting session timer...');
      
      // Update immediately
      updateTimerDisplay(state.session.startTime);
      
      // Update every second
      timerIntervalRef.current = setInterval(() => {
        const elapsedSeconds = updateTimerDisplay(state.session.startTime);
        
        // Send timer update to backend every minute
        const elapsedMinutes = Math.floor(elapsedSeconds / 60);
        if (elapsedSeconds > 0 && elapsedSeconds % 60 === 0) {
          console.log(`⏱️ Sending timer update: ${elapsedMinutes} minutes`);
          invoke('update_session_timer', {
            sessionId: state.session.sessionId,
            elapsedMinutes,
            isFinal: false
          }).catch(err => {
            console.warn('Failed to update timer on backend:', err);
          });
        }
      }, 1000);
    }
    
    // Cleanup on unmount or when session becomes inactive
    return () => {
      if (timerIntervalRef.current) {
        clearInterval(timerIntervalRef.current);
        timerIntervalRef.current = null;
      }
    };
  }, [state.session.isActive, state.session.startTime]);

  // Fetch available models from backend on app initialization
  useEffect(() => {
    const fetchModels = async () => {
      try {
        console.log('🔍 Fetching available models from backend...');
        const models = await invoke<Array<{id: string, name: string, provider: string}>>('get_available_models');
        console.log('✅ Models fetched from backend:', models);
        
        if (models && models.length > 0) {
          setState(prev => ({
            ...prev,
            availableModels: models,
            // Set default model to first available model if current selected model is not in the list
            selectedModel: models.find(m => m.id === prev.selectedModel) ? prev.selectedModel : models[0].id
          }));
          console.log('🔧 Models updated in state, default model set');
        }
      } catch (error) {
        console.error('❌ Failed to fetch models from backend:', error);
        // Set fallback models if backend fetch fails
        const fallbackModels = [
          { id: 'llama', name: 'Llama (Fast)', provider: 'pollinations' },
          { id: 'openai', name: 'OpenAI GPT-4', provider: 'pollinations' },
          { id: 'mistral', name: 'Mistral', provider: 'pollinations' },
        ];
        setState(prev => ({
          ...prev,
          availableModels: fallbackModels,
          selectedModel: 'llama' // Fast fallback model
        }));
        console.log('🔧 Using fallback models due to backend fetch error');
      }
    };

    fetchModels();
  }, []); // Run once on component mount

  // Update window size when screen changes
  useEffect(() => {
    const updateWindowForScreen = async () => {
      try {
        await autoResize(false, state.currentScreen);
      } catch (error) {
        console.error('Failed to resize window for screen change:', error);
      }
    };

    updateWindowForScreen();
  }, [state.currentScreen, autoResize]);

  // Set up transcription event listeners
  useEffect(() => {
    let unlistenTranscription: (() => void) | null = null;

    const setupTranscriptionListener = async () => {
      try {
        // Listen for transcription results from backend
        unlistenTranscription = await listen('transcription-result', (event: any) => {
          const payload = event.payload;
          console.log('Transcription received:', payload);
          
          if (payload.is_final) {
            // Final transcription - append to full transcription
            setState(prev => {
              const newFullTranscription = prev.fullTranscription 
                ? prev.fullTranscription + ' ' + payload.text 
                : payload.text;
              
              return {
                ...prev,
                fullTranscription: newFullTranscription,
                interimTranscription: '', // Clear interim
                transcriptionText: newFullTranscription
              };
            });
          } else {
            // Interim transcription - show in interim field
            setState(prev => ({
              ...prev,
              interimTranscription: payload.text
            }));
          }
        });
        console.log('Transcription listener set up successfully');
      } catch (error) {
        console.error('Failed to set up transcription listener:', error);
      }
    };

    setupTranscriptionListener();

    // Cleanup on unmount
    return () => {
      if (unlistenTranscription) {
        unlistenTranscription();
      }
    };
  }, []);

  // Auto-scroll transcription to right when new text is added
  const transcriptionRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (transcriptionRef.current) {
      transcriptionRef.current.scrollLeft = transcriptionRef.current.scrollWidth;
    }
  }, [state.fullTranscription, state.interimTranscription]);

  // Set up AI streaming event listeners
  useEffect(() => {
    let unlistenAIStart: (() => void) | null = null;
    let unlistenAIToken: (() => void) | null = null;
    let unlistenAIComplete: (() => void) | null = null;
    let unlistenAIError: (() => void) | null = null;

    const setupAIStreamingListeners = async () => {
      try {
        // Listen for AI streaming start
        unlistenAIStart = await listen('ai-stream-start', (_event: any) => {
          console.log('AI streaming started - response will show in embedded window');
          setState(prev => ({ 
            ...prev, 
            isStreaming: true, 
            isLoading: true,
            accumulatedAIResponse: '', // Reset accumulated response
            aiResponseVisible: true,
            aiResponseContent: '',
            aiResponseStreaming: true
          }));
        });

        // Listen for AI streaming tokens
        unlistenAIToken = await listen('ai-stream-token', (event: any) => {
          const token = event.payload.token || event.payload.text || event.payload;
          console.log('AI stream token received (displaying in embedded window):', token);
          
          // Accumulate response text for database saving and embedded display
          if (token && typeof token === 'string') {
            setState(prev => ({ 
              ...prev, 
              accumulatedAIResponse: prev.accumulatedAIResponse + token,
              aiResponseContent: prev.aiResponseContent + token
            }));
          }
        });

        // Listen for AI streaming completion
        unlistenAIComplete = await listen('ai-stream-complete', (event: any) => {
          console.log('🎉 AI streaming completed:', event.payload);
          console.log('🔍 Backend completion event payload type:', typeof event.payload);
          console.log('🔍 Backend completion event payload content:', event.payload);
          
          // Save answer to database if we have a current question ID
          const saveAnswer = async () => {
            const currentState = stateRef.current; // Use ref to get current state
            console.log('🔍 Checking answer save conditions:', {
              hasQuestionId: !!currentState.currentQuestionId,
              isSessionActive: currentState.session.isActive,
              hasSessionId: !!currentState.session.sessionId,
              questionId: currentState.currentQuestionId,
              accumulatedResponseLength: currentState.accumulatedAIResponse.length
            });
            
            if (currentState.currentQuestionId && currentState.session.isActive && currentState.session.sessionId) {
              try {
                // Backend sends full_response directly as payload string
                // Priority: accumulated text -> direct payload string -> fallback
                const answerText = currentState.accumulatedAIResponse || 
                                 (typeof event.payload === 'string' ? event.payload : '') || 
                                 event.payload?.text || 
                                 event.payload?.final_response || 
                                 event.payload?.content || 
                                 'AI response not available';
                
                console.log('💾 Attempting to save answer:', {
                  sessionId: currentState.session.sessionId,
                  questionId: currentState.currentQuestionId,
                  answerLength: answerText.length,
                  answerPreview: answerText.substring(0, 100),
                  isFromAccumulated: !!currentState.accumulatedAIResponse,
                  payloadType: typeof event.payload
                });
                
                const answerId = await invoke('save_interview_answer', {
                  sessionId: currentState.session.sessionId,
                  questionId: currentState.currentQuestionId,
                  answerText: answerText,
                  responseTime: 30, // Default response time in seconds
                  aiFeedback: null,
                  aiScore: null
                });
                console.log('✅ Answer saved to database with ID:', answerId);
              } catch (dbError) {
                console.error('❌ Failed to save answer to database:', dbError);
              }
            } else {
              console.warn('⚠️ Answer not saved - missing requirements');
            }
          };
          
          // Save answer immediately
          saveAnswer();
          
          // Update state to clear loading and question ID
          setState(prev => ({ 
            ...prev, 
            isStreaming: false, 
            isLoading: false, 
            currentQuestionId: null,
            aiResponseStreaming: false
          }));
        });

        // Listen for AI streaming errors
        unlistenAIError = await listen('ai-stream-error', (event: any) => {
          console.error('AI streaming error:', event.payload);
          setState(prev => ({ 
            ...prev, 
            isStreaming: false, 
            isLoading: false,
            aiResponseStreaming: false,
            aiResponseContent: prev.aiResponseContent + '\n\n[ERROR] ' + (event.payload || 'Unknown error occurred')
          }));
        });

        console.log('AI streaming listeners set up successfully');
      } catch (error) {
        console.error('Failed to set up AI streaming listeners:', error);
      }
    };

    setupAIStreamingListeners();

    // Cleanup on unmount
    return () => {
      if (unlistenAIStart) unlistenAIStart();
      if (unlistenAIToken) unlistenAIToken();
      if (unlistenAIComplete) unlistenAIComplete();
      if (unlistenAIError) unlistenAIError();
    };
  }, []);

  // Model selection functions
  const toggleModelDropdown = () => {
    setState(prev => ({ ...prev, isModelDropdownOpen: !prev.isModelDropdownOpen }));
  };

  const selectModel = (modelId: string) => {
    const selectedModelObj = state.availableModels.find(model => model.id === modelId);
    if (selectedModelObj) {
      setState(prev => ({ 
        ...prev, 
        selectedModel: modelId,
        selectedProvider: selectedModelObj.provider,
        isModelDropdownOpen: false 
      }));
      console.log('Model selected:', selectedModelObj.name);
    }
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Element;
      if (state.isModelDropdownOpen && !target.closest('.model-dropdown')) {
        setState(prev => ({ ...prev, isModelDropdownOpen: false }));
      }
    };

    if (state.isModelDropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [state.isModelDropdownOpen]);

  // Handle microphone toggle
  const toggleMicrophone = async () => {
    try {
      if (state.isMicOn) {
        // Stop transcription
        await invoke('stop_transcription');
        setState(prev => ({ ...prev, isMicOn: false, isTranscriptionActive: false }));
      } else {
        // Start microphone transcription
        await invoke('start_microphone_transcription');
        setState(prev => ({ ...prev, isMicOn: true, isTranscriptionActive: true }));
      }
    } catch (error) {
      console.error('Failed to toggle microphone:', error);
    }
  };

  // Handle system audio toggle
  const toggleSystemAudio = async () => {
    try {
      if (state.isSystemSoundOn) {
        // Stop transcription
        await invoke('stop_transcription');
        setState(prev => ({ ...prev, isSystemSoundOn: false }));
      } else {
        // Start system audio transcription  
        await invoke('start_system_audio_transcription');
        setState(prev => ({ ...prev, isSystemSoundOn: true }));
      }
    } catch (error) {
      console.error('Failed to toggle system audio:', error);
    }
  };

  // Handle text input submission
  const handleSubmit = async () => {
    if (!state.inputText.trim() || state.isLoading) {
      console.warn('Send button: No input text or already loading');
      return;
    }

    console.log('🚀 SEND BUTTON: Starting submission with text:', state.inputText);
    setState(prev => ({ ...prev, isLoading: true, isStreaming: true }));
    
    try {
      // Reset response window size first (using DPI-FIXED below-main enhanced window)
      console.log('📰 SEND BUTTON: Resetting AI response window (DPI-FIXED below-main enhanced)...');
      await invoke('reset_ai_response_window_enhanced_below_size').catch(err => {
        console.warn('⚠️ Failed to reset DPI-fixed AI response window size:', err);
        // Try to create the window if it doesn't exist
        invoke('create_ai_response_window_enhanced_below').catch(e => 
          console.warn('⚠️ Failed to create DPI-fixed below-main enhanced window:', e)
        );
      });
      
      // Expand main window for response
      console.log('📰 SEND BUTTON: Expanding main window...');
      await autoResize(true, 'main');
      
      // Create the payload structure that matches GenerateAnswerPayload
      const payload = {
        question: state.inputText,
        model: state.selectedModel,
        provider: 'pollinations',
        company: null,
        position: null,
        job_description: null
      };
      
      console.log('📦 SEND BUTTON: Payload created:', payload);
      console.log('🔧 SEND BUTTON: Calling backend function: pollinations_generate_answer_streaming');
      
      // Save question to database if session is active
      if (state.session.isActive && state.session.sessionId) {
        try {
          const questionId = await invoke('save_interview_question', {
            sessionId: state.session.sessionId,
            questionNumber: state.questionCounter + 1,
            questionText: state.inputText,
            category: 'user_question',
            difficultyLevel: 'medium',
            expectedDuration: 300 // 5 minutes default
          });
          console.log('💾 Question saved to database with ID:', questionId);
          
          // Store question ID for answer linking
          setState(prev => ({ 
            ...prev, 
            currentQuestionId: questionId as string,
            questionCounter: prev.questionCounter + 1
          }));
        } catch (dbError) {
          console.warn('⚠️ Failed to save question to database:', dbError);
        }
      }
      
      // Call AI generation function with proper payload structure (wrapped in payload key)
      const result = await invoke('pollinations_generate_answer_streaming', { payload });
      console.log('✅ SEND BUTTON: Backend call successful, result:', result);
      
      setState(prev => ({ ...prev, inputText: '' }));
      console.log('🧹 SEND BUTTON: Input field cleared');
      
    } catch (error) {
      console.error('❌ SEND BUTTON: Failed to submit question:', error);
      console.error('❌ SEND BUTTON: Error details:', {
        name: (error as Error).name,
        message: (error as Error).message,
        stack: (error as Error).stack
      });
      setState(prev => ({ ...prev, isLoading: false, isStreaming: false }));
      // Collapse window on error
      await autoResize(false, 'main');
      
      // Show user-friendly error
      alert(`Failed to send question: ${(error as Error).message || error}`);
    }
  };

  // Session management functions
  const connectToSession = async () => {
    if (state.isLoading || !state.session.sessionId.trim()) return;
    
    setState(prev => ({ ...prev, isLoading: true }));
    
    try {
      // Use the correct backend function name for database connection
      const sessionData = await invoke('connect_session', {
        sessionId: state.session.sessionId
      }) as any;
      
      console.log('Session data received:', sessionData);
      
      setState(prev => ({ 
        ...prev, 
        isLoading: false,
        currentScreen: 'confirmation', 
        session: { 
          sessionId: sessionData.session_id || prev.session.sessionId,
          userId: sessionData.user_details?.name || 'Unknown User',
          credits: sessionData.credits_available || 0,
          timer: '00:00',
          startTime: 0,
          isActive: false,
          // Store additional session details
          sessionName: sessionData.session_name || 'Interview Session',
          companyName: sessionData.company_name || 'Company',
          jobTitle: sessionData.job_title || 'Position',
          userEmail: sessionData.user_details?.email || '',
          status: sessionData.status || 'created'
        } 
      }));
      
      // Update window size for confirmation screen
      await autoResize(false, 'confirmation');
    } catch (error) {
      console.error('Failed to connect to session:', error);
      // Show error to user
      alert(`Failed to connect to session: ${error}`);
      setState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const startSession = async () => {
    if (!state.session.sessionId || state.isStartingSession) {
      console.error('No session ID available for activation or already starting');
      return;
    }
    
    setState(prev => ({ ...prev, isStartingSession: true }));
    
    try {
      // Use the correct backend function and pass session ID
      await invoke('activate_session', {
        sessionId: state.session.sessionId
      });
      
      setState(prev => ({ 
        ...prev, 
        isStartingSession: false,
        currentScreen: 'main',
        session: { 
          ...prev.session, 
          isActive: true, 
          startTime: Date.now(),
          status: 'active'
        } 
      }));
      
      // Update window size for main screen
      await autoResize(false, 'main');
    } catch (error) {
      console.error('Failed to start session:', error);
      alert(`Failed to start session: ${error}`);
      setState(prev => ({ ...prev, isStartingSession: false }));
    }
  };

  const disconnectSession = async () => {
    try {
      // Send final timer update before disconnecting
      if (state.session.isActive && state.session.startTime > 0) {
        const currentTime = Date.now();
        const elapsedSeconds = Math.floor((currentTime - state.session.startTime) / 1000);
        const elapsedMinutes = Math.floor(elapsedSeconds / 60);
        
        console.log(`⏱️ Sending final timer update: ${elapsedMinutes} minutes`);
        try {
          await invoke('update_session_timer', {
            sessionId: state.session.sessionId,
            elapsedMinutes,
            isFinal: true
          });
        } catch (timerError) {
          console.warn('Failed to send final timer update:', timerError);
        }
      }
      
      // Clear timer
      if (timerIntervalRef.current) {
        clearInterval(timerIntervalRef.current);
        timerIntervalRef.current = null;
      }
      
      await invoke('disconnect_session');
      setState(prev => ({ 
        ...prev, 
        currentScreen: 'session_connection',
        session: { 
          sessionId: '',
          userId: '',
          isActive: false,
          timer: '00:00',
          credits: 0,
          startTime: 0,
        }
      }));
    } catch (error) {
      console.error('Failed to disconnect session:', error);
    }
  };

  // Generate AI answer
  const generateAIAnswer = async () => {
    if (state.isLoading) {
      console.warn('AI button: Already loading, skipping');
      return;
    }
    
    const questionText = state.fullTranscription || state.inputText || 'Please provide assistance';
    if (!questionText.trim()) {
      console.warn('AI button: No text to send to AI');
      return;
    }
    
    console.log('🤖 AI BUTTON: Starting AI answer generation with text:', questionText);
    setState(prev => ({ ...prev, isLoading: true, isStreaming: true }));
    
    try {
      // Reset response window size first (using DPI-FIXED below-main enhanced window)
      console.log('📰 AI BUTTON: Resetting AI response window (DPI-FIXED below-main enhanced)...');
      await invoke('reset_ai_response_window_enhanced_below_size').catch(err => {
        console.warn('⚠️ Failed to reset DPI-fixed AI response window size:', err);
        // Try to create the window if it doesn't exist
        invoke('create_ai_response_window_enhanced_below').catch(e => 
          console.warn('⚠️ Failed to create DPI-fixed below-main enhanced window:', e)
        );
      });
      
      console.log('🤖 AI BUTTON: Using model:', state.selectedModel);
      
      // Create the payload structure that matches GenerateAnswerPayload
      const payload = {
        question: questionText,
        model: state.selectedModel,
        provider: 'pollinations',
        company: null,
        position: null,
        job_description: null
      };
      
      console.log('📦 AI BUTTON: Payload created:', payload);
      console.log('🔧 AI BUTTON: Calling backend function: pollinations_generate_answer_streaming');
      
      // Save question to database if session is active
      if (state.session.isActive && state.session.sessionId) {
        try {
          const questionId = await invoke('save_interview_question', {
            sessionId: state.session.sessionId,
            questionNumber: state.questionCounter + 1,
            questionText: questionText,
            category: 'transcription',
            difficultyLevel: 'medium',
            expectedDuration: 300 // 5 minutes default
          });
          console.log('💾 Question saved to database with ID:', questionId);
          
          // Store question ID for answer linking
          setState(prev => ({ 
            ...prev, 
            currentQuestionId: questionId as string,
            questionCounter: prev.questionCounter + 1
          }));
        } catch (dbError) {
          console.warn('⚠️ Failed to save question to database:', dbError);
        }
      }
      
      // Call the streaming AI function with wrapped payload parameter
      const result = await invoke('pollinations_generate_answer_streaming', { payload });
      console.log('✅ AI BUTTON: Backend call successful, result:', result);
      
      // Clear the transcription after sending to AI (cut functionality)
      setState(prev => ({ 
        ...prev, 
        fullTranscription: '',
        interimTranscription: '',
        inputText: '',
        transcriptionText: 'Ready to assist you...'
      }));
      
      console.log('🧹 AI BUTTON: Transcription cleared, AI answer generation started...');
    } catch (error) {
      console.error('❌ AI BUTTON: Failed to generate AI answer:', error);
      console.error('❌ AI BUTTON: Error details:', {
        name: (error as Error).name,
        message: (error as Error).message,
        stack: (error as Error).stack
      });
      setState(prev => ({ ...prev, isLoading: false, isStreaming: false }));
      
      // Show user-friendly error
      alert(`Failed to generate AI answer: ${(error as Error).message || error}`);
    }
  };

  // Screen analysis
  const analyzeScreen = async () => {
    if (state.isLoading) return;
    
    setState(prev => ({ ...prev, isLoading: true, isStreaming: true }));
    
    try {
      // Reset response window size first (using DPI-FIXED below-main enhanced window)
      console.log('📰 SCREEN: Resetting AI response window (DPI-FIXED below-main enhanced)...');
      await invoke('reset_ai_response_window_enhanced_below_size').catch(err => {
        console.warn('⚠️ Failed to reset DPI-fixed AI response window size:', err);
        // Try to create the window if it doesn't exist
        invoke('create_ai_response_window_enhanced_below').catch(e => 
          console.warn('⚠️ Failed to create DPI-fixed below-main enhanced window:', e)
        );
      });
      
      // Expand main window for screen analysis response
      await autoResize(true, 'main');
      
      await invoke('analyze_screen_with_ai_streaming');
      console.log('Screen analysis started...');
    } catch (error) {
      console.error('Failed to analyze screen:', error);
      setState(prev => ({ ...prev, isLoading: false, isStreaming: false }));
      // Collapse window on error
      await autoResize(false, 'main');
    }
  };


  // Handle window controls
  const minimizeWindow = () => invoke('minimize_window');
  const closeWindow = () => invoke('close_application');

  // Render Session Connection Screen
  const renderSessionConnectionScreen = () => {
    
    return (
      <div className="w-full h-auto bg-black/70 backdrop-blur-md border border-white/20 rounded-lg shadow-lg">
        {/* Ultra Compact Header - Minimal height */}
        <div className="flex items-center justify-between border-b border-white/10 px-2 py-1 h-8">
          <div className="flex items-center gap-1">
            <div className="w-4 h-4 rounded-full bg-gradient-to-r from-blue-400 to-purple-500 flex items-center justify-center flex-shrink-0">
              <span className="material-icons text-white text-xs">psychology</span>
            </div>
            <h2 className="text-white font-medium text-xs">MockMate</h2>
          </div>
          <Button 
            onClick={closeWindow}
            className="w-4 h-4 bg-red-500/80 hover:bg-red-600 border-0 rounded-full p-0 flex items-center justify-center transition-all flex-shrink-0"
            title="Close"
          >
            <span className="material-icons text-white text-xs">close</span>
          </Button>
        </div>
        
        {/* Compact Vertical Content Area */}
        <div className="px-2 py-2">
          {/* Title Section */}
          <div className="mb-2">
            <h3 className="text-white font-medium text-xs mb-0 leading-tight">Connect to Session</h3>
            <p className="text-gray-300 text-xs leading-tight">Enter session ID</p>
          </div>
          
          {/* Input and Button Row */}
          <div className="flex items-center gap-2">
            <Input
              placeholder="Session ID"
              className="flex-1 bg-white/15 border-white/25 text-white placeholder:text-gray-400 rounded focus:border-blue-400 focus:ring-1 focus:ring-blue-400/50 px-2 py-1.5 text-xs transition-all h-8"
              value={state.session.sessionId}
              onChange={(e) => setState(prev => ({ 
                ...prev, 
                session: { ...prev.session, sessionId: e.target.value } 
              }))}
              onKeyPress={(e) => e.key === 'Enter' && !state.isLoading && state.session.sessionId.trim() && connectToSession()}
            />
            
            <Button 
              onClick={connectToSession}
              className="font-medium text-white bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 border-0 rounded disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 px-3 py-1.5 text-xs whitespace-nowrap h-8 flex items-center gap-1 min-w-[70px]"
              disabled={!state.session.sessionId.trim() || state.isLoading}
              title="Connect to Session"
            >
              {state.isLoading ? (
                <>
                  <span className="material-icons animate-spin text-xs">hourglass_empty</span>
                  <span className="text-xs">Wait</span>
                </>
              ) : (
                <>
                  <span className="material-icons text-xs">link</span>
                  <span className="text-xs">Connect</span>
                </>
              )}
            </Button>
          </div>
        </div>
      </div>
    );
  };

  // Render Confirmation Screen
  const renderConfirmationScreen = () => {
    
    return (
      <div className="w-full h-auto bg-black/70 backdrop-blur-md border border-white/20 rounded-lg shadow-lg">
        {/* Ultra Compact Header - Minimal height (same as session connection) */}
        <div className="flex items-center justify-between border-b border-white/10 px-2 py-1 h-8">
          <div className="flex items-center gap-1">
            <div className="w-4 h-4 rounded-full bg-gradient-to-r from-blue-400 to-purple-500 flex items-center justify-center flex-shrink-0">
              <span className="material-icons text-white text-xs">psychology</span>
            </div>
            <h2 className="text-white font-medium text-xs">MockMate</h2>
          </div>
          <Button 
            onClick={closeWindow}
            className="w-4 h-4 bg-red-500/80 hover:bg-red-600 border-0 rounded-full p-0 flex items-center justify-center transition-all flex-shrink-0"
            title="Close"
          >
            <span className="material-icons text-white text-xs">close</span>
          </Button>
        </div>
        
        {/* Compact Content Area */}
        <div className="px-2 py-2">
          {/* Single Row Layout - More Organized */}
          <div className="flex items-center gap-2 w-full">
            {/* Session Status - Better Organized */}
            <div className="flex-1 bg-white/10 rounded px-2 py-1.5 text-xs">
              <div className="grid grid-cols-3 gap-x-2 gap-y-1">
                <div className="flex justify-between">
                  <span className="text-gray-400">ID:</span>
                  <span className="text-blue-400 font-mono text-xs">{state.session.sessionId.substring(0, 6)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Credits:</span>
                  <span className={`font-bold text-xs ${
                    state.session.credits > 10 ? 'text-green-400' : 
                    state.session.credits > 5 ? 'text-yellow-400' : 'text-red-400'
                  }`}>
                    {state.session.credits}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Status:</span>
                  <span className="text-green-400 text-xs">Ready</span>
                </div>
                <div className="flex justify-between col-span-2">
                  <span className="text-gray-400">User:</span>
                  <span className="text-white text-xs truncate" title={state.session.userId}>{state.session.userId}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Type:</span>
                  <span className="text-blue-300 text-xs truncate" title={state.session.jobTitle || 'Interview'}>
                    {state.session.jobTitle ? state.session.jobTitle.substring(0, 8) + '...' : 'Interview'}
                  </span>
                </div>
              </div>
            </div>
            
            {/* Success Icon */}
            <div className="w-6 h-6 rounded-full bg-gradient-to-r from-green-400 to-green-500 flex items-center justify-center flex-shrink-0">
              <span className="material-icons text-white text-sm">check</span>
            </div>
            
            {/* Action Buttons */}
            <div className="flex gap-1">
              <Button 
                onClick={startSession}
                className="font-medium text-white bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 border-0 rounded disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 px-3 py-1.5 text-xs whitespace-nowrap h-8 flex items-center gap-1 min-w-[60px]"
                disabled={state.isStartingSession}
                title="Start Session"
              >
                {state.isStartingSession ? (
                  <>
                    <span className="material-icons animate-spin text-xs">hourglass_empty</span>
                    <span className="text-xs">Wait</span>
                  </>
                ) : (
                  <>
                    <span className="material-icons text-xs">play_arrow</span>
                    <span className="text-xs">Start</span>
                  </>
                )}
              </Button>
              
              <Button 
                onClick={() => setState(prev => ({ ...prev, currentScreen: 'session_connection' }))}
                className="font-medium text-white bg-white/10 hover:bg-white/20 border border-white/25 rounded transition-all duration-200 px-3 py-1.5 text-xs whitespace-nowrap h-8 flex items-center gap-1 min-w-[50px]"
                title="Go Back"
              >
                <span className="material-icons text-xs">arrow_back</span>
                <span className="text-xs">Back</span>
              </Button>
            </div>
          </div>
        </div>
      </div>
    );
  };

  // Render Main Screen
  const renderMainScreen = () => {
    
    return (
      <div className="w-full h-auto bg-black/70 backdrop-blur-md border border-white/20 rounded-lg shadow-lg">
        {/* Compact Header - Same as session windows */}
        <div className="flex items-center justify-between border-b border-white/10 px-2 py-1 h-8">
          <div className="flex items-center gap-1">
            <div className="w-4 h-4 rounded-full bg-gradient-to-r from-blue-400 to-purple-500 flex items-center justify-center flex-shrink-0">
              <span className="material-icons text-white text-xs">psychology</span>
            </div>
            <h2 className="text-white font-medium text-xs">MockMate</h2>
            {state.session.isActive && (
              <>
                <div className="flex items-center gap-1 ml-2">
                  <div className="w-2 h-2 rounded-full bg-green-400 animate-pulse"></div>
                  <span className="text-green-400 text-xs font-medium">Active</span>
                  {/* Session Timer Display */}
                  <div className="flex items-center gap-1 ml-2 bg-black/30 rounded px-2 py-0.5">
                    <span className="material-icons text-blue-400 text-xs">timer</span>
                    <span className="text-white text-xs font-mono tabular-nums">{state.session.timer}</span>
                  </div>
                </div>
                {/* Model Selection */}
                <div className="relative model-dropdown">
                  <Button
                    onClick={toggleModelDropdown}
                    className="flex items-center gap-1 ml-2 rounded bg-white/10 border border-white/20 px-1.5 py-0.5 hover:bg-white/20 transition-all"
                  >
                    <span className="material-icons text-blue-400 text-xs">psychology</span>
                    <span className="text-white text-xs font-medium">
                      {state.availableModels.find(m => m.id === state.selectedModel)?.name?.substring(0, 15) || state.selectedModel.substring(0, 15)}...
                    </span>
                    <span className={`material-icons text-gray-400 text-xs transition-transform ${
                      state.isModelDropdownOpen ? 'rotate-180' : ''
                    }`}>expand_more</span>
                  </Button>
                  
                  {/* Dropdown Menu */}
                  {state.isModelDropdownOpen && (
                    <div className="absolute top-full left-0 mt-1 bg-black/90 backdrop-blur-sm border border-white/20 rounded shadow-lg z-50 min-w-[200px]">
                      <div className="py-1 max-h-60 overflow-y-auto scrollbar-hide">
                        {state.availableModels.map((model) => (
                          <button
                            key={model.id}
                            onClick={() => selectModel(model.id)}
                            className={`w-full text-left px-3 py-2 text-xs hover:bg-white/10 transition-colors ${
                              model.id === state.selectedModel ? 'bg-blue-500/20 text-blue-300' : 'text-white'
                            }`}
                          >
                            <div className="flex items-center gap-2">
                              <span className="material-icons text-xs opacity-70">psychology</span>
                              <div>
                                <div className="font-medium">{model.name}</div>
                                <div className="text-gray-400 text-xs opacity-70">{model.provider}</div>
                              </div>
                            </div>
                          </button>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              </>
            )}
          </div>
          <div className="flex items-center space-x-1">
            {/* Audio and Session Controls */}
            <Button 
              onClick={toggleMicrophone}
              className={`w-6 h-6 rounded p-0 border-0 flex items-center justify-center transition-all ${
                state.isMicOn ? 'bg-blue-500/80 text-white shadow-lg' : 'bg-white/10 text-gray-300 hover:bg-white/20'
              }`}
              title={state.isMicOn ? 'Stop microphone' : 'Start microphone'}
            >
              <i className="material-icons" style={{ fontSize: '16px' }}>{state.isMicOn ? 'mic' : 'mic_off'}</i>
            </Button>

            <Button 
              onClick={toggleSystemAudio}
              className={`w-6 h-6 rounded p-0 border-0 flex items-center justify-center transition-all ${
                state.isSystemSoundOn ? 'bg-blue-500/80 text-white shadow-lg' : 'bg-white/10 text-gray-300 hover:bg-white/20'
              }`}
              title={state.isSystemSoundOn ? 'Stop system audio' : 'Start system audio'}
            >
              <i className="material-icons" style={{ fontSize: '16px' }}>{state.isSystemSoundOn ? 'volume_up' : 'volume_off'}</i>
            </Button>
            
            <Button 
              onClick={disconnectSession}
              className="w-6 h-6 rounded p-0 border-0 bg-white/10 text-gray-300 hover:bg-white/20 flex items-center justify-center transition-all"
              title="Disconnect Session"
            >
              <i className="material-icons" style={{ fontSize: '16px' }}>logout</i>
            </Button>
            
            <Button 
              onClick={minimizeWindow}
              className="w-6 h-6 bg-gray-500/80 hover:bg-gray-600 border-0 rounded-full p-0 flex items-center justify-center transition-all flex-shrink-0"
              title="Minimize"
            >
              <i className="material-icons text-white" style={{ fontSize: '16px' }}>minimize</i>
            </Button>
            
            <Button 
              onClick={closeWindow}
              className="w-6 h-6 bg-red-500/80 hover:bg-red-600 border-0 rounded-full p-0 flex items-center justify-center transition-all flex-shrink-0"
              title="Close"
            >
              <i className="material-icons text-white" style={{ fontSize: '16px' }}>close</i>
            </Button>
          </div>
        </div>
        
        {/* Ultra Compact Content Area - Two Row Layout */}
        <div className="px-2 py-1">
          <div className="flex flex-col gap-1">
            {/* Live Transcription Area - Single Line with Horizontal Scroll */}
            <div className="w-full bg-white/10 rounded px-2 py-2 relative h-[40px] flex items-center">
              <div 
                ref={transcriptionRef}
                className={`text-lg overflow-x-auto scrollbar-hide pr-6 whitespace-nowrap ${
                  state.isTranscriptionActive ? 'text-white' : 'text-gray-300'
                } ${state.interimTranscription ? 'text-blue-300' : ''} ${state.fullTranscription ? 'text-white' : ''}`}
                style={{ 
                  lineHeight: '1.2',
                  minWidth: '100%',
                  display: 'inline-block'
                }}
              >
                {state.fullTranscription && (
                  <span className="text-white">{state.fullTranscription}</span>
                )}
                {state.interimTranscription && (
                  <span className="text-blue-300 italic"> {state.interimTranscription}</span>
                )}
                {!state.fullTranscription && !state.interimTranscription && (
                  <span>{state.transcriptionText}</span>
                )}
              </div>
              <Button 
                onClick={() => setState(prev => ({ ...prev, fullTranscription: '', interimTranscription: '', transcriptionText: 'Ready to assist you...' }))}
                className="absolute top-1 right-1 w-5 h-5 bg-white/10 hover:bg-white/20 border-0 rounded p-0 flex items-center justify-center transition-all"
                title="Clear transcription"
              >
                <span className="material-icons text-gray-400 text-sm">clear</span>
              </Button>
            </div>
            
            {/* Function Buttons Row - Below Transcription */}
            <div className="flex items-center gap-1">
              {/* AI Action Buttons */}
              <Button 
                onClick={generateAIAnswer}
                className={`font-medium text-white border-0 rounded disabled:opacity-50 transition-all duration-200 px-4 py-1 text-xs h-6 flex items-center gap-0.5 min-w-[158px] ${
                  state.isStreaming 
                    ? 'bg-gradient-to-r from-purple-500 to-purple-600 hover:from-purple-600 hover:to-purple-700 shadow-lg' 
                    : 'bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700'
                }`}
                disabled={state.isLoading || (!state.fullTranscription && !state.inputText.trim())}
                title="Generate AI Answer"
              >
                {state.isStreaming ? (
                  <span className="material-icons animate-spin text-xs">hourglass_empty</span>
                ) : (
                  <span className="material-icons text-xs">auto_awesome</span>
                )}
                <span className="text-xs">AI</span>
              </Button>
              
              <Button 
                onClick={analyzeScreen}
                className="font-medium text-white bg-gradient-to-r from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 border-0 rounded transition-all duration-200 px-4 py-1 text-xs h-6 flex items-center gap-0.5 min-w-[180px]"
                title="Analyze Screen Content"
                disabled={state.isLoading}
              >
                <span className="material-icons text-xs">screenshot_monitor</span>
                <span className="text-xs">Screen</span>
              </Button>
              
              {/* Input field - Takes remaining space */}
              <Input
                value={state.inputText}
                onChange={(e) => setState(prev => ({ ...prev, inputText: e.target.value }))}
                placeholder="Ask a question..."
                className="flex-1 bg-white/15 border-white/25 text-white placeholder:text-gray-400 rounded focus:border-blue-400 focus:ring-1 focus:ring-blue-400/50 px-2 py-0.5 text-xs transition-all h-6"
                onKeyPress={(e) => e.key === 'Enter' && handleSubmit()}
                disabled={state.isLoading}
              />
              
              {/* Send button */}
              <Button
                onClick={handleSubmit}
                className="font-medium text-white bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 border-0 rounded disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 px-3 py-1 text-xs h-6 flex items-center gap-0.5 min-w-[70px]"
                disabled={state.isLoading || !state.inputText.trim()}
                title="Send question"
              >
                {state.isLoading ? (
                  <span className="material-icons animate-spin text-xs">hourglass_empty</span>
                ) : (
                  <span className="material-icons text-xs">send</span>
                )}
                <span className="text-xs">Send</span>
              </Button>
            </div>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="w-screen h-auto bg-transparent">
      {state.currentScreen === 'session_connection' && renderSessionConnectionScreen()}
      {state.currentScreen === 'confirmation' && renderConfirmationScreen()}
      {state.currentScreen === 'main' && renderMainScreen()}
    </div>
  );
}

export default App;
