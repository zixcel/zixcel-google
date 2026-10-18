# Getting started

## Inspect a Google account locally

Install a verified native Google package and the independent
[zix dispatcher](https://github.com/zixcel/cli/blob/main/docs/install.md).
Verify the release identity, archive checksums and bundled file hashes before
registering `google.extension.json` with its `zix-google` executable.
Use the command syntax reported by `zix extensions --help` for your dispatcher.

The native runtime needs no Rust compiler, Node or Google SDK. Supply an existing
short-lived access token only in the local environment as described in
[administration](administration.md); never paste it into an Issue or argument.

```sh
zix google --help
zix describe google --format json
zix google projects list --max-pages 2
zix google billing accounts list --max-pages 2
```

Missing credentials fail without starting a browser. Successful reads can still
be partial; inspect coverage and section failures. Enabled APIs and billing
linkage do not prove workload use or an invoice amount.

## Use approved operations from TypeScript

[provider-google](../packages/provider-google/README.md) obtains operation plans
from the verified native executable, then uses Crowsi for explicit approval and
bounded execution. Its synthetic playground proves local composition only.
The planner itself never executes a write. Real operations require a credential
with the remote permissions and the owner's separate approval.

## Verify the source

```sh
cargo test --locked --workspace --all-targets --all-features
npm --prefix packages/provider-google ci --ignore-scripts
npm --prefix packages/provider-google test
```

The Node source tests build the actual Rust binary; installed runtime users use
the native release instead. See [architecture](architecture.md) for migration
and supported limits, and [releases](registry-release.md) for publication.
