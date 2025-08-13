// Session-based UI Flow Manager for MockMate Desktop App

class SessionFlowManager {
    constructor() {
        this.currentState = 'initial'; // initial, connected, activated
        this.sessionData = null;
        this.userCredits = 0;
        this.timerState = null;
        this.timerInterval = null;
        this.startTime = null;
        this.pausedTime = 0;
        
        console.log('🔄 Initializing Session Flow Manager');
        this.setupInitialState();
        this.setupEventListeners();
    }

    setupInitialState() {
        // Hide main interview components initially
        this.hideMainComponents();
        
        // Show only session connection interface
        this.showSessionConnectionOnly();
        
        // Update body state
        document.body.className = 'session-initial';
        
        console.log('✅ Initial state setup complete - showing connection interface only');
    }

    setupEventListeners() {
        // Listen for session events from the existing session connection manager
        document.addEventListener('sessionConnected', (event) => {
            console.log('📡 Session connected event received:', event.detail);
            this.onSessionConnected(event.detail);
        });

        document.addEventListener('sessionActivated', (event) => {
            console.log('📡 Session activated event received:', event.detail);
            this.onSessionActivated(event.detail);
        });

        document.addEventListener('sessionDisconnected', () => {
            console.log('📡 Session disconnected event received');
            this.onSessionDisconnected();
        });

        // Override the existing connect button to use our flow
        this.setupConnectButton();
        
        // Setup session close button
        this.setupSessionCloseButton();
    }

    setupConnectButton() {
        // Wait for session connection UI to be ready
        const checkAndSetup = () => {
            const connectBtn = document.getElementById('connectBtn');
            if (connectBtn) {
                connectBtn.addEventListener('click', async () => {
                    await this.handleSessionConnection();
                });
            } else {
                setTimeout(checkAndSetup, 100);
            }
        };
        checkAndSetup();
    }

    async handleSessionConnection() {
        const sessionIdInput = document.getElementById('sessionIdInput');
        const sessionId = sessionIdInput?.value?.trim();
        
        if (!sessionId) {
            this.showNotification('Please enter a session ID', 'warning');
            return;
        }

        console.log('🔗 Attempting to connect to session:', sessionId);
        
        try {
            // Show loading state
            this.setConnectButtonLoading(true);
            
            // Extract session ID from URL if needed
            const cleanSessionId = this.extractSessionId(sessionId);
            
            // Show connecting progress
            this.showNotification('🔗 Connecting to session...', 'info');
            
            // Fetch real session details from database using the session ID
            console.log('📞 Fetching session details from database for ID:', cleanSessionId);
            const sessionInfo = await window.safeInvoke('connect_session', {
                sessionId: cleanSessionId
            });
            
            if (!sessionInfo) {
                throw new Error('Session not found or invalid session ID');
            }
            
            console.log('✅ Successfully fetched session details:', sessionInfo);
            this.sessionData = sessionInfo;
            
            // Transition to connected state
            await this.transitionToConnected(sessionInfo);
            
        } catch (error) {
            console.error('❌ Failed to connect to session:', error);
            this.showNotification(`Failed to connect: ${error.message || error}`, 'error');
        } finally {
            this.setConnectButtonLoading(false);
        }
    }

    extractSessionId(input) {
        // Extract session ID from URL if pasted
        if (input.includes('mockmate://session/')) {
            const match = input.match(/mockmate:\/\/session\/([a-f0-9-]+)/);
            return match ? match[1] : input;
        }
        if (input.includes('/session/')) {
            const match = input.match(/\/session\/([a-f0-9-]+)/);
            return match ? match[1] : input;
        }
        return input;
    }

    async transitionToConnected(sessionInfo) {
        this.currentState = 'connected';
        this.sessionData = sessionInfo;
        
        // Update UI to show session info and activation button
        this.showSessionConnectedState(sessionInfo);
        
        // Update body class
        document.body.className = 'session-connected';
        
        console.log('🔄 Transitioned to connected state');
        
        // Resize main window to accommodate the detailed session info and buttons
        try {
            // Use the proper safeInvoke from main.js that handles the Tauri API correctly
            if (window.safeInvoke) {
                await window.safeInvoke('resize_main_window', { width: 800, height: 280 });
            } else {
                console.log('⚠️ safeInvoke not available, skipping window resize');
            }
            console.log('📐 Main window resized to show full session details');
        } catch (resizeError) {
            console.warn('⚠️ Failed to resize main window:', resizeError);
        }
        
        // Show notification
        this.showNotification(`Connected to "${sessionInfo.interview_config.job_title}"`, 'success');
    }

    showSessionConnectedState(sessionInfo) {
        // Update the session connection panel to show connected state
        const panel = document.getElementById('sessionConnectionPanel');
        if (panel) {
            // Format session creation date
            const createdDate = new Date(sessionInfo.created_at).toLocaleDateString('en-US', {
                month: 'short',
                day: 'numeric',
                year: 'numeric'
            });
            
            // Format company name or show "N/A"
            const companyDisplay = sessionInfo.company_name || 'Not specified';
            
            // Format job description preview (first 80 chars)
            const jobDescPreview = sessionInfo.job_description 
                ? (sessionInfo.job_description.length > 80 
                    ? sessionInfo.job_description.substring(0, 80) + '...' 
                    : sessionInfo.job_description)
                : 'No description provided';
            
            panel.innerHTML = `
                <div class="session-connected-content">
                    <div class="session-connection-header">
                        <div class="session-icon">
                            <span class="material-icons">check_circle</span>
                        </div>
                        <div class="session-title-info">
                            <h3>Connected: ${sessionInfo.job_title}</h3>
                            <p class="session-company">📍 ${companyDisplay}</p>
                        </div>
                    </div>
                    
                    <div class="session-details-compact">
                        <div class="detail-row">
                            <span class="detail-label">👤 Interviewer:</span>
                            <span class="detail-value">${sessionInfo.user_details.name}</span>
                        </div>
                        <div class="detail-row">
                            <span class="detail-label">💳 Credits:</span>
                            <span class="detail-value credits-count">${sessionInfo.credits_available}</span>
                        </div>
                        <div class="detail-row">
                            <span class="detail-label">📅 Created:</span>
                            <span class="detail-value">${createdDate}</span>
                        </div>
                        <div class="detail-row">
                            <span class="detail-label">📊 Status:</span>
                            <span class="detail-value status-badge status-${sessionInfo.status}">${sessionInfo.status.toUpperCase()}</span>
                        </div>
                        <div class="detail-row full-width">
                            <span class="detail-label">📝 Description:</span>
                            <span class="detail-value job-desc">${jobDescPreview}</span>
                        </div>
                        <div class="detail-row">
                            <span class="detail-label">🎯 Difficulty:</span>
                            <span class="detail-value difficulty-badge">${sessionInfo.interview_config.difficulty}</span>
                        </div>
                        <div class="detail-row button-row">
                            <button id="activateSessionBtn" class="compact-btn start-btn">
                                <span class="material-icons">play_arrow</span>
                                Start
                            </button>
                            <button id="disconnectSessionBtn" class="compact-btn disconnect-btn">
                                <span class="material-icons">link_off</span>
                                Disconnect
                            </button>
                            <button id="sessionCloseBtn" class="compact-btn close-btn" title="Close Application">
                                <span class="material-icons">power_settings_new</span>
                                Close
                            </button>
                        </div>
                        <div class="detail-row full-width">
                            <span class="detail-label">🔗 Session ID:</span>
                            <span class="detail-value session-id">${sessionInfo.session_id.split('-')[0]}...</span>
                        </div>
                    </div>
                </div>
            `;
        }

        // Setup activation button
        this.setupActivationButton();
        
        // Setup disconnect button
        this.setupDisconnectButton();
        
        // Setup close button for connected state
        this.setupSessionCloseButton();
    }

    setupActivationButton() {
        const activateBtn = document.getElementById('activateSessionBtn');
        if (activateBtn) {
            activateBtn.addEventListener('click', async () => {
                await this.handleSessionActivation();
            });
        }
    }

    setupDisconnectButton() {
        const disconnectBtn = document.getElementById('disconnectSessionBtn');
        if (disconnectBtn) {
            disconnectBtn.addEventListener('click', () => {
                this.handleSessionDisconnection();
            });
        }
    }
    
    setupSessionCloseButton() {
        // Wait for session close button to be ready
        const checkAndSetup = () => {
            const sessionCloseBtn = document.getElementById('sessionCloseBtn');
            if (sessionCloseBtn) {
                sessionCloseBtn.addEventListener('click', async () => {
                    await this.handleApplicationClose();
                });
                console.log('✅ Session close button event listener attached');
            } else {
                setTimeout(checkAndSetup, 100);
            }
        };
        checkAndSetup();
    }
    
    async handleApplicationClose() {
        console.log('🔴 Application close requested from session screen');
        
        // If there's an active session, stop the timer first
        if (this.currentState === 'connected' || this.currentState === 'activated') {
            this.stopTimer();
        }
        
        // Use the same close command as the main close button
        try {
            await window.safeInvoke('close_application');
        } catch (error) {
            console.error('❌ Failed to close application:', error);
            // Fallback: try to close the window directly
            if (window.__TAURI__ && window.__TAURI__.window) {
                try {
                    await window.__TAURI__.window.appWindow.close();
                } catch (fallbackError) {
                    console.error('❌ Fallback close also failed:', fallbackError);
                }
            }
        }
    }

    async handleSessionActivation() {
        if (!this.sessionData) {
            this.showNotification('No session connected', 'error');
            return;
        }

        console.log('🚀 Attempting to activate session:', this.sessionData.session_id);
        
        try {
            // Show loading state
            this.setActivateButtonLoading(true);
            
            // Activate the session
            const result = await window.safeInvoke('activate_session', { sessionId: this.sessionData.session_id });
            
            console.log('✅ Session activated successfully:', result);
            
            // Initialize timer state (no backend timer start needed)
            this.timerState = { 
                started_at: new Date().toISOString(), 
                status: 'active' 
            };
            console.log('✅ Timer state initialized:', this.timerState);
            
            // Transition to activated state
            this.transitionToActivated();
            
        } catch (error) {
            console.error('❌ Failed to activate session:', error);
            this.showNotification(`Failed to activate: ${error.message || error}`, 'error');
        } finally {
            this.setActivateButtonLoading(false);
        }
    }

    transitionToActivated() {
        this.currentState = 'activated';
        
        // Hide session connection panel
        this.hideSessionConnectionPanel();
        
        // Show all main interview components
        this.showMainComponents();
        
        // Update header with session info
        this.updateHeaderWithSessionInfo();
        
        // Start the timer display
        this.startTimer();
        
        // Initialize QA Storage Manager for real-time data storage
        this.initializeQAStorageManager();
        
        // Initialize Session Completion Monitor for real-time session monitoring
        this.initializeSessionMonitor();
        
        // Update body class
        document.body.className = 'session-activated';
        
        console.log('🔄 Transitioned to activated state - full interview interface available');
        
        // Show notification
        this.showNotification('Interview session activated! Timer started.', 'success');
        
        // Initialize main controller if not already done
        if (window.mockMateController) {
            window.mockMateController.onSessionActivated(this.sessionData, this.timerState);
        }
    }

    async handleSessionDisconnection() {
        // Confirm disconnection
        if (confirm('Are you sure you want to disconnect from the current session?')) {
            await this.onSessionDisconnected();
        }
    }

    async onSessionDisconnected() {
        // Stop the timer if running
        this.stopTimer();
        
        this.currentState = 'initial';
        this.sessionData = null;
        this.timerState = null;
        this.userCredits = 0;
        
        // Reset to initial state
        this.setupInitialState();
        
        // Resize window back to smaller size for initial state
        try {
            // Use the proper safeInvoke from main.js that handles the Tauri API correctly
            if (window.safeInvoke) {
                await window.safeInvoke('resize_main_window', { width: 800, height: 160 });
            } else {
                console.log('⚠️ safeInvoke not available, skipping window resize');
            }
            console.log('📰 Main window resized back to initial size');
        } catch (resizeError) {
            console.warn('⚠️ Failed to resize main window:', resizeError);
        }
        
        console.log('🔄 Session disconnected - returned to initial state');
        this.showNotification('Session disconnected', 'info');
    }

    // UI State Management Methods
    hideMainComponents() {
        const componentsToHide = [
            '.transcription-container',
            '.input-section',
            '.session-info'  // Header session info
        ];
        
        componentsToHide.forEach(selector => {
            const element = document.querySelector(selector);
            if (element) {
                element.style.display = 'none';
            }
        });
    }

    showMainComponents() {
        const componentsToShow = [
            '.transcription-container',
            '.input-section'
        ];
        
        componentsToShow.forEach(selector => {
            const element = document.querySelector(selector);
            if (element) {
                element.style.display = '';
            }
        });
    }

    showSessionConnectionOnly() {
        // Session connection UI is already in the HTML
        // No need to initialize conflicting session connection manager
        console.log('Session connection UI is ready');
    }

    hideSessionConnectionPanel() {
        const panel = document.getElementById('sessionConnectionPanel');
        if (panel) {
            panel.style.display = 'none';
        }
    }

    updateHeaderWithSessionInfo() {
        const sessionInfo = document.getElementById('sessionInfo');
        if (sessionInfo && this.sessionData) {
            sessionInfo.classList.add('active');
            
            const sessionTitle = document.getElementById('sessionTitle');
            const sessionCredits = document.getElementById('sessionCredits');
            
            if (sessionTitle) {
                sessionTitle.textContent = `${this.sessionData.interview_config.job_title}`;
            }
            
            if (sessionCredits) {
                sessionCredits.textContent = `${this.sessionData.credits_available} credits`;
            }
        }
    }

    // Button State Management
    setConnectButtonLoading(loading) {
        const connectBtn = document.getElementById('connectBtn');
        if (connectBtn) {
            connectBtn.disabled = loading;
            connectBtn.innerHTML = loading 
                ? '<span class="material-icons">hourglass_empty</span>Connecting...'
                : '<span class="material-icons">link</span>Connect';
        }
    }

    setActivateButtonLoading(loading) {
        const activateBtn = document.getElementById('activateSessionBtn');
        if (activateBtn) {
            activateBtn.disabled = loading;
            activateBtn.innerHTML = loading
                ? '<span class="material-icons">hourglass_empty</span>Starting...'
                : '<span class="material-icons">play_arrow</span>Start';
        }
    }

    // Notification System
    showNotification(message, type = 'info') {
        // Create notification element if it doesn't exist
        let notificationContainer = document.querySelector('.notification-container');
        if (!notificationContainer) {
            notificationContainer = document.createElement('div');
            notificationContainer.className = 'notification-container';
            document.body.appendChild(notificationContainer);
        }

        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.innerHTML = `
            <span class="material-icons">${this.getNotificationIcon(type)}</span>
            <span class="notification-message">${message}</span>
        `;

        notificationContainer.appendChild(notification);

        // Auto-remove after 5 seconds
        setTimeout(() => {
            notification.remove();
        }, 5000);

        console.log(`📢 Notification (${type}): ${message}`);
    }

    getNotificationIcon(type) {
        const icons = {
            success: 'check_circle',
            error: 'error',
            warning: 'warning',
            info: 'info'
        };
        return icons[type] || 'info';
    }

    // Public API for other components
    getCurrentState() {
        return this.currentState;
    }

    getSessionData() {
        return this.sessionData;
    }

    getTimerState() {
        return this.timerState;
    }

    isSessionActivated() {
        return this.currentState === 'activated';
    }
    
    // Timer Management Methods
    startTimer() {
        if (this.timerInterval) {
            clearInterval(this.timerInterval);
        }
        
        // Set start time from timer state or current time
        if (this.timerState && this.timerState.started_at) {
            this.startTime = new Date(this.timerState.started_at).getTime();
        } else {
            this.startTime = new Date().getTime();
        }
        
        // Show timer display
        const timerElement = document.getElementById('sessionTimer');
        if (timerElement) {
            timerElement.classList.add('active');
        }
        
        // Update timer every second
        this.timerInterval = setInterval(() => {
            this.updateTimerDisplay();
        }, 1000);
        
        // Initial update
        this.updateTimerDisplay();
        
        console.log('⏱️ Timer started at:', new Date(this.startTime));
    }
    
    updateTimerDisplay() {
        if (!this.startTime) return;
        
        const now = new Date().getTime();
        const elapsed = Math.floor((now - this.startTime - this.pausedTime) / 1000);
        
        const minutes = Math.floor(elapsed / 60);
        const seconds = elapsed % 60;
        
        const formattedTime = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
        
        const timerDisplay = document.getElementById('timerDisplay');
        if (timerDisplay) {
            timerDisplay.textContent = formattedTime;
        }
        
        // Save timer state periodically (every 10 seconds)
        if (elapsed % 10 === 0 && elapsed > 0) {
            this.saveTimerState(elapsed);
        }
    }
    
    stopTimer() {
        if (this.timerInterval) {
            clearInterval(this.timerInterval);
            this.timerInterval = null;
        }
        
        // Hide timer display
        const timerElement = document.getElementById('sessionTimer');
        if (timerElement) {
            timerElement.classList.remove('active');
        }
        
        // Calculate final duration and save
        if (this.startTime) {
            const now = new Date().getTime();
            const totalSeconds = Math.floor((now - this.startTime - this.pausedTime) / 1000);
            const totalMinutes = Math.floor(totalSeconds / 60);
            
            console.log('⏱️ Timer stopped. Total duration:', totalMinutes, 'minutes');
            
            // Save final timer state to database
            this.saveTimerState(totalSeconds, true);
        }
        
        // Reset timer state
        this.startTime = null;
        this.pausedTime = 0;
    }
    
    async saveTimerState(elapsedSeconds, isFinal = false) {
        if (!this.sessionData || !this.sessionData.session_id) {
            return;
        }
        
        try {
            const minutes = Math.floor(elapsedSeconds / 60);
            
            await window.safeInvoke('update_session_timer', {
                sessionId: this.sessionData.session_id,
                elapsedMinutes: minutes,
                isFinal: isFinal
            });
            
            if (isFinal) {
                console.log('✅ Final timer state saved:', minutes, 'minutes');
            }
        } catch (error) {
            console.warn('⚠️ Failed to save timer state:', error);
        }
    }
    
    pauseTimer() {
        if (this.timerInterval) {
            clearInterval(this.timerInterval);
            this.timerInterval = null;
            
            if (this.startTime) {
                this.pausedTime += new Date().getTime() - this.startTime;
            }
            
            console.log('⏸️ Timer paused');
        }
    }
    
    resumeTimer() {
        if (!this.timerInterval && this.startTime) {
            this.startTime = new Date().getTime();
            this.timerInterval = setInterval(() => {
                this.updateTimerDisplay();
            }, 1000);
            
        console.log('▶️ Timer resumed');
        }
    }
    
    // Real-time Question & Answer Storage Integration
    initializeQAStorageManager() {
        if (!window.qaStorageManager) {
            console.warn('⚠️ QA Storage Manager not available');
            return;
        }
        
        // Get auth token from session data or use a mock token for desktop app
        const authToken = this.sessionData?.auth_token || this.sessionData?.user_details?.auth_token || 'desktop-app-token';
        
        console.log('🔑 Using auth token for QA Storage Manager:', authToken ? 'Token available' : 'No token');
        
        // Initialize QA Storage Manager with session data
        window.qaStorageManager.initialize(
            { id: this.sessionData.session_id }, 
            authToken, 
            this.sessionData.user_details?.user_id
        );
        console.log('✅ QA Storage Manager initialized for real-time Q&A storage');
        
        // Hook into main controller for question and answer events
        if (window.mockMateController) {
            // Store original methods to wrap them
            const originalGenerateAnswer = window.mockMateController.generateAnswer;
            const originalSendManualQuestion = window.mockMateController.sendManualQuestion;
            const originalSendToAiWindow = window.mockMateController.sendToAiWindow;
            
            // Initialize question counter
            window.qaStorageManager.currentQuestionNumber = 1;
            
            // Wrap generateAnswer to store questions from transcription
            window.mockMateController.generateAnswer = async function(...args) {
                const questionText = this.fullTranscription || document.getElementById('questionInput')?.value;
                
                if (questionText && questionText.trim()) {
                    try {
                        // Store question immediately with correct format
                        const result = await window.qaStorageManager.storeQuestion({
                            questionText: questionText.trim(),
                            questionNumber: window.qaStorageManager.currentQuestionNumber,
                            category: 'general',
                            difficultyLevel: 'medium',
                            source: 'transcribed',
                            metadata: {
                                timestamp: new Date().toISOString(),
                                session_id: window.sessionFlowManager?.sessionData?.session_id
                            }
                        });
                        console.log('✅ Transcribed question stored successfully:', result);
                        
                        // Increment question number
                        window.qaStorageManager.currentQuestionNumber += 1;
                    } catch (error) {
                        console.error('❌ Failed to store transcribed question:', error);
                    }
                }
                
                // Call original method
                return originalGenerateAnswer.apply(this, args);
            };
            
            // Wrap sendManualQuestion to store manual questions
            window.mockMateController.sendManualQuestion = async function(...args) {
                const questionText = document.getElementById('questionInput')?.value;
                
                if (questionText && questionText.trim()) {
                    try {
                        // Store question immediately with correct format
                        const result = await window.qaStorageManager.storeQuestion({
                            questionText: questionText.trim(),
                            questionNumber: window.qaStorageManager.currentQuestionNumber,
                            category: 'general',
                            difficultyLevel: 'medium',
                            source: 'manual_input',
                            metadata: {
                                timestamp: new Date().toISOString(),
                                session_id: window.sessionFlowManager?.sessionData?.session_id
                            }
                        });
                        console.log('✅ Manual question stored successfully:', result);
                        
                        // Increment question number
                        window.qaStorageManager.currentQuestionNumber += 1;
                    } catch (error) {
                        console.error('❌ Failed to store manual question:', error);
                    }
                }
                
                // Call original method
                return originalSendManualQuestion.apply(this, args);
            };
            
            // Wrap sendToAiWindow to capture and store AI responses
            window.mockMateController.sendToAiWindow = async function(type, content, ...args) {
                // Store answer when response is complete
                if (type === 'complete' && content && typeof content === 'string' && content.trim()) {
                    try {
                        const result = await window.qaStorageManager.storeAnswer({
                            answerText: content.trim(),
                            questionId: window.qaStorageManager.currentQuestionId,
                            source: 'ai_response',
                            metadata: {
                                timestamp: new Date().toISOString(),
                                session_id: window.sessionFlowManager?.sessionData?.session_id,
                                model: this.selectedModel,
                                provider: this.selectedProvider
                            }
                        });
                        console.log('✅ AI answer stored successfully:', result);
                    } catch (error) {
                        console.error('❌ Failed to store AI answer:', error);
                    }
                }
                
                // Call original method
                return originalSendToAiWindow.apply(this, [type, content, ...args]);
            };
            
            console.log('✅ Hooked into main controller for real-time Q&A storage');
        }
    }
    
    // Real-time Session Completion Monitoring
    initializeSessionMonitor() {
        if (!window.sessionCompletionMonitor) {
            console.warn('⚠️ Session Completion Monitor not available');
            return;
        }
        
        // Get auth token from session data or use mock token for desktop app
        const authToken = this.sessionData?.auth_token || this.sessionData?.user_details?.auth_token || 'desktop-app-token';
        
        console.log('🔑 Using auth token for Session Monitor:', authToken ? 'Token available' : 'No token');
        
        // Initialize session completion monitor
        const callbacks = {
            onCompleted: async (statusData) => {
                console.log('🎯 Session completed callback triggered');
                
                // Stop timer and cleanup
                this.stopTimer();
                
                // Ensure final Q&A sync
                if (window.qaStorageManager) {
                    await window.qaStorageManager.forceFinalSync();
                }
                
                console.log('✅ Session completion cleanup finished');
            },
            
            onStopped: async (statusData, reason) => {
                console.log(`⏹️ Session stopped callback triggered: ${reason}`);
                
                // Stop timer and cleanup
                this.stopTimer();
                
                // Ensure final Q&A sync
                if (window.qaStorageManager) {
                    await window.qaStorageManager.forceFinalSync();
                }
                
                console.log('✅ Session stop cleanup finished');
            }
        };
        
        window.sessionCompletionMonitor.initialize(this.sessionData, authToken, callbacks);
        console.log('✅ Session Completion Monitor initialized for real-time monitoring');
    }
}

// Additional CSS for session flow states
const sessionFlowStyles = `
    /* Session Flow States */
    body.session-initial .main-window {
        display: none !important;
    }
    
    body.session-initial .session-connection-container {
        display: flex !important;
    }

    body.session-connected .main-window {
        display: none !important;
    }
    
    body.session-connected .session-connection-container {
        display: flex !important;
    }

    body.session-activated .session-connection-container {
        display: none !important;
    }
    
    body.session-activated .main-window {
        display: flex !important;
    }

    /* Session Connected State Styles */
    .session-connected-content {
        display: flex;
        flex-direction: column;
        gap: 12px;
        font-size: 12px;
    }
    
    .session-connection-header {
        display: flex;
        align-items: flex-start;
        gap: 12px;
    }
    
    .session-title-info h3 {
        margin: 0;
        font-size: 14px;
        font-weight: 600;
        color: var(--success);
    }
    
    .session-company {
        margin: 4px 0 0 0;
        font-size: 11px;
        color: var(--text-muted);
        font-weight: 500;
    }
    
    .session-details-compact {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 4px 8px;
        font-size: 10px;
        background: rgba(0, 0, 0, 0.1);
        padding: 8px;
        border-radius: 6px;
        border: 1px solid var(--border);
    }
    
    .detail-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 2px 0;
    }
    
    .detail-row.full-width {
        grid-column: 1 / -1;
        flex-direction: column;
        align-items: flex-start;
        gap: 2px;
        border-top: 1px solid var(--border);
        padding-top: 6px;
        margin-top: 4px;
    }
    
    .detail-row.button-row {
        grid-column: 1 / -1;
        justify-content: center;
        padding: 8px 0;
        margin: 4px 0;
        border-top: 1px solid rgba(255, 255, 255, 0.1);
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }
    
    .detail-label {
        font-weight: 500;
        color: var(--text-muted);
        white-space: nowrap;
        font-size: 10px;
    }
    
    .detail-value {
        font-weight: 600;
        color: var(--text-primary);
        text-align: right;
        font-size: 10px;
    }
    
    .detail-row.full-width .detail-value {
        text-align: left;
        font-size: 9px;
        line-height: 1.3;
        color: var(--text-secondary);
    }
    
    .credits-count {
        background: var(--accent-secondary);
        color: white;
        padding: 2px 6px;
        border-radius: 3px;
        font-size: 9px;
        font-weight: 700;
    }
    
    .status-badge {
        padding: 2px 4px;
        border-radius: 3px;
        font-size: 8px;
        font-weight: 700;
        text-transform: uppercase;
    }
    
    .status-created {
        background: rgba(0, 123, 255, 0.2);
        color: var(--accent);
    }
    
    .status-active {
        background: rgba(0, 200, 150, 0.2);
        color: var(--success);
    }
    
    .difficulty-badge {
        background: rgba(255, 165, 2, 0.2);
        color: var(--warning);
        padding: 2px 4px;
        border-radius: 3px;
        font-size: 8px;
        font-weight: 600;
    }
    
    .session-id {
        font-family: monospace;
        background: rgba(0, 0, 0, 0.1);
        padding: 1px 4px;
        border-radius: 3px;
        font-size: 8px;
    }

    .session-info-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .session-status-indicator {
        display: flex;
        align-items: center;
        gap: 8px;
        font-weight: 500;
        font-size: 14px;
    }

    .session-status-indicator.connected {
        color: var(--success);
    }

    .status-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: var(--success);
        animation: pulse 2s infinite;
    }

    .session-details-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8px 16px;
    }

    .session-detail-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 12px;
    }

    .detail-label {
        color: var(--text-muted);
        font-weight: 500;
    }

    .detail-value {
        color: var(--text-primary);
        font-weight: 600;
    }

    .credits-count {
        color: var(--accent-secondary);
        background: rgba(255, 107, 53, 0.1);
        padding: 2px 6px;
        border-radius: 4px;
    }

    .session-activation-section {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        padding: 12px 0;
        border-top: 1px solid var(--border);
    }

    .activate-btn {
        background: linear-gradient(135deg, var(--success), rgba(0, 200, 150, 0.8));
        color: white;
        padding: 10px 20px;
        font-size: 14px;
        font-weight: 600;
    }

    .activate-btn:hover {
        background: linear-gradient(135deg, rgba(0, 200, 150, 0.9), var(--success));
        transform: translateY(-2px);
        box-shadow: 0 4px 16px rgba(0, 200, 150, 0.3);
    }

    .activate-btn:disabled {
        background: rgba(0, 200, 150, 0.3);
        transform: none;
        box-shadow: none;
        cursor: not-allowed;
    }

    .activation-note {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 11px;
        color: var(--text-muted);
    }

    .activation-note .material-icons {
        font-size: 14px;
    }

    /* Notification System */
    .notification-container {
        position: fixed;
        top: 20px;
        right: 20px;
        z-index: 10000;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .notification {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 12px 16px;
        border-radius: 8px;
        background: rgba(0, 0, 0, 0.9);
        border: 1px solid var(--border);
        color: var(--text-primary);
        font-size: 13px;
        min-width: 300px;
        animation: slideInFromRight 0.3s ease;
        backdrop-filter: blur(10px);
    }

    .notification-success {
        border-color: var(--success);
        background: rgba(0, 200, 150, 0.1);
    }

    .notification-error {
        border-color: var(--danger);
        background: rgba(255, 71, 87, 0.1);
    }

    .notification-warning {
        border-color: var(--warning);
        background: rgba(255, 165, 2, 0.1);
    }

    .notification-info {
        border-color: var(--accent);
        background: rgba(0, 212, 255, 0.1);
    }

    @keyframes slideInFromRight {
        from {
            transform: translateX(100%);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }

    /* Compact Action Buttons */
    .session-action-buttons {
        display: flex;
        gap: 8px;
        justify-content: center;
        margin-top: 8px;
    }
    
    .compact-btn {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 6px 12px;
        font-size: 11px;
        font-weight: 600;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;
        text-transform: none;
        min-width: auto;
        height: 28px;
    }
    
    .compact-btn .material-icons {
        font-size: 14px;
    }
    
    .start-btn {
        background: linear-gradient(135deg, var(--success), rgba(0, 200, 150, 0.8));
        color: white;
    }
    
    .start-btn:hover {
        background: linear-gradient(135deg, rgba(0, 200, 150, 0.9), var(--success));
        transform: translateY(-1px);
        box-shadow: 0 2px 8px rgba(0, 200, 150, 0.3);
    }
    
    .start-btn:disabled {
        background: rgba(0, 200, 150, 0.3);
        transform: none;
        box-shadow: none;
        cursor: not-allowed;
    }
    
    .disconnect-btn {
        background: linear-gradient(135deg, var(--danger), rgba(255, 71, 87, 0.8));
        color: white;
    }
    
    .disconnect-btn:hover {
        background: linear-gradient(135deg, rgba(255, 71, 87, 0.9), var(--danger));
        transform: translateY(-1px);
        box-shadow: 0 2px 8px rgba(255, 71, 87, 0.3);
    }
    
    .disconnect-btn:disabled {
        background: rgba(255, 71, 87, 0.3);
        transform: none;
        box-shadow: none;
        cursor: not-allowed;
    }
    
    .close-btn {
        background: linear-gradient(135deg, var(--danger), rgba(255, 71, 87, 0.8));
        color: white;
    }
    
    .close-btn:hover {
        background: linear-gradient(135deg, rgba(255, 71, 87, 0.9), var(--danger));
        transform: translateY(-1px);
        box-shadow: 0 2px 8px rgba(255, 71, 87, 0.3);
    }
    
    .close-btn:disabled {
        background: rgba(255, 71, 87, 0.3);
        transform: none;
        box-shadow: none;
        cursor: not-allowed;
    }

    /* Animation for session state transitions */
    .session-connected-content {
        animation: fadeInUp 0.4s ease;
    }

    @keyframes fadeInUp {
        from {
            opacity: 0;
            transform: translateY(20px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
`;

// Add the styles to the document
const sessionFlowStyleEl = document.createElement('style');
sessionFlowStyleEl.textContent = sessionFlowStyles;
document.head.appendChild(sessionFlowStyleEl);

// Initialize the session flow manager
window.sessionFlowManager = new SessionFlowManager();

// Use global safeInvoke from main.js instead of defining our own

console.log('✅ Session Flow Manager loaded successfully');
