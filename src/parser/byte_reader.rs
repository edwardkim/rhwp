//! 바이너리 데이터 읽기 유틸리티
//!
//! HWP 레코드 내부의 바이너리 필드를 순차적으로 읽기 위한 커서 기반 리더.
//! HWP는 리틀 엔디안, UTF-16LE 문자열을 사용한다.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Cursor, Read};

/// 바이트 리더 (커서 기반)
pub struct ByteReader<'a> {
    cursor: Cursor<&'a [u8]>,
    len: usize,
}

impl<'a> ByteReader<'a> {
    /// 새 ByteReader 생성
    pub fn new(data: &'a [u8]) -> Self {
        ByteReader {
            cursor: Cursor::new(data),
            len: data.len(),
        }
    }

    /// 현재 읽기 위치
    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    /// 남은 바이트 수
    pub fn remaining(&self) -> usize {
        self.len.saturating_sub(self.position())
    }

    /// 읽기가 끝났는지 확인
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// u8 읽기
    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.cursor.read_u8()
    }

    /// u16 읽기 (LE)
    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<LittleEndian>()
    }

    /// u32 읽기 (LE)
    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<LittleEndian>()
    }

    /// i8 읽기
    pub fn read_i8(&mut self) -> io::Result<i8> {
        self.cursor.read_i8()
    }

    /// i16 읽기 (LE)
    pub fn read_i16(&mut self) -> io::Result<i16> {
        self.cursor.read_i16::<LittleEndian>()
    }

    /// i32 읽기 (LE)
    pub fn read_i32(&mut self) -> io::Result<i32> {
        self.cursor.read_i32::<LittleEndian>()
    }

    /// i64 읽기 (LE)
    pub fn read_i64(&mut self) -> io::Result<i64> {
        self.cursor.read_i64::<LittleEndian>()
    }

    /// 지정 길이의 바이트 읽기
    pub fn read_bytes(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// 읽기 위치를 직접 설정
    pub fn set_position(&mut self, pos: usize) {
        self.cursor.set_position(pos as u64);
    }

    /// N 바이트 건너뛰기
    pub fn skip(&mut self, n: usize) -> io::Result<()> {
        let pos = self.cursor.position() + n as u64;
        if pos > self.len as u64 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "skip 범위 초과",
            ));
        }
        self.cursor.set_position(pos);
        Ok(())
    }

    /// HWP 문자열 읽기 (2바이트 길이 접두사 + UTF-16LE)
    ///
    /// 형식: [u16 글자수] + [UTF-16LE 바이트 * 글자수]
    pub fn read_hwp_string(&mut self) -> io::Result<String> {
        let char_count = self.read_u16()? as usize;
        if char_count == 0 {
            return Ok(String::new());
        }
        self.read_utf16_string(char_count)
    }

    /// UTF-16LE 문자열 읽기 (지정 글자 수)
    pub fn read_utf16_string(&mut self, char_count: usize) -> io::Result<String> {
        let byte_count = char_count * 2;
        let bytes = self.read_bytes(byte_count)?;

        let utf16: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        String::from_utf16(&utf16).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("UTF-16 디코딩 실패: {}", e),
            )
        })
    }

    /// ColorRef 읽기 (4바이트, 0x00BBGGRR 형식)
    pub fn read_color_ref(&mut self) -> io::Result<u32> {
        self.read_u32()
    }

    /// 나머지 바이트 전부 읽기
    pub fn read_remaining(&mut self) -> io::Result<Vec<u8>> {
        let remaining = self.remaining();
        self.read_bytes(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x42];
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_u8().unwrap(), 0x42);
        assert!(reader.is_empty());
    }

    #[test]
    fn test_read_u16_le() {
        let data = [0x34, 0x12];
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_u16().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x78, 0x56, 0x34, 0x12];
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i16_negative() {
        let data = (-100i16).to_le_bytes();
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_i16().unwrap(), -100);
    }

    #[test]
    fn test_read_i32_negative() {
        let data = (-7200i32).to_le_bytes();
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_i32().unwrap(), -7200);
    }

    #[test]
    fn test_read_hwp_string() {
        // "한글" = U+D55C U+AE00 → UTF-16LE
        let mut data = Vec::new();
        data.extend_from_slice(&2u16.to_le_bytes()); // 글자 수
        data.extend_from_slice(&0xD55Cu16.to_le_bytes()); // '한'
        data.extend_from_slice(&0xAE00u16.to_le_bytes()); // '글'

        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_hwp_string().unwrap(), "한글");
    }

    #[test]
    fn test_read_empty_hwp_string() {
        let data = [0x00, 0x00]; // 글자 수 0
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_hwp_string().unwrap(), "");
    }

    #[test]
    fn test_read_ascii_hwp_string() {
        // "ABC" in UTF-16LE
        let mut data = Vec::new();
        data.extend_from_slice(&3u16.to_le_bytes());
        data.extend_from_slice(&0x0041u16.to_le_bytes()); // 'A'
        data.extend_from_slice(&0x0042u16.to_le_bytes()); // 'B'
        data.extend_from_slice(&0x0043u16.to_le_bytes()); // 'C'

        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_hwp_string().unwrap(), "ABC");
    }

    #[test]
    fn test_read_bytes() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_bytes(3).unwrap(), [0x01, 0x02, 0x03]);
        assert_eq!(reader.remaining(), 2);
    }

    #[test]
    fn test_skip() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);
        reader.skip(2).unwrap();
        assert_eq!(reader.read_u8().unwrap(), 0x03);
    }

    #[test]
    fn test_position_and_remaining() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.position(), 0);
        assert_eq!(reader.remaining(), 4);

        reader.read_u16().unwrap();
        assert_eq!(reader.position(), 2);
        assert_eq!(reader.remaining(), 2);
    }

    #[test]
    fn test_color_ref() {
        // BGR: Blue=0xFF, Green=0x80, Red=0x40 → 0x00FF8040
        let data = [0x40, 0x80, 0xFF, 0x00];
        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_color_ref().unwrap(), 0x00FF8040);
    }

    #[test]
    fn test_sequential_reads() {
        let mut data = Vec::new();
        data.extend_from_slice(&42u8.to_le_bytes());
        data.extend_from_slice(&1000u16.to_le_bytes());
        data.extend_from_slice(&(-500i32).to_le_bytes());

        let mut reader = ByteReader::new(&data);
        assert_eq!(reader.read_u8().unwrap(), 42);
        assert_eq!(reader.read_u16().unwrap(), 1000);
        assert_eq!(reader.read_i32().unwrap(), -500);
        assert!(reader.is_empty());
    }
}
