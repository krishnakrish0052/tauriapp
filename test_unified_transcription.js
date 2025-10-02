// Test script to directly call the unified transcription commands
// This helps us debug if the new commands are working correctly

import { invoke } from '@tauri-apps/api/core';

console.log('🧪 Testing unified Pluely-Deepgram transcription commands...');

// Test the test command first
console.log('📡 Testing Deepgram connection...');
try {
    const testResult = await invoke('test_deepgram_streaming_direct');
    console.log('✅ Deepgram test result:', testResult);
} catch (error) {
    console.error('❌ Deepgram test failed:', error);
}

// Test starting the unified transcription
console.log('🚀 Starting unified transcription...');
try {
    await invoke('start_pluely_deepgram_transcription');
    console.log('✅ Unified transcription started successfully');
    
    // Wait a few seconds
    console.log('⏳ Waiting 5 seconds...');
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    // Check status
    console.log('📊 Checking transcription status...');
    const status = await invoke('get_pluely_deepgram_transcription_status');
    console.log('📊 Status:', status);
    
    // Stop transcription
    console.log('🛑 Stopping unified transcription...');
    await invoke('stop_pluely_deepgram_transcription');
    console.log('✅ Unified transcription stopped successfully');
    
} catch (error) {
    console.error('❌ Unified transcription test failed:', error);
}

console.log('🧪 Test completed');