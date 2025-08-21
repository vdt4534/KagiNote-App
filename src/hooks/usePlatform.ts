import { useEffect, useState } from 'react';
import { platform } from '@tauri-apps/plugin-os';

export type Platform = 'macos' | 'windows' | 'linux' | 'web';

export interface PlatformInfo {
  platform: Platform;
  isLoading: boolean;
  isMacOS: boolean;
  isWindows: boolean;
  isLinux: boolean;
  isWeb: boolean;
}

export const usePlatform = (): PlatformInfo => {
  const [currentPlatform, setCurrentPlatform] = useState<Platform>('web');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const detectPlatform = async () => {
      try {
        const platformName = await platform();
        
        // Map Tauri platform names to our platform type
        let detectedPlatform: Platform;
        switch (platformName as string) {
          case 'darwin':
            detectedPlatform = 'macos';
            break;
          case 'win32':
            detectedPlatform = 'windows';
            break;
          case 'linux':
            detectedPlatform = 'linux';
            break;
          default:
            detectedPlatform = 'web';
        }
        
        setCurrentPlatform(detectedPlatform);
        
        // Add platform class to document for CSS targeting
        document.documentElement.classList.remove('platform-macos', 'platform-windows', 'platform-linux', 'platform-web');
        document.documentElement.classList.add(`platform-${detectedPlatform}`);
        
        // Set CSS custom property for platform
        document.documentElement.style.setProperty('--platform', detectedPlatform);
        
      } catch (error) {
        // Fallback to web if Tauri API not available (development mode)
        console.warn('Tauri platform API not available, defaulting to web');
        setCurrentPlatform('web');
        document.documentElement.classList.add('platform-web');
        document.documentElement.style.setProperty('--platform', 'web');
      } finally {
        setIsLoading(false);
      }
    };

    detectPlatform();
  }, []);

  return {
    platform: currentPlatform,
    isLoading,
    isMacOS: currentPlatform === 'macos',
    isWindows: currentPlatform === 'windows',
    isLinux: currentPlatform === 'linux',
    isWeb: currentPlatform === 'web',
  };
};

/**
 * Hook to get platform-specific configuration
 */
export const usePlatformConfig = () => {
  const { platform } = usePlatform();
  
  const config = {
    titlebarHeight: platform === 'macos' ? 44 : 32,
    windowControlsLeft: platform === 'macos',
    hasTrafficLights: platform === 'macos',
    supportsBlur: platform === 'macos' || platform === 'windows',
    
    // Platform-specific keyboard shortcuts
    shortcuts: {
      meta: platform === 'macos' ? 'cmd' : 'ctrl',
      metaKey: platform === 'macos' ? 'metaKey' : 'ctrlKey',
    },
    
    // Platform-specific styling
    styling: {
      borderRadius: platform === 'windows' ? '8px' : '12px',
      backdropBlur: platform === 'macos' ? '20px' : platform === 'windows' ? '40px' : 'none',
      titlebarBlur: platform === 'macos' || platform === 'windows',
    },
  };
  
  return config;
};

/**
 * Hook for platform-specific keyboard shortcuts
 */
export const usePlatformShortcuts = () => {
  const { platform } = usePlatform();
  
  const getShortcutDisplay = (keys: string[]): string => {
    const symbols = {
      macos: {
        meta: '⌘',
        shift: '⇧',
        alt: '⌥',
        ctrl: '⌃',
      },
      windows: {
        meta: 'Ctrl',
        shift: 'Shift',
        alt: 'Alt',
        ctrl: 'Ctrl',
      },
      linux: {
        meta: 'Ctrl',
        shift: 'Shift',
        alt: 'Alt',
        ctrl: 'Ctrl',
      },
    };
    
    const platformSymbols = symbols[platform as keyof typeof symbols] || symbols.windows;
    
    return keys
      .map(key => {
        const lowerKey = key.toLowerCase();
        return platformSymbols[lowerKey as keyof typeof platformSymbols] || key;
      })
      .join(platform === 'macos' ? '' : '+');
  };
  
  const isMetaKey = (event: KeyboardEvent): boolean => {
    return platform === 'macos' ? event.metaKey : event.ctrlKey;
  };
  
  const matchesShortcut = (
    event: KeyboardEvent, 
    shortcut: { key: string; meta?: boolean; shift?: boolean; alt?: boolean }
  ): boolean => {
    const keyMatches = event.key.toLowerCase() === shortcut.key.toLowerCase();
    const metaMatches = !shortcut.meta || isMetaKey(event);
    const shiftMatches = !shortcut.shift || event.shiftKey;
    const altMatches = !shortcut.alt || event.altKey;
    
    return keyMatches && metaMatches && shiftMatches && altMatches;
  };
  
  return {
    getShortcutDisplay,
    isMetaKey,
    matchesShortcut,
    platform,
  };
};