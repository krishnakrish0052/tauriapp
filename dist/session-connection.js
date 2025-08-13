// Session Connection Management for MockMate Desktop App

class SessionConnectionManager {
    constructor() {
        this.currentSession = null;
        this.connectionStatus = 'disconnected';
        
        // Set initial body state immediately for first run
        console.log('🎬 Setting initial body state for session welcome');
        document.body.classList.remove('session-connected');
        document.body.classList.add('session-welcome');
        
        this.setupUI();
        this.setupEventListeners();
        this.setupProtocolHandling(); // Add protocol handling
        
        // Ensure session panel is visible on first run
        setTimeout(() => {
            this.ensureFirstRunState();
            // Test toast notification
            this.showNotification('🎯 Session Manager Initialized', 'success', 2000);
        }, 100); // Slightly longer delay to ensure everything is ready
    }

    setupUI() {
        // Create session connection UI elements
        const existingContainer = document.querySelector('.session-connection-container');
        if (existingContainer) {
            existingContainer.remove();
        }

        const sessionContainer = document.createElement('div');
        sessionContainer.className = 'session-connection-container';
        sessionContainer.innerHTML = `
            <div class="session-connection-panel" id="sessionConnectionPanel" style="display: block;">
                <div class="session-input-section">
                    <div class="session-input-group">
                        <label for="sessionIdInput">Session ID:</label>
                        <input type="text" id="sessionIdInput" placeholder="Enter session ID or paste URL" 
                               class="session-input" />
                        <button id="connectBtn" class="session-btn connect-btn">
                            <span class="material-icons">link</span>
                            <span>Connect</span>
                        </button>
                    </div>
                </div>
                
                <div class="session-status-section" id="sessionStatusDisplay" style="display: none;">
                    <div class="session-info">
                        <div class="session-details">
                            <div class="session-title" id="sessionTitle">No Session Connected</div>
                            <div class="session-meta" id="sessionMeta"></div>
                        </div>
                        <div class="session-actions">
                            <button id="disconnectBtn" class="session-btn disconnect-btn">
                                <span class="material-icons">link_off</span>
                                Disconnect
                            </button>
                        </div>
                    </div>
                    <div class="connection-indicator" id="connectionIndicator">
                        <span class="indicator-dot"></span>
                        <span class="indicator-text">Connected</span>
                    </div>
                </div>
            </div>
        `;

        // Insert the session UI at the top of the main container
        const mainWindow = document.querySelector('.main-window');
        if (mainWindow) {
            const header = mainWindow.querySelector('.header');
            if (header) {
                mainWindow.insertBefore(sessionContainer, header.nextSibling);
            } else {
                mainWindow.insertBefore(sessionContainer, mainWindow.firstChild);
            }
        }

        // Add CSS styles
        this.addStyles();
    }

    addStyles() {
        const existingStyles = document.querySelector('#session-connection-styles');
        if (existingStyles) {
            existingStyles.remove();
        }

        const styles = document.createElement('style');
        styles.id = 'session-connection-styles';
        styles.textContent = `
            .session-connection-container {
                width: 100%;
                background: rgba(0, 0, 0, 0.8);
                border-bottom: 1px solid var(--border);
                backdrop-filter: blur(10px);
            }

            .session-connection-panel {
                padding: 12px 16px;
                display: flex;
                flex-direction: column;
                gap: 12px;
            }

            .session-input-section {
                display: flex;
                align-items: center;
                gap: 12px;
            }

            .session-input-group {
                display: flex;
                align-items: center;
                gap: 8px;
                flex: 1;
            }

            .session-input-group label {
                font-size: 12px;
                color: var(--text-secondary);
                min-width: 70px;
            }

            .session-input {
                flex: 1;
                padding: 6px 10px;
                background: rgba(255, 255, 255, 0.1);
                border: 1px solid var(--border);
                border-radius: 6px;
                color: var(--text-primary);
                font-size: 12px;
                min-width: 300px;
            }

            .session-input:focus {
                outline: none;
                border-color: var(--accent);
                box-shadow: 0 0 8px rgba(0, 212, 255, 0.3);
            }

            .session-input::placeholder {
                color: var(--text-muted);
            }

            .session-btn {
                padding: 4px 8px;
                border: none;
                border-radius: 6px;
                font-size: 11px;
                font-weight: 500;
                cursor: pointer;
                display: flex;
                align-items: center;
                gap: 3px;
                transition: all 0.2s ease;
                white-space: nowrap;
                min-width: 0;
                flex-shrink: 0;
            }
            
            .session-btn .material-icons {
                font-size: 14px;
                line-height: 1;
            }
            
            .session-btn span:not(.material-icons) {
                font-size: 11px;
                line-height: 1;
            }

            .connect-btn {
                background: var(--accent);
                color: white;
            }

            .connect-btn:hover {
                background: rgba(0, 212, 255, 0.8);
                transform: translateY(-1px);
            }

            .connect-btn:disabled {
                background: rgba(0, 212, 255, 0.3);
                cursor: not-allowed;
                transform: none;
            }

            .disconnect-btn {
                background: var(--danger);
                color: white;
            }

            .disconnect-btn:hover {
                background: rgba(255, 71, 87, 0.8);
                transform: translateY(-1px);
            }

            .session-status-section {
                display: flex;
                justify-content: space-between;
                align-items: center;
            }

            .session-info {
                display: flex;
                align-items: center;
                gap: 12px;
                flex: 1;
            }

            .session-details {
                display: flex;
                flex-direction: column;
                gap: 2px;
            }

            .session-title {
                font-size: 14px;
                font-weight: 600;
                color: var(--text-primary);
            }

            .session-meta {
                font-size: 11px;
                color: var(--text-muted);
            }

            .connection-indicator {
                display: flex;
                align-items: center;
                gap: 6px;
                font-size: 12px;
                color: var(--success);
            }

            .indicator-dot {
                width: 8px;
                height: 8px;
                background: var(--success);
                border-radius: 50%;
                animation: pulse-green 2s infinite;
            }

            .connection-indicator.connecting .indicator-dot {
                background: var(--warning);
                animation: pulse-yellow 1s infinite;
            }

            .connection-indicator.error .indicator-dot {
                background: var(--danger);
                animation: pulse-red 1s infinite;
            }

            .connection-indicator.error .indicator-text {
                color: var(--danger);
            }

            .connection-indicator.connecting .indicator-text {
                color: var(--warning);
            }

            @keyframes pulse-green {
                0%, 100% { opacity: 1; transform: scale(1); }
                50% { opacity: 0.7; transform: scale(1.2); }
            }

            @keyframes pulse-yellow {
                0%, 100% { opacity: 1; transform: scale(1); }
                50% { opacity: 0.5; transform: scale(1.1); }
            }

            @keyframes pulse-red {
                0%, 100% { opacity: 1; transform: scale(1); }
                50% { opacity: 0.8; transform: scale(1.15); }
            }
        `;

        document.head.appendChild(styles);
    }

    setupEventListeners() {
        // Setup UI event listeners
        document.addEventListener('DOMContentLoaded', () => {
            this.bindUIEvents();
        });

        // If DOM is already loaded, bind events immediately
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => {
                this.bindUIEvents();
            });
        } else {
            setTimeout(() => this.bindUIEvents(), 100); // Small delay to ensure UI is rendered
        }

        // Setup Tauri event listeners for session management
        this.setupTauriEventListeners();
    }

    bindUIEvents() {
        const connectBtn = document.getElementById('connectBtn');
        const disconnectBtn = document.getElementById('disconnectBtn');
        const sessionIdInput = document.getElementById('sessionIdInput');

        if (connectBtn) {
            connectBtn.addEventListener('click', () => {
                this.connectToSession();
            });
        }

        if (disconnectBtn) {
            disconnectBtn.addEventListener('click', () => {
                this.disconnectFromSession();
            });
        }

        if (sessionIdInput) {
            sessionIdInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    this.connectToSession();
                }
            });

            // Auto-extract session ID from URLs
            sessionIdInput.addEventListener('paste', (e) => {
                setTimeout(() => {
                    const value = e.target.value;
                    const extractedId = this.extractSessionIdFromUrl(value);
                    if (extractedId && extractedId !== value) {
                        e.target.value = extractedId;
                    }
                }, 10);
            });
        }
    }

    async setupTauriEventListeners() {
        try {
            const { listen } = window.__TAURI__.event;

            // Session status updates
            await listen('session-status', (event) => {
                console.log('Session status:', event.payload);
                this.updateConnectionStatus(event.payload.status, event.payload);
            });

            // Session connected successfully
            await listen('session-connected', (event) => {
                console.log('Session connected:', event.payload);
                this.onSessionConnected(event.payload);
            });

            // Session disconnected
            await listen('session-disconnected', (event) => {
                console.log('Session disconnected:', event.payload);
                this.onSessionDisconnected(event.payload);
            });

            // Session errors
            await listen('session-error', (event) => {
                console.log('Session error:', event.payload);
                this.onSessionError(event.payload);
            });

            console.log('✅ Session event listeners setup successfully');
        } catch (error) {
            console.error('Failed to setup session event listeners:', error);
        }
    }

    extractSessionIdFromUrl(input) {
        // Extract session ID from various URL formats
        const patterns = [
            /\/session\/([a-f0-9-]{36})/i,  // /session/uuid
            /session[_-]?id[=:]([a-f0-9-]{36})/i, // session_id=uuid or sessionId:uuid
            /([a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12})/i // UUID pattern
        ];

        for (const pattern of patterns) {
            const match = input.match(pattern);
            if (match) {
                return match[1];
            }
        }

        return input.trim(); // Return as-is if no pattern matches
    }

    async connectToSession() {
        const sessionIdInput = document.getElementById('sessionIdInput');
        const connectBtn = document.getElementById('connectBtn');
        
        if (!sessionIdInput || !connectBtn) return;

        const sessionId = this.extractSessionIdFromUrl(sessionIdInput.value.trim());
        
        if (!sessionId) {
            this.showNotification('⚠️ Please enter a valid session ID', 'error');
            return;
        }

        // Validate UUID format
        const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
        if (!uuidPattern.test(sessionId)) {
            this.showNotification('❌ Invalid session ID format - must be a valid UUID', 'error');
            return;
        }

        try {
            // Show connecting state
            connectBtn.disabled = true;
            connectBtn.innerHTML = '<span class="material-icons">hourglass_empty</span><span>Connecting...</span>';
            this.updateConnectionStatus('connecting');
            this.showNotification('🔗 Connecting to session...', 'info');
            
            const { invoke } = window.__TAURI__.core;
            
            // First validate the session
            this.showNotification('🔍 Validating session ID...', 'info');
            const validationResult = await invoke('validate_session_id', { sessionId });
            
            if (!validationResult.valid) {
                throw new Error(validationResult.message);
            }
            
            this.showNotification('✅ Session validation successful!', 'success');
            
            // Connect to the session
            this.showNotification('🔌 Establishing connection...', 'info');
            const connectionResult = await invoke('connect_session', { sessionId });
            
            this.showNotification('✅ Session connected successfully!', 'success');
            
            // Activate the session
            this.showNotification('🚀 Activating session...', 'info');
            const activationResult = await invoke('activate_session', { sessionId });
            
            if (activationResult) {
                this.showNotification('🎯 Session activated! Main features are now available.', 'success', 5000);
                
                // Store session info and show main UI
                this.currentSession = connectionResult;
                this.onSessionConnected({ session: connectionResult });
            } else {
                throw new Error('Failed to activate session');
            }
            
        } catch (error) {
            console.error('Failed to connect to session:', error);
            const errorMessage = error.toString().replace('Error: ', '');
            this.showNotification(`❌ Connection failed: ${errorMessage}`, 'error', 8000);
            this.updateConnectionStatus('error', { error: errorMessage });
        } finally {
            connectBtn.disabled = false;
            connectBtn.innerHTML = '<span class="material-icons">link</span><span>Connect</span>';
        }
    }

    async disconnectFromSession() {
        if (!this.currentSession) {
            this.showNotification('⚠️ No active session to disconnect from', 'warning');
            return;
        }

        const disconnectBtn = document.getElementById('disconnectBtn');
        
        try {
            if (disconnectBtn) {
                disconnectBtn.disabled = true;
                disconnectBtn.innerHTML = '<span class="material-icons">hourglass_empty</span>Disconnecting...';
            }
            
            this.showNotification('🔌 Disconnecting from session...', 'info');
            this.updateConnectionStatus('connecting'); // Reuse connecting animation for disconnecting
            
            const { invoke } = window.__TAURI__.core;
            const sessionId = this.currentSession.session_id || this.currentSession.id;
            
            if (sessionId) {
                await invoke('disconnect_session', { sessionId });
            }
            
            this.showNotification('✅ Successfully disconnected from session', 'success');
            this.onSessionDisconnected({ reason: 'User initiated disconnect' });
            
        } catch (error) {
            console.error('Failed to disconnect from session:', error);
            const errorMessage = error.toString().replace('Error: ', '');
            this.showNotification(`❌ Failed to disconnect: ${errorMessage}`, 'error');
        } finally {
            if (disconnectBtn) {
                disconnectBtn.disabled = false;
                disconnectBtn.innerHTML = '<span class="material-icons">link_off</span>Disconnect';
            }
        }
    }

    updateConnectionStatus(status, data = {}) {
        this.connectionStatus = status;
        
        const indicator = document.getElementById('connectionIndicator');
        const statusDisplay = document.getElementById('sessionStatusDisplay');
        const connectionPanel = document.getElementById('sessionConnectionPanel');
        
        if (!indicator || !statusDisplay) return;

        // Remove existing status classes
        indicator.classList.remove('connecting', 'error');
        
        switch (status) {
            case 'connecting':
                indicator.classList.add('connecting');
                indicator.querySelector('.indicator-text').textContent = 'Connecting...';
                if (connectionPanel) connectionPanel.style.display = 'block';
                break;
                
            case 'connected':
                indicator.querySelector('.indicator-text').textContent = 'Connected';
                break;
                
            case 'disconnected':
                indicator.querySelector('.indicator-text').textContent = 'Disconnected';
                statusDisplay.style.display = 'none';
                if (connectionPanel) connectionPanel.style.display = 'none';
                this.currentSession = null;
                break;
                
            case 'error':
                indicator.classList.add('error');
                indicator.querySelector('.indicator-text').textContent = 'Error';
                if (connectionPanel) connectionPanel.style.display = 'block';
                break;
        }
    }

    onSessionConnected(data) {
        this.currentSession = data.session;
        
        const sessionTitle = document.getElementById('sessionTitle');
        const sessionMeta = document.getElementById('sessionMeta');
        const statusDisplay = document.getElementById('sessionStatusDisplay');
        const connectionPanel = document.getElementById('sessionConnectionPanel');
        
        if (sessionTitle && this.currentSession) {
            sessionTitle.textContent = this.currentSession.job_title || 'Interview Session';
        }
        
        if (sessionMeta && this.currentSession) {
            const metaInfo = [];
            if (this.currentSession.company_name) metaInfo.push(this.currentSession.company_name);
            if (this.currentSession.difficulty_level) metaInfo.push(`${this.currentSession.difficulty_level} difficulty`);
            if (this.currentSession.interview_type) metaInfo.push(this.currentSession.interview_type);
            
            sessionMeta.textContent = metaInfo.join(' • ');
        }
        
        if (statusDisplay) statusDisplay.style.display = 'block';
        if (connectionPanel) connectionPanel.style.display = 'block';
        
        this.updateConnectionStatus('connected', data);
        
        // Clear the session ID input
        const sessionIdInput = document.getElementById('sessionIdInput');
        if (sessionIdInput) sessionIdInput.value = '';
        
        // Show main application functionality after successful connection
        this.showMainApplicationUI();
        
        this.showNotification('Session connected! Main features are now available.', 'success');
    }

    onSessionDisconnected(data) {
        this.updateConnectionStatus('disconnected', data);
        
        // Hide main application functionality when disconnected
        this.hideMainApplicationUI();
        
        this.showNotification('Session disconnected. Please connect to a session to use main features.', 'info');
    }

    onSessionError(data) {
        this.updateConnectionStatus('error', data);
        
        if (data.error) {
            this.showNotification(data.error, 'error');
        }
    }

    showNotification(message, type = 'info', duration = 3000) {
        // Always use our custom toast system for better control
        console.log(`${type.toUpperCase()}: ${message}`);
        
        // Create a toast notification
        this.createToast(message, type, duration);
    }

    createToast(message, type, duration = 3000) {
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        
        // Create toast content with icon and message
        const toastContent = document.createElement('div');
        toastContent.style.cssText = 'display: flex; align-items: center; gap: 8px;';
        
        // Add icon based on type
        const icon = document.createElement('span');
        icon.className = 'toast-icon';
        switch(type) {
            case 'success':
                icon.textContent = '✅';
                break;
            case 'error':
                icon.textContent = '❌';
                break;
            case 'warning':
                icon.textContent = '⚠️';
                break;
            case 'info':
            default:
                icon.textContent = 'ℹ️';
                break;
        }
        
        const messageSpan = document.createElement('span');
        messageSpan.textContent = message;
        
        toastContent.appendChild(icon);
        toastContent.appendChild(messageSpan);
        toast.appendChild(toastContent);
        
        // Set background color based on type
        let backgroundColor;
        switch(type) {
            case 'success':
                backgroundColor = '#10B981'; // Green
                break;
            case 'error':
                backgroundColor = '#EF4444'; // Red
                break;
            case 'warning':
                backgroundColor = '#F59E0B'; // Orange
                break;
            case 'info':
            default:
                backgroundColor = '#3B82F6'; // Blue
                break;
        }
        
        toast.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 12px 16px;
            background: ${backgroundColor};
            color: white;
            border-radius: 8px;
            font-size: 13px;
            font-weight: 500;
            box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
            z-index: 10000;
            animation: slideInFromRight 0.3s ease;
            min-width: 250px;
            max-width: 400px;
            word-wrap: break-word;
            cursor: pointer;
            user-select: none;
        `;
        
        // Stack multiple toasts
        const existingToasts = document.querySelectorAll('.toast');
        if (existingToasts.length > 0) {
            const offset = existingToasts.length * 70; // 70px spacing between toasts
            toast.style.top = `${20 + offset}px`;
        }

        document.body.appendChild(toast);
        
        // Add click to dismiss
        toast.addEventListener('click', () => {
            this.dismissToast(toast);
        });

        // Auto dismiss after duration
        setTimeout(() => {
            this.dismissToast(toast);
        }, duration);
    }
    
    dismissToast(toast) {
        if (!toast || !toast.parentNode) return;
        
        toast.style.animation = 'slideOutToRight 0.3s ease';
        setTimeout(() => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
                
                // Reposition remaining toasts
                const remainingToasts = document.querySelectorAll('.toast');
                remainingToasts.forEach((t, index) => {
                    t.style.top = `${20 + (index * 70)}px`;
                });
            }
        }, 300);
    }

    // Public methods for external use
    isConnected() {
        return this.connectionStatus === 'connected' && this.currentSession !== null;
    }

    getCurrentSession() {
        return this.currentSession;
    }

    showConnectionUI() {
        const connectionPanel = document.getElementById('sessionConnectionPanel');
        if (connectionPanel) {
            connectionPanel.style.display = 'block';
        }
    }

    hideConnectionUI() {
        const connectionPanel = document.getElementById('sessionConnectionPanel');
        if (connectionPanel) {
            connectionPanel.style.display = 'none';
        }
    }
    
    // Show main application UI after session connection
    showMainApplicationUI() {
        console.log('🔗 Session connected - showing main application UI');
        
        // Adjust window size for connected state
        document.body.classList.remove('session-welcome');
        document.body.classList.add('session-connected');
        
        // Hide first run screen
        const firstRunScreen = document.getElementById('sessionFirstRun');
        if (firstRunScreen) {
            firstRunScreen.classList.add('session-connected');
        }
        
        // Show main controls and functionality
        const mainControlsBar = document.getElementById('mainControlsBar');
        const transcriptionContainer = document.getElementById('transcriptionContainer');
        const inputSection = document.getElementById('inputSection');
        
        if (mainControlsBar) {
            mainControlsBar.classList.add('session-connected');
        }
        
        if (transcriptionContainer) {
            transcriptionContainer.classList.add('session-connected');
        }
        
        if (inputSection) {
            inputSection.classList.add('session-connected');
        }
        
        // Hide session connection panel
        this.hideConnectionUI();
        
        console.log('✅ Main application UI is now visible');
    }
    
    // Hide main application UI when session is disconnected
    hideMainApplicationUI() {
        console.log('🔗 Session disconnected - hiding main application UI');
        
        // Adjust window size for welcome state
        document.body.classList.remove('session-connected');
        document.body.classList.add('session-welcome');
        
        // Show first run screen
        const firstRunScreen = document.getElementById('sessionFirstRun');
        if (firstRunScreen) {
            firstRunScreen.classList.remove('session-connected');
        }
        
        // Hide main controls and functionality
        const mainControlsBar = document.getElementById('mainControlsBar');
        const transcriptionContainer = document.getElementById('transcriptionContainer');
        const inputSection = document.getElementById('inputSection');
        
        if (mainControlsBar) {
            mainControlsBar.classList.remove('session-connected');
        }
        
        if (transcriptionContainer) {
            transcriptionContainer.classList.remove('session-connected');
        }
        
        if (inputSection) {
            inputSection.classList.remove('session-connected');
        }
        
        // Show session connection panel for reconnection
        this.showConnectionUI();
        
        console.log('✅ Main application UI is now hidden - first run screen visible');
    }
    
    // Ensure first run state is properly set
    ensureFirstRunState() {
        console.log('🎆 Ensuring first run state...');
        
        if (!this.isConnected()) {
            console.log('Not connected - showing first run experience');
            
            // Make sure body has welcome class
            document.body.classList.remove('session-connected');
            document.body.classList.add('session-welcome');
            
            // Show session connection panel
            this.showConnectionUI();
            
            // Hide main UI elements
            this.hideMainApplicationUI();
            
            console.log('✅ First run state ensured');
        }
    }
}

// Initialize session connection manager
window.sessionConnectionManager = new SessionConnectionManager();

// Add toast animation styles
const toastStyles = document.createElement('style');
toastStyles.textContent = `
    @keyframes slideInFromRight {
        from { transform: translateX(100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
    
    @keyframes slideOutToRight {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(100%); opacity: 0; }
    }
`;
document.head.appendChild(toastStyles);

console.log('✅ Session Connection Manager initialized');
