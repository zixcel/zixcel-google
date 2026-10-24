# zixcel-google

Google Cloud administration for local tools and applications: inspect resources,
understand billing evidence, and prepare explicit, scoped operations.
Google-specific behavior has one Rust source of truth. [日本語](README.ja.md)

## Choose an entry point

| Component | Purpose |
| --- | --- |
| [google-core](crates/google-core) | Rust observations, validation and HTTP operation plans |
| [google-cli](crates/google-cli) | `zix-google`, the native extension for [zix](https://github.com/zixcel/cli) |
| [provider-google](packages/provider-google) | Thin TypeScript adapter: Rust plans + Crowsi approval/HTTP execution |

Use [the getting-started guide](docs/getting-started.md) for native investigation,
[the TypeScript guide](packages/provider-google/README.md) for approved operations,
and [the playground](playground/provider-operations/README.md) for a synthetic
write/read-back example. Runtime users need a verified native binary, not Rust or
a Google SDK. The TypeScript adapter currently supports Linux, including WSL.

## Supported behavior

The native extension provides 21 bounded read commands for IAM, projects, billing
associations, enabled APIs, selected resource inventory and existing standard or
detailed billing-export rows. See [administration](docs/administration.md).
The Rust control catalog also defines project reads/search, enabled-service reads,
permission tests, billing linkage and etag-bound project label updates. Crowsi
executes approved TypeScript operations; a Rust plan alone does not execute them.

No credentials are embedded or automatically selected. Read-only does not mean
free; resource existence does not prove a charge. Missing permissions, disabled
APIs, incomplete pages and unassessed pricing remain explicit limitations.
FOCUS, invoice reconciliation, OAuth login and arbitrary API execution are not
implemented by this source change. No Cloud setup or paid query is performed.

## Develop and verify

Use the declared Rust toolchain and Node version in owner CI:

```sh
cargo test --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
npm --prefix packages/provider-google ci --ignore-scripts
npm --prefix packages/provider-google test
```

The Node tests build and invoke the actual Rust CLI, with synthetic credentials
and fake HTTP. [Architecture](docs/architecture.md) describes trust boundaries,
removed scaffolding, version changes and what remains outside source acceptance.

[Contribution](CONTRIBUTING.md) · [Security](SECURITY.md) · [Apache-2.0](LICENSE) ·
[Notices](NOTICE) · [Source contract](docs/oss-source-contract.md) ·
[Dependency updates](docs/dependency-updates.md) · [Releases](docs/registry-release.md)

The Rust source API can pass existing export bytes through provider-specific normalization into the shared bounded usage monitor; see [usage cost observations](docs/usage-cost-observations.md). Usage evidence remains separate from issued invoices and payment evidence.

[Reviewed API lifecycle plans](docs/api-lifecycle.md) reuse the shared Zix importer for IAM and VM request planning and explicit completion evidence. These credential-free protocols do not activate Cloud operations.
