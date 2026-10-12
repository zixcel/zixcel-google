import { localCommandCredential } from '@crowsi/provider-http-transport/local-control'
import type { LocalAccount } from '@crowsi/provider-http-transport/local-control'
export function cliCredentials(account: LocalAccount) {
  if (account.provider !== 'google-cloud' || !/^[A-Za-z0-9._+@-]{3,256}$/.test(account.subject)) {
    throw new Error('provider/credential/account-invalid')
  }
  return localCommandCredential('gcloud', ['auth', 'print-access-token', '--account', account.subject],
    text => text.trim(), account)
}
