import { createHash } from 'node:crypto'
import { closeSync, constants, fstatSync, openSync, readSync } from 'node:fs'
import { isAbsolute } from 'node:path'

export function selectNative(executable: string | undefined, sha256: string | undefined) {
  if (!executable || !isAbsolute(executable) || !sha256 || !/^[a-f0-9]{64}$/.test(sha256)) {
    throw new Error('Set ZIX_GOOGLE_TEST_BINARY and ZIX_GOOGLE_TEST_SHA256 to a verified native artifact; npm test never builds Rust')
  }
  const fd = openSync(executable, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK)
  try {
    const before = fstatSync(fd)
    if (!before.isFile() || before.size < 64 || before.size > 64 * 1024 * 1024) {
      throw new Error('Native test artifact must be a regular file of 64 bytes to 64 MiB')
    }
    const hash = createHash('sha256'), buffer = Buffer.alloc(65536)
    let offset = 0
    while (offset < before.size) {
      const count = readSync(fd, buffer, 0, Math.min(buffer.length, before.size - offset), offset)
      if (count === 0) throw new Error('Native test artifact was truncated')
      hash.update(buffer.subarray(0, count)); offset += count
    }
    const after = fstatSync(fd)
    if (after.size !== before.size || after.mtimeMs !== before.mtimeMs || after.ctimeMs !== before.ctimeMs
      || hash.digest('hex') !== sha256) throw new Error('Native test artifact changed or checksum mismatched')
    return { executable, sha256 }
  } finally { closeSync(fd) }
}

export const selection = selectNative(process.env.ZIX_GOOGLE_TEST_BINARY, process.env.ZIX_GOOGLE_TEST_SHA256)
export const executable = selection.executable
