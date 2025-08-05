// Example usage of the system sound capturing functionality
// This file demonstrates how to use the new audio commands from the frontend

import { invoke } from '@tauri-apps/api/tauri';

class AudioManager {
    constructor() {
        this.isRecording = false;
        this.devices = [];
        this.currentConfig = null;
    }

    // Get list of available audio input devices
    async getAudioDevices() {
        try {
            const devices = await invoke('get_audio_devices');
            this.devices = devices;
            console.log('Available audio devices:', devices);
            return devices;
        } catch (error) {
            console.error('Failed to get audio devices:', error);
            throw error;
        }
    }

    // Check current audio status
    async checkStatus() {
        try {
            const status = await invoke('check_audio_status');
            this.isRecording = status.is_recording;
            this.currentConfig = status.config;
            console.log('Audio status:', status);
            return status;
        } catch (error) {
            console.error('Failed to check audio status:', error);
            throw error;
        }
    }

    // Start audio capture with default settings
    async startCapture() {
        try {
            const result = await invoke('start_audio_stream');
            this.isRecording = true;
            console.log('Audio capture started:', result);
            return result;
        } catch (error) {
            console.error('Failed to start audio capture:', error);
            throw error;
        }
    }

    // Start audio capture with custom configuration
    async startCaptureWithConfig(config = {}) {
        const audioConfig = {
            sample_rate: config.sampleRate || 44100,
            channels: config.channels || 2,
            buffer_size: config.bufferSize || 4096
        };

        try {
            const result = await invoke('start_audio_with_config', { config: audioConfig });
            this.isRecording = true;
            this.currentConfig = audioConfig;
            console.log('Audio capture started with config:', result);
            return result;
        } catch (error) {
            console.error('Failed to start audio capture with config:', error);
            throw error;
        }
    }

    // Stop audio capture
    async stopCapture() {
        try {
            const result = await invoke('stop_audio_stream');
            this.isRecording = false;
            console.log('Audio capture stopped:', result);
            return result;
        } catch (error) {
            console.error('Failed to stop audio capture:', error);
            throw error;
        }
    }

    // Test audio capture for a specific duration
    async testCapture(durationSeconds = 5) {
        try {
            console.log(`Starting test capture for ${durationSeconds} seconds...`);
            const result = await invoke('test_audio_capture', { duration: durationSeconds });
            console.log('Test capture completed:', result);
            return result;
        } catch (error) {
            console.error('Test capture failed:', error);
            throw error;
        }
    }

    // Toggle audio capture on/off
    async toggleCapture() {
        if (this.isRecording) {
            return await this.stopCapture();
        } else {
            return await this.startCapture();
        }
    }
}

// Example usage
const audioManager = new AudioManager();

// Example functions to demonstrate usage
export async function initializeAudio() {
    console.log('Initializing audio system...');
    
    try {
        // Get available devices
        const devices = await audioManager.getAudioDevices();
        console.log(`Found ${devices.length} audio devices`);
        
        // Check current status
        const status = await audioManager.checkStatus();
        console.log('Current audio status:', status);
        
        return { devices, status };
    } catch (error) {
        console.error('Failed to initialize audio:', error);
        throw error;
    }
}

export async function startAudioCapture(customConfig = null) {
    console.log('Starting audio capture...');
    
    try {
        if (customConfig) {
            return await audioManager.startCaptureWithConfig(customConfig);
        } else {
            return await audioManager.startCapture();
        }
    } catch (error) {
        console.error('Failed to start audio capture:', error);
        throw error;
    }
}

export async function stopAudioCapture() {
    console.log('Stopping audio capture...');
    
    try {
        return await audioManager.stopCapture();
    } catch (error) {
        console.error('Failed to stop audio capture:', error);
        throw error;
    }
}

export async function testAudioSystem(duration = 5) {
    console.log(`Testing audio system for ${duration} seconds...`);
    
    try {
        return await audioManager.testCapture(duration);
    } catch (error) {
        console.error('Audio test failed:', error);
        throw error;
    }
}

// Example HTML integration
export function createAudioControls() {
    const controlsHTML = `
        <div id="audio-controls">
            <h3>System Audio Capture</h3>
            
            <div class="audio-status">
                <span id="recording-status">Not Recording</span>
                <div id="config-display"></div>
            </div>
            
            <div class="audio-buttons">
                <button id="init-audio">Initialize</button>
                <button id="start-capture">Start Capture</button>
                <button id="stop-capture">Stop Capture</button>
                <button id="test-capture">Test (5s)</button>
                <button id="get-devices">Get Devices</button>
            </div>
            
            <div class="audio-config">
                <h4>Custom Configuration</h4>
                <label>Sample Rate: <input id="sample-rate" type="number" value="44100"></label>
                <label>Channels: <input id="channels" type="number" value="2" min="1" max="8"></label>
                <label>Buffer Size: <input id="buffer-size" type="number" value="4096"></label>
                <button id="start-with-config">Start with Config</button>
            </div>
            
            <div id="devices-list"></div>
            <div id="audio-log"></div>
        </div>
    `;
    
    return controlsHTML;
}

// Event handlers for the HTML controls
export function setupAudioControlHandlers() {
    document.getElementById('init-audio')?.addEventListener('click', async () => {
        try {
            const { devices, status } = await initializeAudio();
            updateUI(devices, status);
        } catch (error) {
            logError('Initialization failed: ' + error);
        }
    });
    
    document.getElementById('start-capture')?.addEventListener('click', async () => {
        try {
            const result = await startAudioCapture();
            logSuccess(result);
            updateStatus();
        } catch (error) {
            logError('Start failed: ' + error);
        }
    });
    
    document.getElementById('stop-capture')?.addEventListener('click', async () => {
        try {
            const result = await stopAudioCapture();
            logSuccess(result);
            updateStatus();
        } catch (error) {
            logError('Stop failed: ' + error);
        }
    });
    
    document.getElementById('test-capture')?.addEventListener('click', async () => {
        try {
            const result = await testAudioSystem(5);
            logSuccess(result);
        } catch (error) {
            logError('Test failed: ' + error);
        }
    });
    
    document.getElementById('start-with-config')?.addEventListener('click', async () => {
        const config = {
            sampleRate: parseInt(document.getElementById('sample-rate').value),
            channels: parseInt(document.getElementById('channels').value),
            bufferSize: parseInt(document.getElementById('buffer-size').value)
        };
        
        try {
            const result = await startAudioCapture(config);
            logSuccess(result);
            updateStatus();
        } catch (error) {
            logError('Start with config failed: ' + error);
        }
    });
    
    document.getElementById('get-devices')?.addEventListener('click', async () => {
        try {
            const devices = await audioManager.getAudioDevices();
            updateDevicesList(devices);
        } catch (error) {
            logError('Get devices failed: ' + error);
        }
    });
}

// UI helper functions
function updateUI(devices, status) {
    updateDevicesList(devices);
    updateStatusDisplay(status);
}

function updateDevicesList(devices) {
    const devicesList = document.getElementById('devices-list');
    if (devicesList) {
        devicesList.innerHTML = `
            <h4>Available Devices (${devices.length})</h4>
            <ul>
                ${devices.map(device => `<li>${device}</li>`).join('')}
            </ul>
        `;
    }
}

function updateStatusDisplay(status) {
    const statusElement = document.getElementById('recording-status');
    const configElement = document.getElementById('config-display');
    
    if (statusElement) {
        statusElement.textContent = status.is_recording ? 'Recording' : 'Not Recording';
        statusElement.className = status.is_recording ? 'recording' : 'not-recording';
    }
    
    if (configElement) {
        configElement.innerHTML = `
            <div>Sample Rate: ${status.config.sample_rate} Hz</div>
            <div>Channels: ${status.config.channels}</div>
            <div>Buffer Size: ${status.config.buffer_size}</div>
        `;
    }
}

async function updateStatus() {
    try {
        const status = await audioManager.checkStatus();
        updateStatusDisplay(status);
    } catch (error) {
        console.error('Failed to update status:', error);
    }
}

function logSuccess(message) {
    const log = document.getElementById('audio-log');
    if (log) {
        log.innerHTML += `<div class="success">${new Date().toLocaleTimeString()}: ${message}</div>`;
        log.scrollTop = log.scrollHeight;
    }
    console.log(message);
}

function logError(message) {
    const log = document.getElementById('audio-log');
    if (log) {
        log.innerHTML += `<div class="error">${new Date().toLocaleTimeString()}: ${message}</div>`;
        log.scrollTop = log.scrollHeight;
    }
    console.error(message);
}

// Export the audio manager for direct use
export { audioManager };
