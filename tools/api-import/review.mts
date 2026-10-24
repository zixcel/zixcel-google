/** Reuse the reviewed Zix importer. No downloader, schema engine or auto-adoption. */
import { createHash } from 'node:crypto'
import { readFileSync, realpathSync, statSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import { resolve, isAbsolute } from 'node:path'
const root = resolve(import.meta.dirname, '../..')
const [binary, directory] = process.argv.slice(2)
if (!binary || !directory || !isAbsolute(binary) || !isAbsolute(directory)) throw Error('Usage: review.mts ABSOLUTE_VERIFIED_ZIX ABSOLUTE_SPEC_CACHE')
const lock = JSON.parse(readFileSync(resolve(root, 'api/sources.lock.json'), 'utf8')) as {
  importer: { binary_sha256: string }; sources: { api: string; catalog: string; review: string; source_sha256: string; catalog_sha256: string }[]
}
const hash = (b: Uint8Array) => createHash('sha256').update(b).digest('hex')
const selected = realpathSync(binary), stat = statSync(selected)
if (!stat.isFile() || stat.size > 64 * 1024 * 1024 || (stat.mode & 0o022) !== 0
  || hash(readFileSync(selected)) !== lock.importer.binary_sha256) throw Error('Importer identity mismatch; explicit source review required')
for (const source of lock.sources) {
  const path = resolve(directory, `${source.api}.json`)
  if (statSync(path).size > 8 * 1024 * 1024 || hash(readFileSync(path)) !== source.source_sha256) throw Error('Source hash mismatch; review candidate separately')
  const catalog = readFileSync(resolve(root, 'api', source.catalog))
  if (hash(catalog) !== source.catalog_sha256) throw Error('Accepted catalog hash mismatch')
  const actual = execFileSync(selected, ['api', 'import', path, resolve(root, 'api', source.review)], {
    env: {}, timeout: 30000, maxBuffer: 8 * 1024 * 1024,
  })
  if (!actual.equals(catalog)) throw Error('Imported contract drift; nothing adopted')
  console.log(`${source.api}: exact source, selected contracts, reference closure and catalog verified; no binding enabled`)
}
