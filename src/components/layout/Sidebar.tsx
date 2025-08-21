import React from 'react';
import { cn } from '@/lib/utils';
import { usePlatform } from '@/hooks/usePlatform';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';

export interface SidebarItem {
  id: string;
  label: string;
  icon: string;
  href?: string;
  active?: boolean;
  badge?: string | number;
  onClick?: () => void;
}

export interface SidebarSection {
  title?: string;
  items: SidebarItem[];
}

export interface SidebarProps {
  sections?: SidebarSection[];
  collapsed?: boolean;
  onToggleCollapse?: () => void;
  className?: string;
}

const defaultSections: SidebarSection[] = [
  {
    items: [
      { id: 'home', label: 'Dashboard', icon: 'home', active: true },
      { id: 'record', label: 'New Recording', icon: 'microphone' },
      { id: 'files', label: 'Transcripts', icon: 'document-text', badge: '3' },
      { id: 'settings', label: 'Settings', icon: 'cog' },
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
];

const Sidebar: React.FC<SidebarProps> = ({
  sections = defaultSections,
  collapsed = false,
  onToggleCollapse,
  className,
}) => {
  const { isMacOS } = usePlatform();

  const sidebarClass = cn(
    'flex flex-col w-sidebar h-full border-r border-neutral-200 dark:border-neutral-700',
    'bg-neutral-50 dark:bg-neutral-800',
    isMacOS && 'macos-blur dark:macos-dark-blur',
    collapsed && 'w-16',
    className
  );

  const renderSidebarItem = (item: SidebarItem) => {
    const itemClass = cn(
      'flex items-center gap-3 px-3 py-2 mx-2 rounded-md transition-colors',
      'hover:bg-neutral-100 dark:hover:bg-neutral-700',
      'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2',
      item.active && 'bg-primary-100 dark:bg-primary-900 text-primary-900 dark:text-primary-100',
      collapsed && 'justify-center px-2'
    );

    const handleClick = () => {
      if (item.onClick) {
        item.onClick();
      }
    };

    const content = (
      <>
        <Icon 
          name={item.icon} 
          size="base"
          className={cn(
            'text-neutral-600 dark:text-neutral-400',
            item.active && 'text-primary-600 dark:text-primary-400'
          )}
        />
        
        {!collapsed && (
          <>
            <span className="flex-1 text-sm font-medium text-neutral-900 dark:text-neutral-100">
              {item.label}
            </span>
            
            {item.badge && (
              <Badge variant="neutral" size="sm">
                {item.badge}
              </Badge>
            )}
          </>
        )}
      </>
    );

    if (item.href) {
      return (
        <a
          key={item.id}
          href={item.href}
          className={itemClass}
          onClick={handleClick}
          title={collapsed ? item.label : undefined}
        >
          {content}
        </a>
      );
    }

    return (
      <button
        key={item.id}
        className={itemClass}
        onClick={handleClick}
        title={collapsed ? item.label : undefined}
        type="button"
      >
        {content}
      </button>
    );
  };

  return (
    <nav className={sidebarClass} role="navigation" aria-label="Main navigation">
      <div className="flex-1 py-4 space-y-6 overflow-y-auto scrollbar-thin">
        {sections.map((section, sectionIndex) => (
          <div key={sectionIndex} className="space-y-1">
            {section.title && !collapsed && (
              <h3 className="px-3 text-xs font-semibold text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                {section.title}
              </h3>
            )}
            
            <div className="space-y-1">
              {section.items.map(renderSidebarItem)}
            </div>
          </div>
        ))}
      </div>
      
      {/* Privacy indicator at bottom */}
      <div className="p-4 border-t border-neutral-200 dark:border-neutral-700">
        {!collapsed ? (
          <div className="flex items-center gap-2 text-xs text-secondary-600 dark:text-secondary-400">
            <Icon name="shield-check" size="sm" />
            <span className="font-medium">100% Local Privacy</span>
          </div>
        ) : (
          <div className="flex justify-center">
            <Icon 
              name="shield-check" 
              size="base" 
              className="text-secondary-600 dark:text-secondary-400"
              aria-label="100% Local Privacy"
            />
          </div>
        )}
      </div>
    </nav>
  );
};

Sidebar.displayName = 'Sidebar';

export { Sidebar };