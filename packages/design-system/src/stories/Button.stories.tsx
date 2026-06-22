import type { Meta, StoryObj } from '@storybook/react';
import { Button } from '../components/Button/Button.js';
import { Stack } from '../primitives/Stack/Stack.js';
import { SearchIcon, CheckIcon } from '../primitives/Icon/Icon.js';

const meta: Meta<typeof Button> = {
  title: 'Components/Button',
  component: Button,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'secondary', 'ghost', 'destructive'],
    },
    size: { control: 'select', options: ['sm', 'md', 'lg'] },
    loading: { control: 'boolean' },
    disabled: { control: 'boolean' },
    children: { control: 'text' },
  },
};
export default meta;

type Story = StoryObj<typeof Button>;

export const Primary: Story = {
  args: { variant: 'primary', children: 'Get started' },
};

export const Secondary: Story = {
  args: { variant: 'secondary', children: 'Cancel' },
};

export const Ghost: Story = {
  args: { variant: 'ghost', children: 'Learn more' },
};

export const Destructive: Story = {
  args: { variant: 'destructive', children: 'Delete account' },
};

export const Loading: Story = {
  args: { variant: 'primary', loading: true, children: 'Saving…' },
};

export const Disabled: Story = {
  args: { variant: 'primary', disabled: true, children: 'Unavailable' },
};

export const WithIcons: Story = {
  args: {
    variant: 'primary',
    startIcon: <SearchIcon size="sm" />,
    endIcon: <CheckIcon size="sm" />,
    children: 'Search',
  },
};

export const Sizes: StoryObj = {
  name: 'All sizes',
  render: () => (
    <Stack direction="row" gap="3" align="center">
      <Button variant="primary" size="sm">Small</Button>
      <Button variant="primary" size="md">Medium</Button>
      <Button variant="primary" size="lg">Large</Button>
    </Stack>
  ),
};

export const AllVariants: StoryObj = {
  name: 'All variants',
  render: () => (
    <Stack direction="row" gap="3" align="center" wrap>
      <Button variant="primary">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="destructive">Destructive</Button>
    </Stack>
  ),
};
