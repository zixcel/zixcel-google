import { parameters, segment, text } from '@crowsi/provider-http-transport/local-control'
import type { LocalOperation } from '@crowsi/provider-http-transport/local-control'
export const discoveryOperations: readonly LocalOperation[] = [
  { id: 'accounts.search', effect: 'read', requiredPermissions: ['resourcemanager.projects.get'],
    build(input) { const p = parameters(input, ['pageToken']); const query = new URLSearchParams({ pageSize: '50' })
      if (p.pageToken !== null) query.set('pageToken', text(p.pageToken, 2048))
      return { method: 'GET', url: `https://cloudresourcemanager.googleapis.com/v3/projects:search?${query}` } } },
  { id: 'services.list', effect: 'read', requiredPermissions: ['serviceusage.services.list'],
    build(input, a) { const p = parameters(input, ['pageToken'])
      if (!/^[0-9]{1,20}$/.test(a.account)) throw new Error('provider/input/project-number-required')
      const query = new URLSearchParams({ pageSize: '50', filter: 'state:ENABLED' })
      if (p.pageToken !== null) query.set('pageToken', text(p.pageToken, 2048))
      return { method: 'GET', url: `https://serviceusage.googleapis.com/v1/projects/${segment(a.account)}/services?${query}` } } },
]
