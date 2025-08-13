// Q&A Debug Logger for Desktop App
// This script adds extensive logging to track Q&A storage calls

(function() {
    console.log('🔬 Q&A Debug Logger initialized');

    // Wait for the main components to be available
    function waitForComponents() {
        return new Promise((resolve) => {
            const checkInterval = setInterval(() => {
                if (window.mockMateController && window.qaStorageManager && window.sessionFlowManager) {
                    clearInterval(checkInterval);
                    resolve();
                }
            }, 100);
        });
    }

    // Initialize debug logging when components are ready
    waitForComponents().then(() => {
        console.log('🔬 All components detected, setting up Q&A debug logging...');
        
        // Add debug logging to QA Storage Manager
        if (window.qaStorageManager) {
            const originalStoreQuestion = window.qaStorageManager.storeQuestion;
            const originalStoreAnswer = window.qaStorageManager.storeAnswer;
            const originalInitialize = window.qaStorageManager.initialize;
            
            // Wrap storeQuestion
            window.qaStorageManager.storeQuestion = async function(questionData) {
                console.log('🔬 [QA DEBUG] storeQuestion called with:', questionData);
                console.log('🔬 [QA DEBUG] Current session:', this.currentSession);
                console.log('🔬 [QA DEBUG] Auth token present:', !!this.authToken);
                console.log('🔬 [QA DEBUG] Is online:', this.isOnline);
                
                try {
                    const result = await originalStoreQuestion.call(this, questionData);
                    console.log('🔬 [QA DEBUG] storeQuestion result:', result);
                    return result;
                } catch (error) {
                    console.error('🔬 [QA DEBUG] storeQuestion error:', error);
                    throw error;
                }
            };
            
            // Wrap storeAnswer
            window.qaStorageManager.storeAnswer = async function(answerData) {
                console.log('🔬 [QA DEBUG] storeAnswer called with:', answerData);
                console.log('🔬 [QA DEBUG] Current session:', this.currentSession);
                console.log('🔬 [QA DEBUG] Current question ID:', this.currentQuestionId);
                
                try {
                    const result = await originalStoreAnswer.call(this, answerData);
                    console.log('🔬 [QA DEBUG] storeAnswer result:', result);
                    return result;
                } catch (error) {
                    console.error('🔬 [QA DEBUG] storeAnswer error:', error);
                    throw error;
                }
            };
            
            // Wrap initialize
            window.qaStorageManager.initialize = function(sessionData, token, userId) {
                console.log('🔬 [QA DEBUG] initialize called with:');
                console.log('🔬 [QA DEBUG] - sessionData:', sessionData);
                console.log('🔬 [QA DEBUG] - token present:', !!token);
                console.log('🔬 [QA DEBUG] - userId:', userId);
                
                const result = originalInitialize.call(this, sessionData, token, userId);
                console.log('🔬 [QA DEBUG] initialize completed');
                return result;
            };
        }
        
        // Add debug logging to main controller methods
        if (window.mockMateController) {
            // Check if methods exist
            console.log('🔬 [QA DEBUG] Main controller methods:');
            console.log('🔬 [QA DEBUG] - generateAnswer:', typeof window.mockMateController.generateAnswer);
            console.log('🔬 [QA DEBUG] - sendManualQuestion:', typeof window.mockMateController.sendManualQuestion);
            console.log('🔬 [QA DEBUG] - sendToAiWindow:', typeof window.mockMateController.sendToAiWindow);
            
            // Store original methods
            const originalGenerateAnswer = window.mockMateController.generateAnswer;
            const originalSendManualQuestion = window.mockMateController.sendManualQuestion;
            const originalSendToAiWindow = window.mockMateController.sendToAiWindow;
            
            // Log when methods are called
            if (originalGenerateAnswer) {
                window.mockMateController.generateAnswer = async function(...args) {
                    console.log('🔬 [QA DEBUG] generateAnswer called');
                    console.log('🔬 [QA DEBUG] - fullTranscription:', this.fullTranscription);
                    console.log('🔬 [QA DEBUG] - input field value:', document.getElementById('questionInput')?.value);
                    
                    const result = await originalGenerateAnswer.apply(this, args);
                    console.log('🔬 [QA DEBUG] generateAnswer completed');
                    return result;
                };
            }
            
            if (originalSendManualQuestion) {
                window.mockMateController.sendManualQuestion = async function(...args) {
                    console.log('🔬 [QA DEBUG] sendManualQuestion called');
                    console.log('🔬 [QA DEBUG] - input field value:', document.getElementById('questionInput')?.value);
                    
                    const result = await originalSendManualQuestion.apply(this, args);
                    console.log('🔬 [QA DEBUG] sendManualQuestion completed');
                    return result;
                };
            }
            
            if (originalSendToAiWindow) {
                window.mockMateController.sendToAiWindow = async function(type, content, ...args) {
                    console.log('🔬 [QA DEBUG] sendToAiWindow called with type:', type);
                    console.log('🔬 [QA DEBUG] - content length:', content?.length);
                    console.log('🔬 [QA DEBUG] - content preview:', content?.substring(0, 50) + '...');
                    
                    const result = await originalSendToAiWindow.apply(this, [type, content, ...args]);
                    console.log('🔬 [QA DEBUG] sendToAiWindow completed');
                    return result;
                };
            }
        }
        
        // Add debug logging to session flow
        if (window.sessionFlowManager) {
            console.log('🔬 [QA DEBUG] Session flow manager state:');
            console.log('🔬 [QA DEBUG] - current state:', window.sessionFlowManager.currentState);
            console.log('🔬 [QA DEBUG] - session data:', window.sessionFlowManager.sessionData);
            
            // Log when QA storage manager is initialized
            const originalInitializeQAStorageManager = window.sessionFlowManager.initializeQAStorageManager;
            if (originalInitializeQAStorageManager) {
                window.sessionFlowManager.initializeQAStorageManager = function() {
                    console.log('🔬 [QA DEBUG] initializeQAStorageManager called in session flow');
                    console.log('🔬 [QA DEBUG] - session data:', this.sessionData);
                    console.log('🔬 [QA DEBUG] - qa manager available:', !!window.qaStorageManager);
                    console.log('🔬 [QA DEBUG] - main controller available:', !!window.mockMateController);
                    
                    const result = originalInitializeQAStorageManager.call(this);
                    console.log('🔬 [QA DEBUG] initializeQAStorageManager completed');
                    return result;
                };
            }
        }
        
        // Log DOM events
        document.addEventListener('click', (event) => {
            const target = event.target;
            if (target.id === 'generateAnswerBtn' || target.id === 'sendBtn') {
                console.log('🔬 [QA DEBUG] Button clicked:', target.id);
                console.log('🔬 [QA DEBUG] - Question input value:', document.getElementById('questionInput')?.value);
                console.log('🔬 [QA DEBUG] - Full transcription:', window.mockMateController?.fullTranscription);
            }
        });
        
        console.log('✅ Q&A Debug Logger setup completed');
    });
})();
