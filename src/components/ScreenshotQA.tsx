import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';

interface ScreenshotQAProps {
  selectedModel: string;
  isLoading: boolean;
  setIsLoading: (isLoading: boolean) => void;
  setIsStreaming: (isStreaming: boolean) => void;
  sessionDetails?: {
    companyName?: string;
    jobTitle?: string;
  };
  onError: (title: string, message: string) => void;
}

const ScreenshotQA: React.FC<ScreenshotQAProps> = ({
  selectedModel,
  isLoading,
  setIsLoading,
  setIsStreaming,
  sessionDetails,
  onError
}) => {
  const [isCapturing, setIsCapturing] = useState(false);
  
  // Ultra-accurate vision models ranked by performance for Q&A scenarios
  const ultraAccuracyModels = [
    "gemini-search", // #1: Gemini 2.5 Flash + Google Search (seed tier) - Best overall
    "openai-reasoning", // #2: OpenAI o4-mini (seed tier) - Best for complex reasoning  
    "gemini",        // #3: Gemini 2.5 Flash Lite (seed tier) - Fast and accurate
    "openai",        // #4: OpenAI GPT-5 Nano (anonymous tier) - Reliable backup
    "openai-fast",   // #5: OpenAI GPT-4.1 Nano (anonymous tier) - Speed optimized
    "openai-audio",  // #6: GPT-4o Mini Audio Preview (seed tier) - Multi-modal
    "bidara",        // #7: BIDARA by NASA (anonymous tier, community) - Specialized
    "unity",         // #8: Unity Unrestricted Agent (seed tier, community)
    "evil",          // #9: Evil (Uncensored) (seed tier, community)
  ];

  // Automatically select optimal ultra-accurate model for multi-question scenarios
  const selectOptimalModel = (currentModel: string): string => {
    if (ultraAccuracyModels.includes(currentModel)) {
      return currentModel;
    }
    // Default to gemini-search for maximum accuracy with Google Search integration
    return "gemini-search";
  };
  
  const handleScreenshotQA = async () => {
    if (isLoading || isCapturing) return;
    
    setIsCapturing(true);
    setIsLoading(true);
    setIsStreaming(true);
    
    try {
      // Reset response window size first (using DPI-FIXED below-main enhanced window)
      console.log('📸 SCREENSHOT QA: Resetting AI response window...');
      await invoke('reset_ai_response_window_enhanced_below_size').catch((err: any) => {
        console.warn('⚠️ Failed to reset AI response window size:', err);
        // Try to create the window if it doesn't exist
        invoke('create_ai_response_window_enhanced_below').catch((e: any) => 
          console.warn('⚠️ Failed to create window:', e)
        );
      });
      
      // Note: We'll let the natural resize mechanism handle window expansion
      // to avoid conflicts with the ResizeObserver and prevent height accumulation
      console.log('📸 SCREENSHOT QA: Window will expand naturally when AI response appears...');
      
      // Get optimal ultra-accurate model for multi-question scenarios
      const modelToUse = selectOptimalModel(selectedModel);
      console.log(`🔥🎯 ULTRA Q&A: Auto-selecting optimal model: ${modelToUse} (original: ${selectedModel}) - Optimized for 100% accuracy`);
      
      // Create the payload for ultra-accurate Q&A with multi-question support
      const payload = {
        model: modelToUse,
        provider: 'pollinations',
        company: sessionDetails?.companyName || null,
        position: sessionDetails?.jobTitle || null,
        job_description: null,
        system_prompt: null,
      };
      
      console.log('🔥🎯 ULTRA Q&A: Optimized payload created:', payload);
      console.log('🔥🎯 ULTRA Q&A: Capturing screenshot and analyzing with advanced AI for 100% accuracy...');
      
      // Call the ULTRA-ACCURATE backend function with advanced prompt engineering
      await invoke('enhanced_qa_with_vision_streaming', { payload });
      console.log('🚀🎯 ULTRA Q&A: Processing started with optimally-selected model and multi-question support...');
      
      // The streaming response will be handled by the event listeners
    } catch (error) {
      console.error('❌ SCREENSHOT QA: Failed to analyze screenshot:', error);
      console.error('❌ SCREENSHOT QA: Error details:', {
        name: (error as Error).name,
        message: (error as Error).message,
        stack: (error as Error).stack
      });
      
      setIsLoading(false);
      setIsStreaming(false);
      setIsCapturing(false);
      
      // Window will naturally resize when loading state changes
      
      // Show user-friendly error
      onError('Ultra Q&A Error', `Failed ultra-accurate analysis: ${(error as Error).message || error}`);
    } finally {
      setIsCapturing(false);
    }
  };

  return (
    <Button
      onClick={handleScreenshotQA}
      className="font-medium text-white bg-gradient-to-r from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 border-0 rounded transition-all duration-200 px-4 py-1 text-xs h-6 flex items-center gap-0.5 min-w-[100px] shadow-lg"
      disabled={isLoading || isCapturing}
    >
      {isCapturing ? (
        <>
          <span className="material-icons animate-pulse text-xs">psychology</span>
          <span className="text-xs">Ultra-Analyzing...</span>
        </>
      ) : (
        <>
          <span className="material-icons text-xs">psychology</span>
          <span className="text-xs">Ultra Q&A</span>
        </>
      )}
    </Button>
  );
};

export default ScreenshotQA;
