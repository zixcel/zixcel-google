use crate::{Error, Result};
pub(super) fn check(status: u16, body: &[u8]) -> Result<()> {
    if status == 200 {
        return Ok(());
    }
    if (status == 400 || status == 403)
        && let Ok(value) = serde_json::from_slice::<serde_json::Value>(body)
    {
        for detail in value
            .pointer("/error/details")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if detail["@type"] != "type.googleapis.com/google.rpc.ErrorInfo"
                || detail["domain"] != "googleapis.com"
            {
                continue;
            }
            match detail["reason"].as_str() {
                Some("SERVICE_DISABLED") => return Err(Error::ServiceDisabled),
                Some("ACCESS_TOKEN_SCOPE_INSUFFICIENT") => {
                    return Err(Error::ScopeInsufficient);
                }
                Some("USER_PROJECT_DENIED" | "USER_PROJECT_MISSING") => {
                    return Err(Error::QuotaProjectUnavailable);
                }
                _ => {}
            }
        }
    }
    Err(match status {
        401 => Error::Unauthorized,
        403 => Error::Forbidden,
        404 => Error::Unavailable,
        429 => Error::RateLimited,
        _ => Error::HttpStatus(status),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_http_failures_retain_only_numeric_status_not_private_body() {
        for status in [400, 402, 500, 503] {
            let error = check(status, b"SYNTHETIC_PRIVATE_RESPONSE").unwrap_err();
            assert_eq!(error.http_status(), Some(status));
            assert_eq!(error.code(), "http_failed");
            assert!(!error.to_string().contains("SYNTHETIC_PRIVATE_RESPONSE"));
        }
    }
    #[test]
    fn only_known_typed_provider_reasons_change_the_static_error_code() {
        for (reason, error) in [
            ("SERVICE_DISABLED", Error::ServiceDisabled),
            ("ACCESS_TOKEN_SCOPE_INSUFFICIENT", Error::ScopeInsufficient),
            ("USER_PROJECT_DENIED", Error::QuotaProjectUnavailable),
            ("UNKNOWN", Error::Forbidden),
        ] {
            let value = serde_json::json!({"error":{"message":"SYNTHETIC_PRIVATE_BODY","details":[{"@type":"type.googleapis.com/google.rpc.ErrorInfo","domain":"googleapis.com","reason":reason}]}});
            let actual = check(403, &serde_json::to_vec(&value).unwrap()).unwrap_err();
            assert_eq!(actual, error);
            assert!(!actual.to_string().contains("SYNTHETIC_PRIVATE_BODY"));
        }
        assert_eq!(check(403, b"{invalid").unwrap_err(), Error::Forbidden);
        assert_eq!(check(404, b"{}").unwrap_err(), Error::Unavailable);
    }
}
