use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidInput,
    CredentialMissing,
    CredentialInvalid,
    Transport,
    Deadline,
    Unauthorized,
    Forbidden,
    Unavailable,
    AccountUnavailable,
    ProjectUnavailable,
    KeyListUnavailable,
    RateLimited,
    HttpStatus,
    ServiceDisabled,
    ScopeInsufficient,
    QuotaProjectUnavailable,
    ResponseTooLarge,
    InvalidResponse,
    ScopeMismatch,
    Protocol,
}
impl Error {
    pub(crate) fn at_project(self) -> Self {
        if self == Self::Unavailable {
            Self::ProjectUnavailable
        } else {
            self
        }
    }
    pub(crate) fn at_account(self) -> Self {
        if self == Self::Unavailable {
            Self::AccountUnavailable
        } else {
            self
        }
    }
    pub(crate) fn at_key_list(self) -> Self {
        if self == Self::Unavailable {
            Self::KeyListUnavailable
        } else {
            self
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::CredentialMissing => "credential_missing",
            Self::CredentialInvalid => "credential_invalid",
            Self::Transport => "transport_failed",
            Self::Deadline => "deadline_exceeded",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::Unavailable => "resource_unavailable",
            Self::AccountUnavailable => "account_unavailable",
            Self::ProjectUnavailable => "project_unavailable",
            Self::KeyListUnavailable => "key_list_unavailable",
            Self::RateLimited => "rate_limited",
            Self::HttpStatus => "http_failed",
            Self::ServiceDisabled => "service_disabled",
            Self::ScopeInsufficient => "oauth_scope_insufficient",
            Self::QuotaProjectUnavailable => "quota_project_unavailable",
            Self::ResponseTooLarge => "response_too_large",
            Self::InvalidResponse => "invalid_response",
            Self::ScopeMismatch => "scope_mismatch",
            Self::Protocol => "protocol_rejected",
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
