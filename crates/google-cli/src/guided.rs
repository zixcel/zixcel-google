use zix_native_provider::{Error, Failure, GuidanceCatalog, Manifest, Request, Result};

pub fn metadata() -> Result<(Manifest, GuidanceCatalog)> {
    let mut manifest: Manifest = serde_json::from_value(crate::protocol::manifest())
        .map_err(|_| Error("invalid_manifest"))?;
    manifest.protocol = "zix-provider/v2".into();
    zix_native_provider::guidance::decorate(&mut manifest, include_bytes!("../api/guidance.json"))?;
    let catalog =
        zix_native_provider::guidance::catalog(&manifest, include_bytes!("../api/guidance.json"))?;
    Ok((manifest, catalog))
}
pub fn execute(request: &Request) -> std::result::Result<serde_json::Value, Failure> {
    let legacy = crate::protocol::Request {
        schema: crate::protocol::SCHEMA.into(),
        id: request.id.clone(),
        operation: request.operation.clone(),
        arguments: request.arguments.clone(),
        limits: crate::protocol::Limits {
            timeout_ms: request.limits.timeout_ms,
            maximum_response_bytes: request.limits.maximum_response_bytes,
        },
    };
    let result = crate::protocol::execute(&legacy);
    if let Some(code) = result.error {
        return Err(Failure {
            code,
            http_status: result.http_status,
        });
    }
    result
        .result
        .ok_or_else(|| Error("invalid_response").into())
}
pub fn entry() -> std::process::ExitCode {
    match metadata() {
        Ok((m, c)) => zix_native_provider::serve_guided(m, c, execute),
        Err(e) => {
            eprintln!("{{\"error\":\"{}\"}}", e.0);
            std::process::ExitCode::from(2)
        }
    }
}
