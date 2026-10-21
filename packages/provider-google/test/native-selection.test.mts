import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { mkdtempSync, readdirSync, rmSync, symlinkSync, truncateSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test from 'node:test'
import { selectNative } from './native-support.mts'

test('native selection fails before execution and releases descriptors on rejection', () => {
  const root = mkdtempSync(join(tmpdir(), 'google-native-selection-'))
  try {
    const binary = join(root, 'synthetic'), link = join(root, 'link')
    const bytes = Buffer.alloc(128, 1), digest = createHash('sha256').update(bytes).digest('hex')
    writeFileSync(binary, bytes); symlinkSync(binary, link)
    assert.throws(() => selectNative(undefined, digest), /never builds Rust/)
    assert.throws(() => selectNative('relative', digest), /never builds Rust/)
    assert.throws(() => selectNative(binary, 'not-a-checksum'), /never builds Rust/)
    assert.throws(() => selectNative(link, digest))
    assert.throws(() => selectNative(root, digest), /regular file/)
    const descriptors = readdirSync('/proc/self/fd').length
    for (let i = 0; i < 64; i++) assert.throws(() => selectNative(binary, '0'.repeat(64)), /checksum/)
    assert.equal(readdirSync('/proc/self/fd').length, descriptors)
    assert.deepEqual(selectNative(binary, digest), { executable: binary, sha256: digest })
    truncateSync(binary, 64 * 1024 * 1024 + 1)
    assert.throws(() => selectNative(binary, digest), /64 MiB/)
    truncateSync(binary, 0)
    assert.throws(() => selectNative(binary, digest), /regular file/)
  } finally { rmSync(root, { recursive: true, force: true }) }
})
