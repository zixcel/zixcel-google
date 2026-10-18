import assert from 'node:assert/strict'
import test from 'node:test'
import { createControl, observeTestedPermissions, createNativePlanner } from '../dist/index.mjs'
import { selection } from './native-support.mjs'
const planner = createNativePlanner(selection)
const account = { provider: 'google-cloud', account: '1234567890', subject: 'synthetic-owner' }
async function observed() {
  const control = createControl(account, { acquire: async () => 'synthetic-never-real-credential' }, planner,
    async () => new Response('{"permissions":["resourcemanager.projects.get"]}'))
  const plan = control.prepare('permissions.test', { permissions: ['resourcemanager.projects.get', 'resourcemanager.projects.update'] })
  const receipt = await control.execute(control.approve(plan, Date.now() + 10000).id, plan)
  return { plan, receipt }
}
test('tested rights are scoped/fresh observations and omitted rights are not inferred as denial', async () => {
  const { plan, receipt } = await observed()
  const info = observeTestedPermissions(account, plan, receipt, planner)
  assert.equal(info.authority, 'none')
  assert.equal(info.coverage, 'requested-permission-test-only')
  assert.deepEqual(info.permissions.map(p => p.state), ['returned-by-test', 'not-returned-or-resource-unavailable'])
  assert.ok(Object.isFrozen(info.permissions[0]))
})
test('substituted account, request, payload and stale/future observations are rejected', async () => {
  const { plan, receipt } = await observed()
  assert.throws(() => observeTestedPermissions({ ...account, account: 'other' }, plan, receipt, planner), /binding-rejected/)
  assert.throws(() => observeTestedPermissions(account, { ...plan, account: { ...account, subject: 'other' } }, receipt, planner), /binding-rejected/)
  assert.throws(() => observeTestedPermissions(account, plan, { ...receipt, data: {} }, planner), /binding-rejected/)
  assert.throws(() => observeTestedPermissions(account, plan, receipt, planner, receipt.observedAt - 1), /stale/)
  assert.throws(() => observeTestedPermissions(account, plan, receipt, planner, receipt.observedAt + 300001), /stale/)
})
