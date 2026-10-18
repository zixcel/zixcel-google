# Google administration architecture

## Physical ownership

```text
zixcel-google/
  crates/google-core/       # Google-specific models, observations and control plans
  crates/google-cli/        # bounded native process entrypoints and packaging
  packages/provider-google/ # TypeScript to Rust/Crowsi adapter
  playground/              # synthetic operations and bounded local report consumers
  docs/                    # scope, permissions, costs and release contracts
```

The root Cargo manifest is a virtual workspace. The old configuration-only
planner and its unused Google Workspace service declarations were removed after
checking active source consumers. Historical review snapshots are not consumers.
No compatibility planner or alternate Google HTTP catalog remains in TypeScript.

## Execution boundaries

`google-core` owns selected read observations and six reviewed control request
builders. Its control plans contain method, fixed Google origin, path and body;
they never obtain credentials, call HTTP, grant permissions or authorize execution.

`google-cli` owns bounded stdin/stdout, process errors and environment token intake
for native read observations. `--manifest` remains the 21-command read-only zix
extension. `--control-manifest` and `--prepare-control-v1` perform local planning
only, without credentials or cloud calls. A Rust core consumer can inject a
`GoogleRead` transport; model fields remain private with read-only accessors.

The TypeScript adapter invokes an explicitly selected native ELF with a supplied
SHA256. It opens a non-symlink regular file, rejects group/world-writable files,
checks owner, size and hash, and invokes the open descriptor with an empty
environment. It never invokes a PATH command, shell, gcloud or automatic installer.
The owner must protect the installed executable and parent directories from
untrusted writers; SHA256 is an integrity binding, not proof of release provenance.

Crowsi remains the owner of generic credentials, approval, bounded HTTP execution,
account/request digest binding and one-use grants. Google-specific request
construction is in Rust; generic execution is not reimplemented. The trusted owner
supplies the planner and credential source, never LLM-generated code. Permission
observations are information only and confer no remote or local authority.

## Operations and costs

The core catalog separates read/write effect, required permissions, resource
creation and request pricing. Pricing is currently `not_assessed`; this is not a
free-use guarantee. None of the six control operations creates paid resources.
Permission tests use POST but remain reads. Label changes require a numeric project
resource and the observed project etag. Google rejects stale etags; a PATCH reply
can be an asynchronous Operation and is not evidence of completed mutation.
No automatic retries or asynchronous polling are added. If a write outcome is
unknown, inspect state with a new approved read before considering a new write.

Project billing linkage, resource inventory, usage-cost records and finalized
invoices are different evidence. Existing export readers retain decimal strings,
source hashes and completeness. FOCUS and common invoice/monitor normalization
are not implemented here. No export creation, SQL job, IAM grant or API activation
is performed by these local plans or source tests.

## Versions and release paths

This source changes the native package identity to `zixcel-google-cli` (0.4.0),
with `zixcel-google-core` at the same workspace version. The native release tag is
`google/v<google-cli version>`. Previous `google-admin/*` releases remain immutable.
The executable and read protocol remain `zix-google` and `zix-provider/v1`.

The TypeScript candidate is 0.2.0: callers explicitly create/pass a native planner
and a CredentialSource. `cliCredentials`, static operation builders and arbitrary
Google extension catalogs are removed. Native package hashes and npm versions
are independently released; no package is claimed published by a manifest change.
The crate packages remain `publish=false`; source builds use public pinned managed
dependencies, without a sibling checkout or private registry.

## Acceptance

Rust tests cover typed scope, invalid inputs, selected projections, pagination,
source binding and HTTP limits. CLI tests exercise actual bounded processes.
Node tests call the real native planner with isolated credentials/fake HTTP,
including write/read-back, replay and permission failure. The independent npm
consumer extracts a packed candidate and uses a copied native executable.
Owner CI runs each entrypoint; release CI builds the Linux musl binary and retains
checksums, source identity, dependency notices and build provenance. Live account
operations, financial reconciliation and publication require separate evidence.

Official label API: [Project etag](https://docs.cloud.google.com/resource-manager/reference/rest/v3/projects)
and [projects.patch](https://docs.cloud.google.com/resource-manager/reference/rest/v3/projects/patch).
