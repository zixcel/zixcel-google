import assert from 'node:assert/strict'
import { createControl, createNativePlanner } from '@zixcel/provider-google'
import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
const executable = process.env.ZIX_GOOGLE_TEST_BINARY
if (!executable) throw new Error('Select the verified native binary for this synthetic demonstration')
const planner = createNativePlanner({ executable, sha256: createHash('sha256').update(readFileSync(executable)).digest('hex') })
const operations = planner.catalog.operations
const account = { provider: "google-cloud", subject: 'synthetic-owner', account: '1234567890' }
const samples = {"billing.project-read":{},"account.read": {}, "permissions.test": {"permissions": ["resourcemanager.projects.get"]}, "project.set-labels": {"labels": {"env": "test"}, "etag": "synthetic-etag"}}
let state = {}, credentialUses = 0
const control = createControl(account, { acquire: async () => { credentialUses++; return 'synthetic-only-not-a-real-credential' } }, planner, async (_url, init) => {
  if (init.method !== 'GET') state = JSON.parse(init.body)
  return new Response(JSON.stringify(state))
})
const write = operations.find(operation => operation.effect === 'write')
assert.ok(write)
const plan = control.prepare(write.id, samples[write.id])
const grant = control.approve(plan, Date.now() + 30000)
const changed = await control.execute(grant.id, plan)
const read = control.prepare('account.read', {}), readGrant = control.approve(read, Date.now() + 30000)
const observed = await control.execute(readGrant.id, read)
assert.deepEqual(observed.data, changed.data)
const used = credentialUses
await assert.rejects(control.execute(grant.id, plan))
assert.equal(credentialUses, used)
console.log(JSON.stringify({ provider: account.provider, write: write.id, fakeReadBack: true, replayRefused: true, realAccount: false }))
