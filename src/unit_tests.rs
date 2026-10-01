use crate::boundary::{identifier, reference, secret_ref};

#[test]
fn boundary_accepts_only_opaque_bounded_identity_values() {
    assert!(identifier("id", "publishing").is_ok());
    assert!(identifier("id", "Publishing").is_err());
    assert!(reference("ref", "workspace:publishing").is_ok());
    assert!(reference("ref", "workspace publishing").is_err());
    assert!(secret_ref("secret://google/workspace/observer").is_ok());
    assert!(secret_ref("raw-token").is_err());
}
