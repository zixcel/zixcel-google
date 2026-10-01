# zixcel-google

An independent connector validating Google Workspace, Analytics and AdSense configuration and generating deterministic plans. The CLI has no HTTP client or credential store and makes no external API calls.

```bash
cargo run --offline -- doctor
cargo run --offline -- capabilities
cargo run --offline -- validate examples/config.toml
cargo run --offline -- plan examples/config.toml
```

Only `secret://...` credential references are accepted. Token or password fields fail validation. `plan` describes future observations; communication, authorization and execution belong to separate boundaries.

Library use combines `parse_config` and `build_plan`. Input is closed TOML up to 1 MiB; output is a deterministic proposal without communication. The crate uses `publish = false` during local validation.

## Quality gate

These checks run within this crate without connecting to Google APIs.

```bash
# WONDERLAND_ROOT is the workspace checkout root.
"$WONDERLAND_ROOT/bin/verify-repositories" --rust --tier standard
```

## Package integration

The package is an independently consumable unit. Callers reference its documented
interface through a versioned dependency and own application-specific composition
and integration.

## License

Apache-2.0. Copyright 2026 HAT Inc. See [LICENSE](LICENSE) and [NOTICE](NOTICE). External dependencies retain their respective licenses.
