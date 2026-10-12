import { billingOperations } from './billing.mjs'
import { createLocalProviderControl, parameters, segment, labels } from '@crowsi/provider-http-transport/local-control'
import type { CredentialSource, LocalAccount, LocalOperation } from '@crowsi/provider-http-transport/local-control'
const origin = 'https://cloudresourcemanager.googleapis.com'
const resource = (a: LocalAccount) => `projects/${segment(a.account)}`
export const operations: readonly LocalOperation[] = Object.freeze([
  ...billingOperations,
  { id: 'account.read', effect: 'read', requiredPermissions: ['resourcemanager.projects.get'],
    build(input, a) { parameters(input, []); return { method: 'GET', url: `${origin}/v3/${resource(a)}` } } },
  { id: 'permissions.test', effect: 'read', requiredPermissions: [],
    build(input, a) { const p = parameters(input, ['permissions']);
      if (!Array.isArray(p.permissions) || p.permissions.length < 1 || p.permissions.length > 32
        || p.permissions.some(x => typeof x !== 'string' || !/^[a-zA-Z][a-zA-Z0-9.]{1,127}$/.test(x))) throw new Error('provider/input/permissions')
      return { method: 'POST', url: `${origin}/v3/${resource(a)}:testIamPermissions`, body: JSON.stringify({ permissions: p.permissions }) } } },
  { id: 'project.set-labels', effect: 'write', requiredPermissions: ['resourcemanager.projects.update'],
    build(input, a) { const p = parameters(input, ['labels']); return { method: 'PATCH',
      url: `${origin}/v3/${resource(a)}?updateMask=labels`, body: JSON.stringify({ name: resource(a), labels: labels(p.labels) }) } } },
])
export function createControl(account: LocalAccount, credentials: CredentialSource, fetchImplementation?: typeof fetch, extensions: readonly LocalOperation[] = []) {
  if (account.provider !== 'google-cloud') throw new Error('provider/account/mismatch')
  segment(account.account)
  return createLocalProviderControl({ account, credentials, operations: [...operations, ...extensions], allowedOrigins: [origin, 'https://cloudbilling.googleapis.com'], fetchImplementation })
}

export { cliCredentials } from './credentials.mjs'
