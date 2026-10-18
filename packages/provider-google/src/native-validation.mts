import type { BoundRequest, LocalAccount } from '@crowsi/provider-http-transport/local-control'
export interface Operation {
  readonly id: string; readonly effect: 'read' | 'write'; readonly required_permissions: readonly string[]
  readonly request_charge: 'not_assessed'; readonly creates_billable_resources: false
}
export interface Catalog { readonly operations: readonly Operation[]; readonly allowed_origins: readonly string[] }
function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('provider/native/response-invalid')
  return value as Record<string, unknown>
}
function texts(value: unknown, maximum: number): readonly string[] {
  if (!Array.isArray(value) || value.length > maximum
    || value.some(item => typeof item !== 'string' || item.length > 256)) throw new Error('provider/native/response-invalid')
  return Object.freeze([...value] as string[])
}
export function catalog(value: unknown): Catalog {
  const record = object(value)
  if (record.schema !== 'zix-google-control/v1' || record.plan_only !== true || record.approval_required !== true
    || !Array.isArray(record.operations) || record.operations.length < 1 || record.operations.length > 64) {
    throw new Error('provider/native/catalog-invalid')
  }
  const ids = new Set<string>()
  const operations = record.operations.map((value: unknown): Operation => {
    const op = object(value)
    if (typeof op.id !== 'string' || !/^[a-z][a-z.-]{1,63}$/.test(op.id) || ids.has(op.id)
      || (op.effect !== 'read' && op.effect !== 'write') || op.request_charge !== 'not_assessed'
      || op.creates_billable_resources !== false) throw new Error('provider/native/catalog-invalid')
    ids.add(op.id)
    return Object.freeze({ id: op.id, effect: op.effect, request_charge: op.request_charge,
      creates_billable_resources: false, required_permissions: texts(op.required_permissions, 32) })
  })
  const origins = texts(record.allowed_origins, 10)
  if (!origins.length || origins.some(origin => !/^https:\/\/[a-z]+\.googleapis\.com$/.test(origin))) {
    throw new Error('provider/native/catalog-invalid')
  }
  return Object.freeze({ operations: Object.freeze(operations), allowed_origins: origins })
}
export function prepared(value: unknown, operation: string, account: LocalAccount): BoundRequest {
  const result = object(value), bound = object(result.account), request = object(result.request)
  if (result.schema !== 'zix-google-control/v1' || result.operation !== operation
    || bound.provider !== account.provider || bound.subject !== account.subject || bound.account !== account.account
    || !['GET', 'POST', 'PATCH'].includes(String(request.method)) || typeof request.url !== 'string'
    || (request.body !== undefined && typeof request.body !== 'string')) throw new Error('provider/native/binding-invalid')
  return Object.freeze({ method: request.method as BoundRequest['method'], url: request.url,
    ...(request.body === undefined ? {} : { body: request.body as string }) })
}
