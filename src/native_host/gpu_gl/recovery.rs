//! Process admission counts live and permanently quarantined GL contexts.

use std::sync::Mutex;

use serde::Serialize;

use super::{NativeGlError, NativeGlFailureKind, NativeGlResult};

pub const MAX_GL_CONTEXT_CREDITS: usize = 4;
static COUNTS: Mutex<(usize, usize)> = Mutex::new((0, 0));

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeGlContextCredits {
    pub limit: usize,
    pub in_use: usize,
    pub orphaned: usize,
}

pub fn context_credits() -> NativeGlContextCredits {
    let counts = COUNTS.lock().unwrap_or_else(|error| error.into_inner());
    NativeGlContextCredits {
        limit: MAX_GL_CONTEXT_CREDITS,
        in_use: counts.0,
        orphaned: counts.1,
    }
}

pub(super) struct ContextCredit {
    touched: bool,
    orphaned: bool,
}

impl ContextCredit {
    pub fn reserve() -> NativeGlResult<Self> {
        let mut counts = COUNTS.lock().unwrap_or_else(|error| error.into_inner());
        if counts.0 >= MAX_GL_CONTEXT_CREDITS {
            return Err(NativeGlError::new(
                NativeGlFailureKind::RecoveryBudgetExhausted,
                "GL live/orphan context credit limit reached; no unproven credit can be forgiven",
            ));
        }
        counts.0 += 1;
        Ok(Self {
            touched: false,
            orphaned: false,
        })
    }

    pub fn touch(&mut self) {
        self.touched = true;
    }

    pub fn proven_release(mut self) {
        self.touched = false;
    }

    pub fn orphan(&mut self) {
        if self.touched && !self.orphaned {
            COUNTS.lock().unwrap_or_else(|error| error.into_inner()).1 += 1;
            self.orphaned = true;
        }
    }
}

impl Drop for ContextCredit {
    fn drop(&mut self) {
        self.orphan();
        if !self.orphaned {
            COUNTS.lock().unwrap_or_else(|error| error.into_inner()).0 -= 1;
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeGlRecoveryInfo {
    pub attempts: u64,
    pub last_attempted_context_id: u64,
    pub last_attempted_device_generation: u64,
    pub failure: Option<super::NativeGlFailureInfo>,
}
