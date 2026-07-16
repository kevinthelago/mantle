import { describe, it, expect } from 'vitest'
import { renderToString } from 'react-dom/server'
import { createElement } from 'react'
import { List, ListItem } from './List.js'

describe('List', () => {
  it('renders a ul with role=list by default', () => {
    const html = renderToString(createElement(List, null, createElement(ListItem, null, 'Item')))
    expect(html).toContain('<ul')
    expect(html).toContain('role="list"')
  })

  it('uses role=listbox when interactive=true', () => {
    const html = renderToString(
      createElement(List, { interactive: true }, createElement(ListItem, null, 'Item')),
    )
    expect(html).toContain('role="listbox"')
  })
})

describe('ListItem', () => {
  it('renders as li element', () => {
    const html = renderToString(createElement(ListItem, null, 'Row'))
    expect(html).toContain('<li')
    expect(html).toContain('Row')
  })

  it('sets role=option and aria-selected when interactive', () => {
    const html = renderToString(
      createElement(ListItem, { interactive: true, selected: false }, 'Option'),
    )
    expect(html).toContain('role="option"')
    expect(html).toContain('aria-selected="false"')
  })

  it('marks aria-selected=true when selected', () => {
    const html = renderToString(
      createElement(ListItem, { interactive: true, selected: true }, 'Selected'),
    )
    expect(html).toContain('aria-selected="true"')
  })

  it('marks aria-disabled when disabled', () => {
    const html = renderToString(createElement(ListItem, { disabled: true }, 'Disabled'))
    expect(html).toContain('aria-disabled="true"')
  })

  it('renders startSlot with aria-hidden', () => {
    const icon = createElement('span', null, '★')
    const html = renderToString(createElement(ListItem, { startSlot: icon }, 'With icon'))
    expect(html).toContain('aria-hidden="true"')
    expect(html).toContain('★')
  })

  it('renders description text', () => {
    const html = renderToString(createElement(ListItem, { description: 'Secondary text' }, 'Title'))
    expect(html).toContain('Secondary text')
    expect(html).toContain('Title')
  })

  it('renders endSlot', () => {
    const badge = createElement('span', null, '3')
    const html = renderToString(createElement(ListItem, { endSlot: badge }, 'Notifications'))
    expect(html).toContain('>3<')
  })
})
