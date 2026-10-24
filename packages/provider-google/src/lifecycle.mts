import { invokeNative } from './native-executable.mjs'
import type { NativeExecutable } from './native-executable.mjs'
import type { LocalAccount } from '@crowsi/provider-http-transport/local-control'
export type LifecycleTarget =
  | { readonly kind: 'service_account'; readonly unique_id: string }
  | { readonly kind: 'key'; readonly unique_id: string; readonly key_id: string }
  | { readonly kind: 'instance'; readonly zone: string; readonly name: string; readonly expected_id: string }
export interface LifecycleRequest {
  readonly schema: 'zix-google-lifecycle/v1'
  readonly operation: string
  readonly account: LocalAccount
  readonly target: LifecycleTarget
  readonly request_id?: string | null
  readonly operation_name?: string | null
}
export interface LifecycleEvidence { readonly status: number | null; readonly body: unknown }
export interface LifecycleReconciliation {
  readonly plan_sha256: string
  readonly request: LifecycleRequest
  readonly before: LifecycleEvidence
  readonly mutation: LifecycleEvidence
  readonly operation?: LifecycleEvidence | null
  readonly after: LifecycleEvidence
}
export type LifecycleState = 'complete' | 'pending' | 'unknown' | 'rejected' | 'operation_failed' | 'target_changed'
export interface LifecyclePlan {
  readonly digest: string
  readonly schema: 'zix-google-lifecycle/v1'
  readonly operation: string
  readonly account: LocalAccount
  readonly request: { readonly method: 'GET' | 'POST' | 'DELETE'; readonly url: string; readonly body?: string }
  readonly target_resource: string
  readonly expected_id: string
  readonly atomic_identity_precondition: boolean
  readonly contract: Readonly<Record<string, unknown>>
}
/** Pure Google policy adapter. No credentials, approval, execution or automatic polling.
 * Deliberately separate from GooglePlanner: generic control cannot silently discard
 * lifecycle target/context evidence while approving only the HTTP request.
 */
export function createLifecyclePlanner(selection: NativeExecutable) {
  const binary = Object.freeze({ ...selection })
  return Object.freeze({
    prepare(input: LifecycleRequest): LifecyclePlan {
      return invokeNative(binary, '--prepare-lifecycle-v1', input) as LifecyclePlan
    },
    reconcile(input: LifecycleReconciliation): LifecycleState {
      const state = invokeNative(binary, '--reconcile-lifecycle-v1', input)
      if (!['complete', 'pending', 'unknown', 'rejected', 'operation_failed', 'target_changed'].includes(String(state))) {
        throw new Error('provider/native/lifecycle-response-invalid')
      }
      return state as LifecycleState
    },
  })
}
