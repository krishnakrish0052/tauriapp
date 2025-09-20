import React from 'react';
import { Button } from '@/components/ui/button';

interface NotificationDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  onClose: () => void;
  windowHeight?: number;
}

const NotificationDialog: React.FC<NotificationDialogProps> = ({
  isOpen,
  title,
  message,
  onClose,
  windowHeight = 400,
}) => {
  if (!isOpen) return null;

  // Calculate responsive dimensions based on window height
  const maxHeight = Math.min(windowHeight * 0.7, 300); // Max 70% of window height or 300px
  const dialogHeight = Math.max(120, Math.min(maxHeight, 200)); // Between 120-200px
  const padding = windowHeight < 300 ? 12 : 16; // Smaller padding for smaller windows
  const fontSize = windowHeight < 300 ? 'text-xs' : 'text-sm';
  const titleSize = windowHeight < 300 ? 'text-sm' : 'text-base';
  
  // Truncate message if it's too long for small windows
  const maxMessageLength = windowHeight < 300 ? 100 : 200;
  const truncatedMessage = message.length > maxMessageLength 
    ? message.substring(0, maxMessageLength) + '...' 
    : message;

  return (
    <div 
      className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50"
      onClick={onClose}
    >
      <div 
        className="bg-black/90 border border-white/20 rounded-lg shadow-xl mx-4 animate-in fade-in zoom-in duration-200"
        style={{ 
          maxWidth: '90%', 
          width: '320px', 
          height: `${dialogHeight}px`,
          minHeight: '120px'
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className={`flex flex-col h-full p-${padding < 16 ? '3' : '4'}`}>
          {/* Header */}
          <div className="flex items-center justify-between mb-3">
            <h3 className={`font-semibold text-white ${titleSize} truncate flex-1 mr-2`}>
              {title}
            </h3>
            <div className="w-2 h-2 rounded-full bg-blue-400 flex-shrink-0" />
          </div>
          
          {/* Message Content */}
          <div className="flex-1 overflow-hidden">
            <p className={`text-gray-300 ${fontSize} leading-relaxed break-words`}>
              {truncatedMessage}
            </p>
          </div>
          
          {/* Footer */}
          <div className="flex justify-end mt-4">
            <Button
              onClick={onClose}
              className={`bg-blue-500 hover:bg-blue-600 text-white border-0 rounded transition-colors px-4 py-1.5 ${fontSize} h-auto min-h-[28px] flex items-center justify-center`}
            >
              OK
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default NotificationDialog;
