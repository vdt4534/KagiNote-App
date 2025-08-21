import React, { useState } from 'react';
import { cn } from '@/lib/utils';
import { usePlatform } from '@/hooks/usePlatform';
import { TitleBar } from './TitleBar';
import { Sidebar } from './Sidebar';
import { StatusBar } from './StatusBar';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

export interface AppLayoutProps {
  children: React.ReactNode;
  title?: string;
  subtitle?: string;
  loading?: boolean;
  modelInfo?: {
    name: string;
    status: 'ready' | 'loading' | 'error';
  };
  recordingInfo?: {
    isRecording: boolean;
    duration?: string;
    status?: string;
  };
  systemInfo?: {
    privacy: boolean;
    cpu?: string;
    memory?: string;
  };
  className?: string;
}

const AppLayout: React.FC<AppLayoutProps> = ({
  children,
  title,
  subtitle,
  loading = false,
  modelInfo,
  recordingInfo,
  systemInfo = { privacy: true },
  className,
}) => {
  const { platform, isLoading: platformLoading } = usePlatform();
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  // Show loading spinner while platform detection is in progress
  if (platformLoading) {
    return (
      <div className="flex items-center justify-center h-screen bg-neutral-50 dark:bg-neutral-900">
        <div className="text-center space-y-4">
          <LoadingSpinner size="lg" />
          <p className="text-neutral-600 dark:text-neutral-400">
            Initializing KagiNote...
          </p>
        </div>
      </div>
    );
  }

  return (
    <div
      className={cn(
        'flex flex-col h-screen bg-neutral-50 text-neutral-900',
        'dark:bg-neutral-900 dark:text-neutral-100',
        className
      )}
    >
      {/* Skip link for accessibility */}
      <a
        href="#main-content"
        className="skip-link"
      >
        Skip to main content
      </a>
      
      {/* Title Bar */}
      <TitleBar 
        title={title}
        subtitle={subtitle}
      />
      
      {/* Main Content Area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <Sidebar
          collapsed={sidebarCollapsed}
          onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed)}
        />
        
        {/* Main Content */}
        <main 
          id="main-content"
          className="flex-1 flex flex-col overflow-hidden"
          role="main"
        >
          <div className="flex-1 overflow-auto p-6">
            {loading ? (
              <div className="flex items-center justify-center h-full">
                <div className="text-center space-y-4">
                  <LoadingSpinner size="lg" />
                  <p className="text-neutral-600 dark:text-neutral-400">
                    Loading...
                  </p>
                </div>
              </div>
            ) : (
              children
            )}
          </div>
        </main>
      </div>
      
      {/* Status Bar */}
      <StatusBar
        modelInfo={modelInfo}
        recordingInfo={recordingInfo}
        systemInfo={systemInfo}
      />
    </div>
  );
};

AppLayout.displayName = 'AppLayout';

export { AppLayout };