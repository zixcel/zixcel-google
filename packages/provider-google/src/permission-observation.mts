import { createHash } from 'node:crypto'
import { segment } from '@crowsi/provider-http-transport/local-control'
import type { LocalAccount, LocalPlan } from '@crowsi/provider-http-transport/local-control'
interface Receipt {
  schema: string; operation: string; requestDigest: string; responseDigest: string;
  observedAt: number; status: number; data: unknown;
}
const hash = (value: unknown) => createHash('sha256').update(JSON.stringify(value)).digest('hex')
const sameAccount = (a: LocalAccount, b: LocalAccount) =>
  a.provider === b.provider && a.account === b.account && a.subject === b.subject

/** Information-only interpretation of the requested test, never an authorization constructor. */
export function observeTestedPermissions(account: LocalAccount, plan: LocalPlan, receipt: Receipt, now = Date.now()) {
  const { digest, ...source } = plan
  if (account.provider !== 'google-cloud' || !sameAccount(account, plan.account)
    || plan.operation !== 'permissions.test' || plan.request.method !== 'POST'
    || plan.request.url !== `https://cloudresourcemanager.googleapis.com/v3/projects/${segment(account.account)}:testIamPermissions`
    || hash(source) !== digest || receipt.requestDigest !== digest || receipt.operation !== plan.operation
    || receipt.schema !== 'crowsi://provider-operation-receipt/v1' || receipt.status !== 200
    || hash(receipt.data) !== receipt.responseDigest) throw new Error('provider/observation/binding-rejected')
  if (!Number.isSafeInteger(now) || !Number.isSafeInteger(receipt.observedAt)
    || receipt.observedAt < 0 || receipt.observedAt > Number.MAX_SAFE_INTEGER - 300000 || now < receipt.observedAt || now - receipt.observedAt > 300000) {
    throw new Error('provider/observation/stale')
  }
  const query = JSON.parse(plan.request.body ?? '{}') as { permissions?: unknown }
  const requested = query.permissions
  if (!Array.isArray(requested) || requested.length < 1 || requested.length > 32
    || requested.some(p => typeof p !== 'string' || !/^[A-Za-z][A-Za-z0-9.]{1,127}$/.test(p))
    || new Set(requested).size !== requested.length) throw new Error('provider/observation/query-rejected')
  if (!receipt.data || typeof receipt.data !== 'object' || Array.isArray(receipt.data)) {
    throw new Error('provider/observation/response-rejected')
  }
  const field = (receipt.data as { permissions?: unknown }).permissions
  const values = field === undefined ? [] : field
  if (!Array.isArray(values) || values.length > requested.length || new Set(values).size !== values.length
    || values.some(p => !requested.includes(p))) throw new Error('provider/observation/response-rejected')
  return Object.freeze({ account: Object.freeze({ ...account }), observedAt: receipt.observedAt,
    validUntil: receipt.observedAt + 300000, requestDigest: digest, responseDigest: receipt.responseDigest,
    authority: 'none' as const, coverage: 'requested-permission-test-only' as const,
    permissions: Object.freeze(requested.map(permission => Object.freeze({ permission,
      state: values.includes(permission) ? 'returned-by-test' as const : 'not-returned-or-resource-unavailable' as const }))),
  })
}
