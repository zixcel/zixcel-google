import assert from 'node:assert/strict'
import test from 'node:test'
import { mkdtempSync, writeFileSync, symlinkSync, copyFileSync, chmodSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { createControl, createNativePlanner } from '../dist/index.mjs'
import { selection } from './native-support.mjs'
const account = { provider: 'google-cloud', subject: 'synthetic-owner', account: '1234567890' }
test('native selection rejects PATH, digest substitution, scripts, symlinks and writable executables', () => {
  assert.throws(() => createNativePlanner({ executable: 'zix-google', sha256: selection.sha256 }))
  assert.throws(() => createNativePlanner({ ...selection, sha256: '0'.repeat(64) }))
  const directory = mkdtempSync(join(tmpdir(), 'google-native-boundary-'))
  try {
    const link = join(directory, 'link'), script = join(directory, 'script'), binary = join(directory, 'binary')
    symlinkSync(selection.executable, link)
    writeFileSync(script, '#!/bin/sh\nprintf unsafe\n', { mode: 0o700 })
    copyFileSync(selection.executable, binary); chmodSync(binary, 0o777)
    for (const executable of [link, script, binary, directory]) {
      assert.throws(() => createNativePlanner({ ...selection, executable }), /request-rejected/)
    }
  } finally { rmSync(directory, { recursive: true, force: true }) }
})
test('planning rejects stale-label inputs and invalid scope before acquiring credentials or HTTP', () => {
  const planner = createNativePlanner(selection)
  let credentials = 0, requests = 0
  const control = createControl(account, { acquire: async () => { credentials++; return 'synthetic-only-not-a-real-token' } }, planner,
    async () => { requests++; return new Response('{}') })
  assert.throws(() => control.prepare('project.set-labels', { labels: {} }), /request-rejected/)
  assert.throws(() => control.prepare('permissions.test', { permissions: ['a.b', 'a.b'] }), /request-rejected/)
  assert.throws(() => control.prepare('account.read', { access_token: 'SYNTHETIC_PRIVATE' }),
    error => !error.message.includes('SYNTHETIC_PRIVATE'))
  assert.equal(credentials, 0); assert.equal(requests, 0)
})
test('write grants are one-use and failed requests do not get an automatic replay', async () => {
  const planner = createNativePlanner(selection)
  let requests = 0, credentials = 0
  const control = createControl(account, { acquire: async () => { credentials++; return 'synthetic-only-not-a-real-token' } }, planner,
    async () => { requests++; throw new Error('SYNTHETIC_PRIVATE_TRANSPORT') })
  const plan = control.prepare('project.set-labels', { labels: { env: 'test' }, etag: 'observed-etag' })
  const grant = control.approve(plan, Date.now() + 30000)
  await assert.rejects(control.execute(grant.id, plan))
  await assert.rejects(control.execute(grant.id, plan))
  assert.equal(requests, 1); assert.equal(credentials, 1)
})
