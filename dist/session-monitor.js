// Session Completion Monitor
// Monitors session status in real-time and handles automatic app closure

class SessionCompletionMonitor {
    constructor() {
        this.currentSession = null;
        this.authToken = null;
        this.apiBaseUrl = 'http://localhost:3001/api';
        this.monitorInterval = null;
        this.isMonitoring = false;
        this.checkIntervalMs = 5000; // Check every 5 seconds
        this.lastKnownStatus = null;
        this.onSessionCompletedCallback = null;
        this.onSessionStoppedCallback = null;
        
        console.log('👁️ Session Completion Monitor initialized');
    }

    // Initialize monitoring for a session
    initialize(sessionData, token, callbacks = {}) {
        this.currentSession = sessionData;
        this.authToken = token;
        this.lastKnownStatus = sessionData.status;
        this.onSessionCompletedCallback = callbacks.onCompleted || null;
        this.onSessionStoppedCallback = callbacks.onStopped || null;
        
        // Get session ID from either sessionData.id or sessionData.session_id
        const sessionId = sessionData.session_id || sessionData.id;
        console.log('🔍 Session Monitor initialized for:', sessionId, 'Status:', sessionData.status);
        
        // For desktop app, disable API monitoring since we don't have a backend API running
        console.log('📱 Desktop app mode - API monitoring disabled');
        console.log('✅ Session Monitor ready for local callbacks only');
    }

    // Start real-time monitoring
    startMonitoring() {
        if (this.isMonitoring) {
            console.log('⚠️ Session monitoring already active');
            return;
        }

        if (!this.currentSession || !this.authToken) {
            console.error('❌ Cannot start monitoring: missing session or auth token');
            return;
        }

        this.isMonitoring = true;
        console.log('📱 Desktop app session monitoring initialized (API calls disabled)');
        console.log('✅ Session monitoring ready for local callbacks only');
        
        // For desktop app, we don't need to poll an API since we manage the session locally
        // The monitoring is just to provide the callback interface for session lifecycle events
    }

    // Stop monitoring
    stopMonitoring() {
        if (this.monitorInterval) {
            clearInterval(this.monitorInterval);
            this.monitorInterval = null;
        }
        
        this.isMonitoring = false;
        console.log('⏹️ Session monitoring stopped');
    }

    // Check session status from the server
    async checkSessionStatus() {
        if (!this.currentSession || !this.authToken) {
            console.log('⚠️ No session data available for status check');
            return;
        }

        try {
            const response = await fetch(`${this.apiBaseUrl}/sessions/${this.currentSession.id}/status`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.authToken}`,
                    'Content-Type': 'application/json'
                }
            });

            if (!response.ok) {
                if (response.status === 404) {
                    console.log('🚨 Session no longer exists on server');
                    await this.handleSessionDeleted();
                    return;
                }
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const statusData = await response.json();
            await this.processStatusUpdate(statusData);

        } catch (error) {
            console.error('❌ Failed to check session status:', error);
            
            // Don't stop monitoring on network errors, just log them
            if (error.message.includes('fetch')) {
                console.log('📡 Network error - will retry on next check');
            }
        }
    }

    // Process status update from server
    async processStatusUpdate(statusData) {
        const { 
            status, 
            active, 
            stoppedExternally, 
            desktopConnected,
            endedAt,
            timestamp
        } = statusData;

        // Log status if it changed
        if (status !== this.lastKnownStatus) {
            console.log(`📊 Session status changed: ${this.lastKnownStatus} → ${status}`);
            this.lastKnownStatus = status;
        }

        // Check for completion
        if (status === 'completed') {
            console.log('🎯 Session completed externally!');
            await this.handleSessionCompleted(statusData);
            return;
        }

        // Check for cancellation
        if (status === 'cancelled') {
            console.log('❌ Session cancelled externally!');
            await this.handleSessionCancelled(statusData);
            return;
        }

        // Check if stopped externally while we thought it was active
        if (stoppedExternally && this.lastKnownStatus === 'active') {
            console.log('⏹️ Session stopped externally while active!');
            await this.handleSessionStopped(statusData);
            return;
        }

        // Check for desktop disconnection
        if (!desktopConnected && this.lastKnownStatus === 'active') {
            console.log('📱 Desktop connection lost');
            // Don't close automatically for connection loss, just log it
        }

        // Update session data
        this.currentSession = { ...this.currentSession, ...statusData };
    }

    // Handle session completion
    async handleSessionCompleted(statusData) {
        console.log('✅ Handling session completion...');
        
        // Stop monitoring
        this.stopMonitoring();
        
        // Show user notification
        await this.showNotification(
            'Session Completed', 
            'This interview session has been completed. The application will close automatically.',
            'success'
        );

        // Call completion callback if provided
        if (this.onSessionCompletedCallback) {
            try {
                await this.onSessionCompletedCallback(statusData);
            } catch (error) {
                console.error('❌ Error in session completion callback:', error);
            }
        }

        // Wait a moment for user to see the notification
        setTimeout(() => {
            this.closeApplication('Session completed');
        }, 3000);
    }

    // Handle session cancellation
    async handleSessionCancelled(statusData) {
        console.log('❌ Handling session cancellation...');
        
        // Stop monitoring
        this.stopMonitoring();
        
        // Show user notification
        await this.showNotification(
            'Session Cancelled', 
            'This interview session has been cancelled. The application will close automatically.',
            'warning'
        );

        // Call stopped callback if provided
        if (this.onSessionStoppedCallback) {
            try {
                await this.onSessionStoppedCallback(statusData, 'cancelled');
            } catch (error) {
                console.error('❌ Error in session stopped callback:', error);
            }
        }

        // Wait a moment for user to see the notification
        setTimeout(() => {
            this.closeApplication('Session cancelled');
        }, 3000);
    }

    // Handle session stopped externally
    async handleSessionStopped(statusData) {
        console.log('⏹️ Handling session stopped externally...');
        
        // Stop monitoring
        this.stopMonitoring();
        
        // Show user notification
        await this.showNotification(
            'Session Stopped', 
            'This interview session has been stopped from the web interface. The application will close automatically.',
            'info'
        );

        // Call stopped callback if provided
        if (this.onSessionStoppedCallback) {
            try {
                await this.onSessionStoppedCallback(statusData, 'stopped');
            } catch (error) {
                console.error('❌ Error in session stopped callback:', error);
            }
        }

        // Wait a moment for user to see the notification
        setTimeout(() => {
            this.closeApplication('Session stopped externally');
        }, 3000);
    }

    // Handle session deletion
    async handleSessionDeleted() {
        console.log('🗑️ Session deleted from server');
        
        // Stop monitoring
        this.stopMonitoring();
        
        // Show user notification
        await this.showNotification(
            'Session Deleted', 
            'This interview session has been deleted. The application will close automatically.',
            'error'
        );

        // Wait a moment for user to see the notification
        setTimeout(() => {
            this.closeApplication('Session deleted');
        }, 2000);
    }

    // Show notification to user
    async showNotification(title, message, type = 'info') {
        console.log(`📢 ${type.toUpperCase()}: ${title} - ${message}`);
        
        // Try to use the main app's notification system if available
        if (window.mockMateController && window.mockMateController.showNotification) {
            window.mockMateController.showNotification(`${title}: ${message}`, type);
        }
        
        // Also try to use system notifications if available
        if ('Notification' in window && Notification.permission === 'granted') {
            new Notification(title, {
                body: message,
                icon: '/assets/icon.png' // Adjust path as needed
            });
        }
        
        // Fallback: Log prominently
        console.log(`\n${'='.repeat(60)}\n📢 ${title}\n${message}\n${'='.repeat(60)}\n`);
    }

    // Close the application
    async closeApplication(reason) {
        console.log(`🚪 Closing application: ${reason}`);
        
        try {
            // Try to close via the main controller first
            if (window.mockMateController && window.mockMateController.closeApplication) {
                await window.mockMateController.closeApplication();
                return;
            }
            
            // Try direct Tauri close
            if (window.__TAURI__ && window.__TAURI__.window) {
                await window.__TAURI__.window.appWindow.close();
                return;
            }
            
            // Try via safeInvoke
            if (window.safeInvoke) {
                await window.safeInvoke('close_application');
                return;
            }
            
            console.error('❌ No method available to close application');
            
        } catch (error) {
            console.error('❌ Failed to close application:', error);
            
            // Last resort: try to force close
            if (window.__TAURI__ && window.__TAURI__.window) {
                try {
                    await window.__TAURI__.window.appWindow.close();
                } catch (forceError) {
                    console.error('❌ Force close also failed:', forceError);
                }
            }
        }
    }

    // Update monitoring frequency
    setCheckInterval(intervalMs) {
        if (intervalMs < 1000) {
            console.warn('⚠️ Minimum check interval is 1 second');
            intervalMs = 1000;
        }
        
        if (intervalMs > 60000) {
            console.warn('⚠️ Maximum check interval is 60 seconds');
            intervalMs = 60000;
        }
        
        this.checkIntervalMs = intervalMs;
        
        // Restart monitoring with new interval if currently active
        if (this.isMonitoring) {
            this.stopMonitoring();
            this.startMonitoring();
        }
        
        console.log(`⏱️ Check interval updated to ${intervalMs/1000} seconds`);
    }

    // Get current monitoring status
    getMonitoringStatus() {
        return {
            isMonitoring: this.isMonitoring,
            currentSession: this.currentSession?.id || null,
            lastKnownStatus: this.lastKnownStatus,
            checkInterval: this.checkIntervalMs,
            hasAuthToken: !!this.authToken
        };
    }

    // Manual status check (for testing)
    async forceStatusCheck() {
        console.log('🔄 Forcing status check...');
        await this.checkSessionStatus();
    }

    // Cleanup when shutting down
    cleanup() {
        console.log('🧹 Cleaning up Session Monitor');
        this.stopMonitoring();
        
        // Reset state
        this.currentSession = null;
        this.authToken = null;
        this.lastKnownStatus = null;
        this.onSessionCompletedCallback = null;
        this.onSessionStoppedCallback = null;
    }
}

// Global instance
window.sessionCompletionMonitor = new SessionCompletionMonitor();

console.log('✅ Session Completion Monitor loaded successfully');
