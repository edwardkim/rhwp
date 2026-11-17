//! Process-wide admission for live and permanently quarantined Metal owners.
//! This policy does not establish GPU completion; only its caller can confirm a
//! successful presenter release. Construction reservations can roll back until
//! a successfully created presenter is exposed to owner operations.

use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) const MAX_PROCESS_PRESENTERS: usize = 4;
const QUARANTINE_UNIT: usize = MAX_PROCESS_PRESENTERS + 1;

pub(crate) static PROCESS_ADMISSION: Admission = Admission::new();

/// Encode total reservations and quarantines in one atomic word so readers do
/// not observe a quarantine count from a different admission snapshot.
pub(crate) struct Admission {
    state: AtomicUsize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stats {
    pub in_use: usize,
    pub quarantined: usize,
    pub limit: usize,
}

impl Admission {
    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(0),
        }
    }

    pub fn stats(&self) -> Stats {
        let state = self.state.load(Ordering::Acquire);
        Stats {
            in_use: state % QUARANTINE_UNIT,
            quarantined: state / QUARANTINE_UNIT,
            limit: MAX_PROCESS_PRESENTERS,
        }
    }

    pub fn reserve(&self) -> Option<Credit<'_>> {
        rhwp_codec_resource::atomic::try_update_usize(
            &self.state,
            Ordering::AcqRel,
            Ordering::Acquire,
            |state| (state % QUARANTINE_UNIT < MAX_PROCESS_PRESENTERS).then_some(state + 1),
        )
        .ok()?;
        Some(Credit {
            admission: self,
            state: CreditState::Reserved,
        })
    }
}

#[derive(Clone, Copy)]
enum CreditState {
    Reserved,
    Live,
    Released,
    Quarantined,
}

pub(crate) struct Credit<'a> {
    admission: &'a Admission,
    state: CreditState,
}

impl Credit<'_> {
    /// Call immediately after construction succeeds, before info/publication.
    pub fn mark_live(&mut self) {
        if matches!(self.state, CreditState::Reserved) {
            self.state = CreditState::Live;
        }
    }

    /// A Native presenter release, not abandon or owner destruction, is the
    /// proof authorizing this transition. A quarantine can never be refunded.
    pub fn release_confirmed(&mut self) -> bool {
        if !matches!(self.state, CreditState::Live) {
            return false;
        }
        self.state = CreditState::Released;
        self.admission.state.fetch_sub(1, Ordering::AcqRel);
        true
    }

    pub fn quarantine(&mut self) {
        if matches!(self.state, CreditState::Live) {
            self.state = CreditState::Quarantined;
            self.admission
                .state
                .fetch_add(QUARANTINE_UNIT, Ordering::AcqRel);
        }
    }
}

impl Drop for Credit<'_> {
    fn drop(&mut self) {
        match self.state {
            CreditState::Reserved => {
                // Constructor errors have not exposed a presenter or recorded
                // a frame. Their reservation can return without a GPU fence.
                self.admission.state.fetch_sub(1, Ordering::AcqRel);
            }
            CreditState::Live => self.quarantine(),
            CreditState::Released | CreditState::Quarantined => {}
        }
    }
}
