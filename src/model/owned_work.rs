//! Shared logical work for copies of audited model records, without JSON/IR output.
//! Inline values are deliberately overcounted. Capacity, allocator nodes and RSS
//! are separate limits. Callers account serde-skipped owned data and audit custom
//! serializers that could allocate before invoking this visitor.
use serde::{ser, Serialize};
use std::fmt;

#[derive(Debug)]
pub(crate) struct BudgetError(&'static str);
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

pub(crate) fn measure<T: Serialize + ?Sized>(
    value: &T,
    limit: usize,
    skipped: usize,
) -> Result<usize, BudgetError> {
    let mut meter = Meter {
        bytes: 0,
        limit,
        depth: 0,
    };
    meter.add(skipped)?;
    meter.value(value)?;
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

    fn collect_str<T: fmt::Display + ?Sized>(self, value: &T) -> Result<(), BudgetError> {
        // serde's default formats a complete String first. Count streaming
        // writes instead so the preflight cannot create that unbounded copy.
        struct Counter<'a> {
            meter: &'a mut Meter,
            failure: Option<BudgetError>,
        }
        impl fmt::Write for Counter<'_> {
            fn write_str(&mut self, text: &str) -> fmt::Result {
                if self.failure.is_some() {
                    return Err(fmt::Error);
                }
                self.meter.add(text.len()).map_err(|error| {
                    self.failure = Some(error);
                    fmt::Error
                })
            }
        }
        let mut counter = Counter {
            meter: self,
            failure: None,
        };
        let result = fmt::write(&mut counter, format_args!("{value}"));
        match counter.failure {
            Some(error) => Err(error),
            None => result.map_err(|_| BudgetError("unsupported serialization in budget")),
        }
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
