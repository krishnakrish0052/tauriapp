// Robust Tauri API initialization with retry logic
let invoke, listen, isTauriReady = false;

// Function to check if Tauri is available
function checkTauriAvailability() {
    return new Promise((resolve) => {
        console.log('🔍 Checking for Tauri API availability...');
        console.log('Environment info:', {
            userAgent: navigator.userAgent,
            hasWindow: typeof window !== 'undefined',
            hasTauriGlobal: typeof window !== 'undefined' && 'window.__TAURI__' in window,
            windowTauri: typeof window !== 'undefined' ? window.__TAURI__ : 'window undefined'
        });
        
        let attempts = 0;
        const checkTauri = () => {
            attempts++;
            console.log(`🔍 Attempt ${attempts}: Checking Tauri structure:`, {
                hasTauri: !!window.__TAURI__,
                tauriKeys: window.__TAURI__ ? Object.keys(window.__TAURI__) : 'none',
                hasInvoke: !!(window.__TAURI__ && window.__TAURI__.invoke),
                hasCore: !!(window.__TAURI__ && window.__TAURI__.core),
                hasTauriProp: !!(window.__TAURI__ && window.__TAURI__.tauri)
            });
            
            // Check for different possible Tauri API structures
            let tauriInvoke, tauriListen;
            
            if (window.__TAURI__) {
                // Try direct invoke (Tauri v2 style)
                if (window.__TAURI__.invoke) {
                    tauriInvoke = window.__TAURI__.invoke;
                    tauriListen = window.__TAURI__.event?.listen;
                }
                // Try core.invoke (alternative structure)
                else if (window.__TAURI__.core?.invoke) {
                    tauriInvoke = window.__TAURI__.core.invoke;
                    tauriListen = window.__TAURI__.event?.listen;
                }
                // Try tauri.invoke (nested structure)
                else if (window.__TAURI__.tauri?.invoke) {
                    tauriInvoke = window.__TAURI__.tauri.invoke;
                    tauriListen = window.__TAURI__.event?.listen;
                }
            }
            
            if (tauriInvoke && tauriListen) {
                invoke = tauriInvoke;
                listen = tauriListen;
                isTauriReady = true;
                console.log(`✅ Tauri API loaded successfully after ${attempts} attempts`);
                console.log('Available Tauri APIs:', Object.keys(window.__TAURI__));
                console.log('Using invoke from:', tauriInvoke.name || 'anonymous function');
                resolve(true);
            } else {
                // Fast initial checks, then slower checks
                const delay = attempts < 20 ? 50 : attempts < 50 ? 100 : 200;
                if (attempts <= 5 || attempts % 10 === 0) {
                    console.log(`⏳ Tauri API not ready yet (attempt ${attempts}), retrying in ${delay}ms...`);
                }
                setTimeout(checkTauri, delay);
            }
        };
        checkTauri();
        
        // Timeout after 10 seconds (increased from 5)
        setTimeout(() => {
            if (!isTauriReady) {
                console.error('❌ Tauri API not available after 10 seconds');
                console.error('💡 Make sure you are running the app with "npm run dev" or "tauri dev", not opening the HTML file directly in a browser!');
                // Create fallback functions
                invoke = async (cmd, args) => {
                    const errorMsg = `Tauri not available - command: ${cmd}. Please run the app using "npm run dev" instead of opening HTML directly in browser.`;
                    console.warn(`🚫 Tauri invoke attempted: ${cmd}`, args);
                    throw new Error(errorMsg);
                };
                listen = async (event, handler) => {
                    console.warn(`🚫 Tauri listen attempted: ${event}`);
                    return () => {}; // Return empty unsubscribe function
                };
                resolve(false);
            }
        }, 10000);
    });
}

// Safe invoke function
async function safeInvoke(command, args = {}) {
    console.log(`🔧 safeInvoke called: ${command}, isTauriReady: ${isTauriReady}`);
    if (!isTauriReady) {
        throw new Error(`Tauri not ready - cannot invoke: ${command}`);
    }
    try {
        console.log(`📞 Invoking Tauri command: ${command}`);
        return await invoke(command, args);
    } catch (error) {
        console.error(`❌ Tauri invoke failed for ${command}:`, error);
        throw error;
    }
}

class MockMateController {
    constructor() {
        this.isMicOn = false;
        this.isSystemSoundOn = false;
        this.isTranscriptionActive = false;
        this.currentTranscription = '';
        this.selectedModel = 'gpt-4-turbo';
        this.models = [
            { 
                name: 'GPT-4 Turbo', 
                value: 'gpt-4-turbo',
                icon: '<svg viewBox="0 0 24 24"><path fill="currentColor" d="M12,2C6.5,2,2,6.5,2,12s4.5,10,10,10s10-4.5,10-10S17.5,2,12,2z M12,20c-4.4,0-8-3.6-8-8s3.6-8,8-8s8,3.6,8,8S16.4,20,12,20z M12,6c-3.3,0-6,2.7-6,6s2.7,6,6,6s6-2.7,6-6S15.3,6,12,6z M12,16c-2.2,0-4-1.8-4-4s1.8-4,4-4s4,1.8,4,4S14.2,16,12,16z"></path></svg>' 
            },
            { 
                name: 'GPT-3.5 Turbo', 
                value: 'gpt-3.5-turbo',
                icon: '<svg viewBox="0 0 24 24"><path fill="currentColor" d="M12,2C6.5,2,2,6.5,2,12s4.5,10,10,10s10-4.5,10-10S17.5,2,12,2z M12,20c-4.4,0-8-3.6-8-8s3.6-8,8-8s8,3.6,8,8S16.4,20,12,20z"></path></svg>' 
            },
            { 
                name: 'Gemini Pro', 
                value: 'gemini-pro',
                icon: '<svg viewBox="0 0 24 24"><path fill="currentColor" d="M19.7,6.9c-0.5-0.7-1.2-1.3-2-1.8c-1.6-1-3.5-1.5-5.6-1.5c-2.9,0-5.6,1.1-7.6,3.1c-2,2-3.1,4.7-3.1,7.6c0,2.9,1.1,5.6,3.1,7.6c2,2,4.7,3.1,7.6,3.1c2.1,0,4.1-0.5,5.8-1.6c1.7-1,3.2-2.5,4.1-4.3c0.2-0.4,0.3-0.8,0.3-1.2c0-1.1-0.9-2-2-2c-0.5,0-0.9,0.2-1.3,0.5c-0.9,0.7-2,1.2-3.2,1.5c-1.4,0.3-2.8,0.2-4.2-0.5c-1.3-0.6-2.5-1.6-3.3-2.9c-0.8-1.2-1.2-2.7-1.2-4.2c0-1.5,0.4-3,1.2-4.2c0.8-1.2,2-2.2,3.3-2.9c1.4-0.6,2.8-0.8,4.2-0.5c1.2,0.3,2.3,0.8,3.2,1.5c0.4,0.3,0.8,0.5,1.3,0.5c1.1,0,2-0.9,2-2C20,7.7,19.9,7.3,19.7,6.9z M12,14c-1.1,0-2-0.9-2-2s0.9-2,2-2s2,0.9,2,2S13.1,14,12,14z"></path></svg>' 
            },
            { 
                name: 'Claude 3.5', 
                value: 'claude-3-5-sonnet',
                icon: '<svg viewBox="0 0 24 24"><path fill="currentColor" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-12h2v4h-2zm0 6h2v2h-2z"></path></svg>' 
            }
        ];
        this.init();
    }

    async init() {
        try {
            console.log('🚀 Starting MockMate Controller initialization...');
            
            console.log('📋 Setting up custom select...');
            await this.setupCustomSelect();
            
            console.log('🔗 Setting up event listeners...');
            await this.setupEventListeners();
            
            console.log('📡 Setting up Tauri event listeners...');
            await this.setupTauriEventListeners();
            
            console.log('📝 Updating transcription state...');
            await this.updateTranscriptionState();
            
            console.log('🎤 Initializing audio...');
            await this.initializeAudio();
            
            console.log('✅ MockMate Controller initialized successfully');
            this.showNotification('MockMate initialized successfully!', 'success');
        } catch (error) {
            console.error('❌ Failed to initialize MockMate Controller:', error);
            if (error.message && error.message.includes('Tauri not available')) {
                this.showNotification('Please run the app with "npm run dev" - not by opening HTML directly', 'warning');
            } else {
                this.showNotification('Failed to initialize application', 'error');
            }
        }
    }

    setupTimer() {
        const timerEl = document.getElementById('timer');
        const updateTime = () => {
            timerEl.textContent = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        };
        updateTime();
        setInterval(updateTime, 1000);
    }

    setupCustomSelect() {
        const selectContainer = document.getElementById('customSelectItems');
        this.models.forEach(model => {
            const item = document.createElement('div');
            item.className = 'custom-select-item';
            item.innerHTML = `<div class="model-icon">${model.icon}</div><span>${model.name}</span>`;
            item.addEventListener('click', () => {
                this.selectedModel = model.value;
                document.getElementById('selectedModelName').textContent = model.name;
                document.getElementById('selectedModelIcon').innerHTML = model.icon;
                selectContainer.style.display = 'none';
                document.getElementById('customSelect').classList.remove('open');
                console.log('Model selected:', model.name);
            });
            selectContainer.appendChild(item);
        });
    }

    async setupEventListeners() {
        const customSelect = document.getElementById('customSelect');
        const customSelectItems = document.getElementById('customSelectItems');
        const micBtn = document.getElementById('micBtn');
        const systemSoundBtn = document.getElementById('systemSoundBtn');
        const clearBtn = document.getElementById('clearBtn');
        const closeBtn = document.getElementById('closeBtn');
        const generateAnswerBtn = document.getElementById('generateAnswerBtn');
        const analyzeScreenBtn = document.getElementById('analyzeScreenBtn');
        const uploadResumeBtn = document.getElementById('uploadResumeBtn');
        const sendBtn = document.getElementById('sendBtn');
        const questionInput = document.getElementById('questionInput');

        // Check if all elements exist
        console.log('🔍 Button elements check:', {
            customSelect: !!customSelect,
            micBtn: !!micBtn,
            systemSoundBtn: !!systemSoundBtn,
            closeBtn: !!closeBtn,
            clearBtn: !!clearBtn,
            generateAnswerBtn: !!generateAnswerBtn,
            analyzeScreenBtn: !!analyzeScreenBtn,
            uploadResumeBtn: !!uploadResumeBtn,
            sendBtn: !!sendBtn,
            questionInput: !!questionInput
        });

        // Custom select dropdown
        customSelect.addEventListener('click', () => {
            const isOpen = customSelect.classList.toggle('open');
            customSelectItems.style.display = isOpen ? 'block' : 'none';
        });

        document.addEventListener('click', (e) => {
            if (!customSelect.contains(e.target)) {
                customSelect.classList.remove('open');
                customSelectItems.style.display = 'none';
            }
        });

        // Microphone button
        micBtn.addEventListener('click', async () => {
            await this.toggleMicrophone();
        });

        // System sound button
        systemSoundBtn.addEventListener('click', async () => {
            await this.toggleSystemSound();
        });

        // Clear transcription button
        clearBtn.addEventListener('click', () => {
            this.clearTranscription();
        });

        // Close application button
        closeBtn.addEventListener('click', async () => {
            await this.closeApplication();
        });

        // Generate answer button
        generateAnswerBtn.addEventListener('click', async () => {
            await this.generateAnswer();
        });

        // Analyze screen button
        analyzeScreenBtn.addEventListener('click', async () => {
            await this.analyzeScreen();
        });

        // Upload resume button
        uploadResumeBtn.addEventListener('click', async () => {
            await this.uploadResume();
        });

        // Send button and Enter key
        sendBtn.addEventListener('click', async () => {
            await this.sendManualQuestion();
        });

        questionInput.addEventListener('keypress', async (e) => {
            if (e.key === 'Enter') {
                await this.sendManualQuestion();
            }
        });
    }

    async setupTauriEventListeners() {
        try {
            // Listen for transcription events from Deepgram
            await listen('transcription-result', (event) => {
                console.log('Transcription result:', event.payload);
                this.updateTranscription(event.payload.text || event.payload);
            });

            // Listen for audio status changes
            await listen('audio-status-changed', (event) => {
                console.log('Audio status changed:', event.payload);
                this.updateAudioStatus(event.payload);
            });

            // Listen for WebSocket events
            await listen('websocket-message', (event) => {
                console.log('WebSocket message:', event.payload);
                this.handleWebSocketMessage(event.payload);
            });

            console.log('Event listeners setup successfully');
        } catch (error) {
            console.error('Failed to setup event listeners:', error);
        }
    }

    async initializeAudio() {
        try {
            const devices = await safeInvoke('get_audio_devices');
            console.log('Available audio devices:', devices);
            
            const status = await safeInvoke('check_audio_status');
            console.log('Audio status:', status);
            this.updateAudioStatus(status);
        } catch (error) {
            console.error('Failed to initialize audio:', error);
            this.showNotification('Failed to initialize audio system', 'error');
        }
    }

    async toggleMicrophone() {
        try {
            const micBtn = document.getElementById('micBtn');
            
            if (this.isMicOn) {
                // Stop microphone
                await safeInvoke('stop_audio_stream');
                await safeInvoke('stop_deepgram_transcription');
                this.isMicOn = false;
                micBtn.classList.remove('active');
                this.showNotification('Microphone stopped', 'success');
            } else {
                // Start microphone
                await safeInvoke('start_microphone_capture');
                await safeInvoke('start_deepgram_transcription');
                this.isMicOn = true;
                micBtn.classList.add('active');
                this.showNotification('Microphone started', 'success');
            }
            
            this.updateTranscriptionState();
            this.updateRecordingStatus();
        } catch (error) {
            console.error('Failed to toggle microphone:', error);
            if (error.message && error.message.includes('Tauri not ready')) {
                this.showNotification('Please wait for app to finish initializing...', 'warning');
            } else {
                this.showNotification(`Failed to ${this.isMicOn ? 'stop' : 'start'} microphone: ${error}`, 'error');
            }
        }
    }

    async toggleSystemSound() {
        try {
            const systemSoundBtn = document.getElementById('systemSoundBtn');
            
            if (this.isSystemSoundOn) {
                // Stop system sound
                await safeInvoke('stop_audio_stream');
                await safeInvoke('stop_deepgram_transcription');
                this.isSystemSoundOn = false;
                systemSoundBtn.classList.remove('active');
                this.showNotification('System sound stopped', 'success');
            } else {
                // Start system sound
                await safeInvoke('start_system_audio_capture');
                await safeInvoke('start_deepgram_transcription');
                this.isSystemSoundOn = true;
                systemSoundBtn.classList.add('active');
                this.showNotification('System sound started', 'success');
            }
            
            this.updateTranscriptionState();
            this.updateRecordingStatus();
        } catch (error) {
            console.error('Failed to toggle system sound:', error);
            if (error.message && error.message.includes('Tauri not ready')) {
                this.showNotification('Please wait for app to finish initializing...', 'warning');
            } else {
                this.showNotification(`Failed to ${this.isSystemSoundOn ? 'stop' : 'start'} system sound: ${error}`, 'error');
            }
        }
    }

    async closeApplication() {
        try {
            // Stop any active recordings first
            if (this.isMicOn || this.isSystemSoundOn) {
                await safeInvoke('stop_audio_stream');
                await safeInvoke('stop_deepgram_transcription');
            }
            
            // Close the application
            await safeInvoke('close_application');
        } catch (error) {
            console.error('Failed to close application:', error);
            // Try force close as fallback
            try {
                await safeInvoke('force_close_application');
            } catch (forceError) {
                console.error('Failed to force close application:', forceError);
            }
        }
    }

    async generateAnswer() {
        try {
            const questionInput = document.getElementById('questionInput');
            const companyInput = document.getElementById('companyInput');
            const jobDescriptionInput = document.getElementById('jobDescriptionInput');
            
            const question = questionInput.value.trim() || this.currentTranscription || 'Please provide a general interview answer';
            
            if (!question) {
                this.showNotification('No question available to generate answer', 'warning');
                return;
            }

            this.showNotification('Generating AI answer...', 'info');
            
            const payload = {
                question: question,
                model: this.selectedModel,
                company: companyInput.value.trim() || null,
                position: null,
                job_description: jobDescriptionInput.value.trim() || null
            };

            const answer = await safeInvoke('generate_ai_answer', { payload });
            
            // Display the answer in a new popup or update the transcription area
            this.displayAnswer(answer);
            this.showNotification('Answer generated successfully', 'success');
            
        } catch (error) {
            console.error('Failed to generate answer:', error);
            if (error.message && error.message.includes('Tauri not ready')) {
                this.showNotification('Please wait for app to finish initializing...', 'warning');
            } else {
                this.showNotification(`Failed to generate answer: ${error}`, 'error');
            }
        }
    }

    async analyzeScreen() {
        try {
            this.showNotification('Screen analysis feature coming soon...', 'info');
            // Placeholder for future screen analysis functionality
            // This would require screen capture capabilities
        } catch (error) {
            console.error('Failed to analyze screen:', error);
            this.showNotification(`Failed to analyze screen: ${error}`, 'error');
        }
    }

    async uploadResume() {
        try {
            const result = await safeInvoke('upload_resume');
            this.showNotification(result, 'info');
        } catch (error) {
            console.error('Failed to upload resume:', error);
            if (error.message && error.message.includes('Tauri not ready')) {
                this.showNotification('Please wait for app to finish initializing...', 'warning');
            } else {
                this.showNotification(`Failed to upload resume: ${error}`, 'error');
            }
        }
    }


    async sendManualQuestion() {
        try {
            const questionInput = document.getElementById('questionInput');
            const question = questionInput.value.trim();
            
            if (!question) {
                this.showNotification('Please enter a question', 'warning');
                return;
            }

            // For now, just generate an answer for the manual question
            const payload = {
                question: question,
                model: this.selectedModel,
                company: document.getElementById('companyInput').value.trim() || null,
                position: null,
                job_description: document.getElementById('jobDescriptionInput').value.trim() || null
            };

            this.showNotification('Processing your question...', 'info');
            const answer = await safeInvoke('generate_ai_answer', { payload });
            
            this.displayAnswer(answer);
            questionInput.value = '';
            this.showNotification('Answer generated for your question', 'success');
            
        } catch (error) {
            console.error('Failed to send manual question:', error);
            if (error.message && error.message.includes('Tauri not ready')) {
                this.showNotification('Please wait for app to finish initializing...', 'warning');
            } else {
                this.showNotification(`Failed to process question: ${error}`, 'error');
            }
        }
    }

    updateTranscriptionState() {
        const transcriptionEl = document.getElementById('transcriptionText');
        const isListening = this.isMicOn || this.isSystemSoundOn;

        if (isListening) {
            if (!this.currentTranscription) {
                transcriptionEl.textContent = 'Listening...';
                transcriptionEl.classList.add('listening');
                transcriptionEl.classList.remove('active');
            }
        } else {
            if (!this.currentTranscription) {
                transcriptionEl.textContent = 'Enable Mic or System Sound to start transcription...';
                transcriptionEl.classList.remove('active', 'listening');
            }
        }
    }

    updateTranscription(text) {
        if (text && text.trim()) {
            this.currentTranscription = text;
            const transcriptionEl = document.getElementById('transcriptionText');
            transcriptionEl.textContent = `"${text}"`;
            transcriptionEl.classList.add('active');
            transcriptionEl.classList.remove('listening');
        }
    }

    clearTranscription() {
        this.currentTranscription = '';
        const transcriptionEl = document.getElementById('transcriptionText');
        transcriptionEl.textContent = '';
        this.updateTranscriptionState();
    }

    updateRecordingStatus() {
        const recordingStatus = document.getElementById('recordingStatus');
        const recordingIndicator = document.getElementById('recordingIndicator');
        
        const isRecording = this.isMicOn || this.isSystemSoundOn;
        
        if (isRecording) {
            recordingStatus.textContent = 'Recording';
            recordingIndicator.style.display = 'block';
        } else {
            recordingStatus.textContent = 'Stopped';
            recordingIndicator.style.display = 'none';
        }
    }

    updateAudioStatus(status) {
        if (status && typeof status === 'object') {
            // Update internal state based on backend status
            if (status.is_recording !== undefined) {
                // Update recording status based on backend
                this.updateRecordingStatus();
            }
        }
    }

    displayAnswer(answer) {
        // For now, we'll show the answer in the transcription area
        // In a full implementation, this might open a modal or separate panel
        const transcriptionEl = document.getElementById('transcriptionText');
        transcriptionEl.innerHTML = `<strong>AI Answer:</strong><br>${answer}`;
        transcriptionEl.classList.add('active');
        transcriptionEl.classList.remove('listening');
    }

    handleWebSocketMessage(message) {
        console.log('Handling WebSocket message:', message);
        // Handle different types of WebSocket messages
        if (message.type === 'transcription') {
            this.updateTranscription(message.text);
        }
    }

    showNotification(message, type = 'info') {
        // Simple notification system - could be enhanced with a proper notification library
        const notification = document.createElement('div');
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            left: 20px;
            padding: 12px 20px;
            border-radius: 8px;
            color: white;
            font-size: 14px;
            z-index: 1000;
            max-width: 300px;
            word-wrap: break-word;
            animation: slideInFromLeft 0.3s ease;
        `;
        
        switch (type) {
            case 'success':
                notification.style.background = 'var(--success)';
                break;
            case 'error':
                notification.style.background = 'var(--danger)';
                break;
            case 'warning':
                notification.style.background = 'var(--warning)';
                break;
            default:
                notification.style.background = 'var(--accent)';
        }
        
        notification.textContent = message;
        document.body.appendChild(notification);
        
        setTimeout(() => {
            notification.style.animation = 'slideOut 0.3s ease forwards';
            setTimeout(() => {
                document.body.removeChild(notification);
            }, 300);
        }, 3000);
        
        console.log(`[${type.toUpperCase()}] ${message}`);
    }
}

// Add CSS for notification animations
const style = document.createElement('style');
style.textContent = `
    @keyframes slideInFromLeft {
        from { transform: translateX(-100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
    
    @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(-100%); opacity: 0; }
    }
`;
document.head.appendChild(style);

// Initialize the controller when the DOM is loaded AND Tauri is ready
document.addEventListener('DOMContentLoaded', async () => {
    console.log('DOM loaded, waiting for Tauri...');
    
    // Show loading state
    showInitializationState('loading');
    
    // Wait for Tauri to be ready
    const tauriReady = await checkTauriAvailability();
    
    if (tauriReady) {
        console.log('✅ Tauri is ready, initializing MockMate Controller...');
        showInitializationState('ready');
        
        // Only create controller after Tauri is confirmed ready
        new MockMateController();
    } else {
        console.log('⚠️ Tauri not available, initializing with limited functionality...');
        showInitializationState('error');
        
        // Still create controller for UI functionality, but backend calls will be handled gracefully
        new MockMateController();
    }
});

// Show initialization state to user
function showInitializationState(state) {
    const transcriptionEl = document.getElementById('transcriptionText');
    const statusPill = document.querySelector('.status-pill');
    
    switch (state) {
        case 'loading':
            transcriptionEl.textContent = 'Initializing MockMate... Please wait.';
            transcriptionEl.className = 'transcription-text listening';
            statusPill.textContent = 'Initializing...';
            statusPill.style.background = 'rgba(255, 165, 2, 0.15)';
            statusPill.style.color = 'var(--warning)';
            statusPill.style.borderColor = 'rgba(255, 165, 2, 0.2)';
            break;
            
        case 'ready':
            transcriptionEl.textContent = 'Enable Mic or System Sound to start transcription...';
            transcriptionEl.className = 'transcription-text';
            statusPill.textContent = 'Live';
            statusPill.style.background = 'rgba(0, 200, 150, 0.15)';
            statusPill.style.color = 'var(--success)';
            statusPill.style.borderColor = 'rgba(0, 200, 150, 0.2)';
            break;
            
        case 'error':
            transcriptionEl.textContent = 'Please run with "npm run dev" - HTML opened directly in browser';
            transcriptionEl.className = 'transcription-text';
            transcriptionEl.style.color = 'var(--warning)';
            statusPill.textContent = 'Error';
            statusPill.style.background = 'rgba(255, 71, 87, 0.15)';
            statusPill.style.color = 'var(--danger)';
            statusPill.style.borderColor = 'rgba(255, 71, 87, 0.2)';
            break;
    }
}

// Global error handler
window.addEventListener('error', (event) => {
    console.error('Global error:', event.error);
});

window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason);
});
