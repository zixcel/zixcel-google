import { createLocalProviderControl } from '@crowsi/provider-http-transport/local-control'
import type { CredentialSource, LocalAccount, LocalOperation } from '@crowsi/provider-http-transport/local-control'
import type { GooglePlanner } from './native-planner.mjs'
export { createNativePlanner } from './native-planner.mjs'
export type { GooglePlanner, NativeExecutable } from './native-planner.mjs'

/** The trusted owner selects the verified native planner and supplies credentials separately. */
export function createControl(account: LocalAccount, credentials: CredentialSource,
  planner: GooglePlanner, fetchImplementation?: typeof fetch) {
  if (account.provider !== 'google-cloud') throw new Error('provider/account/mismatch')
  const operations: readonly LocalOperation[] = planner.catalog.operations.map(operation => ({
    id: operation.id, effect: operation.effect, requiredPermissions: operation.required_permissions,
    build(input, boundAccount) { return planner.prepare(operation.id, boundAccount, input) },
  }))
  return createLocalProviderControl({ account, credentials, operations,
    allowedOrigins: [...planner.catalog.allowed_origins], fetchImplementation })
}

export function describeService(planner: GooglePlanner) {
  return Object.freeze({ provider: 'google-cloud', permissionState: 'not-observed' as const,
    credentialSource: 'Explicit trusted-owner CredentialSource; no implicit SDK or account selection.',
    billingCoverage: 'Billing linkage is not spend or an invoice.', operations: planner.catalog.operations })
}
export { observeTestedPermissions } from './permission-observation.mjs'
export { createLifecyclePlanner } from './lifecycle.mjs'
export type { LifecycleTarget, LifecycleRequest, LifecycleEvidence, LifecycleReconciliation, LifecycleState, LifecyclePlan } from './lifecycle.mjs'
