# Reviewed Google API lifecycle contracts

The shared [Zix importer](https://github.com/zixcel/cli/blob/main/docs/api-specifications.md)
imports Discovery, validates exact hashes, closes local references and reports
contract drift. Google does not own another importer, schema engine or SDK generator.
`api/sources.lock.json` pins official IAM/Compute v1 revision 20261016 and complete
selected schema closures. The two catalogs contain ten selected methods; all Zix
execution bindings remain null. `api/operations.json` separately reviews IAM
permission declarations, effects, completion rules and unassessed request pricing.
These declarations are not effective grants or free-use guarantees.

## Reproduce and update

Download the exact official source URLs in the lock separately to a private cache,
using a bounded HTTPS client. A changed upstream document is a candidate, not an
accepted update. With an explicitly verified installed Zix binary:

```sh
node tools/api-import/review.mts /absolute/zix /absolute/spec-cache
node tools/api-import/check-lock.mts
```

The first command verifies importer/source/catalog hashes and invokes the existing
importer; it never downloads or modifies files. If upstream bytes changed, create
separate candidate reviews/catalogs using `zix api import`, compare with `zix api
diff` and review the complete schemas, parameters, scopes, origin/path and response
semantics before changing the lock/policy/source/tests. No cron/update workflow,
runtime discovery, credential selection or automatic grant expansion is added.

## Plans and completion

`zix-google --lifecycle-manifest`, `--prepare-lifecycle-v1` and
`--reconcile-lifecycle-v1` are bounded, credential-free local protocols. Rust
`lifecycle::prepare` and `lifecycle::reconcile` own the same implementation.
The npm `createLifecyclePlanner` is a thin native adapter. See the exported
`LifecycleRequest`/`LifecycleReconciliation` types. Plans bind exact imported
catalog/source and owner-policy hashes; old plans do not silently acquire new
schema semantics. The six-operation existing `createControl` interface is unchanged.

Lifecycle targets use a scoped numeric service-account unique ID, user-managed
key ID or project/zone/VM name with an independently observed decimal-string ID.
Project wildcards, supplied URIs, zero request UUIDs and unsupported operations
fail closed. IAM unique-ID addressing prevents a recreated same-email account
from being selected as its predecessor. Compute names lack that atomic guarantee:
`atomic_identity_precondition=false` explicitly warns of the check/write race.
Before destructive use, the trusted host must freshly inspect identity and exclude
concurrent replacement; a request UUID only deduplicates a request, not a resource
version. This library does not assert that the remote API supports a missing
conditional precondition.

A trusted executor retains the original exact plan and time-bounded request/response
bindings, obtains one-use approval from Crowsi and explicitly supplies evidence.
The pure reconciler does not authenticate caller-supplied JSON or issue authority.
It distinguishes pending, complete, rejected, operation_failed, target_changed and
unknown. A grant authorizes a bounded dispatch; it does not prove remote completion.
Each follow-up read needs its own explicit executor admission; no automatic poll or
mutation retry is added. Importing an operation cannot execute it through `zix api`.
The lifecycle planner is deliberately not structurally compatible with the generic
six-operation control planner: discarding the target/evidence context would lose
part of the reviewed decision. No new live-execution interface is implicitly enabled.

IAM disable/delete acknowledges an Empty object, then requires exact identity and
disabled=true or an explicitly observed post-delete 404. A 403/timeout is unknown,
never absence. Keys must be USER_MANAGED. Compute stop/delete requires scoped
Operation targetLink/targetId/zone/selfLink/type/clientOperationId, DONE without
error, then TERMINATED for the same ID or post-delete 404. A later operation read
must name the original operation. Unknown enums and numeric IDs rounded into JSON
numbers are refused. Output is a state, never a copied raw error body or key data.
No cloud resource, credential, IAM role, billing linkage or API enablement is changed
by these protocols/tests.

## Reuse and footprint decision

The official generated [Compute Rust client](https://docs.rs/google-cloud-compute-v1/latest/google_cloud_compute_v1/)
was evaluated first: per-client features are available, and its Instances client
supports endpoint/credential overrides but defaults to Application Default
Credentials. It is an async network client, whereas this owner needs a deterministic
credential-free request plan that leaves execution with Crowsi. Retaining the
existing bounded executor and imported data avoids a second HTTP/auth stack.
This is a boundary decision, not a claim the SDK cannot be configured safely.

No Rust dependency is added and existing Rust dependency versions are retained.
The TypeScript adapter adopts the already published Crowsi HTTP transport 0.12.1
for explicit indeterminate-write errors; it does not duplicate that generic logic. The
hosted footprint job compares this candidate with accepted baseline 7ceb3d1 using
two independent cold target directories, records elapsed build time, binary size,
locked package count and maintained source bytes. Timings are runner-specific;
added lifecycle behavior is expected to increase code/data. No size or speed
reduction is claimed. Source/conformance acceptance and formal distribution are
verified separately; no held local heavy build is resumed.

Official behavior: [IAM disable](https://docs.cloud.google.com/iam/docs/reference/rest/v1/projects.serviceAccounts/disable),
[key delete](https://docs.cloud.google.com/iam/docs/reference/rest/v1/projects.serviceAccounts.keys/delete),
[VM stop](https://docs.cloud.google.com/compute/docs/reference/rest/v1/instances/stop),
[Compute Operation](https://docs.cloud.google.com/compute/docs/reference/rest/v1/zoneOperations).

The selected Discovery descriptions include Google documentation and schema text.
See `api/NOTICE` for upstream attribution and terms; first-party implementation
remains Apache-2.0. Schemas are selected/reformatted without relicensing upstream
prose or claiming trademark rights.
