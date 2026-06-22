import { create } from '@storybook/theming/create'

export const mantleDarkTheme = create({
  base: 'dark',
  brandTitle: 'Mantle Design System',
  brandUrl: 'https://github.com/kevinthelago/mantle',

  colorPrimary: '#e8956d',
  colorSecondary: '#e8956d',

  /* UI */
  appBg: '#1a1a1a',
  appContentBg: '#242424',
  appBorderColor: '#383838',
  appBorderRadius: 6,

  /* Text */
  textColor: '#f5f5f5',
  textInverseColor: '#1a1a1a',
  textMutedColor: '#6e6e6e',

  /* Toolbar */
  barTextColor: '#a8a8a8',
  barHoverColor: '#f5f5f5',
  barSelectedColor: '#e8956d',
  barBg: '#242424',

  /* Inputs */
  inputBg: '#2e2e2e',
  inputBorder: '#383838',
  inputTextColor: '#f5f5f5',
  inputBorderRadius: 6,
})

export const mantleLightTheme = create({
  base: 'light',
  brandTitle: 'Mantle Design System',
  brandUrl: 'https://github.com/kevinthelago/mantle',

  colorPrimary: '#b85a37',
  colorSecondary: '#b85a37',

  appBg: '#f5f5f5',
  appContentBg: '#ffffff',
  appBorderColor: '#e1e1e1',
  appBorderRadius: 6,

  textColor: '#1a1a1a',
  textInverseColor: '#ffffff',
  textMutedColor: '#8a8a8a',

  barTextColor: '#6e6e6e',
  barHoverColor: '#1a1a1a',
  barSelectedColor: '#b85a37',
  barBg: '#f5f5f5',

  inputBg: '#ffffff',
  inputBorder: '#e1e1e1',
  inputTextColor: '#1a1a1a',
  inputBorderRadius: 6,
})
