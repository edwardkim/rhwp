//! 모델 값의 다이제스트용 이진 인코딩.
//!
//! 봉인 다이제스트는 같은 프로세스 안에서 같은 타입의 값끼리만 비교한다(저장 시 재사용
//! 판정). 그 비교에 필요한 것은 **같은 타입 안에서 다른 값이 다른 바이트 열이 되는 것**
//! 뿐이므로, JSON 처럼 문자열을 이스케이프하고 숫자를 십진 표기로 바꿀 필요가 없다.
//!
//! - 모든 값 앞에 종류 태그를 쓴다.
//! - 문자열·바이트는 길이를 앞에 쓴다.
//! - 시퀀스·맵은 원소마다 표지를 쓰고 끝 표지로 닫는다(길이를 몰라도 접두 부호가 된다).
//! - 구조체 필드는 이름을 함께 쓴다(`skip_serializing_if` 로 필드가 빠져도 모호하지 않다).
//! - 열거형 변형은 변형 번호를 쓴다.
//! - float 는 비트 그대로 쓴다. JSON 이 null 로 합치던 NaN·무한대도 구별한다.
//!
//! 맵 순회 순서는 직렬화 대상이 정한다(JSON 인코딩과 같다).

use serde::ser::{self, Serialize};

/// 버퍼에 모아 큰 덩어리로 해시에 넘긴다.
pub(super) struct DigestWriter {
    hasher: blake3::Hasher,
    buffer: Vec<u8>,
}

const FLUSH_AT: usize = 64 * 1024;

impl DigestWriter {
    pub(super) fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
            buffer: Vec::with_capacity(FLUSH_AT + 1024),
        }
    }

    pub(super) fn finish(mut self) -> [u8; 32] {
        self.hasher.update(&self.buffer);
        *self.hasher.finalize().as_bytes()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
        if self.buffer.len() >= FLUSH_AT {
            self.hasher.update(&self.buffer);
            self.buffer.clear();
        }
    }

    fn tag(&mut self, tag: u8) {
        self.write(&[tag]);
    }

    fn len(&mut self, len: usize) {
        self.write(&(len as u64).to_le_bytes());
    }

    fn str(&mut self, value: &str) {
        self.len(value.len());
        self.write(value.as_bytes());
    }
}

mod tag {
    pub(super) const BOOL: u8 = 1;
    pub(super) const I8: u8 = 2;
    pub(super) const I16: u8 = 3;
    pub(super) const I32: u8 = 4;
    pub(super) const I64: u8 = 5;
    pub(super) const I128: u8 = 6;
    pub(super) const U8: u8 = 7;
    pub(super) const U16: u8 = 8;
    pub(super) const U32: u8 = 9;
    pub(super) const U64: u8 = 10;
    pub(super) const U128: u8 = 11;
    pub(super) const F32: u8 = 12;
    pub(super) const F64: u8 = 13;
    pub(super) const CHAR: u8 = 14;
    pub(super) const STR: u8 = 15;
    pub(super) const BYTES: u8 = 16;
    pub(super) const NONE: u8 = 17;
    pub(super) const SOME: u8 = 18;
    pub(super) const UNIT: u8 = 19;
    pub(super) const UNIT_STRUCT: u8 = 20;
    pub(super) const UNIT_VARIANT: u8 = 21;
    pub(super) const NEWTYPE_STRUCT: u8 = 22;
    pub(super) const NEWTYPE_VARIANT: u8 = 23;
    pub(super) const SEQ: u8 = 24;
    pub(super) const TUPLE: u8 = 25;
    pub(super) const TUPLE_STRUCT: u8 = 26;
    pub(super) const TUPLE_VARIANT: u8 = 27;
    pub(super) const MAP: u8 = 28;
    pub(super) const STRUCT: u8 = 29;
    pub(super) const STRUCT_VARIANT: u8 = 30;
    pub(super) const ELEMENT: u8 = 31;
    pub(super) const KEY: u8 = 32;
    pub(super) const VALUE: u8 = 33;
    pub(super) const FIELD: u8 = 34;
    pub(super) const SKIPPED_FIELD: u8 = 35;
    pub(super) const END: u8 = 36;
}

#[derive(Debug)]
pub(super) struct DigestError(String);

impl std::fmt::Display for DigestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DigestError {}

impl ser::Error for DigestError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

type Result<T = ()> = std::result::Result<T, DigestError>;

impl<'a> ser::Serializer for &'a mut DigestWriter {
    type Ok = ();
    type Error = DigestError;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    fn serialize_bool(self, v: bool) -> Result {
        self.write(&[tag::BOOL, u8::from(v)]);
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result {
        self.tag(tag::I8);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result {
        self.tag(tag::I16);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result {
        self.tag(tag::I32);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result {
        self.tag(tag::I64);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_i128(self, v: i128) -> Result {
        self.tag(tag::I128);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result {
        self.write(&[tag::U8, v]);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result {
        self.tag(tag::U16);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result {
        self.tag(tag::U32);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result {
        self.tag(tag::U64);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result {
        self.tag(tag::U128);
        self.write(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result {
        self.tag(tag::F32);
        self.write(&v.to_bits().to_le_bytes());
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result {
        self.tag(tag::F64);
        self.write(&v.to_bits().to_le_bytes());
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result {
        self.tag(tag::CHAR);
        self.write(&u32::from(v).to_le_bytes());
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result {
        self.tag(tag::STR);
        self.str(v);
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result {
        self.tag(tag::BYTES);
        self.len(v.len());
        self.write(v);
        Ok(())
    }
    fn serialize_none(self) -> Result {
        self.tag(tag::NONE);
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result {
        self.tag(tag::SOME);
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result {
        self.tag(tag::UNIT);
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result {
        self.tag(tag::UNIT_STRUCT);
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result {
        self.tag(tag::UNIT_VARIANT);
        self.write(&variant_index.to_le_bytes());
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result {
        self.tag(tag::NEWTYPE_STRUCT);
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result {
        self.tag(tag::NEWTYPE_VARIANT);
        self.write(&variant_index.to_le_bytes());
        value.serialize(self)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Compound<'a>> {
        self.tag(tag::SEQ);
        Ok(Compound(self))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Compound<'a>> {
        self.tag(tag::TUPLE);
        Ok(Compound(self))
    }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Compound<'a>> {
        self.tag(tag::TUPLE_STRUCT);
        Ok(Compound(self))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>> {
        self.tag(tag::TUPLE_VARIANT);
        self.write(&variant_index.to_le_bytes());
        Ok(Compound(self))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Compound<'a>> {
        self.tag(tag::MAP);
        Ok(Compound(self))
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Compound<'a>> {
        self.tag(tag::STRUCT);
        Ok(Compound(self))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>> {
        self.tag(tag::STRUCT_VARIANT);
        self.write(&variant_index.to_le_bytes());
        Ok(Compound(self))
    }
}

pub(super) struct Compound<'a>(&'a mut DigestWriter);

impl Compound<'_> {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result {
        self.0.tag(tag::ELEMENT);
        value.serialize(&mut *self.0)
    }

    fn field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result {
        self.0.tag(tag::FIELD);
        self.0.str(key);
        value.serialize(&mut *self.0)
    }

    fn skipped_field(&mut self, key: &'static str) -> Result {
        self.0.tag(tag::SKIPPED_FIELD);
        self.0.str(key);
        Ok(())
    }

    fn end(self) -> Result {
        self.0.tag(tag::END);
        Ok(())
    }
}

impl ser::SerializeSeq for Compound<'_> {
    type Ok = ();
    type Error = DigestError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result {
        self.element(value)
    }
    fn end(self) -> Result {
        Compound::end(self)
    }
}

impl ser::SerializeTuple for Compound<'_> {
    type Ok = ();
    type Error = DigestError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result {
        self.element(value)
    }
    fn end(self) -> Result {
        Compound::end(self)
    }
}

impl ser::SerializeTupleStruct for Compound<'_> {
    type Ok = ();
    type Error = DigestError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result {
        self.element(value)
    }
    fn end(self) -> Result {
        Compound::end(self)
    }
}

impl ser::SerializeTupleVariant for Compound<'_> {
    type Ok = ();
    type Error = DigestError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result {
        self.element(value)
    }
    fn end(self) -> Result {
        Compound::end(self)
    }
}

impl ser::SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = DigestError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result {
        self.0.tag(tag::KEY);
        key.serialize(&mut *self.0)
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result {
        self.0.tag(tag::VALUE);
        value.serialize(&mut *self.0)
    }
    fn end(self) -> Result {
        Compound::end(self)
    }
}

impl ser::SerializeStruct for Compound<'_> {
    type Ok = ();
    type Error = DigestError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result {
        self.field(key, value)
    }
    fn skip_field(&mut self, key: &'static str) -> Result {
        self.skipped_field(key)
    }
    fn end(self) -> Result {
        Compound::end(self)
    }
}

impl ser::SerializeStructVariant for Compound<'_> {
    type Ok = ();
    type Error = DigestError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result {
        self.field(key, value)
    }
    fn skip_field(&mut self, key: &'static str) -> Result {
        self.skipped_field(key)
    }
    fn end(self) -> Result {
        Compound::end(self)
    }
}
