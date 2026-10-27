//! Use the canonical native-package notice collector, including Git package ownership.
use std::path::Path;
pub fn collect(output: &Path) -> Result<(), &'static str> {
    zix_cli::release_bundle::licenses::collect(output)
}
