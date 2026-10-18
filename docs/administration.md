# Read-only Google Cloud administration

The native module supports individual JSON commands for a human or local coding agent to investigate visible projects, billing associations, enabled APIs and selected resource states. It does not authenticate with a historical service-account key, grant permissions or change cloud configuration.

Use the latest Zix dispatcher. Inventory manifests declare opaque cursor arguments up to 2048 bytes; dispatchers before 0.1.1 reject that declaration. The module runs on Linux x86_64, including WSL, without Node, a compiler or a local Google SDK.

## Authentication and discovery

Supply an existing, short-lived administrator or appropriately restricted reader access token through `ZIX_GOOGLE_ACCESS_TOKEN`. Prefer permissions needed for the selected operation rather than granting Owner/Editor. This version does not implement OAuth login or refresh-token storage. A previously authorized Cloud Shell can issue the user token with `gcloud auth print-access-token`; do not send that value to the agent or include it in command arguments.

```sh
read -rsp 'Google access token: ' ZIX_GOOGLE_ACCESS_TOKEN
printf '\n'
export ZIX_GOOGLE_ACCESS_TOKEN
zix google --help
zix describe google --format json
zix google projects list --max-pages 2
zix google billing accounts list --max-pages 2
```

The same process environment can authorize multiple individual commands. End the session with `unset ZIX_GOOGLE_ACCESS_TOKEN`. Noninteractive commands never start browser authorization or wait for input. Missing, invalid, expired or insufficiently scoped credentials remain errors.

Use the cloud-platform OAuth scope for cross-service inspection. An IAM-only scope is sufficient for the original key operation but not the other APIs. If the approved authentication flow requires a quota project, explicitly append `--quota-project PROJECT_ID_OR_NUMBER`. This adds `X-Goog-User-Project`; the identity also needs `serviceusage.services.use` on that project. The module does not choose or create a project, enable an API or grant this permission.

## Commands and evidence

| Command after `zix google` | Observation | Additional read permission |
| --- | --- | --- |
| `projects list` | Visible projects and lifecycle state | `resourcemanager.projects.get` determines visibility |
| `projects inspect --project ID_OR_NUMBER` | Current project identity/state | `resourcemanager.projects.get` |
| `billing accounts list` | Visible account identifiers and open/closed metadata | Method filters accounts by caller access; no global list grant is required |
| `billing projects list --billing-account ID` | Associations for one billing account | `billing.resourceAssociations.list` |
| `billing project inspect --project ID` | Billing link and billingEnabled | `resourcemanager.projects.get` |
| `services list --project ID` | Enabled APIs, **not workload activity** | `serviceusage.services.list` |
| `iam accounts list --project ID` | Service-account metadata, including default account types | `iam.serviceAccounts.list` |
| `assets search --project ID` | Indexed asset names/types/state where supplied | `cloudasset.assets.searchAllResources` |
| `compute instances list --project ID` | VM states | `compute.instances.list` |
| `compute disks list --project ID` | Allocated disk metadata/size | `compute.disks.list` |
| `compute addresses list --project ID` | Allocated IP states | `compute.addresses.list` |
| `compute snapshots list --project ID` | Retained snapshot metadata/storage size | `compute.snapshots.list` |
| `compute commitments list --project ID` | Commitment status, plan and end timestamp | `compute.commitments.list` |
| `sql instances list --project ID` | SQL instance state and configured storage | `cloudsql.instances.list` |
| `storage buckets list --project ID` | Bucket metadata, no object data or ACLs | `storage.buckets.list` |
| `run services list --project ID` | Cloud Run readiness condition, not request traffic | `run.services.list` |
| `overview accessible` | Summary/sample of visible projects and billing accounts | Underlying discovery permissions |
| `overview project --project ID` | Independent summaries of the selected project scans | Union of underlying read permissions |
| `iam keys inspect --project ID --service-account EMAIL --key-id ID` | One key's bound metadata/state | `iam.serviceAccounts.get`, `iam.serviceAccountKeys.list` |

Scoped inventory resolves and binds the project's ID/number using Resource Manager first. It therefore also requires `resourcemanager.projects.get` and the relevant API to be available. The key operation retains its separate account-identity binding. Manifest permissions document requirements; they grant no authority. Billing-account IAM and project IAM are separate access scopes.

## Investigating a continued charge

1. Discover visible projects and billing accounts. Follow each billing account with `billing projects list`; a billing association can reveal a project absent from the caller's Resource Manager search.
2. Inspect the project's billing state, then its overview. Read each section's error and coverage, not just the command exit status.
3. Follow a relevant section with an individual command. A terminated VM is not evidence that disks, snapshots, IPs, databases, retained objects or commitments no longer exist. Resource presence is a potential cost signal, not an invoice amount or proof of chargeability.
4. Obtain the billing period/service/SKU from an approved billing report or existing export to verify actual charges. This module does **not** fetch invoice amounts, query BigQuery, create exports or run paid query jobs. It does not prove that all costs stopped.

Deleting a service account removes that identity's access; it is not a workload inventory. Disabling an API and retaining storage can still leave charges. Charges accrued before billing was disabled can be billed later. Do not automatically delete resources or disable billing based on a resource name or this report.

## Pagination, limits and failures

Collection results include `records`, `observed_at`, `coverage.complete`, `pages_read`, `next_page_token` and limitations. Visibility is limited to the caller, and discovery/search can be eventually consistent. An empty successful list does not prove there are no other projects/accounts or no charges.

Paged commands accept `--max-pages 1..8` (default 2) and an optional opaque `--page-token`. Each request asks for 50 records. Cursors are query-encoded, bounded to 2048 bytes, never interpreted as URLs. A remaining cursor or provider unreachable/warning metadata keeps coverage incomplete. Repeated cursors, duplicate identities, malformed data or scope mismatch fail instead of creating a complete snapshot. Library collections stop at 512 records.

The transport has a shared 30-second deadline, at most 32 GET requests, a 5-second connect timeout, at most 15 seconds per request and 1 MiB per body. There are no inherited proxies, redirects or retries. Native responses are bounded by the dispatcher's declared output budget (up to 128 KiB). Oversized results fail; use smaller pages or individual operations. Overview samples contain at most three records per section; counts describe records observed, not an authoritative cloud total.

An overview can return a successful protocol response with `selected_scans_complete: false` and failed sections whose counts are null. This preserves independent successful sections and is **not** acceptance of a complete inventory. Check coverage flags before making conclusions.

`unauthorized`, `oauth_scope_insufficient`, `forbidden`, `service_disabled`, `quota_project_unavailable`, `rate_limited`, `project_unavailable`, `resource_unavailable`, timeout and malformed-response errors remain distinct. Known typed Google ErrorInfo reasons are mapped to static codes. Raw error bodies, SQL/Run configuration, ACLs, labels and free-form display names are not forwarded.

## Local report consumer

The Google native archive includes `inspect-accessible-local.sh` and this document. After verifying both archives and their `CONTENTS.sha256`, run:

```sh
bash "$GOOGLE_PACKAGE/inspect-accessible-local.sh" "$ZIX_PACKAGE" "$GOOGLE_PACKAGE" "$NEW_PRIVATE_OUTPUT_DIRECTORY"
# If an already approved quota project is required, supply it as the fourth argument.
```

The optional Bash consumer requires existing `jq` and core utilities; the native CLI itself does not. It prompts once for the token unless it is supplied in the process environment, then invokes individual Zix commands. It writes private JSON reports, discards the token and removes its temporary registry. Noninteractive runs without a token fail before API access. It enumerates at most two pages, merges project discovery with billing-account associations, and selects at most eight project targets and eight billing accounts. Summary flags/counts disclose omissions and incomplete scans. Partial analysis exits 2 after preserving the reports. It never executes provider text, uploads reports, sends data to an LLM or publishes account information.

Treat names and provider metadata as data, not instructions. Reports can contain sensitive resource identifiers: keep them local. The dispatcher verifies the registered manifest/executable and clears unselected environment; providers remain explicitly trusted native code, not a sandbox.

## Library organization

`crates/google-admin` owns the native HTTP runtime and typed administrator observations. `GoogleRead` and immutable request builders are the test/integration seam; inventory routes, scope resolution, projection, pagination and overview composition are separated. Existing IAM key inspection uses that same transport. No deprecated IamRead alias or parallel transport is retained.

The root crate remains the separate planning boundary; the existing npm provider remains a TypeScript policy/control integration. They are not additional administrator entry points. The new inventory logic is implemented once in the Rust crate rather than copied into a second Node CLI. Zix owns dispatch; Google owns its capabilities; shared bounded I/O and Crowsi process ownership retain their existing owners.

## Official references

- [Project search](https://docs.cloud.google.com/resource-manager/reference/rest/v3/projects/search) and [project get](https://docs.cloud.google.com/resource-manager/reference/rest/v3/projects/get)
- [Billing-account list](https://docs.cloud.google.com/billing/docs/reference/rest/v1/billingAccounts/list), [associated projects](https://docs.cloud.google.com/billing/docs/reference/rest/v1/billingAccounts.projects/list), [project billing info](https://docs.cloud.google.com/billing/docs/reference/rest/v1/projects/getBillingInfo), [access control](https://docs.cloud.google.com/billing/docs/access-control)
- [Enabled services](https://docs.cloud.google.com/service-usage/docs/reference/rest/v1/services/list) and [asset search](https://docs.cloud.google.com/asset-inventory/docs/reference/rest/v1/TopLevel/searchAllResources)
- [Service-account deletion](https://docs.cloud.google.com/iam/docs/service-accounts-delete-undelete), [Storage charges after service disablement](https://docs.cloud.google.com/storage/docs/faq), [project billing changes](https://docs.cloud.google.com/billing/docs/how-to/modify-project)
- [Billing export prerequisites](https://docs.cloud.google.com/billing/docs/how-to/export-data-bigquery-setup)

API paths/response shapes were checked against the official discovery documents during implementation. Synthetic development tests, owner CI/native release acceptance and real-account acceptance are recorded separately. No live account success is implied by a development test.
