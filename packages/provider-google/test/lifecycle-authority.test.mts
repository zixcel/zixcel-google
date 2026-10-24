import assert from 'node:assert/strict'
import test from 'node:test'
import { createLocalProviderControl } from '@crowsi/provider-http-transport/local-control'
import { createLifecyclePlanner } from '../dist/index.mjs'
import { selection } from './native-support.mts'
const planner = createLifecyclePlanner(selection)
const request = { schema: 'zix-google-lifecycle/v1' as const, operation: 'service-account.disable',
 account: { provider: 'google-cloud', subject: 'synthetic-owner', account: 'sample-project' },
 target: { kind: 'service_account' as const, unique_id: '123456789012345678901' } }
test('real generic Crowsi gate consumes once and retains indeterminate write without retry', async () => {
 const native = planner.prepare(request)
 let calls = 0
 const control = createLocalProviderControl({ account: request.account,
  credentials: { acquire: async () => 'synthetic-credential-not-real' },
  allowedOrigins: ['https://iam.googleapis.com'],
  operations: [{id: request.operation, effect: 'write', requiredPermissions: ['iam.serviceAccounts.disable'], build: () => native.request}],
  fetchImplementation: async () => { calls++; throw Error('synthetic disconnected response') },
 })
 const prepared = control.prepare(request.operation, {})
 const grant = control.approve(prepared, Date.now() + 30000)
 await assert.rejects(control.execute(grant.id, prepared), (error: unknown) =>
  error instanceof Error && 'outcome' in error && error.outcome === 'indeterminate-write')
 await assert.rejects(control.execute(grant.id, prepared), /grant\/unknown/)
 assert.equal(calls, 1)
 const before = {status:200,body:{uniqueId:request.target.unique_id,projectId:'sample-project',disabled:false}}
 assert.equal(planner.reconcile({ request, plan_sha256: native.digest, before,
  mutation:{status:null,body:null},after:before }), 'unknown')
})
