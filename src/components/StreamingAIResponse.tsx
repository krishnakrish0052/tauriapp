import React, { memo, useCallback, useRef, useEffect } from 'react';

interface StreamingAIResponseProps {
  content: string;
  isStreaming: boolean;
  className?: string;
}

/**
 * Optimized streaming AI response component that minimizes re-renders
 * and provides smooth token-by-token display with batching for performance.
 */
const StreamingAIResponse: React.FC<StreamingAIResponseProps> = memo(({
  content,
  isStreaming,
  className = '',
}) => {
  const contentRef = useRef<HTMLDivElement>(null);
  const animationFrameRef = useRef<number | null>(null);
  const lastContentRef = useRef<string>('');
  const batchedUpdateRef = useRef<string>('');
  const updateTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Performance-optimized content update with batching
  const updateContent = useCallback((newContent: string) => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
    }

    animationFrameRef.current = requestAnimationFrame(() => {
      if (contentRef.current && newContent !== lastContentRef.current) {
        contentRef.current.textContent = newContent;
        lastContentRef.current = newContent;
        
        // Auto-scroll to bottom during streaming
        if (isStreaming) {
          contentRef.current.scrollTop = contentRef.current.scrollHeight;
        }
      }
    });
  }, [isStreaming]);

  // Optimized updates for real-time streaming visibility
  useEffect(() => {
    if (content === batchedUpdateRef.current) return;

    if (isStreaming) {
      // During streaming: minimal delay for visible streaming effect
      if (updateTimeoutRef.current) {
        clearTimeout(updateTimeoutRef.current);
      }
      
      batchedUpdateRef.current = content;
      updateTimeoutRef.current = setTimeout(() => {
        updateContent(content);
      }, 1); // Very small delay for smoother visual streaming
    } else {
      // When not streaming: immediate update
      updateContent(content);
      batchedUpdateRef.current = content;
    }
  }, [content, isStreaming, updateContent]);

  // Cleanup timeouts
  useEffect(() => {
    return () => {
      if (updateTimeoutRef.current) {
        clearTimeout(updateTimeoutRef.current);
      }
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  return (
    <div className={`relative ${className}`}>
      <div
        ref={contentRef}
        className={`
          streaming-ai-response
          whitespace-pre-wrap break-words
          ${isStreaming ? 'streaming-active' : ''}
        `}
        style={{
          // CSS optimizations for smooth text rendering
          fontKerning: 'normal',
          textRendering: 'optimizeSpeed',
          willChange: isStreaming ? 'contents' : 'auto',
          contain: 'style layout',
          // Prevent layout shifts during streaming
          minHeight: '1.2em',
        }}
      >
        {content}
      </div>
      
      {/* Streaming indicator cursor */}
      {isStreaming && (
        <span 
          className="inline-block w-2 h-5 bg-blue-400 animate-pulse ml-1"
          style={{
            animation: 'blink 1s infinite',
          }}
        />
      )}
    </div>
  );
});

StreamingAIResponse.displayName = 'StreamingAIResponse';

export default StreamingAIResponse;
