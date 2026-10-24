import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
const root = resolve(import.meta.dirname, '../..')
const read = (name: string) => JSON.parse(readFileSync(resolve(root, 'api', name), 'utf8'))
const lock = read('sources.lock.json'), policy = read('operations.json')
const selected = new Set<string>()
for (const source of lock.sources) {
  const bytes = readFileSync(resolve(root, 'api', source.catalog)), catalog = JSON.parse(bytes.toString()), review = read(source.review)
  if (createHash('sha256').update(bytes).digest('hex') !== source.catalog_sha256
    || catalog.source.sha256 !== source.source_sha256 || catalog.source.revision !== source.revision
    || catalog.source.version !== source.version || catalog.source.source_uri !== source.url
    || review.source_sha256 !== source.source_sha256) throw Error('Source/catalog lock drift')
  if (JSON.stringify([...source.selected_methods].sort()) !== JSON.stringify(catalog.operations.map((x: {id: string}) => x.id).sort())) throw Error('Selection drift')
  for (const operation of catalog.operations) {
    if (operation.binding !== null || selected.has(operation.id)) throw Error('Unreviewed executable or duplicate binding')
    selected.add(operation.id)
  }
}
if (policy.operations.length !== selected.size || new Set(policy.operations.map((x: {api_method: string}) => x.api_method)).size !== selected.size
  || policy.operations.some((x: {api_method: string; automatic_retry: boolean}) => !selected.has(x.api_method) || x.automatic_retry !== false)) throw Error('Owner-policy selection drift')
console.log(`Verified ${selected.size} imported contracts with separate explicit owner policies; no executable API aliases`)
