//! Fallible copies of derived state. Shared pointers retain their allocation;
//! owned collection and string backings use the caller's request policy.
use super::Storage;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

pub(crate) trait CloneWithStorage: Clone {
    fn clone_with_storage<S: Storage>(&self, storage: &S) -> Result<Self, S::Error>;
}
pub(crate) fn try_clone<T: CloneWithStorage, S: Storage>(
    value: &T,
    storage: &S,
) -> Result<T, S::Error> {
    if S::UNBOUNDED {
        Ok(value.clone())
    } else {
        value.clone_with_storage(storage)
    }
}
macro_rules! clone_copy {
    ($($ty:ty),* $(,)?) => {$ (
        impl $crate::storage::CloneWithStorage for $ty {
            fn clone_with_storage<S:$crate::storage::Storage>(&self,storage:&S)->Result<Self,S::Error> {
                let _=storage; Ok(*self)
            }
        }
    )*};
}
pub(crate) use clone_copy;
macro_rules! clone_fields {
    ($ty:ty, {$($field:ident),* $(,)?}) => {
        impl $crate::storage::CloneWithStorage for $ty {
            fn clone_with_storage<S:$crate::storage::Storage>(&self,storage:&S)->Result<Self,S::Error> {
                // Exhaustive patterns make future owned fields an explicit policy change.
                let Self { $($field),* }=self;
                Ok(Self { $($field: $crate::storage::try_clone($field,storage)?),* })
            }
        }
    };
}
pub(crate) use clone_fields;
clone_copy!(
    (),
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64
);
impl CloneWithStorage for String {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        s.string(self)
    }
}
impl<T: ?Sized> CloneWithStorage for &T {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let _ = s;
        Ok(*self)
    }
}
impl<T: Copy, const N: usize> CloneWithStorage for [T; N] {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let _ = s;
        Ok(*self)
    }
}
impl<T: CloneWithStorage> CloneWithStorage for Vec<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let mut out = s.vec(self.len())?;
        for value in self {
            out.push(try_clone(value, s)?);
        }
        Ok(out)
    }
}
impl<T: CloneWithStorage> CloneWithStorage for Option<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        self.as_ref().map(|v| try_clone(v, s)).transpose()
    }
}
impl<T: CloneWithStorage> CloneWithStorage for Box<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        s.bytes(std::mem::size_of::<T>())?;
        Ok(Box::new(try_clone(self.as_ref(), s)?))
    }
}
impl<T: ?Sized> CloneWithStorage for std::sync::Arc<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let _ = s;
        Ok(Self::clone(self))
    }
}
impl<T: ?Sized> CloneWithStorage for std::rc::Rc<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let _ = s;
        Ok(Self::clone(self))
    }
}
impl<T: CloneWithStorage> CloneWithStorage for std::cell::RefCell<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        Ok(Self::new(try_clone(&*self.borrow(), s)?))
    }
}
impl<K: Ord + CloneWithStorage, V: CloneWithStorage> CloneWithStorage for BTreeMap<K, V> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let mut out = Self::new();
        for (k, v) in self {
            s.tree_entry::<K, V>()?;
            out.insert(try_clone(k, s)?, try_clone(v, s)?);
        }
        Ok(out)
    }
}
impl<T: Ord + CloneWithStorage> CloneWithStorage for BTreeSet<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let mut out = Self::new();
        for v in self {
            s.tree_entry::<T, ()>()?;
            out.insert(try_clone(v, s)?);
        }
        Ok(out)
    }
}
impl<K: Eq + Hash + CloneWithStorage, V: CloneWithStorage> CloneWithStorage for HashMap<K, V> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let mut out = Self::with_hasher(self.hasher().clone());
        s.reserve_map(&mut out, self.capacity())?;
        for (k, v) in self {
            out.insert(try_clone(k, s)?, try_clone(v, s)?);
        }
        Ok(out)
    }
}
impl<T: Eq + Hash + CloneWithStorage> CloneWithStorage for HashSet<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        let mut out = Self::with_hasher(self.hasher().clone());
        s.reserve_set(&mut out, self.capacity())?;
        for v in self {
            out.insert(try_clone(v, s)?);
        }
        Ok(out)
    }
}
macro_rules! tuple_clone {
    ($($ty:ident:$i:tt),+) => {
        impl<$($ty:CloneWithStorage),+> CloneWithStorage for ($($ty,)+) {
            fn clone_with_storage<S:Storage>(&self,s:&S)->Result<Self,S::Error>{Ok(($(try_clone(&self.$i,s)?,)+))}
        }
    };
}
tuple_clone!(A:0);
tuple_clone!(A:0,B:1);
tuple_clone!(A:0,B:1,C:2);
tuple_clone!(A:0,B:1,C:2,D:3);
tuple_clone!(A:0,B:1,C:2,D:3,E:4);
tuple_clone!(A:0,B:1,C:2,D:3,E:4,F:5);
impl CloneWithStorage for crate::model::paragraph::Paragraph {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        s.clone_paragraph(self)
    }
}
impl CloneWithStorage for crate::model::table::Table {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        s.clone_table(self)
    }
}

impl<T: CloneWithStorage> CloneWithStorage for std::ops::Range<T> {
    fn clone_with_storage<S: Storage>(&self, s: &S) -> Result<Self, S::Error> {
        Ok(try_clone(&self.start, s)?..try_clone(&self.end, s)?)
    }
}

impl CloneWithStorage for crate::model::style::Numbering {
    fn clone_with_storage<S: Storage>(&self, storage: &S) -> Result<Self, S::Error> {
        storage.clone_record(self)
    }
}
impl CloneWithStorage for crate::model::style::Bullet {
    fn clone_with_storage<S: Storage>(&self, storage: &S) -> Result<Self, S::Error> {
        storage.clone_record(self)
    }
}

clone_copy!(
    crate::model::style::Alignment,
    crate::model::style::LineSpacingType,
    crate::model::style::HeadType,
    crate::model::style::CenterLine
);

clone_copy!(
    crate::model::style::ImageFillMode,
    crate::model::image::ImageEffect
);
