import React from 'react';
import { cn } from '@/lib/utils';

// Icon size mapping
const sizeClasses = {
  sm: 'w-4 h-4',
  base: 'w-6 h-6',
  lg: 'w-8 h-8',
  xl: 'w-10 h-10',
} as const;

export interface IconProps {
  name: string;
  size?: keyof typeof sizeClasses;
  className?: string;
  'aria-label'?: string;
}

// Icon components (using heroicons-style SVGs)
const icons = {
  // Navigation & UI
  home: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
    </svg>
  ),
  
  cog: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
      <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
    </svg>
  ),
  
  search: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
  ),
  
  // Audio & Recording
  microphone: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
    </svg>
  ),
  
  'microphone-slash': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M5.586 5.586A2 2 0 017 5v6a3 3 0 005.395 1.74M9 9v3a3 3 0 002.08 2.86l1.92-1.92M15 9V5a3 3 0 10-6 0v1m6 8a7.001 7.001 0 01-11.04-2.05M21 21l-18-18" />
    </svg>
  ),
  
  play: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="currentColor" {...props}>
      <path d="M8 5v14l11-7z"/>
    </svg>
  ),
  
  pause: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="currentColor" {...props}>
      <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/>
    </svg>
  ),
  
  stop: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="currentColor" {...props}>
      <rect x="4" y="4" width="16" height="16" rx="2"/>
    </svg>
  ),
  
  'volume-up': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M11 18h-2a1 1 0 01-1-1v-4a1 1 0 011-1h2l4-4v14l-4-4z" />
    </svg>
  ),
  
  // Privacy & Security
  'shield-check': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
    </svg>
  ),
  
  'lock-closed': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
    </svg>
  ),
  
  // File Operations
  'document-text': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
    </svg>
  ),
  
  folder: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
    </svg>
  ),
  
  upload: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
    </svg>
  ),
  
  download: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
    </svg>
  ),
  
  // Status & Feedback
  'check-circle': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  ),
  
  'x-circle': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  ),
  
  'exclamation-triangle': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
    </svg>
  ),
  
  'information-circle': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  ),
  
  clock: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  ),
  
  // Actions
  plus: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
    </svg>
  ),
  
  pencil: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
    </svg>
  ),
  
  x: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
    </svg>
  ),
  
  'chevron-down': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
    </svg>
  ),
  
  'chevron-right': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
    </svg>
  ),
  
  // Special
  'eye-slash': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21" />
    </svg>
  ),
} as const;

export const Icon: React.FC<IconProps> = ({
  name,
  size = 'base',
  className = '',
  'aria-label': ariaLabel,
}) => {
  const IconComponent = icons[name as keyof typeof icons];
  
  if (!IconComponent) {
    console.warn(`Icon "${name}" not found`);
    return null;
  }

  return (
    <IconComponent
      className={cn(sizeClasses[size], className)}
      aria-label={ariaLabel}
      role={ariaLabel ? 'img' : 'presentation'}
      aria-hidden={!ariaLabel}
    />
  );
};