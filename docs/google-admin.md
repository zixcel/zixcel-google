# Google administrator module

`crates/google-admin` is a Rust library and the `zix-google` native extension. It implements one bounded, read-only IAM operation: `zix google iam keys inspect`. The existing planning crate and separate npm provider-control library retain their own APIs; this module does not claim those APIs implement IAM key inspection.

The Google adapter fetches the selected service-account metadata, binds its project/email/unique identity, then lists only user-managed keys for that account. Returned resource names are bound to the observed identity. Explicit project IDs avoid the misleading missing-resource errors possible with a wildcard project. Keys are never used to authenticate, created, disabled or deleted.

The transport is HTTPS-only to `iam.googleapis.com`, without inherited proxies, redirects or automatic retries. It allows two scoped requests, a total deadline, and bounded response bodies. Provider error bodies and raw diagnostics are never printed. Responses contain only selected key metadata and a non-authoritative observation. IAM access errors never become a successful missing-key result.

From 0.1.1, HTTP 404 is reported as `account_unavailable` during account lookup and `key_list_unavailable` during key listing. These identify the failing phase without asserting that a key or account was deleted. Both remain errors; only a successfully bound account and key list can yield `not_present`. No raw provider response or additional credential is exposed.

Use the independent `zixcel/cli` package to register the verified manifest and executable; the root `zixcel-google` planning binary is not the administrator entry point. [Install and invocation](https://github.com/zixcel/cli/blob/main/docs/install.md).

The extension obtains an externally supplied `ZIX_GOOGLE_ACCESS_TOKEN`; no local Google SDK, old service-account private key, authentication daemon, or embedded OAuth client is used. Token minting/login is outside this initial module. OAuth scopes: IAM or cloud-platform. Required permissions: `iam.serviceAccounts.get` and `iam.serviceAccountKeys.list`. The manifest declaration is not proof of authorization.

Initial scope accepts standard `name@project.iam.gserviceaccount.com` service-account emails and project IDs. Compute-default account addresses and other resource aliases are not accepted as input in this version. Response aliases are accepted only when bound to the observed identity.

Release tags are `google-admin/v<crate version>`. The independent native release workflow builds a Linux musl archive with the manifest, exact source/hash, checksums and original dependency notices. It does not publish this `publish=false` crate to crates.io or change the npm package.

Bounded response intake reuses `zixcel-bounded-io` from the fixed public Git revision recorded in Cargo.toml/Cargo.lock. The managed component is not yet published to crates.io. This is a build dependency, not a sibling-checkout requirement or runtime source download.

Development: `cargo test --locked --workspace --all-targets --all-features`; `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`. Tests use synthetic identities, credential-free mock transports, loopback HTTP and actual CLI processes. Live Google IAM acceptance remains separate and requires an owner-supplied credential.

Official references: [account get](https://docs.cloud.google.com/iam/docs/reference/rest/v1/projects.serviceAccounts/get), [key list](https://docs.cloud.google.com/iam/docs/reference/rest/v1/projects.serviceAccounts.keys/list).
