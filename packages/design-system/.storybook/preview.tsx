import type { Preview, Decorator } from '@storybook/react'
import { mantleDarkTheme, mantleLightTheme } from './mantleTheme.js'
import '../src/styles/tokens.css'

const withTheme: Decorator = (Story, context) => {
  const theme = (context.globals['theme'] as string) ?? 'dark'
  return (
    <div
      data-theme={theme === 'light' ? 'light' : undefined}
      style={{
        background: theme === 'light' ? '#ffffff' : '#1a1a1a',
        minHeight: '100vh',
        padding: '24px',
        boxSizing: 'border-box',
      }}
    >
      <Story />
    </div>
  )
}

const preview: Preview = {
  decorators: [withTheme],
  globalTypes: {
    theme: {
      description: 'Color theme',
      defaultValue: 'dark',
      toolbar: {
        title: 'Theme',
        icon: 'moon',
        items: [
          { value: 'dark', title: 'Dark', icon: 'moon' },
          { value: 'light', title: 'Light', icon: 'sun' },
        ],
        dynamicTitle: true,
      },
    },
  },
  parameters: {
    backgrounds: { disable: true },
    docs: {
      theme: mantleDarkTheme,
    },
    layout: 'padded',
    actions: { argTypesRegex: '^on[A-Z].*' },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
}

export default preview
