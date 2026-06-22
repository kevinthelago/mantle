import type { StorybookConfig } from '@storybook/react-vite';
import { resolve } from 'path';

const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(ts|tsx)'],
  addons: [
    '@storybook/addon-essentials',
    '@storybook/addon-a11y',
  ],
  framework: {
    name: '@storybook/react-vite',
    options: {},
  },
  viteFinal: (viteConfig) => ({
    ...viteConfig,
    resolve: {
      ...viteConfig.resolve,
      alias: {
        ...(viteConfig.resolve?.alias ?? {}),
        '@tokens': resolve(__dirname, '../src/tokens/index.ts'),
        '@primitives': resolve(__dirname, '../src/primitives/index.ts'),
        '@components': resolve(__dirname, '../src/components/index.ts'),
      },
    },
  }),
};

export default config;
