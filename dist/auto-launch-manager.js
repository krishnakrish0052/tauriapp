// Auto Launch Manager - Handles automatic session filling and connection
// Reads URL parameters and localStorage to auto-fill session ID and auto-connect

class AutoLaunchManager {
    constructor() {
        this.urlParams = new URLSearchParams(window.location.search);
        this.launchData = null;
        this.sessionId = null;
        this.autoFill = false;
        this.autoConnect = false;
        
        console.log('🚀 Auto Launch Manager initialized');
        this.initialize();
    }

    initialize() {
        // Extract session ID from URL hash (mockmate://session/ID)
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
        
        // Auto-connect if requested (after a brief delay for UI to load)
        if (this.sessionId && this.autoConnect) {
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
        
        console.log('🔍 Checking URL for session ID...');
        console.log('Hash:', hash);
        console.log('Pathname:', pathname);
        console.log('Full URL:', href);
        
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
                return;
            }
        }
        
        console.log('ℹ️ No session ID found in URL');
    }

    readURLParameters() {
        // Read auto-fill and auto-connect flags from URL parameters
        this.autoFill = this.urlParams.get('auto_fill') === 'true';
        this.autoConnect = this.urlParams.get('auto_connect') === 'true';
        
        // Also check for session ID in query parameters
        if (!this.sessionId) {
            this.sessionId = this.urlParams.get('session_id') || this.urlParams.get('sessionId');
        }
        
        console.log('🔧 URL Parameters:');
        console.log('  auto_fill:', this.autoFill);
        console.log('  auto_connect:', this.autoConnect);
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
        console.log('\n🎯 AUTO LAUNCH SUMMARY:');
        console.log('='.repeat(30));
        console.log(`Session ID: ${this.sessionId || 'Not found'}`);
        console.log(`Auto-fill: ${this.autoFill ? '✅ Enabled' : '❌ Disabled'}`);
        console.log(`Auto-connect: ${this.autoConnect ? '✅ Enabled' : '❌ Disabled'}`);
        console.log(`Launch data: ${this.launchData ? '✅ Available' : '❌ None'}`);
        console.log('='.repeat(30));
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
                
                // Visual feedback
                sessionInput.style.background = 'rgba(0, 255, 0, 0.1)';
                sessionInput.style.borderColor = 'rgba(0, 255, 0, 0.5)';
                
                setTimeout(() => {
                    sessionInput.style.background = '';
                    sessionInput.style.borderColor = '';
                }, 2000);
                
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

    performAutoConnect() {
        if (!this.sessionId) {
            console.warn('⚠️ Cannot auto-connect: No session ID available');
            return false;
        }
        
        console.log('🔗 Auto-connecting to session:', this.sessionId);
        
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
                
                // Visual feedback on the button
                const originalText = connectBtn.innerHTML;
                connectBtn.innerHTML = '<span class="material-icons">hourglass_empty</span>Auto-connecting...';
                connectBtn.style.background = 'rgba(0, 255, 0, 0.2)';
                
                setTimeout(() => {
                    if (connectBtn.innerHTML.includes('Auto-connecting')) {
                        connectBtn.innerHTML = originalText;
                        connectBtn.style.background = '';
                    }
                }, 3000);
                
                // Clear launch data after successful use
                this.clearLaunchData();
                
                return true;
            }
            
            if (attempts < 25) { // Try for 5 seconds
                setTimeout(() => connectAttempts(attempts + 1), 200);
            } else {
                console.error('❌ Failed to find connect button or session input for auto-connect');
                return false;
            }
        };
        
        connectAttempts();
    }

    clearLaunchData() {
        try {
            localStorage.removeItem('mockmate_launch_data');
            console.log('🧹 Launch data cleared from localStorage');
        } catch (error) {
            console.warn('⚠️ Failed to clear launch data:', error);
        }
    }

    // Public API for manual operations
    getSessionId() {
        return this.sessionId;
    }

    isAutoFillEnabled() {
        return this.autoFill;
    }

    isAutoConnectEnabled() {
        return this.autoConnect;
    }

    getLaunchData() {
        return this.launchData;
    }

    // Manually trigger auto-fill
    manualAutoFill() {
        return this.performAutoFill();
    }

    // Manually trigger auto-connect
    manualAutoConnect() {
        return this.performAutoConnect();
    }

    // Update session ID and optionally auto-fill
    updateSessionId(sessionId, shouldAutoFill = false) {
        this.sessionId = sessionId;
        console.log('📝 Session ID updated:', sessionId);
        
        if (shouldAutoFill) {
            this.performAutoFill();
        }
    }
}

// Initialize auto launch manager when page loads
let autoLaunchManager = null;

// Wait for DOM to be ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        autoLaunchManager = new AutoLaunchManager();
        window.autoLaunchManager = autoLaunchManager;
    });
} else {
    // DOM is already ready
    autoLaunchManager = new AutoLaunchManager();
    window.autoLaunchManager = autoLaunchManager;
}

console.log('✅ Auto Launch Manager module loaded');
