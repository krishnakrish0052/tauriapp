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
        this.fullTranscription = '';  // Cumulative transcription text
        this.interimTranscription = ''; // Current interim text
        this.selectedModel = 'gpt-4-turbo';
        this.aiResponseWindow = null;
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
                const payload = event.payload;
                if (payload.text && payload.text.trim()) {
                    this.updateTranscription(payload.text, payload.is_final);
                    if (payload.confidence) {
                        console.log(`Confidence: ${(payload.confidence * 100).toFixed(1)}%`);
                    }
                }
            });

            // Listen for transcription status updates
            await listen('transcription-status', (event) => {
                console.log('Transcription status:', event.payload);
                const payload = event.payload;
                if (payload.status === 'connected') {
                    this.showNotification(`Deepgram connected (ID: ${payload.request_id})`, 'success');
                } else if (payload.status === 'disconnected') {
                    this.showNotification('Deepgram disconnected', 'info');
                }
            });

            // Listen for transcription errors
            await listen('transcription-error', (event) => {
                console.error('Transcription error:', event.payload);
                this.showNotification(`Transcription error: ${event.payload.error}`, 'error');
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
                // Stop microphone transcription
                await safeInvoke('stop_transcription');
                this.isMicOn = false;
                micBtn.classList.remove('active');
                this.showNotification('Microphone stopped', 'success');
            } else {
                // Start microphone transcription
                await safeInvoke('start_microphone_transcription');
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
                // Stop system audio transcription
                await safeInvoke('stop_transcription');
                this.isSystemSoundOn = false;
                systemSoundBtn.classList.remove('active');
                this.showNotification('System sound stopped', 'success');
            } else {
                // Start system audio transcription
                await safeInvoke('start_system_audio_transcription');
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
                await safeInvoke('stop_transcription');
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
            
            const question = questionInput.value.trim() || this.fullTranscription || 'Please provide a general interview answer';
            
            if (!question) {
                this.showNotification('No question available to generate answer', 'warning');
                return;
            }

            this.showNotification('Generating AI answer...', 'info');
            
            // Show the AI response window (it was created at startup)
            try {
                await safeInvoke('show_ai_response_window');
                console.log('AI response window shown successfully');
            } catch (windowError) {
                console.error('Failed to show AI response window:', windowError);
                // Continue with fallback - show in main window or notification
                this.showNotification('Using fallback display for AI response', 'warning');
            }
            
            const payload = {
                question: question,
                model: this.selectedModel,
                company: companyInput.value.trim() || null,
                position: null,
                job_description: jobDescriptionInput.value.trim() || null
            };

            const answer = await safeInvoke('generate_ai_answer', { payload });
            
            // Send the answer to the AI response window using streaming
            await this.sendToAiWindow('stream', answer);
            this.showNotification('Answer generated successfully', 'success');
            
        } catch (error) {
            console.error('Failed to generate answer:', error);
            // Send error to AI window if it exists
            await this.sendToAiWindow('error', error.message || error.toString());
            
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

    updateTranscription(text, isFinal = false) {
        if (text && text.trim()) {
            const transcriptionEl = document.getElementById('transcriptionText');
            
            if (isFinal) {
                // Final transcription - append to full transcription
                if (this.fullTranscription) {
                    this.fullTranscription += ' ' + text;
                } else {
                    this.fullTranscription = text;
                }
                this.currentTranscription = this.fullTranscription;
                this.interimTranscription = '';
                
                // Show full transcription
                transcriptionEl.textContent = this.fullTranscription;
                transcriptionEl.classList.add('active');
                transcriptionEl.classList.remove('listening', 'interim');
            } else {
                // Interim transcription - show with different styling
                this.interimTranscription = text;
                const displayText = this.fullTranscription + 
                    (this.fullTranscription ? ' ' : '') + 
                    `${text}...`;
                
                transcriptionEl.textContent = displayText;
                transcriptionEl.classList.add('interim');
                transcriptionEl.classList.remove('listening');
            }
        }
    }

    clearTranscription() {
        this.currentTranscription = '';
        this.fullTranscription = '';
        this.interimTranscription = '';
        const transcriptionEl = document.getElementById('transcriptionText');
        transcriptionEl.textContent = '';
        this.updateTranscriptionState();
        // Also close AI response window if open
        if (this.aiResponseWindow) {
            this.aiResponseWindow.remove();
            this.aiResponseWindow = null;
        }
    }

    updateRecordingStatus() {
        const recordingStatus = document.getElementById('recordingStatus');
        const recordingIndicator = document.getElementById('recordingIndicator');
        
        const isRecording = this.isMicOn || this.isSystemSoundOn;
        
        // Only update if elements exist (they don't exist in current HTML)
        if (recordingStatus) {
            recordingStatus.textContent = isRecording ? 'Recording' : 'Stopped';
        }
        
        if (recordingIndicator) {
            recordingIndicator.style.display = isRecording ? 'block' : 'none';
        }
        
        // For now, we'll just log the recording status since the UI elements don't exist
        console.log(`🎤 Recording status: ${isRecording ? 'Recording' : 'Stopped'}`);
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

    // Create AI Response Window
    createAiResponseWindow() {
        // Remove existing window if any
        if (this.aiResponseWindow) {
            this.aiResponseWindow.remove();
        }

        const mainWindow = document.querySelector('.main-window');
        const mainWindowRect = mainWindow.getBoundingClientRect();
        
        this.aiResponseWindow = document.createElement('div');
        this.aiResponseWindow.className = 'ai-response-window';
        this.aiResponseWindow.style.cssText = `
            position: absolute;
            top: ${mainWindowRect.bottom + 5}px;
            left: ${mainWindowRect.left}px;
            width: ${mainWindowRect.width}px;
            min-height: 100px;
            max-height: 400px;
            background: rgba(0, 0, 0, 0.85);
            border-radius: 16px;
            border: 1px solid rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(20px);
            -webkit-backdrop-filter: blur(20px);
            overflow: hidden;
            z-index: 1000;
            display: flex;
            flex-direction: column;
            animation: slideInFromBottom 0.3s ease;
        `;
        
        const header = document.createElement('div');
        header.className = 'ai-response-header';
        header.style.cssText = `
            padding: 12px 16px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            background: linear-gradient(145deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.02));
            display: flex;
            align-items: center;
            justify-content: space-between;
        `;
        
        const title = document.createElement('div');
        title.style.cssText = `
            color: var(--text-primary);
            font-weight: 600;
            font-size: 14px;
            display: flex;
            align-items: center;
            gap: 8px;
        `;
        title.innerHTML = `
            <span class="material-icons" style="font-size: 18px; color: var(--accent);">auto_awesome</span>
            AI Assistant Response
        `;
        
        const closeBtn = document.createElement('button');
        closeBtn.style.cssText = `
            background: none;
            border: none;
            color: var(--text-secondary);
            cursor: pointer;
            padding: 4px;
            border-radius: 4px;
            transition: all 0.2s ease;
        `;
        closeBtn.innerHTML = '<span class="material-icons" style="font-size: 18px;">close</span>';
        closeBtn.onclick = () => {
            this.aiResponseWindow.remove();
            this.aiResponseWindow = null;
        };
        closeBtn.onmouseover = () => closeBtn.style.background = 'rgba(255, 255, 255, 0.1)';
        closeBtn.onmouseout = () => closeBtn.style.background = 'none';
        
        header.appendChild(title);
        header.appendChild(closeBtn);
        
        const content = document.createElement('div');
        content.className = 'ai-response-content';
        content.style.cssText = `
            padding: 16px;
            color: var(--text-primary);
            line-height: 1.6;
            font-size: 14px;
            overflow-y: auto;
            flex: 1;
            max-height: 320px;
        `;
        
        this.aiResponseWindow.appendChild(header);
        this.aiResponseWindow.appendChild(content);
        document.body.appendChild(this.aiResponseWindow);
        
        return content;
    }
    
    // Show streaming response in AI window
    showStreamingResponse(text) {
        if (!this.aiResponseWindow) {
            this.createAiResponseWindow();
        }
        
        const content = this.aiResponseWindow.querySelector('.ai-response-content');
        content.innerHTML = `
            <div style="display: flex; align-items: center; gap: 8px; color: var(--accent);">
                <div class="streaming-indicator"></div>
                ${text}
            </div>
        `;
    }
    
    // Stream text to AI response window with typing effect
    streamTextToWindow(text) {
        if (!this.aiResponseWindow) {
            this.createAiResponseWindow();
        }
        
        const content = this.aiResponseWindow.querySelector('.ai-response-content');
        content.innerHTML = '';
        
        let index = 0;
        const streamingSpeed = 30; // milliseconds per character
        
        const typeText = () => {
            if (index < text.length) {
                content.innerHTML = text.substring(0, index + 1) + '<span class="cursor">|</span>';
                index++;
                setTimeout(typeText, streamingSpeed);
                
                // Auto-resize window based on content
                this.adjustAiWindowHeight();
            } else {
                // Remove cursor when done
                content.innerHTML = text;
            }
        };
        
        typeText();
    }
    
    // Adjust AI window height based on content
    adjustAiWindowHeight() {
        if (!this.aiResponseWindow) return;
        
        const content = this.aiResponseWindow.querySelector('.ai-response-content');
        const contentHeight = content.scrollHeight;
        const maxHeight = 400;
        const minHeight = 100;
        
        const newHeight = Math.min(Math.max(contentHeight + 80, minHeight), maxHeight); // +80 for header
        this.aiResponseWindow.style.height = newHeight + 'px';
    }
    
    // Send data to AI response window (for Tauri-based window communication)
    async sendToAiWindow(type, data) {
        try {
            // Use the new Tauri command to send data to AI response window
            const aiResponseData = {
                message_type: type,
                text: typeof data === 'string' ? data : data?.text || null,
                error: typeof data === 'string' && type === 'error' ? data : data?.error || null
            };
            
            console.log('Sending to AI window:', aiResponseData);
            
            await safeInvoke('send_ai_response_data', { data: aiResponseData });
            console.log('Successfully sent data to AI response window');
            
        } catch (error) {
            console.error('Failed to send data to AI window:', error);
            // Fallback: display in main window
            if (type === 'stream' || type === 'complete') {
                const text = typeof data === 'string' ? data : data?.text;
                if (text) {
                    this.displayAnswer(text);
                }
            } else if (type === 'error') {
                const errorMsg = typeof data === 'string' ? data : data?.error || 'Unknown error';
                this.showNotification(`AI Error: ${errorMsg}`, 'error');
            }
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

// Add CSS for notification animations and AI response window
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
    
    @keyframes slideInFromBottom {
        from { transform: translateY(20px); opacity: 0; }
        to { transform: translateY(0); opacity: 1; }
    }
    
    .streaming-indicator {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: var(--accent);
        animation: pulse 1.5s infinite;
        box-shadow: 0 0 4px var(--accent);
    }
    
    .cursor {
        color: var(--accent);
        animation: blink 1s infinite;
    }
    
    @keyframes blink {
        0%, 50% { opacity: 1; }
        51%, 100% { opacity: 0; }
    }
    
    .ai-response-window::-webkit-scrollbar {
        width: 6px;
    }
    
    .ai-response-window::-webkit-scrollbar-track {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 3px;
    }
    
    .ai-response-window::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.3);
        border-radius: 3px;
    }
    
    .ai-response-window::-webkit-scrollbar-thumb:hover {
        background: rgba(255, 255, 255, 0.5);
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
