import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface StealthHotkeyEvent {
  action: string;
  hotkey_id: number;
  timestamp: string;
}

interface UseStealthModeReturn {
  isStealthActive: boolean;
  toggleStealth: () => Promise<void>;
  activateStealth: () => Promise<void>;
  deactivateStealth: () => Promise<void>;
  isLoading: boolean;
  error: string | null;
  lastHotkeyAction: string | null;
}

export const useStealthMode = (): UseStealthModeReturn => {
  const [isStealthActive, setIsStealthActive] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastHotkeyAction, setLastHotkeyAction] = useState<string | null>(null);

  // Apply/remove stealth CSS class when stealth mode changes
  useEffect(() => {
    if (isStealthActive) {
      document.body.classList.add('stealth-mode');
      
      // AGGRESSIVE TOOLTIP REMOVAL - Remove ALL possible tooltip attributes
      const removeAllTooltips = () => {
        // Remove title attributes from ALL elements
        const elementsWithTitle = document.querySelectorAll('[title]');
        elementsWithTitle.forEach(element => {
          element.setAttribute('data-stealth-original-title', element.getAttribute('title') || '');
          element.removeAttribute('title');
        });
        
        // Remove aria-label attributes that show tooltips
        const elementsWithAriaLabel = document.querySelectorAll('[aria-label]');
        elementsWithAriaLabel.forEach(element => {
          element.setAttribute('data-stealth-original-aria-label', element.getAttribute('aria-label') || '');
          element.removeAttribute('aria-label');
        });
        
        // Remove data-tooltip attributes
        const elementsWithDataTooltip = document.querySelectorAll('[data-tooltip]');
        elementsWithDataTooltip.forEach(element => {
          element.setAttribute('data-stealth-original-tooltip', element.getAttribute('data-tooltip') || '');
          element.removeAttribute('data-tooltip');
        });
        
        // Remove alt attributes from images that might show on hover
        const imagesWithAlt = document.querySelectorAll('img[alt]');
        imagesWithAlt.forEach(element => {
          element.setAttribute('data-stealth-original-alt', element.getAttribute('alt') || '');
          element.removeAttribute('alt');
        });
        
        // Remove placeholder text that might give away functionality
        const placeholderElements = document.querySelectorAll('[placeholder]');
        placeholderElements.forEach((element: any) => {
          element.setAttribute('data-stealth-original-placeholder', element.placeholder || '');
          element.placeholder = '';
        });
        
        // Remove any data-tip attributes (another common tooltip library)
        const elementsWithDataTip = document.querySelectorAll('[data-tip]');
        elementsWithDataTip.forEach(element => {
          element.setAttribute('data-stealth-original-tip', element.getAttribute('data-tip') || '');
          element.removeAttribute('data-tip');
        });
      };
      
      // Remove tooltips immediately
      removeAllTooltips();
      
      // Set up mutation observer to remove tooltips from dynamically added elements
      const observer = new MutationObserver(() => {
        removeAllTooltips();
      });
      
      observer.observe(document.body, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ['title', 'aria-label', 'data-tooltip', 'alt', 'placeholder', 'data-tip']
      });
      
      // Store observer for cleanup
      (window as any).stealthObserver = observer;
      
      console.log('🕵️ Stealth mode: ALL tooltips and hover text removed');
      
    } else {
      document.body.classList.remove('stealth-mode');
      
      // Stop mutation observer
      if ((window as any).stealthObserver) {
        (window as any).stealthObserver.disconnect();
        delete (window as any).stealthObserver;
      }
      
      // Restore all original attributes
      const restoreAttributes = (selector: string, originalAttr: string, targetAttr: string) => {
        const elements = document.querySelectorAll(selector);
        elements.forEach(element => {
          const originalValue = element.getAttribute(originalAttr);
          if (originalValue) {
            element.setAttribute(targetAttr, originalValue);
          }
          element.removeAttribute(originalAttr);
        });
      };
      
      restoreAttributes('[data-stealth-original-title]', 'data-stealth-original-title', 'title');
      restoreAttributes('[data-stealth-original-aria-label]', 'data-stealth-original-aria-label', 'aria-label');
      restoreAttributes('[data-stealth-original-tooltip]', 'data-stealth-original-tooltip', 'data-tooltip');
      restoreAttributes('[data-stealth-original-alt]', 'data-stealth-original-alt', 'alt');
      restoreAttributes('[data-stealth-original-tip]', 'data-stealth-original-tip', 'data-tip');
      
      // Restore placeholder text
      const placeholderElements = document.querySelectorAll('[data-stealth-original-placeholder]');
      placeholderElements.forEach((element: any) => {
        const originalPlaceholder = element.getAttribute('data-stealth-original-placeholder');
        if (originalPlaceholder) {
          element.placeholder = originalPlaceholder;
        }
        element.removeAttribute('data-stealth-original-placeholder');
      });
      
      console.log('🔓 Stealth mode: All tooltips and hover text restored');
    }

    return () => {
      document.body.classList.remove('stealth-mode');
    };
  }, [isStealthActive]);

  // Listen for stealth hotkey events
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        unlisten = await listen<StealthHotkeyEvent>('stealth-hotkey', (event) => {
          console.log('🎯 Stealth hotkey detected:', event.payload);
          setLastHotkeyAction(event.payload.action);
          
          // Clear the action after 3 seconds
          setTimeout(() => setLastHotkeyAction(null), 3000);
        });
      } catch (err) {
        console.error('Failed to setup stealth hotkey listener:', err);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const activateStealth = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      // Activate stealth hotkeys
      await invoke<string>('activate_stealth_mode');
      
      // Enable task manager stealth
      await invoke<string>('enable_task_manager_stealth');
      
      // Apply advanced stealth techniques
      await invoke<string>('apply_advanced_stealth');
      
      // Hide from taskbar for maximum stealth
      try {
        await invoke<string>('hide_from_taskbar');
        console.log('🔒 Application hidden from taskbar');
      } catch (taskbarError) {
        console.warn('⚠️ Failed to hide from taskbar:', taskbarError);
      }
      
      setIsStealthActive(true);
      console.log('🕵️ Stealth mode activated successfully');
      
    } catch (err: any) {
      const errorMessage = `Failed to activate stealth mode: ${err}`;
      setError(errorMessage);
      console.error(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const deactivateStealth = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      // Deactivate stealth hotkeys
      await invoke<string>('deactivate_stealth_mode');
      
      // Disable task manager stealth
      await invoke<string>('disable_task_manager_stealth');
      
      // Show in taskbar again
      try {
        await invoke<string>('show_in_taskbar');
        console.log('🔓 Application restored to taskbar');
      } catch (taskbarError) {
        console.warn('⚠️ Failed to restore to taskbar:', taskbarError);
      }
      
      setIsStealthActive(false);
      console.log('🔓 Stealth mode deactivated successfully');
      
    } catch (err: any) {
      const errorMessage = `Failed to deactivate stealth mode: ${err}`;
      setError(errorMessage);
      console.error(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const toggleStealth = useCallback(async () => {
    if (isStealthActive) {
      await deactivateStealth();
    } else {
      await activateStealth();
    }
  }, [isStealthActive, activateStealth, deactivateStealth]);

  return {
    isStealthActive,
    toggleStealth,
    activateStealth,
    deactivateStealth,
    isLoading,
    error,
    lastHotkeyAction
  };
};

export default useStealthMode;
