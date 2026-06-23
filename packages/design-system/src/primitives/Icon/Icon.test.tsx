import { describe, it, expect } from 'vitest'
import { renderToString } from 'react-dom/server'
import { createElement } from 'react'
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
  LoaderIcon,
} from './Icon.js'

describe('Icon', () => {
  it('renders as SVG', () => {
    const html = renderToString(createElement(Icon, null))
    expect(html).toContain('<svg')
  })

  it('is aria-hidden by default (decorative)', () => {
    const html = renderToString(createElement(Icon, null))
    expect(html).toContain('aria-hidden="true"')
    expect(html).not.toContain('role="img"')
  })

  it('adds aria-label and role=img when label provided', () => {
    const html = renderToString(createElement(Icon, { label: 'Close dialog' }))
    expect(html).toContain('aria-label="Close dialog"')
    expect(html).toContain('role="img"')
    expect(html).not.toContain('aria-hidden')
  })

  it('applies size class', () => {
    const html = renderToString(createElement(Icon, { size: 'xl' }))
    expect(html).toContain('size-xl')
  })

  it('has viewBox="0 0 24 24"', () => {
    const html = renderToString(createElement(Icon, null))
    expect(html).toContain('viewBox="0 0 24 24"')
  })
})

describe('named icon exports', () => {
  const icons = [
    ChevronDownIcon,
    CloseIcon,
    CheckIcon,
    SearchIcon,
    InfoIcon,
    WarningIcon,
    ErrorIcon,
    SuccessIcon,
    SettingsIcon,
    LoaderIcon,
  ]

  it.each(icons)('renders %s as an SVG', (IconComponent) => {
    const html = renderToString(createElement(IconComponent, null))
    expect(html).toContain('<svg')
    expect(html).toContain('viewBox="0 0 24 24"')
  })
})
