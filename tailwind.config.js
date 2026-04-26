/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        // Smainer Brand Colors - Meeting Minutes approved
        void: '#09090B',
        'electric-blue': '#3B82F6',
        
        // shadcn/ui system adapted to Smainer brand
        border: '#27272A',         // zinc-800 for subtle borders
        input: '#27272A',          // zinc-800 for inputs
        ring: '#3B82F6',           // electric blue for focus rings
        background: '#09090B',     // void for main background
        foreground: '#FFFFFF',     // white text on void
        
        primary: {
          DEFAULT: '#3B82F6',      // electric blue
          foreground: '#FFFFFF',   // white on blue
        },
        secondary: {
          DEFAULT: '#18181B',      // zinc-900 for secondary surfaces
          foreground: '#FFFFFF',   // white text
        },
        destructive: {
          DEFAULT: '#EF4444',      // red-500 for errors
          foreground: '#FFFFFF',   // white text
        },
        muted: {
          DEFAULT: '#18181B',      // zinc-900 for muted surfaces
          foreground: '#A1A1AA',   // zinc-400 for muted text
        },
        accent: {
          DEFAULT: '#27272A',      // zinc-800 for accents
          foreground: '#FFFFFF',   // white text
        },
        popover: {
          DEFAULT: '#09090B',      // void background
          foreground: '#FFFFFF',   // white text
        },
        card: {
          DEFAULT: '#18181B',      // zinc-900 for cards
          foreground: '#FFFFFF',   // white text
        },
      },
      borderRadius: {
        lg: '0.5rem',
        md: '0.375rem', 
        sm: '0.125rem',
      },
      spacing: {
        // 4px rhythm for consistent spacing
        '1': '4px',
        '2': '8px',
        '3': '12px',
        '4': '16px',
        '5': '20px',
        '6': '24px',
        '8': '32px',
        '10': '40px',
        '12': '48px',
      },
      fontFamily: {
        mono: ['ui-monospace', 'SFMono-Regular', 'Monaco', 'Consolas', 'Liberation Mono', 'Menlo', 'monospace'],
      },
    },
  },
  plugins: [],
}