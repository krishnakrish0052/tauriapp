import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface WindowDimensions {
  width: number;
  height: number;
}

interface ResponsiveConfig {
  minWidth: number;
  minHeight: number;
  maxWidth?: number;
  maxHeight?: number;
  defaultWidth: number;
  defaultHeight: number;
}

const DEFAULT_CONFIG: ResponsiveConfig = {
  minWidth: 400,
  minHeight: 30, // Allow very small windows for compact UI
  maxWidth: 1400,
  maxHeight: 900,
  defaultWidth: 1100,
  defaultHeight: 250,
};

export const useResponsiveWindow = (config: Partial<ResponsiveConfig> = {}) => {
  // Stabilize the config object to prevent infinite re-renders
  const finalConfig = useMemo(() => ({ ...DEFAULT_CONFIG, ...config }), [config]);
  
  // Track initialization to prevent duplicate calls
  const isInitializedRef = useRef(false);
  const isInitializingRef = useRef(false);
  
  // Ref to track current dimensions without causing re-renders
  const dimensionsRef = useRef<WindowDimensions>({
    width: finalConfig.defaultWidth,
    height: finalConfig.defaultHeight,
  });
  
  const [dimensions, setDimensions] = useState<WindowDimensions>({
    width: finalConfig.defaultWidth,
    height: finalConfig.defaultHeight,
  });
  const [screenSize, setScreenSize] = useState<'sm' | 'md' | 'lg' | 'xl'>('md');
  
  // Update dimensions ref whenever dimensions change
  dimensionsRef.current = dimensions;

  // Determine screen size category based on window dimensions and height
  const updateScreenSize = useCallback((width: number, height?: number) => {
    const currentHeight = height || dimensionsRef.current.height;
    
    // More granular responsive breakpoints
    if (width < 640 || currentHeight < 300) setScreenSize('sm');
    else if (width < 900 || currentHeight < 400) setScreenSize('md');
    else if (width < 1200 || currentHeight < 600) setScreenSize('lg');
    else setScreenSize('xl');
  }, []); // No dependencies needed since we use ref

  // Get monitor information and adjust window size accordingly
  const getOptimalSize = useCallback(async () => {
    try {
      const monitors = await invoke<any[]>('get_monitors_info');
      const primaryMonitor = monitors.find(m => m.is_primary) || monitors[0];
      
      if (primaryMonitor && primaryMonitor.size) {
        const { width: screenWidth, height: screenHeight } = primaryMonitor.size;
        const scaleFactor = primaryMonitor.scale_factor || 1;
        
        // Calculate optimal size based on screen size and scale factor
        const availableWidth = screenWidth / scaleFactor;
        const availableHeight = screenHeight / scaleFactor;
        
        let optimalWidth = finalConfig.defaultWidth;
        let optimalHeight = finalConfig.defaultHeight;
        
        // Scale down for smaller screens (both width and height)
        if (availableWidth < 1024) {
          optimalWidth = Math.min(availableWidth * 0.9, finalConfig.defaultWidth);
        }
        
        if (availableHeight < 600) {
          optimalHeight = Math.min(availableHeight * 0.85, finalConfig.defaultHeight);
        }
        
        // Ensure minimum and maximum constraints
        optimalWidth = Math.max(finalConfig.minWidth, Math.min(optimalWidth, finalConfig.maxWidth || availableWidth));
        optimalHeight = Math.max(finalConfig.minHeight, Math.min(optimalHeight, finalConfig.maxHeight || availableHeight));
        
        return { width: optimalWidth, height: optimalHeight };
      }
    } catch (error) {
      console.warn('Failed to get monitor info, using default size:', error);
    }
    
    return { width: finalConfig.defaultWidth, height: finalConfig.defaultHeight };
  }, [finalConfig]);

  // Resize window with responsive constraints
  const resizeWindow = useCallback(async (newDimensions: Partial<WindowDimensions>) => {
    try {
      // Use current dimensions as fallback instead of calling getOptimalSize recursively
      const targetWidth = newDimensions.width || dimensions.width;
      const targetHeight = newDimensions.height || dimensions.height;
      
      // Apply constraints
      const constrainedWidth = Math.max(
        finalConfig.minWidth,
        Math.min(targetWidth, finalConfig.maxWidth || targetWidth)
      );
      // For very small target heights, don't enforce minimum height
      const constrainedHeight = targetHeight < 150 ? targetHeight : Math.max(
        finalConfig.minHeight,
        Math.min(targetHeight, finalConfig.maxHeight || targetHeight)
      );
      
      // Only resize if dimensions actually changed
      if (constrainedWidth !== dimensions.width || constrainedHeight !== dimensions.height) {
        await invoke('resize_window_scale', {
          width: constrainedWidth,
          height: constrainedHeight,
        });
        
        setDimensions({ width: constrainedWidth, height: constrainedHeight });
        updateScreenSize(constrainedWidth, constrainedHeight);
      }
    } catch (error) {
      console.error('Failed to resize window:', error);
    }
  }, [finalConfig, dimensions.width, dimensions.height, updateScreenSize]);

  // Auto-resize based on content and screen type - enhanced for screen compatibility
  const autoResize = useCallback(async (contentExpanded: boolean = false, screenType?: 'session_connection' | 'confirmation' | 'main') => {
    let targetHeight = finalConfig.defaultHeight;
    let targetWidth = finalConfig.defaultWidth;
    
    // Get current screen size to adapt dimensions
    const optimalSize = await getOptimalSize();
    const baseWidth = optimalSize.width;
    
    // Dynamic dimensions based on screen type and expansion
    switch (screenType) {
      case 'session_connection':
        // Responsive width based on screen size, minimal height (reduced by 25%)
        targetWidth = Math.min(baseWidth, 600); // Reduced from 800px to 600px
        targetHeight = 31;
        break;
      case 'confirmation':
        // Match session connection width for consistency (reduced by 25%)
        targetWidth = Math.min(baseWidth, 600); // Reduced from 800px to 600px
        targetHeight = 27;
        break;
      case 'main':
        // Use reduced width (25% smaller) for consistency
        targetWidth = Math.min(baseWidth, 600); // Reduced from 800px to 600px
        targetHeight = contentExpanded ? 400 : 110;
        break;
      default:
        targetWidth = baseWidth;
        targetHeight = contentExpanded ? finalConfig.defaultHeight * 2 : finalConfig.defaultHeight;
    }
    
    // Store current dimensions for AI response window matching
    (window as any).mockMateWindowWidth = targetWidth;
    
    await resizeWindow({ width: targetWidth, height: targetHeight });
  }, [finalConfig.defaultHeight, finalConfig.defaultWidth, resizeWindow, screenSize, getOptimalSize]);

  // Initialize window size on mount (only once)
  useEffect(() => {
    if (isInitializedRef.current || isInitializingRef.current) {
      return;
    }
    
    isInitializingRef.current = true;
    
    const initializeWindow = async () => {
      try {
        const optimalSize = await getOptimalSize();
        
        // Only resize if dimensions actually changed
        if (optimalSize.width !== dimensions.width || optimalSize.height !== dimensions.height) {
          await invoke('resize_window_scale', {
            width: optimalSize.width,
            height: optimalSize.height,
          });
          
          setDimensions(optimalSize);
          updateScreenSize(optimalSize.width, optimalSize.height);
        }
        
        // Ensure window is visible and properly positioned
        await invoke('ensure_window_visible');
        
        isInitializedRef.current = true;
      } catch (error) {
        console.error('Failed to initialize window:', error);
      } finally {
        isInitializingRef.current = false;
      }
    };

    initializeWindow();
  }, []); // Empty dependency array - only run once on mount

  // Listen for window resize events (if implemented in backend)
  useEffect(() => {
    const handleResize = () => {
      updateScreenSize(
        window.innerWidth || dimensionsRef.current.width,
        window.innerHeight || dimensionsRef.current.height
      );
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []); // Empty dependency array to prevent re-registration

  // Get responsive classes for current screen size with height awareness
  const getResponsiveClasses = useCallback(() => {
    const classes = {
      container: `w-full h-full flex flex-col transition-all duration-300`,
      header: `mockmate-header transition-all duration-200 ${
        screenSize === 'sm' ? 'px-3 py-2' : 
        screenSize === 'md' ? 'px-4 py-2.5' : 
        screenSize === 'lg' ? 'px-6 py-3' :
        'px-8 py-4'
      }`,
      content: `flex-1 overflow-hidden transition-all duration-300 ${
        screenSize === 'sm' ? 'min-h-[120px] p-3' : 
        screenSize === 'md' ? 'min-h-[160px] p-4' : 
        screenSize === 'lg' ? 'min-h-[200px] p-5' :
        'min-h-[240px] p-6'
      }`,
      footer: `mockmate-input-section transition-all duration-200 ${
        screenSize === 'sm' ? 'px-3 py-2 gap-2' : 
        screenSize === 'md' ? 'px-4 py-2.5 gap-2.5' : 
        screenSize === 'lg' ? 'px-6 py-3 gap-3' :
        'px-8 py-4 gap-4'
      }`,
      // Typography scaling
      text: {
        xs: screenSize === 'sm' ? 'text-xs' : screenSize === 'md' ? 'text-sm' : 'text-base',
        sm: screenSize === 'sm' ? 'text-sm' : screenSize === 'md' ? 'text-base' : 'text-lg',
        base: screenSize === 'sm' ? 'text-base' : screenSize === 'md' ? 'text-lg' : 'text-xl',
        lg: screenSize === 'sm' ? 'text-lg' : screenSize === 'md' ? 'text-xl' : 'text-2xl',
        xl: screenSize === 'sm' ? 'text-xl' : screenSize === 'md' ? 'text-2xl' : 'text-3xl',
      },
      // Spacing system
      spacing: {
        xs: screenSize === 'sm' ? 'gap-1' : screenSize === 'md' ? 'gap-1.5' : 'gap-2',
        sm: screenSize === 'sm' ? 'gap-2' : screenSize === 'md' ? 'gap-2.5' : 'gap-3',
        md: screenSize === 'sm' ? 'gap-3' : screenSize === 'md' ? 'gap-4' : 'gap-5',
        lg: screenSize === 'sm' ? 'gap-4' : screenSize === 'md' ? 'gap-5' : 'gap-6',
      },
      // Icon scaling
      icons: {
        xs: screenSize === 'sm' ? 'w-3 h-3' : screenSize === 'md' ? 'w-3.5 h-3.5' : 'w-4 h-4',
        sm: screenSize === 'sm' ? 'w-4 h-4' : screenSize === 'md' ? 'w-5 h-5' : 'w-6 h-6',
        md: screenSize === 'sm' ? 'w-5 h-5' : screenSize === 'md' ? 'w-6 h-6' : 'w-7 h-7',
        lg: screenSize === 'sm' ? 'w-6 h-6' : screenSize === 'md' ? 'w-7 h-7' : 'w-8 h-8',
        xl: screenSize === 'sm' ? 'w-8 h-8' : screenSize === 'md' ? 'w-10 h-10' : 'w-12 h-12',
        // Material Icons font sizes
        material: {
          sm: screenSize === 'sm' ? 'text-base' : screenSize === 'md' ? 'text-lg' : 'text-xl',
          md: screenSize === 'sm' ? 'text-lg' : screenSize === 'md' ? 'text-xl' : 'text-2xl',
          lg: screenSize === 'sm' ? 'text-xl' : screenSize === 'md' ? 'text-2xl' : 'text-3xl',
        }
      },
      // Button system with consistent scaling
      buttons: {
        xs: {
          base: screenSize === 'sm' ? 'px-2 py-1 text-xs' : screenSize === 'md' ? 'px-2.5 py-1.5 text-xs' : 'px-3 py-2 text-sm',
          icon: screenSize === 'sm' ? 'w-6 h-6 p-1' : screenSize === 'md' ? 'w-7 h-7 p-1.5' : 'w-8 h-8 p-2',
        },
        sm: {
          base: screenSize === 'sm' ? 'px-3 py-1.5 text-xs' : screenSize === 'md' ? 'px-4 py-2 text-sm' : 'px-5 py-2.5 text-base',
          icon: screenSize === 'sm' ? 'w-7 h-7 p-1.5' : screenSize === 'md' ? 'w-8 h-8 p-2' : 'w-9 h-9 p-2.5',
        },
        md: {
          base: screenSize === 'sm' ? 'px-4 py-2 text-sm' : screenSize === 'md' ? 'px-5 py-2.5 text-base' : 'px-6 py-3 text-lg',
          icon: screenSize === 'sm' ? 'w-8 h-8 p-2' : screenSize === 'md' ? 'w-9 h-9 p-2.5' : 'w-10 h-10 p-3',
        },
        lg: {
          base: screenSize === 'sm' ? 'px-6 py-3 text-base' : screenSize === 'md' ? 'px-7 py-3.5 text-lg' : 'px-8 py-4 text-xl',
          icon: screenSize === 'sm' ? 'w-10 h-10 p-2.5' : screenSize === 'md' ? 'w-11 h-11 p-3' : 'w-12 h-12 p-3.5',
        },
        // Close/minimize buttons
        control: screenSize === 'sm' ? 'w-7 h-7 p-1.5' : screenSize === 'md' ? 'w-8 h-8 p-2' : 'w-9 h-9 p-2.5',
      },
      // Input system
      inputs: {
        sm: screenSize === 'sm' ? 'px-3 py-2 text-sm' : screenSize === 'md' ? 'px-4 py-2.5 text-base' : 'px-5 py-3 text-lg',
        md: screenSize === 'sm' ? 'px-4 py-2.5 text-base' : screenSize === 'md' ? 'px-5 py-3 text-lg' : 'px-6 py-4 text-xl',
        lg: screenSize === 'sm' ? 'px-5 py-3 text-lg' : screenSize === 'md' ? 'px-6 py-4 text-xl' : 'px-8 py-5 text-2xl',
      },
      // Brand logo sizing
      brand: {
        icon: screenSize === 'sm' ? 'w-7 h-7' : screenSize === 'md' ? 'w-8 h-8' : 'w-9 h-9',
        text: screenSize === 'sm' ? 'text-lg' : screenSize === 'md' ? 'text-xl' : 'text-2xl',
      },
      // Window controls specific sizing
      windowControls: {
        button: screenSize === 'sm' ? 'w-7 h-7' : screenSize === 'md' ? 'w-8 h-8' : 'w-9 h-9',
        icon: screenSize === 'sm' ? 'text-sm' : screenSize === 'md' ? 'text-base' : 'text-lg',
      }
    };
    
    return classes;
  }, [screenSize]);

  return {
    dimensions,
    screenSize,
    resizeWindow,
    autoResize,
    getResponsiveClasses,
    isSmallScreen: screenSize === 'sm',
    isMediumScreen: screenSize === 'md',
    isLargeScreen: screenSize === 'lg' || screenSize === 'xl',
  };
};
