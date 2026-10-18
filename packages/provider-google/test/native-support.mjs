import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
export const executable = process.env.ZIX_GOOGLE_TEST_BINARY
  ?? fileURLToPath(new URL('../../../target/release/zix-google', import.meta.url))
export const selection = { executable, sha256: createHash('sha256').update(readFileSync(executable)).digest('hex') }
