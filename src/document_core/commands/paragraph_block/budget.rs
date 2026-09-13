//! Allocation-free serde visitor: count owned values, never construct JSON/IR.
//! Inline storage is intentionally overcounted. Vec capacity/RSS are not promised.
use crate::{error::HwpError, model::paragraph::Paragraph};
use serde::{ser, Serialize};
use std::fmt;

#[derive(Debug)]
struct BudgetError(&'static str);
impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for BudgetError {}
impl ser::Error for BudgetError {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Self("unsupported serialization in budget")
    }
}

struct Meter {
    bytes: usize,
    limit: usize,
    depth: usize,
}
impl Meter {
    fn add(&mut self, n: usize) -> Result<(), BudgetError> {
        self.bytes = self
            .bytes
            .checked_add(n)
            .ok_or(BudgetError("structure size overflow"))?;
        if self.bytes > self.limit {
            return Err(BudgetError("structure byte budget exceeded"));
        }
        Ok(())
    }
    fn value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), BudgetError> {
        // Also covers recursive extension payloads outside the paragraph tree.
        if self.depth >= 512 {
            return Err(BudgetError("budget traversal depth exceeded"));
        }
        self.add(std::mem::size_of_val(value))?;
        self.depth += 1;
        let result = value.serialize(&mut *self);
        self.depth -= 1;
        result
    }
}

pub(super) fn measure(
    paras: &[Paragraph],
    count: usize,
    limit: usize,
    skipped: usize,
) -> Result<usize, HwpError> {
    let mut meter = Meter {
        bytes: 0,
        limit: limit / count,
        depth: 0,
    };
    // The owned walk charges skipped vpos snapshots at every nesting level.
    // Other skipped seals/flags are inline in the fixed node charge.
    let result = meter.add(skipped).and_then(|()| meter.value(paras));
    result.map_err(|e| super::invalid(e.to_string()))?;
    meter
        .bytes
        .checked_mul(count)
        .ok_or_else(|| super::invalid("structure size overflow"))
}

/// The same allocation-free accounting for detached shared-resource candidates.
pub(super) fn measure_value<T: Serialize + ?Sized>(
    value: &T,
    limit: usize,
) -> Result<usize, HwpError> {
    let mut meter = Meter {
        bytes: 0,
        limit,
        depth: 0,
    };
    meter
        .value(value)
        .map_err(|e| super::invalid(e.to_string()))?;
    Ok(meter.bytes)
}

struct Compound<'a> {
    meter: &'a mut Meter,
}

macro_rules! scalar {
    ($($name:ident($ty:ty)),* $(,)?) => {$(
        fn $name(self, _: $ty) -> Result<(), BudgetError> { Ok(()) }
    )*};
}
impl<'a> ser::Serializer for &'a mut Meter {
    type Ok = ();
    type Error = BudgetError;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;
    scalar!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char)
    );
    fn serialize_str(self, v: &str) -> Result<(), BudgetError> {
        self.add(v.len())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<(), BudgetError> {
        self.add(v.len())
    }
    fn serialize_none(self) -> Result<(), BudgetError> {
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), BudgetError> {
        self.value(v)
    }
    fn serialize_unit(self) -> Result<(), BudgetError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), BudgetError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), BudgetError> {
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), BudgetError> {
        self.value(v)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<(), BudgetError> {
        self.value(v)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'a>, BudgetError> {
        Ok(Compound { meter: self })
    }
    fn serialize_tuple(self, _: usize) -> Result<Compound<'a>, BudgetError> {
        Ok(Compound { meter: self })
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Compound<'a>, BudgetError> {
        Ok(Compound { meter: self })
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Compound<'a>, BudgetError> {
        Ok(Compound { meter: self })
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Compound<'a>, BudgetError> {
        self.add(
            len.unwrap_or(0)
                .checked_mul(64)
                .ok_or(BudgetError("map size overflow"))?,
        )?;
        Ok(Compound { meter: self })
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Compound<'a>, BudgetError> {
        Ok(Compound { meter: self })
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Compound<'a>, BudgetError> {
        Ok(Compound { meter: self })
    }
}

macro_rules! sequence {
    ($trait:ident, $element:ident) => {
        impl ser::$trait for Compound<'_> {
            type Ok = ();
            type Error = BudgetError;
            fn $element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), BudgetError> {
                self.meter.value(v)
            }
            fn end(self) -> Result<(), BudgetError> {
                Ok(())
            }
        }
    };
}
sequence!(SerializeSeq, serialize_element);
sequence!(SerializeTuple, serialize_element);
sequence!(SerializeTupleStruct, serialize_field);
sequence!(SerializeTupleVariant, serialize_field);
impl ser::SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = BudgetError;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), BudgetError> {
        self.meter.value(v)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), BudgetError> {
        self.meter.value(v)
    }
    fn end(self) -> Result<(), BudgetError> {
        Ok(())
    }
}
macro_rules! structure {
    ($trait:ident) => {
        impl ser::$trait for Compound<'_> {
            type Ok = ();
            type Error = BudgetError;
            fn serialize_field<T: Serialize + ?Sized>(
                &mut self,
                _: &'static str,
                v: &T,
            ) -> Result<(), BudgetError> {
                self.meter.value(v)
            }
            fn end(self) -> Result<(), BudgetError> {
                Ok(())
            }
        }
    };
}
structure!(SerializeStruct);
structure!(SerializeStructVariant);
