import { useEffect, useRef, useState } from 'react';
import { Button } from '@/components/ui/button';

interface AIResponseWindowProps {
  isVisible: boolean;
  content: string;
  isStreaming: boolean;
  onClose: () => void;
  onClear: () => void;
}

export const AIResponseWindow: React.FC<AIResponseWindowProps> = ({
  isVisible,
  content,
  isStreaming,
  onClose,
  onClear
}) => {
  const contentRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const headerRef = useRef<HTMLDivElement>(null);
  
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [isInitialized, setIsInitialized] = useState(false);

  // Initialize position to center of parent when visible (DPI-aware)
  useEffect(() => {
    if (isVisible && !isInitialized && containerRef.current) {
      const container = containerRef.current;
      const parent = container.parentElement;
      if (parent) {
        const parentRect = parent.getBoundingClientRect();
        const containerRect = container.getBoundingClientRect();
        
        // DPI-aware centering calculation
        const devicePixelRatio = window.devicePixelRatio || 1;
        const centerX = (parentRect.width - containerRect.width) / 2;
        const centerY = (parentRect.height - containerRect.height) / 2;
        
        console.log('🔍 AIResponseWindow DPI info:', {
          devicePixelRatio,
          parentRect: { width: parentRect.width, height: parentRect.height },
          containerRect: { width: containerRect.width, height: containerRect.height },
          calculatedPosition: { x: centerX, y: centerY }
        });
        
        setPosition({ x: centerX, y: centerY });
        setIsInitialized(true);
      }
    } else if (!isVisible) {
      setIsInitialized(false);
    }
  }, [isVisible, isInitialized]);

  // Auto-scroll to bottom when new content is added
  useEffect(() => {
    if (contentRef.current && isStreaming) {
      contentRef.current.scrollTop = contentRef.current.scrollHeight;
    }
  }, [content, isStreaming]);

  // Handle mouse events for dragging
  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.target === headerRef.current || headerRef.current?.contains(e.target as Node)) {
      setIsDragging(true);
      setDragStart({
        x: e.clientX - position.x,
        y: e.clientY - position.y
      });
      e.preventDefault();
    }
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging || !containerRef.current) return;
    
    const parent = containerRef.current.parentElement;
    if (!parent) return;
    
    const parentRect = parent.getBoundingClientRect();
    const containerRect = containerRef.current.getBoundingClientRect();
    
    let newX = e.clientX - dragStart.x;
    let newY = e.clientY - dragStart.y;
    
    // Constrain to parent bounds
    newX = Math.max(0, Math.min(newX, parentRect.width - containerRect.width));
    newY = Math.max(0, Math.min(newY, parentRect.height - containerRect.height));
    
    setPosition({ x: newX, y: newY });
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  // Add/remove global mouse event listeners
  useEffect(() => {
    if (isDragging) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      document.body.style.userSelect = 'none'; // Prevent text selection while dragging
      
      return () => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
        document.body.style.userSelect = '';
      };
    }
  }, [isDragging, dragStart, position]);

  if (!isVisible) {
    return null;
  }

  return (
    <div 
      ref={containerRef}
      className="absolute bg-black/80 backdrop-blur-md border border-white/20 rounded-lg shadow-lg z-50 w-96 max-w-sm"
      style={{
        left: `${position.x}px`,
        top: `${position.y}px`,
        cursor: isDragging ? 'grabbing' : 'default'
      }}
      onMouseDown={handleMouseDown}
    >
      {/* Header */}
      <div 
        ref={headerRef}
        className="flex items-center justify-between border-b border-white/10 px-2 py-1 h-8 cursor-grab active:cursor-grabbing select-none"
      >
        <div className="flex items-center gap-1">
          <div className="w-4 h-4 rounded-full bg-gradient-to-r from-purple-400 to-pink-500 flex items-center justify-center flex-shrink-0">
            <span className="material-icons text-white text-xs">psychology</span>
          </div>
          <h3 className="text-white font-medium text-xs">AI Response</h3>
          {isStreaming && (
            <div className="flex items-center gap-1 ml-2">
              <div className="w-2 h-2 rounded-full bg-purple-400 animate-pulse"></div>
              <span className="text-purple-400 text-xs">Generating...</span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-1">
          <Button 
            onClick={onClear}
            className="w-4 h-4 bg-white/10 hover:bg-white/20 border-0 rounded p-0 flex items-center justify-center transition-all"
            title="Clear response"
          >
            <span className="material-icons text-gray-400 text-xs">clear</span>
          </Button>
          <Button 
            onClick={onClose}
            className="w-4 h-4 bg-red-500/80 hover:bg-red-600 border-0 rounded-full p-0 flex items-center justify-center transition-all flex-shrink-0"
            title="Close AI Response"
          >
            <span className="material-icons text-white text-xs">close</span>
          </Button>
        </div>
      </div>
      
      {/* Content Area */}
      <div className="p-3">
        <div 
          ref={contentRef}
          className="text-white text-sm leading-relaxed whitespace-pre-wrap max-h-80 overflow-y-auto scrollbar-thin scrollbar-track-white/10 scrollbar-thumb-white/30"
          style={{
            minHeight: content.trim() ? 'auto' : '60px',
            fontFamily: 'system-ui, -apple-system, sans-serif'
          }}
        >
          {content.trim() || (isStreaming ? 'Generating AI response...' : 'No response yet.')}
          {isStreaming && (
            <span className="inline-block w-2 h-4 bg-purple-400 ml-1 animate-pulse"></span>
          )}
        </div>
      </div>
    </div>
  );
};
