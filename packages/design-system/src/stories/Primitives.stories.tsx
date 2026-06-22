import type { Meta, StoryObj } from '@storybook/react'
import { Box } from '../primitives/Box/Box.js'
import { Stack } from '../primitives/Stack/Stack.js'
import { Text } from '../primitives/Text/Text.js'
import {
  Icon,
  ChevronDownIcon,
  CloseIcon,
  CheckIcon,
  SearchIcon,
  InfoIcon,
  WarningIcon,
  ErrorIcon,
  SuccessIcon,
  SettingsIcon,
} from '../primitives/Icon/Icon.js'

/* ---- Box ---- */
const BoxMeta: Meta<typeof Box> = {
  title: 'Primitives/Box',
  component: Box,
  tags: ['autodocs'],
}
export default BoxMeta

export const BoxDefault: StoryObj<typeof Box> = {
  name: 'Default',
  render: () => (
    <Box
      style={{
        padding: 'var(--spacing-4)',
        background: 'var(--color-bg-surface)',
        borderRadius: 'var(--radius-lg)',
        border: '1px solid var(--color-border-default)',
        color: 'var(--color-text-primary)',
        fontFamily: 'var(--font-sans)',
        fontSize: 'var(--font-size-sm)',
      }}
    >
      Box — renders as a &lt;div&gt; by default
    </Box>
  ),
}

export const BoxAsSection: StoryObj<typeof Box> = {
  name: 'Polymorphic (as section)',
  render: () => (
    <Box
      as="section"
      style={{
        padding: 'var(--spacing-4)',
        background: 'var(--color-bg-elevated)',
        borderRadius: 'var(--radius-lg)',
        color: 'var(--color-text-primary)',
        fontFamily: 'var(--font-sans)',
        fontSize: 'var(--font-size-sm)',
      }}
    >
      Renders as &lt;section&gt;
    </Box>
  ),
}

/* ---- Stack ---- */
export const StackStories: StoryObj = {
  name: 'Stack — column',
  render: () => (
    <Stack gap="4">
      {['First', 'Second', 'Third'].map((t) => (
        <Box
          key={t}
          style={{
            padding: 'var(--spacing-3) var(--spacing-4)',
            background: 'var(--color-bg-elevated)',
            borderRadius: 'var(--radius-md)',
            color: 'var(--color-text-primary)',
            fontFamily: 'var(--font-sans)',
            fontSize: 'var(--font-size-sm)',
          }}
        >
          {t}
        </Box>
      ))}
    </Stack>
  ),
}

export const StackRow: StoryObj = {
  name: 'Stack — row',
  render: () => (
    <Stack direction="row" gap="3" align="center">
      {['A', 'B', 'C'].map((t) => (
        <Box
          key={t}
          style={{
            padding: 'var(--spacing-3)',
            background: 'var(--color-brand-subtle)',
            borderRadius: 'var(--radius-md)',
            color: 'var(--color-text-brand)',
            fontFamily: 'var(--font-sans)',
            fontWeight: 'var(--font-weight-medium)',
          }}
        >
          {t}
        </Box>
      ))}
    </Stack>
  ),
}

/* ---- Text ---- */
export const TextScale: StoryObj = {
  name: 'Text — type scale',
  render: () => (
    <Stack gap="3">
      {(['4xl', '3xl', '2xl', 'xl', 'lg', 'md', 'base', 'sm', 'xs', '2xs'] as const).map((size) => (
        <Text key={size} as="p" size={size}>
          {size} — The quick brown fox jumps over the lazy dog
        </Text>
      ))}
    </Stack>
  ),
}

export const TextColors: StoryObj = {
  name: 'Text — colors',
  render: () => (
    <Stack gap="2">
      {(['primary', 'secondary', 'muted', 'brand', 'error', 'success', 'warning'] as const).map(
        (c) => (
          <Text key={c} color={c}>
            color=&quot;{c}&quot;
          </Text>
        ),
      )}
    </Stack>
  ),
}

export const TextMono: StoryObj = {
  name: 'Text — monospace',
  render: () => (
    <Text font="mono" size="sm">
      const greeting = &apos;hello, world&apos;;
    </Text>
  ),
}

/* ---- Icon ---- */
export const IconSizes: StoryObj = {
  name: 'Icon — sizes',
  render: () => (
    <Stack direction="row" gap="4" align="center">
      {(['xs', 'sm', 'md', 'lg', 'xl', '2xl'] as const).map((size) => (
        <Stack key={size} gap="1" align="center">
          <CheckIcon size={size} />
          <Text size="2xs" color="muted">
            {size}
          </Text>
        </Stack>
      ))}
    </Stack>
  ),
}

export const IconGallery: StoryObj = {
  name: 'Icon — gallery',
  render: () => (
    <Stack direction="row" gap="4" align="center" wrap>
      <ChevronDownIcon size="lg" label="Chevron down" />
      <CloseIcon size="lg" label="Close" />
      <CheckIcon size="lg" label="Check" />
      <SearchIcon size="lg" label="Search" />
      <InfoIcon size="lg" label="Info" />
      <WarningIcon size="lg" label="Warning" />
      <ErrorIcon size="lg" label="Error" />
      <SuccessIcon size="lg" label="Success" />
      <SettingsIcon size="lg" label="Settings" />
    </Stack>
  ),
}
