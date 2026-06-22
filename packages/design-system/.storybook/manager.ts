import { addons } from '@storybook/manager-api';
import { mantleDarkTheme } from './mantleTheme.js';

addons.setConfig({
  theme: mantleDarkTheme,
  sidebar: {
    showRoots: true,
  },
});
