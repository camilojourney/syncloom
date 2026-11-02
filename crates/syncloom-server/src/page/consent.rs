use super::*;
use std::{collections::HashMap, sync::Mutex};

/// Browser-only form state. Restart requires fresh acceptance; core dispatch still checks cost.
#[derive(Default)]
pub(crate) struct CostConsents(Mutex<HashMap<String, CostConsent>>);

#[derive(PartialEq, Eq)]
struct CostConsent {
    revision: String,
    estimate: syncloom_core::social::CostEstimate,
}

impl CostConsent {
    fn current(piece: &Piece) -> Self {
        Self {
            revision: piece.revision.clone(),
            estimate: syncloom_core::social::estimate(&piece.text, piece.visual.is_some()),
        }
    }
}

impl CostConsents {
    #[expect(
        clippy::expect_used,
        reason = "Poisoned cost-consent state cannot safely authorize spend; preserve fail-closed behavior."
    )]
    pub(super) fn accepted(&self, piece: &Piece) -> Option<u64> {
        let current = CostConsent::current(piece);
        let mut records = self.0.lock().expect("cost consent lock");
        if piece.channel == syncloom_core::Channel::X
            && records.get(&piece.piece_id) == Some(&current)
        {
            Some(current.estimate.microusd)
        } else {
            // Never reuse a stale acceptance after content or pricing changes.
            records.remove(&piece.piece_id);
            None
        }
    }

    #[expect(
        clippy::expect_used,
        reason = "Poisoned cost-consent state cannot safely authorize spend; preserve fail-closed behavior."
    )]
    pub(super) fn accept(&self, piece: &Piece, form: &PieceForm) -> Result<(), StudioError> {
        if piece.channel != syncloom_core::Channel::X {
            return Err(StudioError::InvalidRequest);
        }
        if piece.revision != form.expected_revision {
            return Err(StudioError::RevisionConflict);
        }
        let current = CostConsent::current(piece);
        if form.accepted_cost_microusd != Some(current.estimate.microusd)
            || form.pricing_as_of.as_deref() != Some(current.estimate.pricing_as_of.as_str())
        {
            return Err(StudioError::CostConfirmationRequired);
        }
        self.0
            .lock()
            .expect("cost consent lock")
            .insert(piece.piece_id.clone(), current);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece() -> Piece {
        serde_json::from_value(serde_json::json!({
            "piece_id":"thought-test-x", "group_id":"test", "channel":"x",
            "platform":"x", "text":"A draft.", "revision":"revision-1",
            "status":"approved", "review_decision_id":"review-1", "publish_status":null,
            "fit":{"verdict":"PASS", "char_count":8,"min_chars":1,"max_chars":280},
            "source":{"type":"text","raw_input":"A thought."},
            "created_at":"2026-10-31T00:00:00Z", "updated_at":"2026-10-31T00:00:00Z"
        }))
        .unwrap()
    }

    // [utest->req~studio-api.browser-x-cost-consent~1]
    #[test]
    fn changed_estimate_or_date_invalidates_consent_even_for_the_same_revision() {
        let piece = piece();
        for change in ["amount", "date"] {
            let records = CostConsents::default();
            let mut old = CostConsent::current(&piece);
            if change == "amount" {
                old.estimate.microusd = 14999;
            } else {
                old.estimate.pricing_as_of = "2026-10-30".into();
            }
            records
                .0
                .lock()
                .unwrap()
                .insert(piece.piece_id.clone(), old);
            assert_eq!(records.accepted(&piece), None, "{change}");
            assert!(
                records.0.lock().unwrap().is_empty(),
                "stale record must be cleared"
            );
        }
    }

    // [utest->req~studio-api.browser-x-cost-consent~1]
    #[test]
    fn consent_is_piece_and_session_scoped() {
        let piece = piece();
        let records = CostConsents::default();
        records
            .0
            .lock()
            .unwrap()
            .insert(piece.piece_id.clone(), CostConsent::current(&piece));
        assert_eq!(records.accepted(&piece), Some(15000));
        let mut other = piece.clone();
        other.piece_id = "thought-other-x".into();
        assert_eq!(records.accepted(&other), None);
        assert_eq!(
            CostConsents::default().accepted(&piece),
            None,
            "new session requires consent"
        );
    }
}
