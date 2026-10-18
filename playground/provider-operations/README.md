# Synthetic Google operation playground

This demonstrates a real Rust operation planner composed with Crowsi grants and
fake HTTP, without contacting Google or changing a real account.

```sh
npm --prefix packages/provider-google ci --ignore-scripts
npm --prefix packages/provider-google test
(cd packages/provider-google && node ../../.github/scripts/provider-consumer.mjs)
```

The independent consumer packs and installs the candidate, supplies its copied
native executable, checks TypeScript types and runs the write/read-back demo.
The demo changes synthetic labels with an explicit etag, reads fake state back,
and verifies that reusing the write grant fails without another credential use.
Its computed binary hash binds the local synthetic build only; production callers
must use a release hash verified through their trusted distribution process.

See [the package guide](../../packages/provider-google/README.md) for real owner
composition. No approval is inferred from this example. Account selection,
credential acquisition, real remote permissions and operation approval remain with
the owner. No secret value or real project identifier belongs in the playground.
