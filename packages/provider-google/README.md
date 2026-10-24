# @zixcel/provider-google

Thin TypeScript integration for Google plans from the Rust core and Crowsi's
one-use administrator authorization/HTTP transport. Apache-2.0.

## Runtime requirements

Linux (including WSL), Node 24 and an explicitly installed, verified `zix-google`
native package supporting `zix-google-control/v1`. Runtime users do not need a
Rust compiler or gcloud. This source is the 0.3.0 candidate; check official release
availability independently before installing it. There is no automatic download,
login, credential-store lookup, HTTP call or Cloud change on package import.

## Compose an approved operation

```ts
import { createNativePlanner, createControl } from '@zixcel/provider-google'
const planner = createNativePlanner({
  executable: installedExecutable, // absolute path in an owner-protected directory
  sha256: verifiedReleaseBinarySha256, // from the separately verified release
})
const control = createControl(account, credentialSource, planner)
const plan = control.prepare('account.read', {})
// Present the exact plan/account/permissions to the trusted administrator.
const grant = control.approve(plan, Date.now() + 30_000)
const receipt = await control.execute(grant.id, plan)
```

`account` has `provider: 'google-cloud'`, `subject` and `account` (project ID or
number). Service Usage and label updates require a project number. The owner
supplies a Crowsi `CredentialSource`; the planner never receives its token.
The owner must verify subject/account selection; a token's identity is not
inferred from the declared account. `describeService(planner)` returns immutable
operation metadata without claiming observed access. Read/write and cost are
separate: `request_charge: not_assessed` does not promise free API usage.

## Supported catalog

Rust alone defines accounts.search, services.list, account.read, permissions.test,
project.set-labels and billing.project-read. Search/list input is `{ pageToken:
null }` for the first page. Each additional page is a separately approved request.
Permission input is `{ permissions: ['resourcemanager.projects.get'] }`.
Label input is `{ labels: { env: 'test' }, etag: observedProjectEtag }`.
Labels replace the supplied label set; review removals as well as additions.

Label etags must come from the current project read. The response to a PATCH can
be a pending Google Operation; an accepted HTTP response does not prove a finished
mutation. No automatic retry/polling is added. Resolve uncertain outcomes with a
new approved read instead of reusing a grant or automatically issuing a new write.
Billing linkage is not actual spend. Native inventory/billing-export reads are
available through `zix google`; this package does not copy those implementations.

`observeTestedPermissions(account, plan, receipt, planner)` validates the native
request and the bound Crowsi receipt before returning requested-name observations.
It reports omitted names as unavailable/not returned, never a complete IAM denial.
The caller owns receipt provenance; these are not signed authorization records.

## Source verification

```sh
npm ci --ignore-scripts
# Select a separately built or downloaded native artifact and its verified checksum.
export ZIX_GOOGLE_TEST_BINARY=/absolute/path/to/zix-google
export ZIX_GOOGLE_TEST_SHA256=replace_with_verified_64_character_sha256
npm test
node ../../.github/scripts/provider-consumer.mts
```

`npm test` never builds or downloads native code. It refuses a missing selection,
checksum mismatch, symlink, non-regular file or oversized native artifact before
running synthetic accounts and fake HTTP tests. Artifact hashing uses a bounded
64 KiB buffer. A checksum identifies the selected bytes; verify their provenance
separately before selecting them.

The owner CI explicitly builds the Rust CLI first with two Cargo jobs and no
incremental/debug artifacts. For a deliberate local build, use
`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build --locked --release -p zixcel-google-cli --bin zix-google`
from the repository root, within your own resource limits, then select the binary
and its independently verified SHA256. Building is never an npm lifecycle hook. The independent consumer tests the packed npm artifact and a copied
native executable. No real credential or Cloud account is used. Release and live
acceptance are separate. The previous implicit gcloud helper, duplicate request
builders and arbitrary extension catalog are removed; callers must adopt the
explicit planner argument. See [architecture](../../docs/architecture.md).

`createLifecyclePlanner(selection)` exposes the native lifecycle planning and evidence classification API. It deliberately provides no credentials, approval or execution. See [lifecycle policy](../../docs/api-lifecycle.md) for exact target bindings and Compute identity limitations. Existing `createControl` operations are unchanged.
