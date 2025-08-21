/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'media', // Respect system preference
  theme: {
    extend: {
      // Typography System
      fontFamily: {
        'sans': [
          'SF Pro Text',
          'SF Pro Display',
          'Segoe UI Variable Text',
          'Segoe UI Variable Display',
          'system-ui',
          '-apple-system',
          'sans-serif'
        ],
        'cjk': [
          'SF Pro Text',
          'Segoe UI Variable Text',
          'Noto Sans CJK JP',
          'Hiragino Kaku Gothic ProN',
          'Yu Gothic Medium',
          'system-ui',
          'sans-serif'
        ],
        'mono': [
          'SF Mono',
          'Monaco',
          'Cascadia Code',
          'Roboto Mono',
          'Courier New',
          'monospace'
        ]
      },
      
      // Color System
      colors: {
        // Primary - Trust Blue
        primary: {
          50: '#EFF6FF',
          100: '#DBEAFE',
          200: '#BFDBFE',
          300: '#93C5FD',
          400: '#60A5FA',
          500: '#3B82F6',
          600: '#2563EB',
          700: '#1D4ED8',
          800: '#1E40AF',
          900: '#1E3A8A',
        },
        // Secondary - Privacy Green
        secondary: {
          50: '#ECFDF5',
          100: '#D1FAE5',
          200: '#A7F3D0',
          300: '#6EE7B7',
          400: '#34D399',
          500: '#10B981',
          600: '#059669',
          700: '#047857',
          800: '#065F46',
          900: '#064E3B',
        },
        // Neutral - Professional Grays
        neutral: {
          50: '#F9FAFB',
          100: '#F3F4F6',
          200: '#E5E7EB',
          300: '#D1D5DB',
          400: '#9CA3AF',
          500: '#6B7280',
          600: '#4B5563',
          700: '#374151',
          800: '#1F2937',
          900: '#111827',
        },
        // Warning
        warning: {
          50: '#FFFBEB',
          100: '#FEF3C7',
          200: '#FDE68A',
          300: '#FCD34D',
          400: '#FBBF24',
          500: '#F59E0B',
          600: '#D97706',
          700: '#B45309',
          800: '#92400E',
          900: '#78350F',
        },
        // Error
        error: {
          50: '#FEF2F2',
          100: '#FEE2E2',
          200: '#FECACA',
          300: '#FCA5A5',
          400: '#F87171',
          500: '#EF4444',
          600: '#DC2626',
          700: '#B91C1C',
          800: '#991B1B',
          900: '#7F1D1D',
        }
      },
      
      // Spacing (4px base unit)
      spacing: {
        '18': '4.5rem',    // 72px
        '88': '22rem',     // 352px
        'sidebar': '17.5rem', // 280px
        'titlebar': '2.75rem', // 44px (macOS)
        'titlebar-win': '2rem', // 32px (Windows)
      },
      
      // Desktop-specific sizing
      width: {
        'sidebar': '17.5rem', // 280px
      },
      height: {
        'titlebar': '2.75rem', // 44px
        'titlebar-win': '2rem', // 32px
        'toolbar': '3.5rem',    // 56px
        'statusbar': '2rem',    // 32px
      },
      
      // Border radius
      borderRadius: {
        'sm': '0.125rem',  // 2px
        'base': '0.25rem', // 4px
        'md': '0.375rem',  // 6px
        'lg': '0.5rem',    // 8px
        'xl': '0.75rem',   // 12px
        '2xl': '1rem',     // 16px
      },
      
      // Shadows
      boxShadow: {
        'window': '0 8px 32px rgba(0, 0, 0, 0.12)',
        'modal': '0 16px 48px rgba(0, 0, 0, 0.24)',
      },
      
      // Backdrop blur (for platform effects)
      backdropBlur: {
        'macos': '20px',
        'windows': '40px',
      },
      
      // Animations
      animation: {
        'pulse-recording': 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'fade-in': 'fadeIn 0.2s ease-out',
        'slide-in': 'slideIn 0.3s ease-out',
        'spin': 'spin 1s linear infinite',
      },
      
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideIn: {
          '0%': { transform: 'translateY(-10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        spin: {
          '0%': { transform: 'rotate(0deg)' },
          '100%': { transform: 'rotate(360deg)' },
        }
      },
      
      // Z-index scale
      zIndex: {
        'titlebar': '1000',
        'sidebar': '900',
        'modal': '2000',
        'tooltip': '3000',
      }
    },
  },
  plugins: [],
}