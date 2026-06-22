#!/usr/bin/env ts-node
import { writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { generateTokensCSS, generateTailwindPreset } from '../src/tokens/generate.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

const css = generateTokensCSS();
writeFileSync(resolve(root, 'src/styles/tokens.css'), css, 'utf-8');
console.log('✓ src/styles/tokens.css');

const preset = generateTailwindPreset();
const presetTs = [
  '/* Generated — do not edit. Run `npm run build:tokens` to regenerate. */',
  "import type { Config } from 'tailwindcss';",
  '',
  `const preset = ${JSON.stringify(preset, null, 2)} satisfies Partial<Config>;`,
  '',
  'export default preset;',
].join('\n');

writeFileSync(resolve(root, 'tailwind.preset.ts'), presetTs, 'utf-8');
console.log('✓ tailwind.preset.ts');
