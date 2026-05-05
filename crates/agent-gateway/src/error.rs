use thiserror::Error;

/// Gateway denial codes — all 14 from spec 07.
/// Each maps to a specific denial reason in the gateway pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DenialCode {
    GrantMissing,
    GrantExpired,
    GrantRevoked,
    PacketMissing,
    PacketNotAssigned,
    PacketScopeDenied,
    ActionDenied,
    FileScopeDenied,
    SchemaScopeDenied,
    BudgetExceeded,
    EvidenceRequired,
    ApprovalRequired,
    SeparationOfDutiesDenied,
    SnapshotAnchorRequired,
}

impl std::fmt::Display for DenialCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DenialCode::*;
        match self {
            GrantMissing => write!(f, "GRANT_MISSING"),
            GrantExpired => write!(f, "GRANT_EXPIRED"),
            GrantRevoked => write!(f, "GRANT_REVOKED"),
            PacketMissing => write!(f, "PACKET_MISSING"),
            PacketNotAssigned => write!(f, "PACKET_NOT_ASSIGNED"),
            PacketScopeDenied => write!(f, "PACKET_SCOPE_DENIED"),
            ActionDenied => write!(f, "ACTION_DENIED"),
            FileScopeDenied => write!(f, "FILE_SCOPE_DENIED"),
            SchemaScopeDenied => write!(f, "SCHEMA_SCOPE_DENIED"),
            BudgetExceeded => write!(f, "BUDGET_EXCEEDED"),
            EvidenceRequired => write!(f, "EVIDENCE_REQUIRED"),
            ApprovalRequired => write!(f, "APPROVAL_REQUIRED"),
            SeparationOfDutiesDenied => write!(f, "SEPARATION_OF_DUTIES_DENIED"),
            SnapshotAnchorRequired => write!(f, "SNAPSHOT_ANCHOR_REQUIRED"),
        }
    }
}

/// Error returned by the agent gateway.
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("denied: {code} — {message}")]
    Denied {
        code: DenialCode,
        message: String,
    },

    #[error("internal gateway error: {0}")]
    Internal(String),

    #[error("store error: {0}")]
    Store(#[from] control_store::StoreError),
}

impl GatewayError {
    pub fn deny(code: DenialCode, message: impl Into<String>) -> Self {
        Self::Denied {
            code,
            message: message.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Map to RFC 7807 problem detail response.
    pub fn problem_detail(&self) -> serde_json::Value {
        let (status, title, detail) = match self {
            Self::Denied { code, message } => {
                let status = match code {
                    DenialCode::BudgetExceeded => 429,
                    DenialCode::EvidenceRequired
                    | DenialCode::PacketMissing
                    | DenialCode::SnapshotAnchorRequired => 400,
                    _ => 403,
                };
                (status, code.to_string(), message.clone())
            }
            Self::Internal(msg) => (500, "INTERNAL_GATEWAY_ERROR".to_string(), msg.clone()),
            Self::Store(e) => (500, "STORE_ERROR".to_string(), e.to_string()),
        };
        serde_json::json!({
            "type": "about:blank",
            "title": title,
            "status": status,
            "detail": detail,
        })
    }
}

impl GatewayError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Denied { code, .. } => match code {
                DenialCode::BudgetExceeded => 429,
                DenialCode::EvidenceRequired
                | DenialCode::PacketMissing
                | DenialCode::SnapshotAnchorRequired => 400,
                _ => 403,
            },
            Self::Internal(_) | Self::Store(_) => 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_denial_codes_display() {
        let codes = [
            DenialCode::GrantMissing,
            DenialCode::GrantExpired,
            DenialCode::GrantRevoked,
            DenialCode::PacketMissing,
            DenialCode::PacketNotAssigned,
            DenialCode::PacketScopeDenied,
            DenialCode::ActionDenied,
            DenialCode::FileScopeDenied,
            DenialCode::SchemaScopeDenied,
            DenialCode::BudgetExceeded,
            DenialCode::EvidenceRequired,
            DenialCode::ApprovalRequired,
            DenialCode::SeparationOfDutiesDenied,
            DenialCode::SnapshotAnchorRequired,
        ];
        for code in &codes {
            let s = code.to_string();
            assert!(!s.is_empty(), "denial code display should not be empty for {code:?}");
        }
    }

    #[test]
    fn denied_error_has_correct_problem_detail() {
        let err = GatewayError::deny(DenialCode::ActionDenied, "write not allowed");
        let detail = err.problem_detail();
        assert_eq!(detail["status"], 403);
        assert_eq!(detail["title"], "ACTION_DENIED");
    }

    #[test]
    fn budget_exceeded_returns_429() {
        let err = GatewayError::deny(DenialCode::BudgetExceeded, "token budget exceeded");
        assert_eq!(err.status_code(), 429);
        let detail = err.problem_detail();
        assert_eq!(detail["status"], 429);
    }

    #[test]
    fn evidence_required_returns_400() {
        let err = GatewayError::deny(DenialCode::EvidenceRequired, "evidence needed");
        assert_eq!(err.status_code(), 400);
    }
}
