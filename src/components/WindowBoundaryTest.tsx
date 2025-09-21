import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface WindowBoundaryTestProps {
  className?: string;
}

const WindowBoundaryTest: React.FC<WindowBoundaryTestProps> = ({ className = '' }) => {
  const [debugResult, setDebugResult] = useState<string>('');
  const [fixResult, setFixResult] = useState<string>('');
  const [nuclearResult, setNuclearResult] = useState<string>('');
  const [isDebugLoading, setIsDebugLoading] = useState(false);
  const [isFixLoading, setIsFixLoading] = useState(false);
  const [isNuclearLoading, setIsNuclearLoading] = useState(false);

  const handleDebugWindow = async () => {
    setIsDebugLoading(true);
    try {
      const result = await invoke<string>('debug_main_window_dimensions');
      setDebugResult(result);
      console.log('🔍 Debug result:', result);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      setDebugResult(`Error: ${errorMsg}`);
      console.error('❌ Debug error:', error);
    } finally {
      setIsDebugLoading(false);
    }
  };

  const handleFixBoundary = async () => {
    setIsFixLoading(true);
    try {
      const result = await invoke<string>('fix_main_window_invisible_boundary');
      setFixResult(result);
      console.log('🔧 Fix result:', result);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      setFixResult(`Error: ${errorMsg}`);
      console.error('❌ Fix error:', error);
    } finally {
      setIsFixLoading(false);
    }
  };

  const handleNuclearFix = async () => {
    setIsNuclearLoading(true);
    try {
      const result = await invoke<string>('nuclear_fix_webview_padding');
      setNuclearResult(result);
      console.log('💥 Nuclear fix result:', result);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      setNuclearResult(`Error: ${errorMsg}`);
      console.error('❌ Nuclear fix error:', error);
    } finally {
      setIsNuclearLoading(false);
    }
  };

  return (
    <div className={`p-4 bg-gray-800 rounded-lg border border-gray-600 ${className}`}>
      <h3 className="text-lg font-semibold text-white mb-4">🔧 Window Boundary Test</h3>
      
      <div className="space-y-4">
        {/* Debug Button */}
        <div>
          <button
            onClick={handleDebugWindow}
            disabled={isDebugLoading}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-800 text-white rounded-md transition-colors"
          >
            {isDebugLoading ? '🔍 Debugging...' : '🔍 Debug Window Dimensions'}
          </button>
          {debugResult && (
            <div className="mt-2 p-2 bg-gray-700 rounded text-sm text-gray-300 font-mono">
              {debugResult}
            </div>
          )}
        </div>

        {/* Fix Button */}
        <div>
          <button
            onClick={handleFixBoundary}
            disabled={isFixLoading}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-red-800 text-white rounded-md transition-colors"
          >
            {isFixLoading ? '🔧 Fixing...' : '🔧 Fix Invisible Boundary'}
          </button>
          {fixResult && (
            <div className="mt-2 p-2 bg-gray-700 rounded text-sm text-gray-300 font-mono">
              {fixResult}
            </div>
          )}
        </div>

        {/* Nuclear Fix Button */}
        <div>
          <button
            onClick={handleNuclearFix}
            disabled={isNuclearLoading}
            className="px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-purple-800 text-white rounded-md transition-colors"
          >
            {isNuclearLoading ? '💥 Nuclear Fixing...' : '💥 NUCLEAR FIX (Windows API)'}
          </button>
          {nuclearResult && (
            <div className="mt-2 p-2 bg-gray-700 rounded text-sm text-gray-300 font-mono">
              {nuclearResult}
            </div>
          )}
        </div>

        {/* Instructions */}
        <div className="text-xs text-gray-400 border-t border-gray-600 pt-3">
          <p><strong>Instructions:</strong></p>
          <ol className="list-decimal list-inside space-y-1 mt-2">
            <li>First click "Debug Window Dimensions" to check for invisible boundary</li>
            <li>If chrome height &gt; 0, try "Fix Invisible Boundary" first</li>
            <li>If still not fixed, use "NUCLEAR FIX" (removes all Windows styles)</li>
            <li>Test by moving cursor below the main window - boundary should be gone</li>
          </ol>
        </div>
      </div>
    </div>
  );
};

export default WindowBoundaryTest;
