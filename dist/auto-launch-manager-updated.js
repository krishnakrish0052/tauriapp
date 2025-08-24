// Auto Launch Manager - Enhanced for temporary token authentication
// Handles automatic session filling and connection using new secure temp tokens

class AutoLaunchManager {
    constructor() {
        this.urlParams = new URLSearchParams(window.location.search);
        this.launchData = null;
        this.sessionId = null;
        this.tempToken = null;
        this.autoFill = false;
        this.autoConnect = false;
        this.tempTokenGenerated = false;
        
        console.log('🚀 Auto Launch Manager initialized (Enhanced)');
        this.initialize();
    }

    initialize() {
        // Extract session ID and temp token from URL
        this.extractSessionFromURL();
        
        // Read launch parameters from URL
        this.readURLParameters();
        
        // Read launch data from localStorage (set by web app)
        this.readLaunchData();
        
        // Log what we found
        this.logLaunchInfo();
        
        // Auto-fill session if requested
        if (this.sessionId && this.autoFill) {
            this.performAutoFill();
        }
        
        // Auto-connect with temp token if available
        if (this.sessionId && this.tempToken && this.autoConnect) {
            setTimeout(() => {
                this.performAutoConnectWithTempToken();
            }, 1000);
        }
        // Fallback to manual connection UI if no temp token
        else if (this.sessionId && this.autoConnect) {
            setTimeout(() => {
                this.performAutoConnect();
            }, 1000);
        }
    }

    extractSessionFromURL() {
        // Check if we were launched via protocol (mockmate://session/ID)
        const hash = window.location.hash;
        const pathname = window.location.pathname;
        const href = window.location.href;
        
        console.log('🔍 Checking URL for session ID and temp token...');
        console.log('Hash:', hash);
        console.log('Pathname:', pathname);
        console.log('Full URL (masked):', href.replace(/temp_token=[^&]+/, 'temp_token=***'));
        
        // Try different patterns to extract session ID
        const patterns = [
            /mockmate:\/\/session\/([a-f0-9-]{36})/i,  // Full protocol URL
            /#\/session\/([a-f0-9-]{36})/i,            // Hash-based routing
            /\/session\/([a-f0-9-]{36})/i,             // Path-based routing
            /session[=\/]([a-f0-9-]{36})/i             // Query parameter style
        ];
        
        for (const pattern of patterns) {
            const match = href.match(pattern);
            if (match) {
                this.sessionId = match[1];
                console.log('✅ Found session ID from URL:', this.sessionId);
                break;
            }
        }
        
        if (!this.sessionId) {
            console.log('ℹ️ No session ID found in URL');
        }
    }

    readURLParameters() {
        // Read auto-fill and auto-connect flags from URL parameters
        this.autoFill = this.urlParams.get('auto_fill') === 'true';
        this.autoConnect = this.urlParams.get('auto_connect') === 'true';
        
        // Extract temporary token from URL parameters
        this.tempToken = this.urlParams.get('temp_token');
        if (this.tempToken) {
            console.log('🔑 Found temporary token from URL:', this.tempToken.substring(0, 8) + '...');
        }
        
        // Also check for session ID in query parameters
        if (!this.sessionId) {
            this.sessionId = this.urlParams.get('session_id') || this.urlParams.get('sessionId');
        }
        
        console.log('🔧 URL Parameters:');
        console.log('  auto_fill:', this.autoFill);
        console.log('  auto_connect:', this.autoConnect);
        console.log('  temp_token present:', !!this.tempToken);
        console.log('  session_id from params:', this.urlParams.get('session_id'));
    }

    readLaunchData() {
        try {
            const launchDataStr = localStorage.getItem('mockmate_launch_data');
            if (launchDataStr) {
                this.launchData = JSON.parse(launchDataStr);
                console.log('📋 Launch data from localStorage:', this.launchData);
                
                // Use launch data if we don't have values from URL
                if (!this.sessionId && this.launchData.sessionId) {
                    this.sessionId = this.launchData.sessionId;
                    console.log('📝 Using session ID from launch data:', this.sessionId);
                }
                
                if (!this.autoFill && this.launchData.autoFill) {
                    this.autoFill = this.launchData.autoFill;
                    console.log('📝 Using auto-fill from launch data:', this.autoFill);
                }
                
                if (!this.autoConnect && this.launchData.autoConnect) {
                    this.autoConnect = this.launchData.autoConnect;
                    console.log('📝 Using auto-connect from launch data:', this.autoConnect);
                }
                
                // Check if temp token was generated (new authentication method)
                if (this.launchData.tempTokenGenerated) {
                    this.tempTokenGenerated = true;
                    console.log('🔐 Temp token was generated for this session');
                }
                
                // Check if launch data is recent (within last 5 minutes)
                const launchAge = Date.now() - (this.launchData.timestamp || 0);
                if (launchAge > 5 * 60 * 1000) { // 5 minutes
                    console.log('⏰ Launch data is old, clearing it');
                    localStorage.removeItem('mockmate_launch_data');
                    this.launchData = null;
                }
            }
        } catch (error) {
            console.warn('⚠️ Failed to read launch data:', error);
            localStorage.removeItem('mockmate_launch_data');
        }
    }

    logLaunchInfo() {
        console.log('\n🎯 AUTO LAUNCH SUMMARY (Enhanced):');
        console.log('='.repeat(40));
        console.log(`Session ID: ${this.sessionId || 'Not found'}`);
        console.log(`Temp Token: ${this.tempToken ? this.tempToken.substring(0, 8) + '...' : 'Not available'}`);
        console.log(`Auto-fill: ${this.autoFill ? '✅ Enabled' : '❌ Disabled'}`);
        console.log(`Auto-connect: ${this.autoConnect ? '✅ Enabled' : '❌ Disabled'}`);
        console.log(`Launch data: ${this.launchData ? '✅ Available' : '❌ None'}`);
        console.log(`Authentication: ${this.tempToken ? '🔐 Secure (Temp Token)' : '⚠️ Legacy/Manual'}`);
        console.log('='.repeat(40));
    }

    performAutoFill() {
        if (!this.sessionId) {
            console.warn('⚠️ Cannot auto-fill: No session ID available');
            return false;
        }
        
        console.log('📝 Auto-filling session ID:', this.sessionId);
        
        // Wait for the session input field to be available
        const fillAttempts = (attempts = 0) => {
            const sessionInput = document.getElementById('sessionIdInput');
            
            if (sessionInput) {
                sessionInput.value = this.sessionId;
                sessionInput.dispatchEvent(new Event('input', { bubbles: true }));
                sessionInput.dispatchEvent(new Event('change', { bubbles: true }));
                
                console.log('✅ Session ID auto-filled successfully');
                
                // Visual feedback - green glow for temp token, blue for manual
                const glowColor = this.tempToken ? 'rgba(0, 255, 0, 0.3)' : 'rgba(0, 150, 255, 0.3)';
                sessionInput.style.background = glowColor;
                sessionInput.style.borderColor = this.tempToken ? 'rgba(0, 255, 0, 0.8)' : 'rgba(0, 150, 255, 0.8)';
                sessionInput.style.boxShadow = this.tempToken ? '0 0 10px rgba(0, 255, 0, 0.5)' : '0 0 10px rgba(0, 150, 255, 0.5)';
                
                setTimeout(() => {
                    sessionInput.style.background = '';
                    sessionInput.style.borderColor = '';
                    sessionInput.style.boxShadow = '';
                }, 3000);
                
                return true;
            }
            
            if (attempts < 20) { // Try for 4 seconds
                setTimeout(() => fillAttempts(attempts + 1), 200);
            } else {
                console.error('❌ Failed to find session input field for auto-fill');
                return false;
            }
        };
        
        fillAttempts();
    }

    async performAutoConnectWithTempToken() {
        if (!this.sessionId || !this.tempToken) {
            console.warn('⚠️ Cannot auto-connect with temp token: Missing session ID or temp token');
            return false;
        }
        
        console.log('🔐 Auto-connecting with temporary token for session:', this.sessionId);
        
        try {
            // Show connecting notification
            this.showNotification('🔐 Connecting with secure token...', 'info');
            
            // Call the new temp token authentication API
            const response = await this.authenticateWithTempToken();
            
            if (response.success) {
                console.log('✅ Successfully authenticated and activated session with temp token');
                
                // Update UI to show connected/activated state
                this.handleSuccessfulConnection(response.session);
                
                // Show success notification
                this.showNotification(`✅ Connected to "${response.session.jobTitle}"`, 'success');
                
                // Clear the temp token for security
                this.tempToken = null;
                localStorage.removeItem('mockmate_launch_data');
                
                return true;
            } else {
                throw new Error(response.error || 'Authentication failed');
            }
            
        } catch (error) {
            console.error('❌ Auto-connect with temp token failed:', error);
            this.showNotification(`❌ Auto-connect failed: ${error.message}`, 'error');
            
            // Fall back to manual connection
            console.log('🔄 Falling back to manual connection...');
            this.performAutoConnect();
            return false;
        }
    }

    async authenticateWithTempToken() {
        console.log('📡 Calling temp token authentication API...');
        
        try {
            const response = await fetch(`http://localhost:5000/api/sessions/${this.sessionId}/connect-with-temp-token`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    tempToken: this.tempToken,
                    desktop_version: '1.0.0',
                    platform: 'windows'
                })
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || `HTTP ${response.status}`);
            }

            const data = await response.json();
            console.log('✅ Temp token authentication successful:', data);
            return data;
            
        } catch (error) {
            console.error('❌ Temp token authentication request failed:', error);
            throw error;
        }
    }

    handleSuccessfulConnection(sessionData) {
        // Store session data globally for other components
        window.currentSession = sessionData;
        window.isSessionActive = true;
        
        // Update UI elements to show connected state
        const sessionPanel = document.getElementById('sessionConnectionPanel');
        if (sessionPanel) {
            // Create connected state UI
            sessionPanel.innerHTML = `
                <div class="session-connected-content">
                    <div class="session-connection-header">
                        <div class="session-header-content">
                            <div class="session-brand">
                                <div class="logo">
                                    <img src="mockmate-logo.png" alt="MockMate" style="width: 40px; height: 40px; border-radius: 8px;">
                                </div>
                                <div>
                                    <h2 style="margin: 0; font-size: 24px; font-weight: 600;">
                                        <span style="color: #ffffff;">Mock</span><span style="color: #ffd700;">Mate</span>
                                    </h2>
                                    <p style="margin: 0; font-size: 16px; color: var(--text-muted);">Connected & Active</p>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <div class="session-main-content">
                        <div class="session-left-panel">
                            <div class="job-title">* ${sessionData.jobTitle}</div>
                            <div class="session-info">Session: ${this.sessionId.substring(0, 8)}... | Credits: ${sessionData.user.credits}</div>
                            <div class="session-status">
                                <span class="status-indicator active"></span>
                                Interview Active - Ready for Questions
                            </div>
                        </div>
                    </div>
                </div>
            `;
        }
        
        // Update body class to show active state
        document.body.className = 'session-active';
        
        // Dispatch custom events for other components
        document.dispatchEvent(new CustomEvent('sessionConnected', {
            detail: sessionData
        }));
        
        document.dispatchEvent(new CustomEvent('sessionActivated', {
            detail: sessionData
        }));
    }

    performAutoConnect() {
        if (!this.sessionId) {
            console.warn('⚠️ Cannot auto-connect: No session ID available');
            return false;
        }
        
        console.log('🔗 Auto-connecting to session (manual flow):', this.sessionId);
        
        // Wait for the connect button to be available
        const connectAttempts = (attempts = 0) => {
            const connectBtn = document.getElementById('connectBtn');
            const sessionInput = document.getElementById('sessionIdInput');
            
            if (connectBtn && sessionInput && sessionInput.value) {
                console.log('🎯 Triggering auto-connect...');
                
                // Ensure session ID is filled
                if (sessionInput.value !== this.sessionId) {
                    sessionInput.value = this.sessionId;
                }
                
                // Trigger the connection
                connectBtn.click();
                
                console.log('✅ Auto-connect triggered successfully');
                return true;
            }
            
            if (attempts < 30) { // Try for 6 seconds
                setTimeout(() => connectAttempts(attempts + 1), 200);
            } else {
                console.error('❌ Failed to find connect button for auto-connect');
                this.showNotification('Please connect manually - auto-connect failed', 'warning');
                return false;
            }
        };
        
        connectAttempts();
    }

    showNotification(message, type = 'info') {
        // Create or update notification element
        let notification = document.getElementById('auto-launch-notification');
        if (!notification) {
            notification = document.createElement('div');
            notification.id = 'auto-launch-notification';
            notification.style.cssText = `
                position: fixed;
                top: 20px;
                right: 20px;
                padding: 12px 16px;
                border-radius: 8px;
                font-size: 14px;
                font-weight: 500;
                z-index: 10000;
                transition: all 0.3s ease;
                max-width: 300px;
                word-wrap: break-word;
            `;
            document.body.appendChild(notification);
        }
        
        // Set styles based on type
        const styles = {
            info: 'background: rgba(59, 130, 246, 0.9); color: white; border: 1px solid rgba(59, 130, 246, 0.5);',
            success: 'background: rgba(16, 185, 129, 0.9); color: white; border: 1px solid rgba(16, 185, 129, 0.5);',
            warning: 'background: rgba(245, 158, 11, 0.9); color: white; border: 1px solid rgba(245, 158, 11, 0.5);',
            error: 'background: rgba(239, 68, 68, 0.9); color: white; border: 1px solid rgba(239, 68, 68, 0.5);'
        };
        
        notification.style.cssText += styles[type] || styles.info;
        notification.textContent = message;
        
        // Auto-hide after 5 seconds for success/info, 10 seconds for warnings/errors
        const hideDelay = (type === 'success' || type === 'info') ? 5000 : 10000;
        setTimeout(() => {
            if (notification && notification.parentNode) {
                notification.style.opacity = '0';
                setTimeout(() => {
                    if (notification && notification.parentNode) {
                        notification.parentNode.removeChild(notification);
                    }
                }, 300);
            }
        }, hideDelay);
        
        console.log(`[${type.toUpperCase()}] ${message}`);
    }
}

// Initialize the enhanced auto launch manager when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.autoLaunchManager = new AutoLaunchManager();
});

// Also initialize immediately if DOM is already loaded
if (document.readyState === 'loading') {
    // DOM is still loading
} else {
    // DOM is already loaded
    window.autoLaunchManager = new AutoLaunchManager();
}
