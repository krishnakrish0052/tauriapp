// Session Launch Handler
// Handles auto-fill and auto-connect functionality for the MockMate desktop app

/**
 * Sets up protocol launch handling for the desktop app
 * @param {object} sessionManager - The session manager instance to interact with
 */
export function setupProtocolLaunchHandler(sessionManager) {
    try {
        console.log('🚀 Setting up protocol launch handler...');
        
        // Listen for deep link protocol events from Tauri backend
        const { listen } = window.__TAURI__.event;
        
        // Listen for session-launch events
        listen('session-launch', (event) => {
            console.log('📡 Protocol launch event received:', event.payload);
            handleProtocolLaunch(event.payload, sessionManager);
        });
        
        // Also check URL parameters for direct launches or testing
        checkUrlForSessionParams(sessionManager);
        
        console.log('✅ Protocol launch handler setup complete');
    } catch (error) {
        console.error('❌ Failed to setup protocol launch handler:', error);
    }
}

/**
 * Handles a protocol launch event with session data
 * @param {object} data - The session launch data from protocol URL
 * @param {object} sessionManager - The session manager to perform connection with
 */
function handleProtocolLaunch(data, sessionManager) {
    try {
        const { session_id, token, user_id } = data;
        
        if (!session_id) {
            console.error('❌ No session ID in protocol launch data');
            return;
        }
        
        console.log('🔗 Handling protocol launch for session:', session_id);
        
        // Find the session ID input field
        const sessionIdInput = document.getElementById('sessionIdInput');
        if (sessionIdInput) {
            // Auto-fill the session ID input
            sessionIdInput.value = session_id;
            console.log('✅ Auto-filled session ID:', session_id);
            
            // Auto-focus the input field
            sessionIdInput.focus();
        }
        
        // Show a notification about auto-connecting
        if (sessionManager.showNotification) {
            sessionManager.showNotification('🚀 Auto-connecting from web app launch...', 'info');
        }
        
        // Automatically connect to the session after a short delay
        setTimeout(() => {
            if (sessionManager.connectToSession) {
                console.log('🔄 Auto-connecting to session...');
                sessionManager.connectToSession();
            } else {
                console.error('❌ Session manager does not have connectToSession method');
                
                // Try to click the connect button as fallback
                const connectButton = document.getElementById('connectBtn');
                if (connectButton) {
                    console.log('🖱️ Clicking connect button as fallback...');
                    connectButton.click();
                }
            }
        }, 500); // Half-second delay to ensure UI is ready
        
    } catch (error) {
        console.error('❌ Failed to handle protocol launch:', error);
        if (sessionManager.showNotification) {
            sessionManager.showNotification(`Protocol launch failed: ${error.message}`, 'error');
        }
    }
}

/**
 * Checks URL parameters for session ID and other parameters
 * Useful for direct launches or testing
 * @param {object} sessionManager - The session manager instance
 */
function checkUrlForSessionParams(sessionManager) {
    try {
        // Check if URL has session parameters
        const urlParams = new URLSearchParams(window.location.search);
        const sessionId = urlParams.get('session');
        const token = urlParams.get('token');
        const userId = urlParams.get('user_id');
        
        // If all required parameters are present, handle as protocol launch
        if (sessionId && token && userId) {
            console.log('📋 Found session data in URL parameters:', { sessionId, userId });
            
            // Create payload similar to protocol launch event
            const payload = {
                session_id: sessionId,
                token: token,
                user_id: userId
            };
            
            // Handle as protocol launch
            handleProtocolLaunch(payload, sessionManager);
        }
    } catch (error) {
        console.error('❌ Failed to check URL parameters:', error);
    }
}

/**
 * Initialize protocol handling for a specific session manager instance
 * @param {object} sessionManager - The session connection manager instance
 */
export function initProtocolHandling(sessionManager) {
    // Setup the protocol handler
    setupProtocolLaunchHandler(sessionManager);
    
    // Add the protocol handler methods to the session manager
    sessionManager.handleProtocolLaunch = (data) => handleProtocolLaunch(data, sessionManager);
    sessionManager.checkUrlForSessionParams = () => checkUrlForSessionParams(sessionManager);
    
    // Return the enhanced session manager
    return sessionManager;
}

export default {
    setupProtocolLaunchHandler,
    handleProtocolLaunch,
    checkUrlForSessionParams,
    initProtocolHandling
};
