import type { BoundRequest, LocalAccount } from '@crowsi/provider-http-transport/local-control'
import { invokeNative } from './native-executable.mjs'
import type { NativeExecutable } from './native-executable.mjs'
import { catalog, prepared } from './native-validation.mjs'
import type { Catalog } from './native-validation.mjs'
export type { NativeExecutable } from './native-executable.mjs'
export interface GooglePlanner {
  readonly catalog: Catalog
  prepare(operation: string, account: LocalAccount, input: unknown): BoundRequest
}
export function createNativePlanner(selection: NativeExecutable): GooglePlanner {
  const binary = Object.freeze({ ...selection })
  const capabilities = catalog(invokeNative(binary, '--control-manifest'))
  return Object.freeze({ catalog: capabilities,
    prepare(operation: string, account: LocalAccount, input: unknown) {
      const result = invokeNative(binary, '--prepare-control-v1', {
        schema: 'zix-google-control/v1', operation, account, input,
      })
      return prepared(result, operation, account)
    },
  })
}
