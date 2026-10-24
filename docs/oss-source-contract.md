# OSS source contract: zixcel/zixcel-google

Google-specific resource observations and scoped operation planning live in one
Rust core. The native CLI and thin TypeScript/Crowsi integration are separate
entrypoints in this repository, not competing implementations.

## Source owners

| Manifest | Package | Responsibility |
| --- | --- | --- |
| `crates/google-core/Cargo.toml` | zixcel-google-core | Google models, observations, scope and control request validation |
| `crates/google-cli/Cargo.toml` | zixcel-google-cli | Native `zix-google` process protocols and packaging |
| `packages/provider-google/package.json` | @zixcel/provider-google | Verified Rust planner to Crowsi approval/execution |

Rust packages are source/build dependencies (`publish=false`). The native package
is distributed through owner GitHub Releases; npm is released separately with
GitHub OIDC. Manifest versions are not proof of registry availability.
The root Cargo manifest is a virtual workspace. The old unused configuration-only
planner is removed; there is no fallback path that bypasses the new core.

## Completion requirements

1. Public interfaces, docs, package manifests and implementation agree.
2. Independent consumers resolve public managed dependencies with committed locks;
   no central private repository, sibling checkout or ambient credential is needed.
3. Rust tests cover scope/HTTP boundaries and actual CLI protocols. Node tests use
   the actual native planner, synthetic credentials and fake HTTP for read/write,
   input denial, permission denial and one-use grant behavior.
4. Crowsi retains generic authorization and bounded transport; no parallel grant
   system is introduced. Credential subject selection and operational approval
   remain the trusted owner's responsibility.
5. Pricing, side effects, resource creation and observation completeness remain
   separate. Missing access and partial pages never imply zero cost or no resource.
6. Source testing, immutable archive publication and real-account acceptance are
   separate. Never complete unexecuted or skipped checks by inference.
7. Apache-2.0, owner NOTICE and third-party notices remain with each distribution.

## Verification and release entrypoints

- [Rust CI](../.github/workflows/ci.yml)
- [TypeScript/native composition](../.github/workflows/provider-control.yml)
- [Repository policy](../.github/workflows/oss-policy.yml)
- [Native release](../.github/workflows/release-google.yml)
- [npm tag release](../.github/workflows/publish-npm.yml)
- [Reviewed npm stage publication](../.github/workflows/publish-npm.yml)

[Architecture](architecture.md), [source map](oss-source-map.json),
[administration](administration.md) and [playground](../playground/provider-operations/README.md)
state the supported interfaces and limits. No OAuth/login, paid query, export
setup, full IAM inventory or finalized-invoice normalization is claimed here.
