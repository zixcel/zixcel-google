import assert from 'node:assert/strict'
import test from 'node:test'
import { createLifecyclePlanner, createControl, createNativePlanner } from '../dist/index.mjs'
import { selection } from './native-support.mts'
const planner = createLifecyclePlanner(selection)
const request = { schema: 'zix-google-lifecycle/v1' as const, operation: 'service-account.disable',
  account: { provider: 'google-cloud', subject: 'synthetic-owner', account: 'sample-project' },
  target: { kind: 'service_account' as const, unique_id: '123456789012345678901' } }
const before = { status: 200, body: { uniqueId: request.target.unique_id, projectId: 'sample-project', disabled: false } }
test('native lifecycle policy is a separate pure adapter; HTTP acknowledgement is not completion', () => {
  const plan = planner.prepare(request)
  assert.equal(plan.request.url, 'https://iam.googleapis.com/v1/projects/sample-project/serviceAccounts/123456789012345678901:disable')
  assert.equal(plan.atomic_identity_precondition, true)
  assert.equal(planner.reconcile({ request, plan_sha256: plan.digest, before, mutation: { status: 200, body: {} }, after: before }), 'pending')
  assert.equal(planner.reconcile({ request, plan_sha256: plan.digest, before, mutation: { status: 200, body: {} },
    after: { status: 200, body: { ...before.body, disabled: true } } }), 'complete')
  assert.equal(planner.reconcile({ request, plan_sha256: plan.digest, before, mutation: { status: null, body: null }, after: before }), 'unknown')
  assert.equal(planner.reconcile({ request, plan_sha256: plan.digest, before, mutation: { status: 403, body: null }, after: before }), 'rejected')
  assert.throws(() => planner.prepare({ ...request, account: { ...request.account, account: '../foreign' } }), /request-rejected/)
})
test('generic six-operation control does not silently acquire lifecycle write authority', () => {
  const control = createControl(request.account, { acquire: async () => { throw Error('must not acquire') } }, createNativePlanner(selection))
  assert.throws(() => control.prepare(request.operation, request.target), /unsupported/)
})
