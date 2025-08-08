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
        this.selectedModel = 'llama-3.1-70b-instruct';
        this.selectedProvider = 'openai';
        this.aiResponseWindow = null;
        this.models = [];
        this.providers = [];
        this.allModels = []; // All available models from backend
        this.streamingText = ''; // Accumulated streaming text
        this.isStreaming = false; // Track streaming state
        this.heightAdjustmentTimeout = null; // Throttling for height adjustments
        this.aiWindowResizeObserver = null; // ResizeObserver for AI content
        this.aiWindowSizePoll = null; // Fallback polling handle
        this.init();
    }

    async init() {
        try {
            console.log('🚀 Starting MockMate Controller initialization...');
            
            console.log('📊 Loading AI providers and models...');
            await this.loadAIProvidersAndModels();
            
            console.log('⚙️ Setting up provider switch...');
            await this.setupProviderSwitch();
            
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

    async loadAIProvidersAndModels() {
        try {
            // Always load providers and models via backend so headers/env are respected
            this.providers = await safeInvoke('get_ai_providers');
            console.log('✅ Loaded AI providers:', this.providers);

            this.allModels = await safeInvoke('get_available_models');
            console.log('✅ Loaded AI models:', this.allModels);
            // Normalize legacy value field to id if needed
            this.allModels = this.allModels.map(m => ({
                id: m.id || m.value,
                name: m.name,
                provider: m.provider,
                icon: m.icon || ''
            }));
            console.log('🔁 Normalized models:', this.allModels);

            // If providers include Pollinations, default to it; otherwise keep current
            if (this.providers.some(p => p.id === 'pollinations')) {
                this.selectedProvider = 'pollinations';
            }

            // Set default models based on selected provider
            this.updateModelsForProvider();
        } catch (err) {
            console.error('❌ Failed to load AI providers/models from backend:', err);
            // Fallback to hardcoded models
            this.providers = [
                { id: 'openai', name: 'OpenAI' },
                { id: 'pollinations', name: 'Self AI' }
            ];
            this.allModels = [
                { name: 'GPT-4 Turbo', value: 'gpt-4-turbo', provider: 'openai', icon: '🤖' },
                { name: 'GPT-3.5 Turbo', value: 'gpt-3.5-turbo', provider: 'openai', icon: '🤖' }
            ];
            this.updateModelsForProvider();
            this.showNotification('Using fallback models - backend failed to load models', 'warning');
        }
    }

    setupProviderSwitch() {
        // Find the provider switch container in the HTML
        const providerContainer = document.querySelector('.provider-switch');
        if (!providerContainer) {
            console.warn('Provider switch container not found in HTML');
            return;
        }

        // Create provider switch buttons
        this.providers.forEach(provider => {
            const button = document.createElement('button');
            button.className = `provider-btn ${provider.id === this.selectedProvider ? 'active' : ''}`;
            button.textContent = provider.name;
            button.addEventListener('click', () => {
                this.switchProvider(provider.id);
            });
            providerContainer.appendChild(button);
        });
    }

    switchProvider(providerId) {
        if (this.selectedProvider === providerId) return;
        
        console.log(`🔄 Switching AI provider from ${this.selectedProvider} to ${providerId}`);
        
        // Update selected provider
        this.selectedProvider = providerId;
        
        // Update provider switch UI
        const providerButtons = document.querySelectorAll('.provider-btn');
        providerButtons.forEach(btn => {
            btn.classList.remove('active');
            if (btn.textContent.toLowerCase().includes(providerId) || 
                (providerId === 'pollinations' && btn.textContent === 'Self AI')) {
                btn.classList.add('active');
            }
        });
        
        // Update available models for the new provider
        this.updateModelsForProvider();
        
        // Update model dropdown
        this.rebuildModelDropdown();
        
        this.showNotification(`Switched to ${this.getProviderDisplayName(providerId)}`, 'success');
    }

    updateModelsForProvider() {
        // Filter models based on selected provider
        this.models = this.allModels.filter(model => model.provider === this.selectedProvider);
        
        // Set default model for the provider if current model doesn't belong to this provider
        const currentModelBelongsToProvider = this.models.some(model => (model.id || model.value) === this.selectedModel);
        if (!currentModelBelongsToProvider && this.models.length > 0) {
            // Try to find "Llama Fast" first, otherwise use first model
            const llamaModel = this.models.find(m => 
                m.name.toLowerCase().includes('llama') && 
                (m.name.toLowerCase().includes('fast') || m.name.toLowerCase().includes('roblox'))
            );
            if (llamaModel) {
                this.selectedModel = llamaModel.id || llamaModel.value;
                console.log(`🎯 Set default model to Llama Fast: ${this.selectedModel}`);
            } else {
                // Fallback to first model
                this.selectedModel = this.models[0].id || this.models[0].value;
                console.log(`🎯 Set fallback default model for ${this.selectedProvider}: ${this.selectedModel}`);
            }
        }
        
        console.log(`📋 Updated models for ${this.selectedProvider}:`, this.models.map(m => m.name));
    }

    rebuildModelDropdown() {
        const selectContainer = document.getElementById('customSelectItems');
        if (!selectContainer) return;
        
        // Clear existing items
        selectContainer.innerHTML = '';
        // Add models for current provider
        this.models.forEach(model => {
            const item = document.createElement('div');
            item.className = 'custom-select-item';
            item.innerHTML = `<span>${model.name}</span>`;
            item.addEventListener('click', () => {
                this.selectedModel = model.id || model.value;
                document.getElementById('selectedModelName').textContent = model.name;
                selectContainer.style.display = 'none';
                document.getElementById('customSelect').classList.remove('open');
                console.log('Model selected:', model.name, '(', this.selectedModel, ')');
            });
            selectContainer.appendChild(item);
        });
        
        // Update selected model display
        if (this.models.length > 0) {
            const selectedModelInfo = this.models.find(m => (m.id || m.value) === this.selectedModel) || this.models[0];
            document.getElementById('selectedModelName').textContent = selectedModelInfo.name;
        }
    }

    getProviderDisplayName(providerId) {
        const provider = this.providers.find(p => p.id === providerId);
        return provider ? provider.name : providerId;
    }

    setupCustomSelect() {
        const selectContainer = document.getElementById('customSelectItems');
        if (!selectContainer) {
            console.warn('Custom select container not found');
            return;
        }
        
        // Clear existing items first
        selectContainer.innerHTML = '';
        
        // Add models for current provider
        this.models.forEach(model => {
            const item = document.createElement('div');
            item.className = 'custom-select-item';
            item.innerHTML = `<span>${model.name}</span>`;
            item.addEventListener('click', () => {
                this.selectedModel = model.id || model.value;
                document.getElementById('selectedModelName').textContent = model.name;
                selectContainer.style.display = 'none';
                document.getElementById('customSelect').classList.remove('open');
                console.log('Model selected:', model.name, '(', this.selectedModel, ')');
            });
            selectContainer.appendChild(item);
        });
        
        // Update selected model display to show default model (prefer Llama Fast)
        if (this.models.length > 0) {
            // Try to find and select "Llama Fast Roblox" by default if no specific model selected yet
            const llamaModel = this.models.find(m => 
                m.name.toLowerCase().includes('llama') && 
                (m.name.toLowerCase().includes('fast') || m.name.toLowerCase().includes('roblox'))
            );
            
            let selectedModelInfo;
            if (llamaModel && (!this.selectedModel || this.selectedModel === 'gpt-4-turbo')) {
                // Use Llama Fast if available and no specific model selected yet
                this.selectedModel = llamaModel.id || llamaModel.value;
                selectedModelInfo = llamaModel;
                console.log('🦙 Auto-selected Llama Fast Roblox as default model');
            } else {
                // Use currently selected model or fallback to first
                selectedModelInfo = this.models.find(m => (m.id || m.value) === this.selectedModel) || this.models[0];
            }
            
            const selectedModelNameEl = document.getElementById('selectedModelName');
            if (selectedModelNameEl && selectedModelInfo) {
                selectedModelNameEl.textContent = selectedModelInfo.name;
            }
        }
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
            // Ensure dropdown appears above and is not clipped
            customSelect.style.position = 'relative';
            customSelect.style.zIndex = 10000;
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

    ensureValidModelSelection() {
        // Ensure selected provider and model are valid based on fetched data
        const validModels = this.allModels.filter(m => m.provider === this.selectedProvider);
        if (!validModels.length) {
            console.warn(`No models available for provider: ${this.selectedProvider}`);
            return; // nothing to validate
        }

        const selectedId = this.selectedModel;
        const isValid = validModels.some(m => (m.id || m.value) === selectedId);
        if (!isValid) {
            // Try to find "Llama Fast" first, otherwise use first valid model
            const llamaModel = validModels.find(m => 
                m.name.toLowerCase().includes('llama') && 
                (m.name.toLowerCase().includes('fast') || m.name.toLowerCase().includes('roblox'))
            );
            const fallback = llamaModel || validModels[0];
            
            this.selectedModel = fallback.id || fallback.value;
            // Reflect in UI if elements exist
            const nameEl = document.getElementById('selectedModelName');
            if (nameEl) nameEl.textContent = fallback.name;
            // Also rebuild dropdown to reflect provider state
            this.models = validModels;
            this.rebuildModelDropdown();
            this.showNotification(`Model not available. Switched to ${fallback.name}`, 'info');
            console.log('ensureValidModelSelection applied fallback model:', this.selectedModel);
        }
    }

    async showResponseWindow() {
        // Always ensure the in-UI response window is visible
        this.ensureUiResponseWindow();
        
        // Also try native window if available
        try {
            await safeInvoke('show_ai_response_window');
        } catch (_) {
            // Native window not available, using in-UI window only
        }
    }

    async pollinationsStreamAnswer(prompt, options = {}) {
        // Build Pollinations URL with streaming
        const base = 'https://text.pollinations.ai';
        const params = new URLSearchParams();
        params.set('prompt', prompt);
        params.set('model', this.selectedModel); // Use the actual model ID without encoding in URL path
        if (options.seed !== undefined) params.set('seed', String(options.seed));
        if (options.temperature !== undefined) params.set('temperature', String(options.temperature));
        if (options.top_p !== undefined) params.set('top_p', String(options.top_p));
        if (options.presence_penalty !== undefined) params.set('presence_penalty', String(options.presence_penalty));
        if (options.frequency_penalty !== undefined) params.set('frequency_penalty', String(options.frequency_penalty));
        if (options.system) params.set('system', options.system);
        params.set('stream', 'true');
        params.set('private', 'true');
        params.set('referrer', 'mockmate-desktop');

        // Use the correct URL format - model goes in params, not path
        const url = `${base}/?${params.toString()}`;
        console.log('Polling Pollinations (stream):', url);

        // Ensure response window visible
        await this.showResponseWindow();
        const contentEl = this.ensureUiResponseWindow();
        contentEl.textContent = '';

        const res = await fetch(url, { method: 'GET' });
        if (!res.ok || !res.body) {
            throw new Error(`Pollinations request failed: ${res.status}`);
        }

        const reader = res.body.getReader();
        const decoder = new TextDecoder();
        let accumulated = '';

        try {
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;

                const chunk = decoder.decode(value, { stream: true });

                // If the API emits SSE (data: ...), parse it; otherwise treat as plain text
                if (chunk.includes('data:')) {
                    const lines = chunk.split('\n');
                    for (const line of lines) {
                        const trimmed = line.trim();
                        if (!trimmed) continue;
                        if (!trimmed.startsWith('data:')) continue;
                        const data = trimmed.replace(/^data:\s?/, '');
                        if (data === '[DONE]') {
                            console.log('Stream completed with [DONE] marker');
                            break;
                        }
                        try {
                            const obj = JSON.parse(data);
                            let textPiece = '';
                            const choice = obj.choices && obj.choices[0];
                            if (choice && choice.delta && typeof choice.delta.content === 'string') {
                                textPiece = choice.delta.content;
                            } else if (typeof obj.text === 'string') {
                                textPiece = obj.text;
                            } else if (typeof obj.content === 'string') {
                                textPiece = obj.content;
                            } else if (choice && typeof choice.text === 'string') {
                                textPiece = choice.text;
                            }
                            if (textPiece) {
                                accumulated += textPiece;
                                await this.sendToAiWindow('stream', accumulated);
                            }
                        } catch {
                            if (data && data !== 'null') {
                                accumulated += data;
                                await this.sendToAiWindow('stream', accumulated);
                            }
                        }
                    }
                } else {
                    // Plain text streaming (most likely for Pollinations text endpoint)
                    accumulated += chunk;
                    await this.sendToAiWindow('stream', accumulated);
                }
            }
        } catch (streamError) {
            console.error('Streaming error:', streamError);
            throw new Error(`Streaming failed: ${streamError.message}`);
        }

        // Final check: if we have accumulated content, mark as complete
        if (accumulated.trim()) {
            console.log('✅ Stream completed successfully, total characters:', accumulated.length);
            await this.sendToAiWindow('complete', accumulated);
        } else {
            console.warn('⚠️ Stream completed but no content accumulated');
            const fallbackMessage = 'Response received but content was empty. This might be a model compatibility issue.';
            await this.sendToAiWindow('complete', fallbackMessage);
        }
        return accumulated;
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

            // Clear the transcription immediately when Generate button is clicked
            // This prevents sending the same text again on next Generate button click
            if (this.fullTranscription) {
                this.clearTranscription();
            }

            this.ensureValidModelSelection();
            this.showNotification('Generating AI answer...', 'info');
            
            // ALWAYS create/show AI response window in initial state when Generate is clicked
            this.createAiResponseWindowInInitialState();
            await this.showResponseWindow();
            
            // Build a system prompt to steer interviewer-style answers
            const systemPrompt = `You are an expert interview assistant. Provide concise, accurate, real-world interview answers. ` +
                `Use the given company and job description context. Avoid irrelevant details. Use bullet points when helpful.`;

            const payload = {
                question: question,
                model: this.selectedModel,
                provider: this.selectedProvider,
                company: companyInput.value.trim() || null,
                position: null,
                job_description: jobDescriptionInput.value.trim() || null,
                system_prompt: systemPrompt
            };
            
            // Log the payload being sent to AI
            console.log('🚀 AI Request Payload:', JSON.stringify(payload, null, 2));
            if (this.selectedProvider === 'pollinations') {
                // Use backend streaming for better UX (recommended for Pollinations)
                try {
                    console.log('🚀 Starting Pollinations streaming response...');
                    // Reset streaming state for new request
                    this.streamingText = '';
                    this.isStreaming = true;
                    
                    const answer = await safeInvoke('pollinations_generate_answer_streaming', { payload: payload });
                    console.log('✅ Pollinations streaming completed with final answer:', answer);
                    // Mark as complete with the final answer
                    await this.sendToAiWindow('complete', answer);
                } catch (streamError) {
                    console.warn('⚠️ Streaming failed, falling back to non-streaming:', streamError);
                    const answer = await safeInvoke('pollinations_generate_answer', { payload: payload });
                    await this.sendToAiWindow('complete', answer);
                }
            } else {
                const answer = await safeInvoke('generate_ai_answer', payload);
                await this.sendToAiWindow('complete', answer);
            }

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

            // Validate selection before sending
            this.ensureValidModelSelection();
            
            // ALWAYS create/show AI response window in initial state when sending manual question
            this.createAiResponseWindowInInitialState();
            await this.showResponseWindow();

            // Build system prompt
            const systemPrompt = `You are an expert interview assistant. Provide concise, accurate answers. Avoid irrelevant details.`;

            this.showNotification('Processing your question...', 'info');

            const payload = {
                question: question,
                model: this.selectedModel,
                provider: this.selectedProvider,
                company: document.getElementById('companyInput').value.trim() || null,
                position: null,
                job_description: document.getElementById('jobDescriptionInput').value.trim() || null,
                system_prompt: systemPrompt
            };
            
            // Log the payload being sent to AI for manual question
            console.log('💬 Manual Question AI Request Payload:', JSON.stringify(payload, null, 2));
            if (this.selectedProvider === 'pollinations') {
                // Use backend streaming for better UX (recommended for Pollinations)
                try {
                    console.log('🚀 Starting Pollinations streaming for manual question...');
                    // Reset streaming state for new request
                    this.streamingText = '';
                    this.isStreaming = true;
                    
                    const answer = await safeInvoke('pollinations_generate_answer_streaming', { payload: payload });
                    console.log('✅ Pollinations manual question streaming completed with final answer:', answer);
                    // Mark as complete with the final answer
                    await this.sendToAiWindow('complete', answer);
                } catch (streamError) {
                    console.warn('⚠️ Manual question streaming failed, falling back to non-streaming:', streamError);
                    const answer = await safeInvoke('pollinations_generate_answer', { payload: payload });
                    await this.sendToAiWindow('complete', answer);
                }
            } else {
                const answer = await safeInvoke('generate_ai_answer', payload);
                await this.sendToAiWindow('complete', answer);
            }
            
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
            const transcriptionArea = transcriptionEl.parentElement; // Get the scrollable container
            
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
            
            // Auto-scroll to show the latest text (scroll to the right)
            if (transcriptionArea) {
                // Use requestAnimationFrame for smooth scrolling
                requestAnimationFrame(() => {
                    transcriptionArea.scrollLeft = transcriptionArea.scrollWidth;
                });
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
        // DON'T automatically close AI response window on clear transcription
        // Let user manually close it if they want to
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

    // Deprecated: Avoid using transcription area to show AI answers
    displayAnswer(answer) {
        const contentEl = this.ensureUiResponseWindow();
        contentEl.textContent = answer;
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
        
        // Calculate position to keep window 400px from screen bottom
        const screenHeight = window.screen.availHeight || window.screen.height || 1080;
        const maxAllowedHeight = 400; // Maximum height from screen bottom
        const windowTop = Math.max(mainWindowRect.bottom + 5, screenHeight - maxAllowedHeight);
        
        this.aiResponseWindow.style.cssText = `
            position: absolute;
            top: ${windowTop}px;
            left: ${mainWindowRect.left}px;
            width: ${mainWindowRect.width}px;
            height: 150px;
            min-height: 100px;
            max-height: ${maxAllowedHeight}px;
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
            box-sizing: border-box;
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
            display: none;
        `;
        closeBtn.innerHTML = '<span class="material-icons" style="font-size: 18px;">close</span>';
        closeBtn.onclick = async () => {
            console.log('🗑️ Closing AI response window completely');
            
            // Close and clean up the in-UI window completely
            this.closeAiResponseWindowCompletely();
            
            // Also try to close native Tauri window if it exists
            try {
                await safeInvoke('close_ai_response_window');
            } catch (error) {
                console.log('Native window close failed (expected if not using native window):', error);
            }
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
            font-size: 7px;
            overflow-y: auto;
            flex: 1;
            max-height: none; // Will be set dynamically
        `;
        
        this.aiResponseWindow.appendChild(header);
        this.aiResponseWindow.appendChild(content);
        document.body.appendChild(this.aiResponseWindow);
        
        // Attach ResizeObserver to auto-adjust height on content changes
        try {
            if (this.aiWindowResizeObserver) {
                try { this.aiWindowResizeObserver.disconnect(); } catch {}
                this.aiWindowResizeObserver = null;
            }
            if ('ResizeObserver' in window) {
                this.aiWindowResizeObserver = new ResizeObserver(() => {
                    this.throttledAdjustHeight();
                });
                this.aiWindowResizeObserver.observe(content);
            } else {
                // Fallback polling every 200ms if ResizeObserver is unavailable
                let lastHeight = content.scrollHeight;
                if (this.aiWindowSizePoll) {
                    clearInterval(this.aiWindowSizePoll);
                }
                this.aiWindowSizePoll = setInterval(() => {
                    const h = content.scrollHeight;
                    if (h !== lastHeight) {
                        lastHeight = h;
                        this.throttledAdjustHeight();
                    }
                }, 200);
            }
        } catch (e) {
            console.warn('Failed to setup ResizeObserver for AI window:', e);
        }
        
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
        if (!content) return;
        
        console.log('📊 Adjusting AI window height:', {
            textLength: content.textContent ? content.textContent.length : 0,
            scrollHeight: content.scrollHeight,
            offsetHeight: content.offsetHeight
        });
        
        // Calculate appropriate height based on content
        const contentHeight = content.scrollHeight;
        const headerHeight = 45; // Header height in pixels
        const padding = 20; // Extra padding
        const maxHeight = Math.floor((window.screen.availHeight || 1080) * 0.8);
        const minHeight = 100;
        
        // Limit max height to 400px from screen bottom
        const screenHeight = window.screen.availHeight || window.screen.height || 1080;
        const maxAllowedHeight = 400;
        
        // Calculate new window height with 400px limit
        let newHeight = Math.min(Math.max(contentHeight + headerHeight + padding, minHeight), maxAllowedHeight);
        
        // Update the window height
        this.aiResponseWindow.style.height = `${newHeight}px`;
        
        // Ensure content area can scroll if needed
        content.style.height = 'auto';
        content.style.maxHeight = `${newHeight - headerHeight - 32}px`; // Account for padding
        content.style.overflowY = contentHeight > (newHeight - headerHeight - 32) ? 'auto' : 'hidden';
        
        console.log(`🔧 Window height adjusted to: ${newHeight}px (content: ${contentHeight}px)`);
    }
    
    // Reset AI window to initial small height
    resetAiWindowHeight() {
        if (!this.aiResponseWindow) return;
        
        console.log('🔄 Resetting AI window height to initial size');
        
        const initialHeight = 150;
        const maxAllowedHeight = 400; // Limit to 400px from screen bottom
        const actualHeight = Math.min(initialHeight, maxAllowedHeight);
        
        this.aiResponseWindow.style.height = `${actualHeight}px`;
        
        const content = this.aiResponseWindow.querySelector('.ai-response-content');
        if (content) {
            content.style.height = 'auto';
            content.style.maxHeight = `${actualHeight - 45 - 32}px`; // Header + padding
            content.style.overflowY = 'hidden';
        }
    }
    
    // Throttled version of height adjustment for streaming content
    throttledAdjustHeight() {
        // Clear any existing timeout
        if (this.heightAdjustmentTimeout) {
            clearTimeout(this.heightAdjustmentTimeout);
        }
        
        // Set a new timeout to throttle the height adjustments
        this.heightAdjustmentTimeout = setTimeout(() => {
            this.adjustAiWindowHeight();
            this.heightAdjustmentTimeout = null;
        }, 150); // Throttle to every 150ms during streaming
    }
    
    // Ensure an in-UI AI response window exists (fallback when native window not available)
    ensureUiResponseWindow() {
        if (!this.aiResponseWindow) {
            console.log('🪟 Creating new AI response window via ensureUiResponseWindow');
            this.createAiResponseWindow();
        } else {
            console.log('✅ AI response window already exists');
        }
        return this.aiResponseWindow.querySelector('.ai-response-content');
    }

    // Create AI response window in clean initial state
    createAiResponseWindowInInitialState() {
        console.log('🪟 Creating AI response window in initial state...');
        
        // Completely clean up any existing window first
        this.closeAiResponseWindowCompletely();
        
        // Create fresh window
        const contentEl = this.createAiResponseWindow();
        
        // Set initial content
        contentEl.textContent = 'Generating response...';
        contentEl.style.fontStyle = 'italic';
        contentEl.style.opacity = '0.7';
        contentEl.style.color = 'var(--text-primary)';
        
        // Reset to initial height
        this.resetAiWindowHeight();
        
        console.log('✅ AI response window created in initial state');
        return contentEl;
    }

    // Completely close and clean up AI response window
    closeAiResponseWindowCompletely() {
        console.log('🗑️ Completely closing and cleaning up AI response window...');
        
        // Stop any streaming
        this.isStreaming = false;
        this.streamingText = '';
        
        // Clean up observers and timers
        if (this.aiWindowResizeObserver) {
            try {
                this.aiWindowResizeObserver.disconnect();
            } catch (e) {
                console.warn('Failed to disconnect ResizeObserver:', e);
            }
            this.aiWindowResizeObserver = null;
        }
        
        if (this.aiWindowSizePoll) {
            clearInterval(this.aiWindowSizePoll);
            this.aiWindowSizePoll = null;
        }
        
        if (this.heightAdjustmentTimeout) {
            clearTimeout(this.heightAdjustmentTimeout);
            this.heightAdjustmentTimeout = null;
        }
        
        // Remove DOM element completely
        if (this.aiResponseWindow) {
            this.aiResponseWindow.remove();
            this.aiResponseWindow = null;
        }
        
        console.log('✅ AI response window completely closed and cleaned up');
    }

    // Send data to AI response window (for Tauri-based window communication)
    async sendToAiWindow(type, data) {
        console.log(`📤 sendToAiWindow called with type: ${type}, window exists: ${!!this.aiResponseWindow}`);
        
        // If window was closed, don't send data to it
        if (!this.aiResponseWindow) {
            console.log('⚠️ AI response window was closed, ignoring sendToAiWindow call');
            return;
        }
        
        // Handle streaming state management
        if (type === 'stream') {
            if (!this.isStreaming) {
                // Start of a new stream - reset accumulator
                this.streamingText = '';
                this.isStreaming = true;
                console.log('🚀 Starting new streaming session');
                // Reset window height at start of new stream
                this.resetAiWindowHeight();
            }
            
            // For streaming, data should be the new token/chunk to add
            const newToken = typeof data === 'string' ? data : data?.text || '';
            if (newToken) {
                // Only add the new token, not replace entire text
                this.streamingText += newToken;
                console.log(`📝 Added token: "${newToken}" | Total length: ${this.streamingText.length}`);
            }
        } else if (type === 'complete') {
            // Stream is complete
            this.isStreaming = false;
            const finalText = typeof data === 'string' ? data : data?.text || this.streamingText;
            this.streamingText = finalText;
            console.log('✅ Stream completed, final text length:', finalText.length);
        } else if (type === 'error') {
            // Error occurred - reset streaming state
            this.isStreaming = false;
            this.streamingText = '';
        }

        // Always mirror into the in-UI response window so users can see content immediately
        const mirrorToUi = () => {
            if (!this.aiResponseWindow) {
                console.log('⚠️ Window closed during mirrorToUi, skipping');
                return;
            }
            
            const contentEl = this.aiResponseWindow.querySelector('.ai-response-content');
            if (!contentEl) {
                console.log('⚠️ Content element not found, skipping mirrorToUi');
                return;
            }
            
            if (type === 'stream') {
                // Show accumulated streaming text with cursor
                contentEl.innerHTML = this.streamingText + '<span class="cursor">|</span>';
                // Adjust window height for streaming content (throttled)
                this.throttledAdjustHeight();
            } else if (type === 'complete') {
                // Show final text without cursor
                const finalText = typeof data === 'string' ? data : data?.text || this.streamingText;
                contentEl.textContent = finalText;
                // Reset content styling
                contentEl.style.fontStyle = 'normal';
                contentEl.style.opacity = '1';
                // Adjust window height for final content (immediate)
                this.adjustAiWindowHeight();
            } else if (type === 'error') {
                const errorMsg = typeof data === 'string' ? data : data?.error || 'Unknown error';
                contentEl.textContent = `Error: ${errorMsg}`;
                // Reset content styling
                contentEl.style.fontStyle = 'normal';
                contentEl.style.opacity = '1';
                contentEl.style.color = 'var(--danger)';
                // Adjust window height for error content (immediate)
                this.adjustAiWindowHeight();
            }
        };

        try {
            // Use the new Tauri command to send data to AI response window
            const aiResponseData = {
                message_type: type,
                text: type === 'stream' ? this.streamingText : (typeof data === 'string' ? data : data?.text || null),
                error: typeof data === 'string' && type === 'error' ? data : data?.error || null
            };
            
            console.log(`Sending to AI window (${type}):`, aiResponseData.text ? `"${aiResponseData.text.substring(0, 100)}..."` : aiResponseData);
            
            await safeInvoke('send_ai_response_data', { data: aiResponseData });

            // Mirror to UI as well for visibility
            mirrorToUi();
        } catch (error) {
            console.warn('Falling back to in-UI response window:', error);
            mirrorToUi();
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
            // Hide the status pill to remove the "Live" label from header
            if (statusPill) {
                statusPill.textContent = '';
                statusPill.style.display = 'none';
            }
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
