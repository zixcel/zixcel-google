import { createHash } from 'node:crypto'
import { constants, openSync, fstatSync, readFileSync, closeSync } from 'node:fs'
import { isAbsolute } from 'node:path'
import { spawnSync } from 'node:child_process'
export interface NativeExecutable { readonly executable: string; readonly sha256: string }

/** Execute an explicitly selected ELF descriptor, never a PATH command or shell wrapper. */
export function invokeNative(binary: NativeExecutable, argument: string, input?: unknown): unknown {
  if (process.platform !== 'linux' || !isAbsolute(binary.executable) || !/^[a-f0-9]{64}$/.test(binary.sha256)) {
    throw new Error('provider/native/selection-invalid')
  }
  let descriptor: number | undefined
  try {
    descriptor = openSync(binary.executable, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK)
    const stat = fstatSync(descriptor)
    if (!stat.isFile() || stat.size < 64 || stat.size > 64 * 1024 * 1024 || (stat.mode & 0o022) !== 0
      || ![0, process.getuid?.()].includes(stat.uid)) throw new Error('rejected')
    const bytes = readFileSync(descriptor)
    if (!bytes.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46]))
      || createHash('sha256').update(bytes).digest('hex') !== binary.sha256) throw new Error('rejected')
    const serialized = input === undefined ? undefined : JSON.stringify(input)
    if (serialized !== undefined && Buffer.byteLength(serialized) > 65536) throw new Error('rejected')
    const result = spawnSync('/proc/self/fd/3', [argument], { env: {}, shell: false,
      stdio: ['pipe', 'pipe', 'pipe', descriptor], input: serialized, timeout: 5000, maxBuffer: 65536 })
    if (result.error || result.status !== 0 || result.signal || result.stderr.length !== 0) throw new Error('rejected')
    return JSON.parse(result.stdout.toString('utf8')) as unknown
  } catch {
    throw new Error('provider/native/request-rejected')
  } finally { if (descriptor !== undefined) closeSync(descriptor) }
}
