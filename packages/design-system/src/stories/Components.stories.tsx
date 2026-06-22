import type { Meta, StoryObj } from '@storybook/react';
import { Badge } from '../components/Badge/Badge.js';
import { IconButton } from '../components/IconButton/IconButton.js';
import { Tooltip } from '../components/Tooltip/Tooltip.js';
import { Popover, MenuItem, MenuSeparator } from '../components/Popover/Popover.js';
import { Slider } from '../components/Slider/Slider.js';
import { ProgressBar } from '../components/ProgressBar/ProgressBar.js';
import { Spinner } from '../components/Spinner/Spinner.js';
import { Separator } from '../components/Separator/Separator.js';
import { List, ListItem } from '../components/List/List.js';
import { Stack } from '../primitives/Stack/Stack.js';
import { Text } from '../primitives/Text/Text.js';
import {
  CloseIcon,
  SettingsIcon,
  ChevronDownIcon,
  SearchIcon,
  CheckIcon,
  InfoIcon,
} from '../primitives/Icon/Icon.js';
import { useState } from 'react';
import { Button } from '../components/Button/Button.js';

/* ---- Badge ---- */
const BadgeMeta: Meta<typeof Badge> = {
  title: 'Components/Badge',
  component: Badge,
  tags: ['autodocs'],
};
export default BadgeMeta;

export const BadgeVariants: StoryObj = {
  name: 'All variants',
  render: () => (
    <Stack direction="row" gap="2" align="center" wrap>
      <Badge>default</Badge>
      <Badge variant="brand">brand</Badge>
      <Badge variant="success">success</Badge>
      <Badge variant="error">error</Badge>
      <Badge variant="warning">warning</Badge>
      <Badge variant="info">info</Badge>
    </Stack>
  ),
};

export const BadgeWithDot: StoryObj = {
  name: 'With dot',
  render: () => (
    <Stack direction="row" gap="2" align="center" wrap>
      <Badge dot>Offline</Badge>
      <Badge dot variant="success">Online</Badge>
      <Badge dot variant="warning">Away</Badge>
      <Badge dot variant="error">Error</Badge>
    </Stack>
  ),
};

/* ---- IconButton ---- */
export const IconButtonGallery: StoryObj = {
  name: 'IconButton — variants',
  render: () => (
    <Stack direction="row" gap="3" align="center">
      <IconButton aria-label="Close" variant="ghost"><CloseIcon size="md" /></IconButton>
      <IconButton aria-label="Settings" variant="secondary"><SettingsIcon size="md" /></IconButton>
      <IconButton aria-label="Search" variant="primary"><SearchIcon size="md" /></IconButton>
      <IconButton aria-label="Delete" variant="destructive"><CloseIcon size="md" /></IconButton>
    </Stack>
  ),
};

/* ---- Tooltip ---- */
export const TooltipPlacements: StoryObj = {
  name: 'Tooltip — placements',
  render: () => (
    <Stack direction="row" gap="8" align="center" justify="center" style={{ padding: '60px' }}>
      <Tooltip content="Tooltip on top" placement="top">
        <Button size="sm">Top</Button>
      </Tooltip>
      <Tooltip content="Tooltip on bottom" placement="bottom">
        <Button size="sm">Bottom</Button>
      </Tooltip>
      <Tooltip content="Tooltip on left" placement="left">
        <Button size="sm">Left</Button>
      </Tooltip>
      <Tooltip content="Tooltip on right" placement="right">
        <Button size="sm">Right</Button>
      </Tooltip>
    </Stack>
  ),
};

/* ---- Popover/Menu ---- */
export const PopoverMenu: StoryObj = {
  name: 'Popover / Menu',
  render: () => (
    <Stack direction="row" gap="4" style={{ padding: '32px' }}>
      <Popover
        trigger={
          <Button variant="secondary" endIcon={<ChevronDownIcon size="sm" />}>
            Options
          </Button>
        }
      >
        <MenuItem icon={<SettingsIcon size="md" />}>Settings</MenuItem>
        <MenuItem icon={<SearchIcon size="md" />}>Search</MenuItem>
        <MenuSeparator />
        <MenuItem danger icon={<CloseIcon size="md" />}>Delete</MenuItem>
      </Popover>
    </Stack>
  ),
};

/* ---- Slider ---- */
export const SliderBasic: StoryObj = {
  name: 'Slider',
  render: () => {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    const [v, setV] = useState(40);
    return (
      <Stack gap="4" style={{ maxWidth: 320 }}>
        <Slider
          label="Volume"
          showValue
          min={0}
          max={100}
          value={v}
          onChange={(e) => setV(Number(e.target.value))}
          formatValue={(n) => `${n}%`}
        />
        <Slider label="Opacity" min={0} max={1} step={0.01} defaultValue={0.8} showValue formatValue={(n) => `${Math.round(n * 100)}%`} />
        <Slider label="Disabled" min={0} max={100} defaultValue={60} disabled />
      </Stack>
    );
  },
};

/* ---- ProgressBar ---- */
export const ProgressBarVariants: StoryObj = {
  name: 'ProgressBar — variants',
  render: () => (
    <Stack gap="4" style={{ maxWidth: 400 }}>
      <ProgressBar value={30} label="Downloading" showValue />
      <ProgressBar value={75} color="success" label="Completed" showValue />
      <ProgressBar value={55} color="warning" label="Warning" showValue />
      <ProgressBar value={20} color="error" label="Failed" showValue />
      <ProgressBar label="Indeterminate" />
    </Stack>
  ),
};

/* ---- Spinner ---- */
export const SpinnerSizes: StoryObj = {
  name: 'Spinner — sizes',
  render: () => (
    <Stack direction="row" gap="4" align="center">
      {(['sm', 'md', 'lg', 'xl'] as const).map((s) => (
        <Stack key={s} gap="1" align="center">
          <Spinner size={s} />
          <Text size="2xs" color="muted">{s}</Text>
        </Stack>
      ))}
    </Stack>
  ),
};

/* ---- Separator ---- */
export const SeparatorDemo: StoryObj = {
  name: 'Separator',
  render: () => (
    <Stack gap="4" style={{ maxWidth: 300 }}>
      <Text>Section A</Text>
      <Separator />
      <Text>Section B</Text>
      <Separator strength="strong" />
      <Text>Section C</Text>
      <Stack direction="row" gap="3" align="center">
        <Text>Item 1</Text>
        <Separator orientation="vertical" />
        <Text>Item 2</Text>
        <Separator orientation="vertical" />
        <Text>Item 3</Text>
      </Stack>
    </Stack>
  ),
};

/* ---- List ---- */
export const ListDemo: StoryObj = {
  name: 'List / ListItem',
  render: () => (
    <Stack gap="4">
      <List style={{ maxWidth: 300 }}>
        <ListItem startSlot={<InfoIcon size="md" />} description="Descriptive text">First item</ListItem>
        <ListItem startSlot={<CheckIcon size="md" />} description="With icon and description">Second item</ListItem>
        <ListItem startSlot={<SettingsIcon size="md" />} endSlot={<ChevronDownIcon size="sm" />}>
          Third item
        </ListItem>
        <ListItem disabled>Disabled item</ListItem>
      </List>

      <List interactive style={{ maxWidth: 300 }}>
        {['Option A', 'Option B', 'Option C'].map((opt, i) => (
          <ListItem key={opt} interactive selected={i === 1}>{opt}</ListItem>
        ))}
      </List>
    </Stack>
  ),
};
