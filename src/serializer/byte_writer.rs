//! 바이너리 데이터 쓰기 유틸리티
//!
//! HWP 레코드 내부의 바이너리 필드를 순차적으로 쓰기 위한 버퍼 기반 라이터.
//! `ByteReader`의 역방향으로, 리틀 엔디안과 UTF-16LE 문자열을 지원한다.

use std::io::{self, Write};

/// BMP(U+0000~U+FFFF) 밖의 문자를 HWP5 WCHAR(16비트) 슬롯에 그대로 `as u16`으로 캐스팅하면
/// 상위 비트가 잘려나가 다른 문자로 손상된다(예: '🔶'(U+1F536) as u16 == 0xF536).
/// HWPX는 이런 문자를 유효하게 담을 수 있으므로(글머리표·각주 기호 등), HWP5로 저장할 때는
/// BMP를 벗어난 문자를 대체 문자(U+FFFD, REPLACEMENT CHARACTER)로 치환해 최소한 손상된 값이
/// 엉뚱한 글자로 조용히 바뀌지 않도록 한다.
pub fn char_to_wchar(c: char) -> u16 {
    if (c as u32) <= 0xFFFF {
        c as u16
    } else {
        0xFFFD
    }
}

/// A deterministic record encoder shared by counting and output writers.
/// Implementations must stream fields without hidden owned scratch.
pub(crate) trait BinaryProducer {
    fn write<W: Write>(&self, writer: &mut ByteWriter<W>) -> io::Result<()>;
}

/// 바이트 라이터. 일반 API의 기본 backing은 Vec이며 요청용 경로는 같은
/// 필드 writer를 counting sink와 미리 예약한 고정 slice에 적용한다.
pub struct ByteWriter<W = Vec<u8>> {
    buf: W,
    position: usize,
}

impl ByteWriter {
    pub fn new() -> Self {
        Self::with_writer(Vec::new())
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }
}

impl<W: Write> ByteWriter<W> {
    pub(crate) fn with_writer(buf: W) -> Self {
        Self { buf, position: 0 }
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn write_u8(&mut self, v: u8) -> io::Result<()> {
        self.write_bytes(&[v])
    }
    pub fn write_u16(&mut self, v: u16) -> io::Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_u32(&mut self, v: u32) -> io::Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_i8(&mut self, v: i8) -> io::Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_i16(&mut self, v: i16) -> io::Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_i32(&mut self, v: i32) -> io::Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_f64(&mut self, v: f64) -> io::Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_bytes(&mut self, data: &[u8]) -> io::Result<()> {
        let end = self
            .position
            .checked_add(data.len())
            .ok_or_else(|| io::Error::other("binary size overflow"))?;
        self.buf.write_all(data)?;
        self.position = end;
        Ok(())
    }
    pub fn write_hwp_string(&mut self, text: &str) -> io::Result<()> {
        // Preserve the old u16 prefix/truncation contract, including a high
        // surrogate cut at 65535 units, without materializing a UTF-16 Vec.
        let mut units = 0usize;
        let mut last = 0u16;
        for unit in text.encode_utf16().take(u16::MAX as usize) {
            units += 1;
            last = unit;
        }
        if (0xD800..=0xDBFF).contains(&last) {
            units -= 1;
        }
        self.write_u16(units as u16)?;
        for unit in text.encode_utf16().take(units) {
            self.write_u16(unit)?;
        }
        Ok(())
    }
    pub fn write_color_ref(&mut self, color: u32) -> io::Result<()> {
        self.write_u32(color)
    }
    pub fn write_zeros(&mut self, mut count: usize) -> io::Result<()> {
        const ZERO: [u8; 256] = [0; 256];
        while count != 0 {
            let n = count.min(ZERO.len());
            self.write_bytes(&ZERO[..n])?;
            count -= n;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::byte_reader::ByteReader;

    #[test]
    fn test_char_to_wchar_bmp_passthrough() {
        assert_eq!(char_to_wchar('A'), 0x0041);
        assert_eq!(char_to_wchar('가'), 0xAC00);
        assert_eq!(char_to_wchar('\u{FFFF}'), 0xFFFF);
    }

    #[test]
    fn test_char_to_wchar_astral_replaces_instead_of_truncating() {
        // '🔶'(U+1F536)를 그냥 `as u16`으로 캐스팅하면 상위 비트가 잘려 0xF536(다른 문자)이
        // 되어 손상된 값이 조용히 기록된다. char_to_wchar는 대신 U+FFFD로 치환해야 한다.
        let c = '\u{1F536}';
        assert_ne!(c as u32 as u16, 0xFFFD); // as u16 캐스팅은 0xF536이 되어 손상된다는 전제 확인
        assert_eq!(char_to_wchar(c), 0xFFFD);
    }

    #[test]
    fn test_write_u8() {
        let mut w = ByteWriter::new();
        w.write_u8(0x42).unwrap();
        assert_eq!(w.into_bytes(), [0x42]);
    }

    #[test]
    fn test_write_u16_le() {
        let mut w = ByteWriter::new();
        w.write_u16(0x1234).unwrap();
        assert_eq!(w.into_bytes(), [0x34, 0x12]);
    }

    #[test]
    fn test_write_u32_le() {
        let mut w = ByteWriter::new();
        w.write_u32(0x12345678).unwrap();
        assert_eq!(w.into_bytes(), [0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_write_i8() {
        let mut w = ByteWriter::new();
        w.write_i8(-1).unwrap();
        assert_eq!(w.into_bytes(), [0xFF]);
    }

    #[test]
    fn test_write_i16_negative() {
        let mut w = ByteWriter::new();
        w.write_i16(-100).unwrap();
        let bytes = w.into_bytes();
        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_i16().unwrap(), -100);
    }

    #[test]
    fn test_write_i32_negative() {
        let mut w = ByteWriter::new();
        w.write_i32(-7200).unwrap();
        let bytes = w.into_bytes();
        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_i32().unwrap(), -7200);
    }

    #[test]
    fn test_write_bytes() {
        let mut w = ByteWriter::new();
        w.write_bytes(&[0x01, 0x02, 0x03]).unwrap();
        assert_eq!(w.into_bytes(), [0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_write_hwp_string_korean() {
        // "한글" → u16 글자수(2) + UTF-16LE
        let mut w = ByteWriter::new();
        w.write_hwp_string("한글").unwrap();
        let bytes = w.into_bytes();

        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_hwp_string().unwrap(), "한글");
    }

    #[test]
    fn test_write_hwp_string_ascii() {
        let mut w = ByteWriter::new();
        w.write_hwp_string("ABC").unwrap();
        let bytes = w.into_bytes();

        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_hwp_string().unwrap(), "ABC");
    }

    #[test]
    fn test_write_hwp_string_empty() {
        let mut w = ByteWriter::new();
        w.write_hwp_string("").unwrap();
        let bytes = w.into_bytes();

        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_hwp_string().unwrap(), "");
    }

    #[test]
    fn test_write_hwp_string_mixed() {
        // 한글 + ASCII 혼합
        let mut w = ByteWriter::new();
        w.write_hwp_string("Hello 세계!").unwrap();
        let bytes = w.into_bytes();

        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_hwp_string().unwrap(), "Hello 세계!");
    }

    #[test]
    fn test_write_hwp_string_overlong_truncates_instead_of_wrapping() {
        // u16::MAX(65535) 초과 길이 문자열을 그대로 쓰면 `as u16` 캐스팅이 wraparound 되어
        // 기록된 길이 필드와 실제 바이트열이 어긋난 손상 레코드가 생긴다.
        let s: String = "A".repeat(u16::MAX as usize + 10);
        let mut w = ByteWriter::new();
        w.write_hwp_string(&s).unwrap();
        let bytes = w.into_bytes();

        let mut reader = ByteReader::new(&bytes);
        let read_back = reader.read_hwp_string().unwrap();
        assert_eq!(read_back.encode_utf16().count(), u16::MAX as usize);
    }

    #[test]
    fn test_write_color_ref() {
        let mut w = ByteWriter::new();
        w.write_color_ref(0x00FF8040).unwrap();
        let bytes = w.into_bytes();

        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_color_ref().unwrap(), 0x00FF8040);
    }

    #[test]
    fn test_write_zeros() {
        let mut w = ByteWriter::new();
        w.write_zeros(5).unwrap();
        assert_eq!(w.into_bytes(), [0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_position() {
        let mut w = ByteWriter::new();
        assert_eq!(w.position(), 0);
        w.write_u16(0x1234).unwrap();
        assert_eq!(w.position(), 2);
        w.write_u32(0).unwrap();
        assert_eq!(w.position(), 6);
    }

    #[test]
    fn test_sequential_writes_roundtrip() {
        let mut w = ByteWriter::new();
        w.write_u8(42).unwrap();
        w.write_u16(1000).unwrap();
        w.write_i32(-500).unwrap();
        let bytes = w.into_bytes();

        let mut reader = ByteReader::new(&bytes);
        assert_eq!(reader.read_u8().unwrap(), 42);
        assert_eq!(reader.read_u16().unwrap(), 1000);
        assert_eq!(reader.read_i32().unwrap(), -500);
        assert!(reader.is_empty());
    }
}
