# zixcel-google

Owner-approved Google Cloud project/service read/write operations and scoped permission observations; separate Rust planning API.

## Rust planning capabilities

- Validate the declared integration settings.
- Inspect a bounded plan and missing prerequisites.

## Rust planning scope

The current package validates configuration and plans operations. It does not provide a complete live service client.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Rust planning verification

Install Rust 1.97 or newer and make the declared dependencies available. Run from this repository:

```sh
cargo test --locked
```

## Documentation and source

[Usage guide](docs/getting-started.md)

[Examples](examples) · [Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)

## Local administrator runtime

See [the separate Node provider package](packages/provider-google/README.md)
for account-bound read/write operations and isolated fake-HTTP tests.
The existing Rust planning API remains independently scoped.

[Isolated public API playground and owner operation boundaries](playground/provider-operations/README.md) · [日本語](README.ja.md)

[OSS responsibility and source completion](docs/oss-source-contract.md) defines the supported package boundary and required acceptance. It does not declare unexecuted source, distribution or production checks complete.

## Node administrator baseline

The runtime implementation is in `packages/provider-google`. Its README defines the reviewed operation catalog, account/credential scope, permission and billing coverage, source additions, examples and isolated test commands. Rust planning-only packages and presentation packages retain their own scopes; a planning fixture is not proof of an external API effect.

Run the Node runtime package's frozen install and tests from that directory. Source additions are reviewed separately from the currently published immutable version and from live-account acceptance. No provider connection or account change occurs during source tests.

Dependency updates: [repository policy](docs/dependency-updates.md).
Registry releases: [version tags and GitHub OIDC](docs/registry-release.md).

## Native administrator extension

[Google IAM observation](docs/google-admin.md) is available as a separate Rust module and `zix google iam keys inspect` extension. The planning crate and npm library keep their distinct APIs. Native release packages require no local Google SDK; access tokens are supplied externally.

## Administrator inventory

Use the native module through `zix google` to investigate visible projects, billing associations, enabled APIs and selected resource states. [Administration guide](docs/administration.md) explains individual JSON commands, required permissions, incomplete visibility and the difference between inventory and actual charges. No cloud mutations or paid query jobs are performed.

Existing billing-export discovery and bounded row reads, plus workload configuration, are documented in [administrator observations](docs/administration.md#existing-billing-exports-and-workload-configuration). No SQL jobs or export setup are performed.
