//! Pure permission and retry/concurrency decisions. No HTTP, SQL, UI or clock.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Owner,
    Editor,
    Reader,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
}

pub fn permits(access: Access, action: Action) -> bool {
    match (access, action) {
        (Access::Owner | Access::Editor, _) | (Access::Reader, Action::Read) => true,
        (Access::Reader, Action::Write) | (Access::None, _) => false,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SaveDecision {
    Append { revision: u64 },
    Replay { revision: u64 },
    Stale { current_revision: u64 },
    OperationReused,
    RevisionLimit,
}

pub struct PriorOperation<'a> {
    pub revision: u64,
    pub expected_revision: u64,
    pub export: &'a [u8],
}

/// Retry identity includes the base AND the entire normalized export, including evidence.
/// A performed-content digest alone cannot identify an accepted operation.
pub fn decide_save(
    head: u64,
    expected: u64,
    export: &[u8],
    prior: Option<PriorOperation<'_>>,
) -> SaveDecision {
    if let Some(prior) = prior {
        return if prior.expected_revision == expected && prior.export == export {
            SaveDecision::Replay {
                revision: prior.revision,
            }
        } else {
            SaveDecision::OperationReused
        };
    }
    if head != expected {
        return SaveDecision::Stale {
            current_revision: head,
        };
    }
    if head >= i64::MAX as u64 {
        return SaveDecision::RevisionLimit;
    }
    SaveDecision::Append { revision: head + 1 }
}

#[cfg(test)]
mod tests {
    use super::{decide_save, permits, Access, Action, PriorOperation, SaveDecision};

    #[test]
    fn permission_table_is_closed_and_fail_closed() {
        for (access, read, write) in [
            (Access::Owner, true, true),
            (Access::Editor, true, true),
            (Access::Reader, true, false),
            (Access::None, false, false),
        ] {
            assert_eq!(permits(access, Action::Read), read);
            assert_eq!(permits(access, Action::Write), write);
        }
    }

    #[test]
    fn replay_precedes_staleness_and_binds_base_and_complete_export() {
        let prior = || {
            Some(PriorOperation {
                revision: 1,
                expected_revision: 0,
                export: b"rights-a",
            })
        };
        assert_eq!(
            decide_save(3, 0, b"rights-a", prior()),
            SaveDecision::Replay { revision: 1 }
        );
        assert_eq!(
            decide_save(3, 1, b"rights-a", prior()),
            SaveDecision::OperationReused
        );
        assert_eq!(
            decide_save(3, 0, b"rights-b", prior()),
            SaveDecision::OperationReused
        );
        assert_eq!(
            decide_save(3, 2, b"x", None),
            SaveDecision::Stale {
                current_revision: 3
            }
        );
        assert_eq!(
            decide_save(3, 3, b"x", None),
            SaveDecision::Append { revision: 4 }
        );
        assert_eq!(
            decide_save(i64::MAX as u64, i64::MAX as u64, b"x", None),
            SaveDecision::RevisionLimit
        );
    }
}
