import React from 'react';
import { cn } from '@/lib/utils';
import { usePlatform } from '@/hooks/usePlatform';
import { Icon } from '@/components/ui/Icon';

export interface TitleBarProps {
  title?: string;
  subtitle?: string;
  showTitle?: boolean;
  className?: string;
  children?: React.ReactNode;
}

const TitleBar: React.FC<TitleBarProps> = ({
  title = 'KagiNote',
  subtitle,
  showTitle = true,
  className,
  children,
}) => {
  const { platform, isMacOS, isWindows } = usePlatform();

  if (isMacOS) {
    return (
      <div
        className={cn(
          'h-titlebar flex items-center titlebar-drag',
          'macos-blur border-b border-neutral-200 dark:border-neutral-700',
          'pl-20', // Space for traffic lights
          className
        )}
        role="banner"
      >
        <div className="titlebar-no-drag flex items-center justify-between w-full pr-6">
          {showTitle && (
            <div className="flex flex-col">
              <h1 className="text-sm font-medium text-neutral-900 dark:text-neutral-100">
                {title}
              </h1>
              {subtitle && (
                <p className="text-xs text-neutral-500 dark:text-neutral-400">
                  {subtitle}
                </p>
              )}
            </div>
          )}
          
          {children && (
            <div className="flex items-center gap-2">
              {children}
            </div>
          )}
        </div>
      </div>
    );
  }

  if (isWindows) {
    return (
      <div
        className={cn(
          'h-titlebar-win flex items-center justify-between titlebar-drag',
          'windows-mica border-b border-neutral-200 dark:border-neutral-700',
          'px-4',
          className
        )}
        role="banner"
      >
        <div className="titlebar-no-drag flex items-center gap-3">
          <div className="flex items-center gap-2">
            <Icon name="shield-check" size="sm" className="text-secondary-600" />
            {showTitle && (
              <div className="flex flex-col">
                <h1 className="text-sm font-medium text-neutral-900 dark:text-neutral-100">
                  {title}
                </h1>
                {subtitle && (
                  <p className="text-xs text-neutral-500 dark:text-neutral-400">
                    {subtitle}
                  </p>
                )}
              </div>
            )}
          </div>
        </div>
        
        <div className="titlebar-no-drag flex items-center gap-2">
          {children}
        </div>
      </div>
    );
  }

  // Linux or fallback
  return (
    <div
      className={cn(
        'h-titlebar-win flex items-center justify-between',
        'bg-neutral-50 dark:bg-neutral-800 border-b border-neutral-200 dark:border-neutral-700',
        'px-4',
        className
      )}
      role="banner"
    >
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <Icon name="shield-check" size="sm" className="text-secondary-600" />
          {showTitle && (
            <div className="flex flex-col">
              <h1 className="text-sm font-medium text-neutral-900 dark:text-neutral-100">
                {title}
              </h1>
              {subtitle && (
                <p className="text-xs text-neutral-500 dark:text-neutral-400">
                  {subtitle}
                </p>
              )}
            </div>
          )}
        </div>
      </div>
      
      <div className="flex items-center gap-2">
        {children}
      </div>
    </div>
  );
};

TitleBar.displayName = 'TitleBar';

export { TitleBar };