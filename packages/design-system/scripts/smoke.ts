/**
 * webkit2gtk smoke render test.
 * Requires: npm install --save-dev playwright @playwright/test
 * Run:      npm run smoke
 *
 * Starts the Storybook dev server, renders key stories in a webkit browser
 * (approximating the webkit2gtk engine Tauri uses), and fails on console errors.
 */

import { chromium, webkit } from 'playwright'
import { spawn, type ChildProcess } from 'child_process'

const SB_PORT = 6007
const SB_URL = `http://localhost:${SB_PORT}`
const TIMEOUT_MS = 60_000

const SMOKE_PATHS = [
  '/?path=/story/primitives-box--box-default',
  '/?path=/story/components-button--primary',
  '/?path=/story/components-button--all-variants',
  '/?path=/story/components-badge--badge-variants',
  '/?path=/story/components-spinner--spinner-sizes',
  '/?path=/story/components-progressbar--progress-bar-variants',
  '/?path=/story/components-separator--separator-demo',
  '/?path=/story/components-list-list-item--list-demo',
]

async function waitForServer(url: string, timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    try {
      const res = await fetch(url)
      if (res.ok) return
    } catch {
      /* not up yet */
    }
    await new Promise((r) => setTimeout(r, 500))
  }
  throw new Error(`Server at ${url} did not start within ${timeoutMs}ms`)
}

async function main(): Promise<void> {
  let sb: ChildProcess | undefined
  let browser

  try {
    console.log('Starting Storybook…')
    sb = spawn('npm', ['run', 'storybook', '--', '--port', String(SB_PORT), '--ci'], {
      shell: true,
      stdio: 'inherit',
    })

    await waitForServer(SB_URL, TIMEOUT_MS)
    console.log('Storybook up. Launching webkit…')

    // Use webkit if available (approximates webkit2gtk), fall back to chromium
    try {
      browser = await webkit.launch({ headless: true })
    } catch {
      console.warn('webkit not installed — falling back to chromium')
      browser = await chromium.launch({ headless: true })
    }

    const page = await browser.newPage()
    const errors: string[] = []
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(msg.text())
    })
    page.on('pageerror', (err) => errors.push(err.message))

    for (const path of SMOKE_PATHS) {
      const url = `${SB_URL}${path}`
      console.log(`  Rendering ${path}`)
      await page.goto(url, { waitUntil: 'networkidle', timeout: 30_000 })
      // Wait for story iframe
      const frame = page.frame({ name: 'storybook-preview-iframe' })
      if (frame) {
        await frame.waitForSelector('body', { timeout: 10_000 })
      }
    }

    if (errors.length > 0) {
      console.error('Console errors detected:')
      errors.forEach((e) => console.error(' ', e))
      process.exitCode = 1
    } else {
      console.log(`✓ Smoke render passed (${SMOKE_PATHS.length} stories)`)
    }
  } finally {
    await browser?.close()
    sb?.kill()
  }
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
