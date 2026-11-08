//! Carry the request through serde's value materialization, including map keys,
//! nested values and Display formatting. Wire output uses the separate byte writer.
use super::{allocation::Allocation, PdfImportError};
use serde::{
    ser::{self, Error as _},
    Serialize, Serializer,
};
use serde_json::Value;

pub(super) fn to_value<T: ?Sized + Serialize>(
    allocation: Allocation<'_>,
    value: &T,
) -> Result<Value, PdfImportError> {
    let result = value.serialize(Guard {
        inner: serde_json::value::Serializer,
        allocation,
        depth: 0,
        key: false,
        number_text: false,
    });
    allocation.0.check().map_err(PdfImportError::backend)?;
    result.map_err(|_| PdfImportError::budget("frontend value serialization failed"))
}

struct Guard<'a, S> {
    inner: S,
    allocation: Allocation<'a>,
    depth: usize,
    key: bool,
    number_text: bool,
}
struct Borrowed<'a, T: ?Sized> {
    value: &'a T,
    allocation: Allocation<'a>,
    depth: usize,
    key: bool,
    number_text: bool,
}
impl<T: ?Sized + Serialize> Serialize for Borrowed<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(Guard {
            inner: serializer,
            allocation: self.allocation,
            depth: self.depth,
            key: self.key,
            number_text: self.number_text,
        })
    }
}
impl<S> Guard<'_, S> {
    fn bytes<E: ser::Error>(&self, count: usize) -> Result<(), E> {
        self.allocation
            .bytes(count)
            .map_err(|_| E::custom("frontend resource reservation failed"))
    }
    fn sequence<E: ser::Error>(&self, len: usize) -> Result<(), E> {
        let bytes = len
            .checked_mul(std::mem::size_of::<Value>())
            .ok_or_else(|| {
                self.allocation.0.reject("frontend value sequence overflow");
                E::custom("frontend value sequence overflow")
            })?;
        self.bytes(bytes)
    }
    fn map<E: ser::Error>(&self, len: usize) -> Result<(), E> {
        // Logical object work: a fixed initial-node reservation plus twice the
        // record/link payload per entry. Strings and nested values are separate.
        // It is not a portable proof of the standard library's allocator layout.
        self.bytes::<E>(1024)?;
        let bytes = len.checked_mul(map_entry_work()).ok_or_else(|| {
            self.allocation.0.reject("frontend value map overflow");
            E::custom("frontend value map overflow")
        })?;
        self.bytes(bytes)
    }
    fn container<E: ser::Error>(&self, levels: usize) -> Result<(), E> {
        if self.depth.saturating_add(levels) > 128 {
            self.allocation.0.reject("frontend value depth");
            return Err(E::custom("frontend value depth"));
        }
        self.allocation
            .0
            .check()
            .map_err(|_| E::custom("frontend request failed"))
    }
}
pub(super) fn map_entry_work() -> usize {
    2 * (std::mem::size_of::<(String, Value)>() + 4 * std::mem::size_of::<usize>())
}

macro_rules! primitive {
    ($method:ident,$ty:ty) => {
        primitive!($method, $ty, |_: $ty| true);
    };
    ($method:ident,$ty:ty,$owns:expr) => {
        fn $method(self, value: $ty) -> Result<Self::Ok, Self::Error> {
            // Numeric primitives may allocate a decimal String when a consumer
            // enables serde_json/arbitrary_precision. Its integer/float emitters
            // format on the stack and copy exactly the resulting decimal bytes.
            // Count that same wire spelling without constructing another String.
            // Supported serde_json versions use a plain numeric enum without
            // arbitrary_precision and an owning String with it. needs_drop is
            // a constant type property, so inline numbers incur no heap work.
            if self.key {
                self.bytes::<Self::Error>(64)?;
            } else if std::mem::needs_drop::<serde_json::Number>() && ($owns)(value) {
                struct Count(usize);
                impl std::io::Write for Count {
                    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                        self.0 = self.0.saturating_add(bytes.len());
                        Ok(bytes.len())
                    }
                    fn flush(&mut self) -> std::io::Result<()> {
                        Ok(())
                    }
                }
                let mut count = Count(0);
                serde_json::to_writer(&mut count, &value).map_err(|_| {
                    self.allocation
                        .0
                        .reject("frontend numeric formatting failed");
                    Self::Error::custom("frontend numeric formatting failed")
                })?;
                self.bytes::<Self::Error>(count.0)?;
            }
            self.inner.$method(value)
        }
    };
}
impl<'a, S: Serializer> Serializer for Guard<'a, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = Sequence<'a, S::SerializeSeq>;
    type SerializeTuple = Sequence<'a, S::SerializeTuple>;
    type SerializeTupleStruct = Sequence<'a, S::SerializeTupleStruct>;
    type SerializeTupleVariant = Sequence<'a, S::SerializeTupleVariant>;
    type SerializeMap = Object<'a, S::SerializeMap>;
    type SerializeStruct = Object<'a, S::SerializeStruct>;
    type SerializeStructVariant = Object<'a, S::SerializeStructVariant>;
    primitive!(serialize_bool, bool, |_: bool| false);
    primitive!(serialize_i8, i8);
    primitive!(serialize_i16, i16);
    primitive!(serialize_i32, i32);
    primitive!(serialize_i64, i64);
    primitive!(serialize_i128, i128);
    primitive!(serialize_u8, u8);
    primitive!(serialize_u16, u16);
    primitive!(serialize_u32, u32);
    primitive!(serialize_u64, u64);
    primitive!(serialize_u128, u128);
    primitive!(serialize_f32, f32, |value: f32| value.is_finite());
    primitive!(serialize_f64, f64, |value: f64| value.is_finite());
    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.bytes::<Self::Error>(value.len_utf8().max(8))?;
        self.inner.serialize_char(value)
    }
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        let work = if self.number_text {
            // serde_json's arbitrary-precision emitter owns the input string,
            // then scans a decimal into a 16-byte String with doubling growth.
            // Reserve those replacement payloads, not a large per-number bound.
            // At most 20 bytes can normalize to i64/u64; that final decimal copy
            // is separate, while longer numbers retain the scanner's String.
            let work = || -> Option<usize> {
                let mut capacity = 16usize;
                let mut work = value.len().checked_add(capacity)?;
                while capacity < value.len() {
                    capacity = capacity.checked_mul(2)?;
                    work = work.checked_add(capacity)?;
                }
                if value.len() <= 20 {
                    work = work.checked_add(value.len())?;
                }
                Some(work)
            };
            work().ok_or_else(|| {
                self.allocation
                    .0
                    .reject("frontend numeric text work overflow");
                Self::Error::custom("frontend numeric text work overflow")
            })?
        } else {
            value.len()
        };
        self.bytes::<Self::Error>(work)?;
        self.inner.serialize_str(value)
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.container::<Self::Error>(1)?;
        self.sequence::<Self::Error>(value.len())?;
        // Value's byte emitter creates Number values directly, without visiting
        // our primitive methods. In an owning-number graph those decimal copies
        // must also be reserved before forwarding the complete byte slice.
        if std::mem::needs_drop::<serde_json::Number>() {
            let work = value
                .iter()
                .try_fold(0usize, |work, &byte| {
                    work.checked_add(if byte < 10 {
                        1
                    } else if byte < 100 {
                        2
                    } else {
                        3
                    })
                })
                .ok_or_else(|| {
                    self.allocation
                        .0
                        .reject("frontend numeric bytes work overflow");
                    Self::Error::custom("frontend numeric bytes work overflow")
                })?;
            self.bytes::<Self::Error>(work)?;
        }
        self.inner.serialize_bytes(value)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_none()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_some(&Borrowed {
            value,
            allocation: self.allocation,
            depth: self.depth,
            key: self.key,
            number_text: self.number_text,
        })
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit()
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit_struct(name)
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.bytes::<Self::Error>(variant.len())?;
        self.inner.serialize_unit_variant(name, index, variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_newtype_struct(
            name,
            &Borrowed {
                value,
                allocation: self.allocation,
                depth: self.depth,
                key: self.key,
                number_text: self.number_text,
            },
        )
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.container::<Self::Error>(1)?;
        self.map::<Self::Error>(1)?;
        self.bytes::<Self::Error>(variant.len())?;
        self.inner.serialize_newtype_variant(
            name,
            index,
            variant,
            &Borrowed {
                value,
                allocation: self.allocation,
                depth: self.depth + 1,
                key: false,
                number_text: false,
            },
        )
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.container::<Self::Error>(1)?;
        self.sequence::<Self::Error>(len.unwrap_or(0))?;
        Ok(Sequence {
            inner: self.inner.serialize_seq(len)?,
            allocation: self.allocation,
            depth: self.depth + 1,
            capacity: len.unwrap_or(0),
            used: 0,
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.container::<Self::Error>(1)?;
        self.sequence::<Self::Error>(len)?;
        Ok(Sequence {
            inner: self.inner.serialize_tuple(len)?,
            allocation: self.allocation,
            depth: self.depth + 1,
            capacity: len,
            used: 0,
        })
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.container::<Self::Error>(1)?;
        self.sequence::<Self::Error>(len)?;
        Ok(Sequence {
            inner: self.inner.serialize_tuple_struct(name, len)?,
            allocation: self.allocation,
            depth: self.depth + 1,
            capacity: len,
            used: 0,
        })
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.container::<Self::Error>(2)?;
        self.map::<Self::Error>(1)?;
        self.bytes::<Self::Error>(variant.len())?;
        self.sequence::<Self::Error>(len)?;
        Ok(Sequence {
            inner: self
                .inner
                .serialize_tuple_variant(name, index, variant, len)?,
            allocation: self.allocation,
            depth: self.depth + 2,
            capacity: len,
            used: 0,
        })
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.container::<Self::Error>(1)?;
        self.map::<Self::Error>(len.unwrap_or(0))?;
        Ok(Object {
            inner: self.inner.serialize_map(len)?,
            allocation: self.allocation,
            depth: self.depth + 1,
            reserved: len.unwrap_or(0),
            number_text: false,
            used: 0,
        })
    }
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let number_text =
            name == "$serde_json::private::Number" && std::mem::needs_drop::<serde_json::Number>();
        // The decimal protocol encodes one scalar, not another JSON container.
        let levels = usize::from(!number_text);
        self.container::<Self::Error>(levels)?;
        // Opaque raw JSON reparses behind serde's normal visitors. The frontend
        // uses typed records/Value, never this native-envelope-only contract.
        // Number is different: its decimal field still visits serialize_str.
        // Reserve that emitter's scan before forwarding the field, so another
        // consumer enabling arbitrary_precision does not disable PDF imports.
        if name == "$serde_json::private::RawValue" {
            self.allocation.0.reject("opaque frontend JSON value");
            return Err(Self::Error::custom("opaque frontend JSON value"));
        }
        if !number_text {
            self.map::<Self::Error>(len)?;
        }
        Ok(Object {
            inner: self.inner.serialize_struct(name, len)?,
            allocation: self.allocation,
            depth: self.depth + levels,
            reserved: len,
            number_text,
            used: 0,
        })
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.container::<Self::Error>(2)?;
        self.map::<Self::Error>(1)?;
        self.bytes::<Self::Error>(variant.len())?;
        self.map::<Self::Error>(len)?;
        Ok(Object {
            inner: self
                .inner
                .serialize_struct_variant(name, index, variant, len)?,
            allocation: self.allocation,
            depth: self.depth + 2,
            reserved: len,
            number_text: false,
            used: 0,
        })
    }
    fn collect_str<T: ?Sized + std::fmt::Display>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let text = self
            .allocation
            .display(value)
            .map_err(|_| Self::Error::custom("frontend display reservation failed"))?;
        self.serialize_str(&text)
    }
    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}

struct Sequence<'a, C> {
    inner: C,
    allocation: Allocation<'a>,
    depth: usize,
    capacity: usize,
    used: usize,
}
impl<C> Sequence<'_, C> {
    fn next<E: ser::Error>(&mut self) -> Result<(), E> {
        if self.used >= self.capacity {
            let capacity = self
                .capacity
                .saturating_mul(2)
                .max(self.used.saturating_add(1))
                .max(4);
            let bytes = capacity
                .checked_mul(std::mem::size_of::<Value>())
                .ok_or_else(|| {
                    self.allocation.0.reject("frontend value capacity overflow");
                    E::custom("frontend value capacity overflow")
                })?;
            self.allocation
                .bytes(bytes)
                .map_err(|_| E::custom("frontend value growth failed"))?;
            self.capacity = capacity;
        }
        self.used += 1;
        Ok(())
    }
}
macro_rules! sequence {
    ($trait:ident,$element:ident) => {
        impl<C: ser::$trait> ser::$trait for Sequence<'_, C> {
            type Ok = C::Ok;
            type Error = C::Error;
            fn $element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
                self.next::<Self::Error>()?;
                self.inner.$element(&Borrowed {
                    value,
                    allocation: self.allocation,
                    depth: self.depth,
                    key: false,
                    number_text: false,
                })
            }
            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.inner.end()
            }
        }
    };
}
sequence!(SerializeSeq, serialize_element);
sequence!(SerializeTuple, serialize_element);
sequence!(SerializeTupleStruct, serialize_field);
sequence!(SerializeTupleVariant, serialize_field);

struct Object<'a, C> {
    inner: C,
    allocation: Allocation<'a>,
    depth: usize,
    reserved: usize,
    number_text: bool,
    used: usize,
}
impl<C> Object<'_, C> {
    fn next<E: ser::Error>(&mut self) -> Result<(), E> {
        if self.used >= self.reserved {
            self.allocation
                .bytes(map_entry_work())
                .map_err(|_| E::custom("frontend object growth failed"))?;
            self.reserved += 1;
        }
        self.used += 1;
        Ok(())
    }
}
impl<C: ser::SerializeMap> ser::SerializeMap for Object<'_, C> {
    type Ok = C::Ok;
    type Error = C::Error;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.next::<Self::Error>()?;
        self.inner.serialize_key(&Borrowed {
            value: key,
            allocation: self.allocation,
            depth: self.depth,
            key: true,
            number_text: false,
        })
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_value(&Borrowed {
            value,
            allocation: self.allocation,
            depth: self.depth,
            key: false,
            number_text: false,
        })
    }
    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        self.next::<Self::Error>()?;
        self.inner.serialize_entry(
            &Borrowed {
                value: key,
                allocation: self.allocation,
                depth: self.depth,
                key: true,
                number_text: false,
            },
            &Borrowed {
                value,
                allocation: self.allocation,
                depth: self.depth,
                key: false,
                number_text: false,
            },
        )
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
macro_rules! object {
    ($trait:ident) => {
        impl<C: ser::$trait> ser::$trait for Object<'_, C> {
            type Ok = C::Ok;
            type Error = C::Error;
            fn serialize_field<T: ?Sized + Serialize>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error> {
                if !self.number_text {
                    self.next::<Self::Error>()?;
                    self.allocation
                        .bytes(key.len())
                        .map_err(|_| Self::Error::custom("frontend field key failed"))?;
                }
                self.inner.serialize_field(
                    key,
                    &Borrowed {
                        value,
                        allocation: self.allocation,
                        depth: self.depth,
                        key: false,
                        number_text: self.number_text,
                    },
                )
            }
            fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
                self.inner.skip_field(key)
            }
            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.inner.end()
            }
        }
    };
}
object!(SerializeStruct);
object!(SerializeStructVariant);
