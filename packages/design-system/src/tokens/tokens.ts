/**
 * Single source of truth for all Mantle design tokens.
 * Run `npm run build:tokens` to regenerate tokens.css and the Tailwind preset.
 */

export const palette = {
  neutral: {
    0: '#ffffff',
    50: '#f5f5f5',
    100: '#ebebeb',
    150: '#e1e1e1',
    200: '#c8c8c8',
    300: '#a8a8a8',
    400: '#8a8a8a',
    500: '#6e6e6e',
    600: '#4a4a4a',
    700: '#383838',
    800: '#2e2e2e',
    850: '#242424',
    900: '#1a1a1a',
    950: '#121212',
  },
  orange: {
    50: '#fef3ee',
    100: '#fde6d4',
    200: '#f9c9a8',
    300: '#f5a77a',
    400: '#f0864b',
    500: '#e8956d',
    600: '#d4754f',
    700: '#b85a37',
    800: '#8f3f22',
    900: '#6b2a10',
  },
  green: {
    400: '#4ade80',
    500: '#22c55e',
  },
  red: {
    400: '#f87171',
    500: '#ef4444',
  },
  yellow: {
    400: '#fbbf24',
    500: '#f59e0b',
  },
  blue: {
    400: '#60a5fa',
    500: '#3b82f6',
  },
} as const;

export const typeScale = {
  fontFamily: {
    sans: "Inter, system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif",
    mono: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Source Code Pro', monospace",
  },
  fontSize: {
    '2xs': '0.625rem',
    xs: '0.75rem',
    sm: '0.8125rem',
    base: '0.875rem',
    md: '1rem',
    lg: '1.125rem',
    xl: '1.25rem',
    '2xl': '1.5rem',
    '3xl': '1.875rem',
    '4xl': '2.25rem',
  },
  fontWeight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
  },
  lineHeight: {
    none: '1',
    tight: '1.25',
    snug: '1.375',
    normal: '1.5',
    relaxed: '1.625',
    loose: '2',
  },
  letterSpacing: {
    tight: '-0.02em',
    normal: '0em',
    wide: '0.02em',
    wider: '0.05em',
  },
} as const;

export const spacing = {
  '0': '0px',
  '0-5': '2px',
  '1': '4px',
  '1-5': '6px',
  '2': '8px',
  '2-5': '10px',
  '3': '12px',
  '3-5': '14px',
  '4': '16px',
  '5': '20px',
  '6': '24px',
  '7': '28px',
  '8': '32px',
  '9': '36px',
  '10': '40px',
  '11': '44px',
  '12': '48px',
  '14': '56px',
  '16': '64px',
  '20': '80px',
  '24': '96px',
  '32': '128px',
} as const;

export const radius = {
  none: '0px',
  xs: '2px',
  sm: '4px',
  md: '6px',
  lg: '8px',
  xl: '12px',
  '2xl': '16px',
  full: '9999px',
} as const;

export const shadow = {
  none: 'none',
  xs: '0 1px 2px 0 rgb(0 0 0 / 0.25)',
  sm: '0 1px 3px 0 rgb(0 0 0 / 0.3), 0 1px 2px -1px rgb(0 0 0 / 0.3)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.3), 0 2px 4px -2px rgb(0 0 0 / 0.3)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.3), 0 4px 6px -4px rgb(0 0 0 / 0.3)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.35), 0 8px 10px -6px rgb(0 0 0 / 0.35)',
} as const;

export const transition = {
  duration: {
    instant: '0ms',
    fast: '100ms',
    normal: '150ms',
    slow: '250ms',
    slower: '350ms',
  },
  easing: {
    linear: 'linear',
    ease: 'ease',
    'ease-in': 'cubic-bezier(0.4, 0, 1, 1)',
    'ease-out': 'cubic-bezier(0, 0, 0.2, 1)',
    'ease-in-out': 'cubic-bezier(0.4, 0, 0.2, 1)',
    spring: 'cubic-bezier(0.175, 0.885, 0.32, 1.275)',
  },
} as const;

export const zIndex = {
  base: '0',
  raised: '10',
  dropdown: '100',
  sticky: '200',
  overlay: '300',
  modal: '400',
  popover: '500',
  tooltip: '600',
  toast: '700',
} as const;

/** Semantic token values for dark theme (default — no data-theme attribute). */
export const semanticDark = {
  color: {
    bg: {
      base: palette.neutral[900],
      surface: palette.neutral[850],
      elevated: palette.neutral[800],
      overlay: palette.neutral[700],
      inverted: palette.neutral[0],
    },
    text: {
      primary: palette.neutral[50],
      secondary: palette.neutral[300],
      muted: palette.neutral[500],
      disabled: palette.neutral[600],
      inverted: palette.neutral[900],
      brand: palette.orange[500],
    },
    brand: {
      default: palette.orange[500],
      hover: palette.orange[400],
      active: palette.orange[600],
      subtle: 'rgb(232 149 109 / 0.15)',
      border: palette.orange[700],
    },
    border: {
      default: palette.neutral[700],
      strong: palette.neutral[600],
      subtle: palette.neutral[800],
      brand: palette.orange[700],
    },
    status: {
      success: palette.green[400],
      'success-subtle': 'rgb(74 222 128 / 0.12)',
      error: palette.red[400],
      'error-subtle': 'rgb(248 113 113 / 0.12)',
      warning: palette.yellow[400],
      'warning-subtle': 'rgb(251 191 36 / 0.12)',
      info: palette.blue[400],
      'info-subtle': 'rgb(96 165 250 / 0.12)',
    },
    interactive: {
      'hover-bg': 'rgb(255 255 255 / 0.05)',
      'active-bg': 'rgb(255 255 255 / 0.08)',
      'focus-ring': palette.orange[500],
    },
  },
} as const;

/** Semantic token values for light theme (data-theme="light"). */
export const semanticLight = {
  color: {
    bg: {
      base: palette.neutral[0],
      surface: palette.neutral[50],
      elevated: palette.neutral[100],
      overlay: palette.neutral[150],
      inverted: palette.neutral[900],
    },
    text: {
      primary: palette.neutral[900],
      secondary: palette.neutral[500],
      muted: palette.neutral[400],
      disabled: palette.neutral[300],
      inverted: palette.neutral[0],
      brand: palette.orange[700],
    },
    brand: {
      default: palette.orange[700],
      hover: palette.orange[600],
      active: palette.orange[800],
      subtle: 'rgb(184 90 55 / 0.1)',
      border: palette.orange[300],
    },
    border: {
      default: palette.neutral[150],
      strong: palette.neutral[200],
      subtle: palette.neutral[100],
      brand: palette.orange[300],
    },
    status: {
      success: palette.green[500],
      'success-subtle': 'rgb(34 197 94 / 0.1)',
      error: palette.red[500],
      'error-subtle': 'rgb(239 68 68 / 0.1)',
      warning: palette.yellow[500],
      'warning-subtle': 'rgb(245 158 11 / 0.1)',
      info: palette.blue[500],
      'info-subtle': 'rgb(59 130 246 / 0.1)',
    },
    interactive: {
      'hover-bg': 'rgb(0 0 0 / 0.04)',
      'active-bg': 'rgb(0 0 0 / 0.08)',
      'focus-ring': palette.orange[700],
    },
  },
} as const;
