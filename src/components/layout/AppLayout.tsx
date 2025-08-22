import React, { useState, useEffect } from 'react';
import { cn } from '@/lib/utils';
import { usePlatform } from '@/hooks/usePlatform';
import { TitleBar } from './TitleBar';
import { Sidebar } from './Sidebar';
import { StatusBar } from './StatusBar';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Button, Sheet, SheetContent, SheetTrigger, Icon } from '@/components/ui/compat';

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
  onNavigate?: (screen: string) => void;
  currentScreen?: string;
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
  onNavigate,
  currentScreen = 'dashboard',
  className,
}) => {
  const { platform, isLoading: platformLoading } = usePlatform();
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [isMobile, setIsMobile] = useState(false);

  // Detect mobile viewport
  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768);
    };
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  }, []);

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
      
      {/* Title Bar with mobile menu button */}
      <div className="relative">
        <TitleBar 
          title={title}
          subtitle={subtitle}
        />
        {isMobile && (
          <Sheet open={mobileMenuOpen} onOpenChange={setMobileMenuOpen}>
            <SheetTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="absolute left-4 top-1/2 -translate-y-1/2 md:hidden"
              >
                <Icon name="bars-3" className="h-5 w-5" />
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="p-0 w-[280px]">
              <Sidebar
                collapsed={false}
                onToggleCollapse={() => {}}
                sections={[
                  {
                    items: [
                      { 
                        id: 'home', 
                        label: 'Dashboard', 
                        icon: 'home', 
                        active: currentScreen === 'dashboard',
                        onClick: () => {
                          onNavigate?.('dashboard');
                          setMobileMenuOpen(false);
                        }
                      },
                      { 
                        id: 'record', 
                        label: 'New Recording', 
                        icon: 'microphone',
                        active: currentScreen === 'recording',
                        onClick: () => {
                          onNavigate?.('recording');
                          setMobileMenuOpen(false);
                        }
                      },
                      { 
                        id: 'files', 
                        label: 'Transcripts', 
                        icon: 'document-text', 
                        badge: '3',
                        active: currentScreen === 'transcripts',
                        onClick: () => {
                          onNavigate?.('transcripts');
                          setMobileMenuOpen(false);
                        }
                      },
                      { 
                        id: 'settings', 
                        label: 'Settings', 
                        icon: 'cog',
                        active: currentScreen === 'settings',
                        onClick: () => {
                          onNavigate?.('settings');
                          setMobileMenuOpen(false);
                        }
                      },
                    ],
                  },
                  {
                    title: 'Privacy',
                    items: [
                      { id: 'privacy', label: 'Local Only', icon: 'shield-check' },
                      { id: 'offline', label: 'No Network', icon: 'eye-slash' },
                      { id: 'device', label: 'On Device', icon: 'lock-closed' },
                    ],
                  },
                ]}
              />
            </SheetContent>
          </Sheet>
        )}
      </div>
      
      {/* Main Content Area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Desktop Sidebar - hidden on mobile */}
        {!isMobile && (
          <Sidebar
            collapsed={sidebarCollapsed}
            onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed)}
            sections={[
              {
                items: [
                  { 
                    id: 'home', 
                    label: 'Dashboard', 
                    icon: 'home', 
                    active: currentScreen === 'dashboard',
                    onClick: () => onNavigate?.('dashboard')
                  },
                  { 
                    id: 'record', 
                    label: 'New Recording', 
                    icon: 'microphone',
                    active: currentScreen === 'recording',
                    onClick: () => onNavigate?.('recording')
                  },
                  { 
                    id: 'files', 
                    label: 'Transcripts', 
                    icon: 'document-text', 
                    badge: '3',
                    active: currentScreen === 'transcripts',
                    onClick: () => onNavigate?.('transcripts')
                  },
                  { 
                    id: 'settings', 
                    label: 'Settings', 
                    icon: 'cog',
                    active: currentScreen === 'settings',
                    onClick: () => onNavigate?.('settings')
                  },
                ],
              },
              {
                title: 'Privacy',
                items: [
                  { id: 'privacy', label: 'Local Only', icon: 'shield-check' },
                  { id: 'offline', label: 'No Network', icon: 'eye-slash' },
                  { id: 'device', label: 'On Device', icon: 'lock-closed' },
                ],
              },
            ]}
          />
        )}
        
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