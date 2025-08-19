// Replace line 1093 in main.js with this:
async analyzeScreen() {
    try {
        console.log('🔍 Starting screen analysis...');
        
        // Validate selection before starting analysis
        this.ensureValidModelSelection();
        
        // Dynamic notification based on user's selected provider and model
        const providerDisplayName = this.getProviderDisplayName(this.selectedProvider);
        const modelDisplayName = this.models.find(m => (m.id || m.value) === this.selectedModel)?.name || this.selectedModel;
        
        this.showNotification(`Analyzing screen with ${providerDisplayName} ${modelDisplayName}...`, 'info');
        
        // Show AI response window in initial state
        this.createAiResponseWindowInInitialState();
        await this.showResponseWindow();

        // ✅ STORE THE QUESTION FIRST for screen analysis
        try {
            console.log('📝 Storing screen analysis question...');
            await window.qaStorageManager.storeQuestion({
                questionText: '[SCREEN_ANALYSIS] Generated question from screen content',
                questionNumber: 1,
                category: 'screen-analysis',
                difficultyLevel: 'medium',
                source: 'screen-analysis',
                metadata: {
                    timestamp: new Date().toISOString(),
                    expectedDuration: 5,
                    analysisType: 'visual',
                    provider: this.selectedProvider,
                    model: this.selectedModel
                }
            });
            console.log('✅ Screen analysis question stored successfully');
        } catch (storageError) {
            console.error('❌ Failed to store screen analysis question:', storageError);
            // Continue with analysis even if storage fails
        }
        
        // Use the same dynamic flow as Send button - respect user's model and provider selection
        const systemPrompt = 'You are an expert technical interviewer analyzing screen content. Generate a specific, relevant technical interview question based on the visible content from applications. Focus on any visible code, applications, documentation, or technical content that could form the basis of a good interview question.';
        
        const payload = {
            model: this.selectedModel,        // Use user's selected model
            provider: this.selectedProvider,  // Use user's selected provider
            company: null,
            position: null, 
            job_description: null,
            system_prompt: systemPrompt
        };
        
        console.log('📸 Screen Analysis Payload (Dynamic):', JSON.stringify(payload, null, 2));
        
        // Use the same streaming pattern as Send button for consistency
        if (this.selectedProvider === 'pollinations') {
            // Use backend streaming for better UX (recommended for Pollinations)
            try {
                console.log('🚀 Starting Pollinations streaming for screen analysis...');
                // Reset streaming state for new request
                this.streamingText = '';
                this.isStreaming = true;
                
                // 🔧 FIX: Add the missing { payload: payload } wrapper
                const result = await safeInvoke('analyze_screen_with_ai_streaming', { payload: payload });
                console.log('✅ Pollinations screen analysis streaming completed:', result);
                
                // Format the result for display
                const responseText = `**🎯 Generated Interview Question:**\n\n${result.generated_question}\n\n**📋 Analysis:**\n\n${result.analysis}\n\n**🎲 Confidence:** ${(result.confidence * 100).toFixed(0)}%\n\n---\n*Generated using ${providerDisplayName} ${modelDisplayName} with Windows Accessibility API*`;
                
                await this.sendToAiWindow('complete', responseText);
            } catch (streamError) {
                console.warn('⚠️ Screen analysis streaming failed, falling back to non-streaming:', streamError);
                const result = await safeInvoke('analyze_screen_with_ai', { payload: payload });
                
                // Handle both structured result and simple string response
                let responseText;
                if (typeof result === 'object' && result.generated_question) {
                    responseText = `**🎯 Generated Interview Question:**\n\n${result.generated_question}\n\n**📋 Analysis:**\n\n${result.analysis}\n\n**🎲 Confidence:** ${(result.confidence * 100).toFixed(0)}%\n\n---\n*Generated using ${providerDisplayName} ${modelDisplayName}*`;
                } else {
                    responseText = result;
                }
                
                await this.sendToAiWindow('complete', responseText);
            }
        } else {
            // For OpenAI and other providers
            const result = await safeInvoke('analyze_screen_with_ai', { payload: payload });
            
            // Handle both structured result and simple string response
            let responseText;
            if (typeof result === 'object' && result.generated_question) {
                responseText = `**🎯 Generated Interview Question:**\n\n${result.generated_question}\n\n**📋 Analysis:**\n\n${result.analysis}\n\n**🎲 Confidence:** ${(result.confidence * 100).toFixed(0)}%\n\n---\n*Generated using ${providerDisplayName} ${modelDisplayName}*`;
            } else {
                responseText = result;
            }
            
            await this.sendToAiWindow('complete', responseText);
        }
        
        this.showNotification('Screen analysis completed! Question generated successfully.', 'success');
        
    } catch (error) {
        console.error('❌ Failed to analyze screen:', error);
        
        // Send error to AI window if it exists
        await this.sendToAiWindow('error', error.message || error.toString());
        
        if (error.message && error.message.includes('Tauri not ready')) {
            this.showNotification('Please wait for app to finish initializing...', 'warning');
        } else if (error.message && error.message.includes('screenshot')) {
            this.showNotification('Failed to capture screenshot. Make sure the screen is accessible.', 'error');
        } else if (error.message && error.message.includes('API')) {
            this.showNotification('AI vision analysis failed. Check Pollinations/OpenAI configuration.', 'error');
        } else {
            this.showNotification(`Screen analysis failed: ${error.message || error}`, 'error');
        }
    }
}
