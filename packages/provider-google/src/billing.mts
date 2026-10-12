import { parameters, segment } from '@crowsi/provider-http-transport/local-control'
import type { LocalOperation } from '@crowsi/provider-http-transport/local-control'
export const billingOperations: readonly LocalOperation[] = [{
  id: 'billing.project-read', effect: 'read', requiredPermissions: ['resourcemanager.projects.get'],
  build(input, account) { parameters(input, []); return { method: 'GET',
    url: `https://cloudbilling.googleapis.com/v1/projects/${segment(account.account)}/billingInfo` } },
}]
