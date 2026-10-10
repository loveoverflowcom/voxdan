//! Pure Studio save state; a transport failure preserves both text and retry identity.
use cantos_api::{ApiError, ErrorCode, RevisionResponse, SaveRevisionRequest};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Dirty,
    Loading,
    Saving,
    Loaded,
    Saved,
    DraftPreserved,
    SignedIn,
    Unauthenticated,
    NotFound,
    Forbidden,
    InvalidRequest,
    InvalidScript,
    EvidenceUnavailable,
    StaleRevision,
    OperationReused,
    ProposalAlreadySubmitted,
    Unavailable,
    CorruptRevision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Intent {
    pub actor: String,
    pub script: String,
    pub request: SaveRevisionRequest,
}

#[derive(Clone, Debug)]
pub struct Editor {
    pub actor: String,
    pub script: String,
    pub draft: String,
    pub base: u64,
    pub pending: Option<Intent>,
    pub ambiguous: bool,
    pub busy: bool,
    pub status: Status,
    pub remote: Option<RevisionResponse>,
    pub ticket: u64,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            actor: String::new(),
            script: String::new(),
            draft: String::new(),
            base: 0,
            pending: None,
            ambiguous: false,
            busy: false,
            status: Status::Idle,
            remote: None,
            ticket: 0,
        }
    }
}

impl Editor {
    pub fn select_script(&self, script: String) -> Self {
        if self.busy || self.pending.is_some() || self.script == script {
            return self.clone();
        }
        let mut next = self.clone();
        next.script = script;
        next.base = 0;
        next.remote = None;
        next.status = if next.draft.is_empty() {
            Status::Idle
        } else {
            Status::Dirty
        };
        next
    }

    pub fn edit(&self, text: String) -> Self {
        let mut next = self.clone();
        next.draft = text;
        if !self.busy
            && self.pending.is_none()
            && matches!(
                self.status,
                Status::Idle
                    | Status::SignedIn
                    | Status::Loaded
                    | Status::Saved
                    | Status::DraftPreserved
                    | Status::Dirty
            )
        {
            next.status = Status::Dirty;
        }
        next
    }
    /// A new intent cannot abandon an ambiguous save. Reconcile the pending snapshot first.
    pub fn start_save(&self, operation: String) -> Option<(Self, Intent)> {
        if self.busy || self.pending.is_some() || self.actor.is_empty() {
            return None;
        }
        let intent = Intent {
            actor: self.actor.clone(),
            script: self.script.clone(),
            request: SaveRevisionRequest {
                expected_revision: self.base,
                operation_id: operation,
                script_json: self.draft.clone(),
            },
        };
        let mut next = self.clone();
        next.busy = true;
        next.status = Status::Saving;
        next.pending = Some(intent.clone());
        next.ambiguous = false;
        Some((next, intent))
    }

    pub fn retry(&self) -> Option<(Self, Intent)> {
        if self.busy {
            return None;
        }
        let intent = self.pending.clone()?;
        if intent.actor != self.actor {
            return None;
        }
        let mut next = self.clone();
        next.busy = true;
        next.status = Status::Saving;
        Some((next, intent))
    }

    pub fn saved(&self, intent: &Intent, response: RevisionResponse) -> Self {
        if self.pending.as_ref() != Some(intent) || self.script != response.script_id {
            return self.clone();
        }
        let mut next = self.clone();
        if response.accepted_by != intent.actor {
            // A different tab may have replaced the session cookie during this request.
            next.busy = false;
            next.ambiguous = true;
            next.actor.clear();
            next.status = Status::Unauthenticated;
            return next;
        }
        next.base = response.revision;
        next.remote = Some(response);
        next.pending = None;
        next.ambiguous = false;
        next.busy = false;
        // Never rewrite the textarea on acknowledgement (including while IME composing).
        next.status = if self.draft == intent.request.script_json {
            Status::Saved
        } else {
            Status::DraftPreserved
        };
        next
    }

    pub fn failed(&self, error: &ApiError) -> Self {
        let mut next = self.clone();
        next.busy = false;
        next.status = error_status(&error.code);
        // A later authorization denial says nothing about an earlier ambiguous commit.
        if matches!(
            error.code,
            ErrorCode::Unavailable | ErrorCode::CorruptRevision
        ) {
            next.ambiguous = next.pending.is_some();
        } else if self.ambiguous
            && matches!(
                error.code,
                ErrorCode::Unauthenticated | ErrorCode::Forbidden | ErrorCode::NotFound
            )
        {
            // Reauthenticate as the submitting actor, then reconcile the original operation.
        } else {
            next.pending = None;
            next.ambiguous = false;
        }
        next
    }

    pub fn session_failed(&self, error: &ApiError) -> Self {
        let mut next = self.clone();
        next.busy = false;
        next.status = error_status(&error.code);
        next
    }

    pub fn start_read(&self) -> Option<Self> {
        if self.busy || self.pending.is_some() {
            return None;
        }
        let mut next = self.clone();
        next.busy = true;
        next.status = Status::Loading;
        next.ticket = self.ticket.checked_add(1)?;
        Some(next)
    }

    pub fn loaded(&self, ticket: u64, response: RevisionResponse) -> Self {
        if ticket != self.ticket || response.script_id != self.script {
            return self.clone();
        }
        let mut next = self.clone();
        next.busy = false;
        if next.draft.is_empty() {
            next.draft = response.script_json.clone();
            next.base = response.revision;
            next.status = Status::Loaded;
        } else {
            next.status = Status::DraftPreserved;
        }
        next.remote = Some(response);
        next
    }

    /// Explicit user reconciliation: keep local text and adopt the displayed remote head.
    pub fn rebase(&self) -> Self {
        let mut next = self.clone();
        if !self.busy && self.pending.is_none() {
            if let Some(remote) = &self.remote {
                next.base = remote.revision;
                next.status = Status::DraftPreserved;
            }
        }
        next
    }
}

pub fn error_status(code: &ErrorCode) -> Status {
    match code {
        ErrorCode::Unauthenticated => Status::Unauthenticated,
        ErrorCode::NotFound => Status::NotFound,
        ErrorCode::Forbidden => Status::Forbidden,
        ErrorCode::InvalidRequest => Status::InvalidRequest,
        ErrorCode::InvalidScript => Status::InvalidScript,
        ErrorCode::EvidenceUnavailable => Status::EvidenceUnavailable,
        ErrorCode::StaleRevision => Status::StaleRevision,
        ErrorCode::OperationReused => Status::OperationReused,
        ErrorCode::ProposalAlreadySubmitted => Status::ProposalAlreadySubmitted,
        ErrorCode::Unavailable => Status::Unavailable,
        ErrorCode::CorruptRevision => Status::CorruptRevision,
    }
}

#[cfg(test)]
mod tests {
    use super::{Editor, Status};
    use cantos_api::{ApiError, ErrorCode, RevisionResponse};

    #[test]
    fn typing_after_an_acknowledged_save_is_visibly_unsaved() {
        let initial = Editor {
            actor: "alice".into(),
            script: "script".into(),
            draft: "original".into(),
            ..Editor::default()
        };
        let (saving, intent) = initial.start_save("op".into()).unwrap();
        let saved = saving.saved(&intent, response(1));
        assert_eq!(saved.status, Status::Saved);
        assert_eq!(saved.edit("new text".into()).status, Status::Dirty);
        assert_eq!(saved.edit("new text".into()).base, 1);
    }

    #[test]
    fn changing_the_save_target_clears_the_saved_claim_and_preserves_the_draft() {
        let initial = Editor {
            actor: "alice".into(),
            script: "script".into(),
            draft: "original".into(),
            ..Editor::default()
        };
        let (saving, intent) = initial.start_save("op".into()).unwrap();
        let saved = saving.saved(&intent, response(1));
        let selected = saved.select_script("other".into());
        assert_eq!(selected.script, "other");
        assert_eq!(selected.draft, "original");
        assert_eq!(selected.base, 0);
        assert!(selected.remote.is_none());
        assert_eq!(selected.status, Status::Dirty);
        assert_eq!(saved.select_script("script".into()).base, 1);
        assert_eq!(saving.select_script("other".into()).script, "script");
    }

    #[test]
    fn expired_session_after_ambiguous_save_keeps_the_original_retry_identity() {
        let initial = Editor {
            actor: "alice".into(),
            script: "script".into(),
            draft: "text".into(),
            ..Editor::default()
        };
        let (saving, intent) = initial.start_save("op".into()).unwrap();
        let unavailable = saving.failed(&ApiError {
            code: ErrorCode::Unavailable,
            current_revision: None,
            issues: vec![],
        });
        let expired = unavailable.failed(&ApiError {
            code: ErrorCode::Unauthenticated,
            current_revision: None,
            issues: vec![],
        });
        assert_eq!(expired.pending, Some(intent));
        assert_eq!(expired.draft, "text");
        let login_denied = expired.session_failed(&ApiError {
            code: ErrorCode::Unauthenticated,
            current_revision: None,
            issues: vec![],
        });
        assert_eq!(login_denied.pending, expired.pending);
        let mut different_actor = login_denied.clone();
        different_actor.actor = "bob".into();
        assert!(different_actor.retry().is_none());
        different_actor.actor = "alice".into();
        assert_eq!(different_actor.retry().unwrap().1, expired.pending.unwrap());
    }

    #[test]
    fn acknowledgement_from_a_changed_cookie_does_not_clear_the_original_intent() {
        let initial = Editor {
            actor: "alice".into(),
            script: "script".into(),
            draft: "text".into(),
            ..Editor::default()
        };
        let (saving, intent) = initial.start_save("op".into()).unwrap();
        let mut unexpected = response(1);
        unexpected.accepted_by = "bob".into();
        let result = saving.saved(&intent, unexpected);
        assert_eq!(result.pending, Some(intent));
        assert!(result.ambiguous);
        assert!(!result.busy);
        assert_eq!(result.base, 0);
        assert_eq!(result.draft, "text");
        assert!(result.retry().is_none());
    }

    fn response(revision: u64) -> RevisionResponse {
        RevisionResponse {
            script_id: "script".into(),
            revision,
            accepted_by: "alice".into(),
            accepted_at: "2026-10-10T00:00:00.000000Z".into(),
            content_digest: "content".into(),
            export_digest: "export".into(),
            script_json: "canonical".into(),
        }
    }
    #[test]
    fn timeout_retry_reuses_snapshot_and_edit_during_save_survives_ack() {
        let initial = Editor {
            actor: "alice".into(),
            script: "script".into(),
            draft: "Ngày mai, mình có diễn tiếp không?".into(),
            ..Editor::default()
        };
        let (saving, intent) = initial.start_save("operation".into()).unwrap();
        assert!(saving.start_save("duplicate".into()).is_none());
        let mut failed = saving.failed(&ApiError {
            code: ErrorCode::Unavailable,
            current_revision: None,
            issues: vec![],
        });
        failed.draft.push_str(" Nhé?");
        assert!(failed.start_save("new".into()).is_none());
        let (retrying, retried) = failed.retry().unwrap();
        assert_eq!(retried, intent);
        let saved = retrying.saved(&retried, response(1));
        assert_eq!(saved.draft, failed.draft);
        assert_eq!(saved.base, 1);
        assert_eq!(saved.status, Status::DraftPreserved);
        assert!(saved.pending.is_none());
    }
    #[test]
    fn conflict_and_reload_preserve_text_until_explicit_rebase() {
        let initial = Editor {
            actor: "alice".into(),
            script: "script".into(),
            draft: "my edit".into(),
            base: 1,
            ..Editor::default()
        };
        let (saving, _) = initial.start_save("op".into()).unwrap();
        let conflict = saving.failed(&ApiError {
            code: ErrorCode::StaleRevision,
            current_revision: Some(2),
            issues: vec![],
        });
        assert_eq!(conflict.draft, "my edit");
        assert_eq!(conflict.base, 1);
        let reading = conflict.start_read().unwrap();
        let loaded = reading.loaded(reading.ticket, response(2));
        assert_eq!(loaded.draft, "my edit");
        assert_eq!(loaded.base, 1);
        assert_eq!(loaded.rebase().base, 2);
    }
    #[test]
    fn late_completed_read_cannot_replace_newer_selection_or_ticket() {
        let initial = Editor {
            actor: "alice".into(),
            script: "script".into(),
            ..Editor::default()
        };
        let mut reading = initial.start_read().unwrap();
        let old = reading.ticket;
        reading.ticket += 1;
        let late = reading.loaded(old, response(1));
        assert!(late.remote.is_none());
        reading.script = "other".into();
        assert!(reading.loaded(reading.ticket, response(1)).remote.is_none());
    }
}
