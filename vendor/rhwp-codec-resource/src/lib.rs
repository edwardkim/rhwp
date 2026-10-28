//! Fallible codec storage and work checks. No process-global allocator changes.
#![cfg_attr(not(feature = "scoped"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;
use alloc::vec::Vec;
use core::fmt;

/// The requesting importer owns counters, limits, cancellation, and sticky errors.
pub trait ResourceBudget: Send + Sync {
    fn allocation_limit(&self) -> usize;
    fn reserve(&self, bytes: usize) -> bool;
    fn work(&self, units: usize) -> bool;
    fn reject(&self);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceError;

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("codec resource limit or allocation failure")
    }
}

#[cfg(feature = "scoped")]
impl std::error::Error for ResourceError {}

#[cfg(feature = "scoped")]
mod scoped {
    use super::ResourceBudget;
    use alloc::rc::Rc;
    use alloc::sync::Arc;
    use core::marker::PhantomData;
    use std::cell::RefCell;

    std::thread_local! {
        pub(super) static ACTIVE: RefCell<Option<Arc<dyn ResourceBudget>>> = const { RefCell::new(None) };
    }

    /// Restores the prior synchronous scope. Not Send/Sync: restoration is local.
    pub struct Scope {
        previous: Option<Arc<dyn ResourceBudget>>,
        _thread: PhantomData<Rc<()>>,
    }

    impl Scope {
        pub fn enter(budget: Arc<dyn ResourceBudget>) -> Self {
            Self {
                previous: ACTIVE.with(|slot| slot.replace(Some(budget))),
                _thread: PhantomData,
            }
        }
    }

    impl Drop for Scope {
        fn drop(&mut self) {
            ACTIVE.with(|slot| *slot.borrow_mut() = self.previous.take());
        }
    }
}

#[cfg(feature = "scoped")]
pub use scoped::Scope;

fn with_budget<T>(f: impl FnOnce(Option<&dyn ResourceBudget>) -> T) -> T {
    #[cfg(feature = "scoped")]
    {
        scoped::ACTIVE.with(|slot| f(slot.borrow().as_deref()))
    }
    #[cfg(not(feature = "scoped"))]
    {
        f(None)
    }
}

pub fn reject() -> ResourceError {
    with_budget(|budget| {
        if let Some(budget) = budget {
            budget.reject();
        }
    });
    ResourceError
}

/// Charge before the corresponding repeated work. Zero checks cancellation too.
pub fn work(units: usize) -> Result<(), ResourceError> {
    with_budget(|budget| match budget {
        Some(budget) if !budget.work(units) => Err(ResourceError),
        _ => Ok(()),
    })
}

pub fn product(factors: &[usize]) -> Result<usize, ResourceError> {
    factors
        .iter()
        .try_fold(1usize, |n, &v| n.checked_mul(v).ok_or_else(reject))
}

pub fn reserve<T>(out: &mut Vec<T>, additional: usize) -> Result<(), ResourceError> {
    work(0)?;
    let needed = out.len().checked_add(additional).ok_or_else(reject)?;
    if needed <= out.capacity() {
        return Ok(());
    }
    let bytes = product(&[needed, core::mem::size_of::<T>()])?;
    with_budget(|budget| match budget {
        Some(budget) if !budget.reserve(bytes) => Err(ResourceError),
        _ => Ok(()),
    })?;
    out.try_reserve_exact(additional).map_err(|_| reject())
}

pub fn capacity<T>(count: usize) -> Result<Vec<T>, ResourceError> {
    let mut out = Vec::new();
    reserve(&mut out, count)?;
    Ok(out)
}

pub fn filled<T: Clone>(value: T, count: usize) -> Result<Vec<T>, ResourceError> {
    work(count)?;
    let mut out = capacity(count)?;
    out.resize(count, value);
    Ok(out)
}

pub fn copy<T: Copy>(source: &[T]) -> Result<Vec<T>, ResourceError> {
    work(source.len())?;
    let mut out = capacity(source.len())?;
    out.extend_from_slice(source);
    Ok(out)
}

pub fn push<T>(out: &mut Vec<T>, value: T) -> Result<(), ResourceError> {
    work(1)?;
    if out.len() == out.capacity() {
        let needed = out.len().checked_add(1).ok_or_else(reject)?;
        let item_size = core::mem::size_of::<T>();
        let ceiling = with_budget(|b| b.map_or(usize::MAX, ResourceBudget::allocation_limit));
        let count_limit = if item_size == 0 {
            usize::MAX
        } else {
            ceiling / item_size
        };
        let desired = out
            .capacity()
            .saturating_mul(2)
            .max(4)
            .min(count_limit)
            .max(needed);
        reserve(out, desired - out.len())?;
    }
    out.push(value);
    Ok(())
}

pub fn resize<T: Clone>(out: &mut Vec<T>, count: usize, value: T) -> Result<(), ResourceError> {
    if count > out.len() {
        work(count - out.len())?;
        reserve(out, count - out.len())?;
    } else {
        work(0)?;
    }
    out.resize(count, value);
    Ok(())
}

pub fn collect<T>(iter: impl IntoIterator<Item = T>) -> Result<Vec<T>, ResourceError> {
    let mut out = Vec::new();
    for item in iter {
        push(&mut out, item)?;
    }
    Ok(out)
}

pub fn extend<T>(out: &mut Vec<T>, iter: impl IntoIterator<Item = T>) -> Result<(), ResourceError> {
    for item in iter {
        push(out, item)?;
    }
    Ok(())
}

pub fn append<T>(out: &mut Vec<T>, other: &mut Vec<T>) -> Result<(), ResourceError> {
    work(other.len())?;
    reserve(out, other.len())?;
    out.append(other);
    Ok(())
}

/// Reserve a finite iterator's work once; unknown-length iterators check each step.
pub fn iterations<I: IntoIterator>(iter: I) -> Result<(I::IntoIter, bool), ResourceError> {
    let iter = iter.into_iter();
    let (lower, upper) = iter.size_hint();
    let prepaid = upper == Some(lower);
    work(if prepaid { lower } else { 0 })?;
    Ok((iter, prepaid))
}

/// Fallible methods preserve normal Vec reborrowing at decoder call sites.
pub trait VecResource<T> {
    fn rhwp_push(&mut self, value: T) -> Result<(), ResourceError>;
    fn rhwp_reserve(&mut self, additional: usize) -> Result<(), ResourceError>;
    fn rhwp_resize(&mut self, count: usize, value: T) -> Result<(), ResourceError>
    where
        T: Clone;
    fn rhwp_extend(&mut self, iter: impl IntoIterator<Item = T>) -> Result<(), ResourceError>;
    fn rhwp_extend_from_slice(&mut self, source: &[T]) -> Result<(), ResourceError>
    where
        T: Copy;
    fn rhwp_append(&mut self, other: &mut Vec<T>) -> Result<(), ResourceError>;
}
impl<T> VecResource<T> for Vec<T> {
    fn rhwp_push(&mut self, value: T) -> Result<(), ResourceError> {
        push(self, value)
    }
    fn rhwp_reserve(&mut self, additional: usize) -> Result<(), ResourceError> {
        reserve(self, additional)
    }
    fn rhwp_resize(&mut self, count: usize, value: T) -> Result<(), ResourceError>
    where
        T: Clone,
    {
        resize(self, count, value)
    }
    fn rhwp_extend(&mut self, iter: impl IntoIterator<Item = T>) -> Result<(), ResourceError> {
        extend(self, iter)
    }
    fn rhwp_extend_from_slice(&mut self, source: &[T]) -> Result<(), ResourceError>
    where
        T: Copy,
    {
        extend(self, source.iter().copied())
    }
    fn rhwp_append(&mut self, other: &mut Vec<T>) -> Result<(), ResourceError> {
        append(self, other)
    }
}

pub trait SliceResource<T: Copy> {
    fn rhwp_copy(&self) -> Result<Vec<T>, ResourceError>;
}
impl<T: Copy> SliceResource<T> for [T] {
    fn rhwp_copy(&self) -> Result<Vec<T>, ResourceError> {
        copy(self)
    }
}
pub trait IteratorResource: Iterator + Sized {
    fn rhwp_collect(self) -> Result<Vec<Self::Item>, ResourceError> {
        collect(self)
    }
}
impl<I: Iterator> IteratorResource for I {}

#[cfg(all(test, feature = "scoped"))]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    struct Budget {
        bytes: usize,
        units: AtomicUsize,
        failed: AtomicBool,
    }
    impl ResourceBudget for Budget {
        fn allocation_limit(&self) -> usize {
            self.bytes
        }
        fn reserve(&self, bytes: usize) -> bool {
            if bytes > self.bytes {
                self.reject();
                return false;
            }
            !self.failed.load(Ordering::Relaxed)
        }
        fn work(&self, units: usize) -> bool {
            if self.failed.load(Ordering::Relaxed) {
                return false;
            }
            if self
                .units
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                    n.checked_sub(units)
                })
                .is_err()
            {
                self.reject();
                return false;
            }
            true
        }
        fn reject(&self) {
            self.failed.store(true, Ordering::Relaxed);
        }
    }
    fn budget(bytes: usize, units: usize) -> Arc<Budget> {
        Arc::new(Budget {
            bytes,
            units: AtomicUsize::new(units),
            failed: AtomicBool::new(false),
        })
    }

    #[test]
    fn stable_sort_preserves_equal_key_order_and_charges_its_storage() {
        let b = budget(64, 1024);
        let _scope = Scope::enter(b.clone());
        let mut data = [(2, 'a'), (1, 'b'), (2, 'c'), (1, 'd')];
        sort(&mut data, |a, b| a.0.cmp(&b.0)).unwrap();
        assert_eq!(data, [(1, 'b'), (1, 'd'), (2, 'a'), (2, 'c')]);
        let _nested = Scope::enter(budget(1, 1024));
        let before = data;
        assert!(sort(&mut data, |a, b| b.0.cmp(&a.0)).is_err());
        assert_eq!(data, before);
    }

    #[test]
    fn rejected_reservation_preserves_the_original_vector() {
        let b = budget(3, 100);
        let _scope = Scope::enter(b.clone());
        let mut out: Vec<u8> = Vec::new();
        assert!(reserve(&mut out, 4).is_err());
        assert_eq!(out.capacity(), 0);
        assert!(b.failed.load(Ordering::Relaxed));
    }

    #[test]
    fn exact_work_limit_and_nested_failure_restore_the_prior_request() {
        let outer = budget(32, 3);
        let _scope = Scope::enter(outer.clone());
        work(3).unwrap();
        {
            let _nested = Scope::enter(budget(0, 0));
            assert!(work(1).is_err());
        }
        work(0).unwrap();
        assert!(!outer.failed.load(Ordering::Relaxed));
        assert!(work(1).is_err());
    }

    #[test]
    fn another_thread_and_the_next_request_have_independent_budgets() {
        let scope = Scope::enter(budget(0, 0));
        assert!(work(1).is_err());
        std::thread::spawn(|| work(1).unwrap()).join().unwrap();
        drop(scope);
        let _next = Scope::enter(budget(8, 8));
        assert_eq!(filled(7u8, 8).unwrap(), [7; 8]);
    }
}

/// Collect fallible elements without an unaccounted intermediate allocation.
pub fn try_collect<T, E: From<ResourceError>>(
    iter: impl IntoIterator<Item = Result<T, E>>,
) -> Result<Vec<T>, E> {
    let mut out = Vec::new();
    for item in iter {
        push(&mut out, item?)?;
    }
    Ok(out)
}
/// Stable ordering with explicitly fallible index storage instead of hidden
/// allocator-owned sort scratch. Ties preserve the original input order.
pub fn sort<T>(
    data: &mut [T],
    mut cmp: impl FnMut(&T, &T) -> core::cmp::Ordering,
) -> Result<(), ResourceError> {
    let count = data.len();
    if count < 2 {
        return work(0);
    }
    let levels = usize::BITS as usize - count.leading_zeros() as usize;
    work(product(&[count, levels, 4])?)?;
    let mut indices = collect(0..count)?;
    indices.sort_unstable_by(|&a, &b| cmp(&data[a], &data[b]).then(a.cmp(&b)));
    let mut destinations = filled(0usize, count)?;
    for (destination, &source) in indices.iter().enumerate() {
        destinations[source] = destination;
    }
    for source in 0..count {
        while destinations[source] != source {
            work(1)?;
            let destination = destinations[source];
            data.swap(source, destination);
            destinations.swap(source, destination);
        }
    }
    Ok(())
}
