import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import dts from 'vite-plugin-dts'
import { resolve } from 'path'

const isStorybook = process.argv.some((arg) => arg.includes('storybook'))

export default defineConfig({
  plugins: [
    react(),
    // Only emit type declarations during library builds, not Storybook.
    ...(!isStorybook
      ? [
          dts({
            include: ['src'],
            exclude: ['src/**/*.stories.tsx'],
          }),
        ]
      : []),
  ],
  resolve: {
    alias: {
      '@tokens': resolve(__dirname, './src/tokens/index.ts'),
      '@primitives': resolve(__dirname, './src/primitives/index.ts'),
      '@components': resolve(__dirname, './src/components/index.ts'),
    },
  },
  css: {
    modules: {
      localsConvention: 'camelCase',
    },
    preprocessorOptions: {
      scss: {
        additionalData: `@use "./src/styles/tokens" as *;`,
      },
    },
  },
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'MantleDesignSystem',
      formats: ['es'],
      fileName: 'index',
    },
    rollupOptions: {
      external: ['react', 'react-dom', 'react/jsx-runtime'],
      output: {
        assetFileNames: (assetInfo) => {
          if (assetInfo.names?.some((n) => n.endsWith('.css'))) {
            return 'styles/[name][extname]'
          }
          return '[name][extname]'
        },
        globals: {
          react: 'React',
          'react-dom': 'ReactDOM',
        },
      },
    },
    cssMinify: false,
  },
})
