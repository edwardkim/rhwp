//! Request-local reservation hooks for the rhwp import path.
use crate::errors::DecodeErrors;
use alloc::vec::Vec;

/// Implemented by the importing request, not by a process-global allocator.
pub trait AllocationBudget {
    /// Charge the complete new reservation before attempting it.
    fn reserve(&self, bytes: usize, kind: &'static str) -> bool;
    /// Record checked-size overflow or a failed fallible reservation.
    fn reject(&self);
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Allocator<'a>(pub(crate) Option<&'a dyn AllocationBudget>);

impl Allocator<'_> {
    pub(crate) fn error(self) -> DecodeErrors {
        if let Some(budget) = self.0 {
            budget.reject();
        }
        DecodeErrors::AllocationLimit
    }
    pub(crate) fn product(self, factors: &[usize]) -> Result<usize, DecodeErrors> {
        factors
            .iter()
            .try_fold(1usize, |n, &v| n.checked_mul(v))
            .ok_or_else(|| self.error())
    }
    pub(crate) fn reserve<T>(
        self,
        out: &mut Vec<T>,
        additional: usize,
        kind: &'static str,
    ) -> Result<(), DecodeErrors> {
        let needed = out
            .len()
            .checked_add(additional)
            .ok_or_else(|| self.error())?;
        if needed <= out.capacity() {
            return Ok(());
        }
        let bytes = self.product(&[needed, core::mem::size_of::<T>()])?;
        if let Some(budget) = self.0 {
            if !budget.reserve(bytes, kind) {
                return Err(DecodeErrors::AllocationLimit);
            }
        }
        out.try_reserve_exact(additional).map_err(|_| self.error())
    }
    pub(crate) fn capacity<T>(
        self,
        count: usize,
        kind: &'static str,
    ) -> Result<Vec<T>, DecodeErrors> {
        let mut out = Vec::new();
        self.reserve(&mut out, count, kind)?;
        Ok(out)
    }
    pub(crate) fn filled<T: Clone>(
        self,
        value: T,
        count: usize,
        kind: &'static str,
    ) -> Result<Vec<T>, DecodeErrors> {
        let mut out = self.capacity(count, kind)?;
        out.resize(count, value);
        Ok(out)
    }
    pub(crate) fn copy<T: Clone>(
        self,
        data: &[T],
        kind: &'static str,
    ) -> Result<Vec<T>, DecodeErrors> {
        let mut out = self.capacity(data.len(), kind)?;
        out.extend_from_slice(data);
        Ok(out)
    }
    pub(crate) fn push<T>(
        self,
        out: &mut Vec<T>,
        value: T,
        kind: &'static str,
    ) -> Result<(), DecodeErrors> {
        self.reserve(out, 1, kind)?;
        out.push(value);
        Ok(())
    }
    pub(crate) fn resize<T: Clone>(
        self,
        out: &mut Vec<T>,
        len: usize,
        value: T,
        kind: &'static str,
    ) -> Result<(), DecodeErrors> {
        if len > out.len() {
            self.reserve(out, len - out.len(), kind)?;
        }
        out.resize(len, value);
        Ok(())
    }
}
