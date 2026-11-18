//! Checked atomic ledger updates shared by import and Native resource owners.
//!
//! Rust 1.93.1 cannot use the renamed standard-library `try_update` API. Keep the
//! same retry, previous-value and ordering contract using stable atomic operations.

#[cfg(any(target_has_atomic = "ptr", target_has_atomic = "64"))]
use core::sync::atomic::Ordering;

#[cfg(any(target_has_atomic = "ptr", target_has_atomic = "64"))]
macro_rules! checked_atomic_update {
    ($name:ident, $atomic:ty, $value:ty) => {
        /// Apply a checked update, returning the previous value on success or rejection.
        ///
        /// The callback can run again after contention or a spurious CAS failure.
        /// `None` rejects without writing. The orderings are the successful CAS
        /// ordering and the load/failed CAS ordering, respectively.
        #[inline]
        pub fn $name(
            counter: &$atomic,
            set_order: Ordering,
            fetch_order: Ordering,
            mut update: impl FnMut($value) -> Option<$value>,
        ) -> Result<$value, $value> {
            let mut observed = counter.load(fetch_order);
            loop {
                let next = update(observed).ok_or(observed)?;
                match counter.compare_exchange_weak(observed, next, set_order, fetch_order) {
                    Ok(previous) => return Ok(previous),
                    Err(current) => observed = current,
                }
            }
        }
    };
}

#[cfg(target_has_atomic = "ptr")]
checked_atomic_update!(try_update_usize, core::sync::atomic::AtomicUsize, usize);

#[cfg(target_has_atomic = "64")]
checked_atomic_update!(try_update_u64, core::sync::atomic::AtomicU64, u64);
