//! CFB (Compound File Binary) 컨테이너 읽기
//!
//! HWP 파일의 OLE/CFB 컨테이너를 열고 스트림을 추출한다.
//! - FileHeader: 256바이트, 비압축
//! - DocInfo: 레코드 스트림, 압축 가능
//! - BodyText/Section{N}: 레코드 스트림, 압축 가능
//! - ViewText/Section{N}: 배포용 문서 (암호화 + 압축)
//! - BinData/BIN{XXXX}.{ext}: 바이너리 데이터

use std::io::{Cursor, Read};

/// CFB 컨테이너 리더
pub struct CfbReader {
    compound: cfb::CompoundFile<Cursor<Vec<u8>>>,
}

/// CFB 리더 에러
#[derive(Debug)]
pub enum CfbError {
    /// OLE 컨테이너 열기 실패
    OpenError(String),
    /// 스트림 읽기 실패
    StreamError(String),
    /// 스트림을 찾을 수 없음
    StreamNotFound(String),
    /// 압축 해제 실패
    DecompressError(String),
}

impl std::fmt::Display for CfbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CfbError::OpenError(e) => write!(f, "CFB 열기 실패: {}", e),
            CfbError::StreamError(e) => write!(f, "스트림 읽기 실패: {}", e),
            CfbError::StreamNotFound(name) => write!(f, "스트림 없음: {}", name),
            CfbError::DecompressError(e) => write!(f, "압축 해제 실패: {}", e),
        }
    }
}

impl std::error::Error for CfbError {}

impl CfbReader {
    /// 바이트 데이터에서 CFB 컨테이너 열기
    pub fn open(data: &[u8]) -> Result<Self, CfbError> {
        let cursor = Cursor::new(data.to_vec());
        let compound = cfb::CompoundFile::open(cursor)
            .map_err(|e| CfbError::OpenError(e.to_string()))?;

        Ok(CfbReader { compound })
    }

    /// 스트림 존재 여부 확인
    pub fn has_stream(&self, path: &str) -> bool {
        self.compound.is_stream(path)
    }

    /// 스트림 원본 데이터 읽기 (압축 해제 없이)
    pub fn read_stream_raw(&mut self, path: &str) -> Result<Vec<u8>, CfbError> {
        if !self.compound.is_stream(path) {
            return Err(CfbError::StreamNotFound(path.to_string()));
        }

        let mut stream = self.compound
            .open_stream(path)
            .map_err(|e| CfbError::StreamError(format!("{}: {}", path, e)))?;

        let mut data = Vec::new();
        stream
            .read_to_end(&mut data)
            .map_err(|e| CfbError::StreamError(format!("{}: {}", path, e)))?;

        Ok(data)
    }

    /// FileHeader 스트림 읽기 (256바이트, 항상 비압축)
    pub fn read_file_header(&mut self) -> Result<Vec<u8>, CfbError> {
        self.read_stream_raw("/FileHeader")
    }

    /// DocInfo 스트림 읽기 (압축 가능)
    pub fn read_doc_info(&mut self, compressed: bool) -> Result<Vec<u8>, CfbError> {
        let raw = self.read_stream_raw("/DocInfo")?;
        if compressed {
            decompress_stream(&raw)
        } else {
            Ok(raw)
        }
    }

    /// 본문 섹션 스트림 읽기
    ///
    /// 배포용 문서의 경우 ViewText/Section{N}에서 읽는다.
    /// 일반 문서는 BodyText/Section{N}에서 읽는다.
    /// 반환값은 압축 해제된 레코드 데이터.
    ///
    /// 배포용 문서의 경우, 이 함수는 raw 암호화 데이터를 반환한다.
    /// 호출자가 별도로 복호화를 수행해야 한다.
    pub fn read_body_text_section(
        &mut self,
        index: u32,
        compressed: bool,
        distribution: bool,
    ) -> Result<Vec<u8>, CfbError> {
        if distribution {
            // 배포용 문서: ViewText 스트림 (암호화됨 → 호출자가 복호화)
            let viewtext_path = format!("/ViewText/Section{}", index);
            if self.has_stream(&viewtext_path) {
                return self.read_stream_raw(&viewtext_path);
            }
        }

        // 일반 문서: BodyText 스트림
        let bodytext_path = format!("/BodyText/Section{}", index);
        if self.has_stream(&bodytext_path) {
            let raw = self.read_stream_raw(&bodytext_path)?;
            return if compressed {
                decompress_stream(&raw)
            } else {
                Ok(raw)
            };
        }

        // 루트 레벨 Section (구버전 호환)
        let section_path = format!("/Section{}", index);
        if self.has_stream(&section_path) {
            let raw = self.read_stream_raw(&section_path)?;
            return if compressed {
                decompress_stream(&raw)
            } else {
                Ok(raw)
            };
        }

        Err(CfbError::StreamNotFound(format!("Section{}", index)))
    }

    /// BinData 스트림 읽기 (BinData/BIN{XXXX}.{ext})
    pub fn read_bin_data(&mut self, storage_name: &str) -> Result<Vec<u8>, CfbError> {
        let path = format!("/BinData/{}", storage_name);
        self.read_stream_raw(&path)
    }

    /// 본문 섹션 수 계산
    pub fn section_count(&self) -> u32 {
        let mut count = 0;
        loop {
            let has_body = self.compound.is_stream(&format!("/BodyText/Section{}", count));
            let has_view = self.compound.is_stream(&format!("/ViewText/Section{}", count));
            let has_root = self.compound.is_stream(&format!("/Section{}", count));

            if has_body || has_view || has_root {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// BinData 스토리지의 스트림 이름 목록
    pub fn list_bin_data(&self) -> Vec<String> {
        let mut names = Vec::new();
        // cfb 크레이트의 walk API로 BinData 하위 항목 탐색
        for entry in self.compound.walk() {
            let path = entry.path().to_string_lossy().replace('\\', "/");
            if path.starts_with("/BinData/") && entry.is_stream() {
                if let Some(name) = path.strip_prefix("/BinData/") {
                    names.push(name.to_string());
                }
            }
        }
        names
    }

    /// 모든 스트림 경로 목록
    pub fn list_streams(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for entry in self.compound.walk() {
            if entry.is_stream() {
                paths.push(entry.path().to_string_lossy().replace('\\', "/"));
            }
        }
        paths
    }

    /// 모든 엔트리(스트림 + 스토리지) 경로와 크기 목록
    pub fn list_all_entries(&self) -> Vec<(String, u64, bool)> {
        let mut entries = Vec::new();
        for entry in self.compound.walk() {
            let path = entry.path().to_string_lossy().replace('\\', "/");
            let size = entry.len();
            let is_stream = entry.is_stream();
            entries.push((path, size, is_stream));
        }
        entries
    }

    /// 미리보기 이미지 스트림 읽기 (PrvImage)
    ///
    /// BMP 또는 GIF 형식의 썸네일 이미지를 반환한다.
    /// 스트림이 없으면 None을 반환한다.
    pub fn read_preview_image(&mut self) -> Option<Vec<u8>> {
        if self.has_stream("/PrvImage") {
            self.read_stream_raw("/PrvImage").ok()
        } else {
            None
        }
    }

    /// 미리보기 텍스트 스트림 읽기 (PrvText)
    ///
    /// UTF-16LE 인코딩된 미리보기 텍스트를 반환한다.
    /// 스트림이 없으면 None을 반환한다.
    pub fn read_preview_text(&mut self) -> Option<String> {
        if !self.has_stream("/PrvText") {
            return None;
        }

        let data = self.read_stream_raw("/PrvText").ok()?;

        // UTF-16LE 디코딩
        if data.len() < 2 {
            return None;
        }

        let text: String = data
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    let code = u16::from_le_bytes([chunk[0], chunk[1]]);
                    char::from_u32(code as u32)
                } else {
                    None
                }
            })
            .collect();

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }
}

/// Lenient CFB 리더 (FAT 검증 무시)
///
/// HWP 프로그램이 생성하는 일부 CFB 파일은 표준 cfb 크레이트의
/// FAT 검증("sector 0 pointed to twice")을 통과하지 못한다.
/// 이 구현체는 FAT 중복을 무시하고 스트림을 추출한다.
pub struct LenientCfbReader {
    data: Vec<u8>,
    sector_size: usize,
    /// Directory entries: (name, start_sector, size, obj_type)
    entries: Vec<(String, u32, u64, u8)>,
    /// FAT table
    fat: Vec<u32>,
    /// Mini-stream data
    mini_stream: Vec<u8>,
    /// Mini-FAT table
    mini_fat: Vec<u32>,
    /// Mini-stream cutoff size
    mini_stream_cutoff: u32,
}

impl LenientCfbReader {
    const END_OF_CHAIN: u32 = 0xFFFFFFFE;
    const FREE_SECT: u32 = 0xFFFFFFFF;

    pub fn open(data: &[u8]) -> Result<Self, CfbError> {
        if data.len() < 512 {
            return Err(CfbError::OpenError("파일이 너무 작음".into()));
        }
        // Magic number 확인
        if &data[0..8] != b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1" {
            return Err(CfbError::OpenError("CFB 매직 넘버 불일치".into()));
        }

        let sector_size_power = u16::from_le_bytes([data[30], data[31]]) as usize;
        let sector_size = 1usize << sector_size_power;
        let mini_sector_size_power = u16::from_le_bytes([data[32], data[33]]) as usize;
        let _mini_sector_size = 1usize << mini_sector_size_power;

        let fat_sectors_count = u32::from_le_bytes([data[44], data[45], data[46], data[47]]) as usize;
        let first_dir_sector = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);
        let mini_stream_cutoff = u32::from_le_bytes([data[56], data[57], data[58], data[59]]);
        let first_mini_fat_sector = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        let mini_fat_sectors_count = u32::from_le_bytes([data[64], data[65], data[66], data[67]]) as usize;
        let first_difat_sector = u32::from_le_bytes([data[68], data[69], data[70], data[71]]);
        let difat_sectors_count = u32::from_le_bytes([data[72], data[73], data[74], data[75]]) as usize;

        // DIFAT 읽기: 헤더의 109개 + 추가 DIFAT 섹터
        let mut fat_sector_ids = Vec::new();
        for i in 0..109.min(fat_sectors_count) {
            let off = 76 + i * 4;
            let sid = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
            if sid != Self::FREE_SECT && sid != Self::END_OF_CHAIN {
                fat_sector_ids.push(sid);
            }
        }
        // 추가 DIFAT 섹터 체인
        if difat_sectors_count > 0 && first_difat_sector != Self::END_OF_CHAIN {
            let mut dsid = first_difat_sector;
            for _ in 0..difat_sectors_count {
                let off = 512 + dsid as usize * sector_size;
                if off + sector_size > data.len() { break; }
                let entries_per = sector_size / 4 - 1;
                for i in 0..entries_per {
                    let eoff = off + i * 4;
                    let sid = u32::from_le_bytes([data[eoff], data[eoff + 1], data[eoff + 2], data[eoff + 3]]);
                    if sid != Self::FREE_SECT && sid != Self::END_OF_CHAIN {
                        fat_sector_ids.push(sid);
                    }
                }
                // 다음 DIFAT 섹터
                let next_off = off + entries_per * 4;
                dsid = u32::from_le_bytes([data[next_off], data[next_off + 1], data[next_off + 2], data[next_off + 3]]);
                if dsid == Self::END_OF_CHAIN || dsid == Self::FREE_SECT { break; }
            }
        }

        // FAT 빌드
        let mut fat = Vec::new();
        for &fsid in &fat_sector_ids {
            let off = 512 + fsid as usize * sector_size;
            if off + sector_size > data.len() { continue; }
            let entries = sector_size / 4;
            for i in 0..entries {
                let eoff = off + i * 4;
                fat.push(u32::from_le_bytes([data[eoff], data[eoff + 1], data[eoff + 2], data[eoff + 3]]));
            }
        }

        // Directory entries 읽기
        let dir_data = Self::read_chain_static(data, &fat, first_dir_sector, sector_size);
        let mut entries = Vec::new();
        let entry_size = 128;
        let n_entries = dir_data.len() / entry_size;
        for i in 0..n_entries {
            let eoff = i * entry_size;
            let name_len = u16::from_le_bytes([dir_data[eoff + 64], dir_data[eoff + 65]]) as usize;
            let name = if name_len > 2 {
                let name_bytes = &dir_data[eoff..eoff + name_len - 2]; // UTF-16LE, exclude null
                String::from_utf16_lossy(
                    &name_bytes.chunks(2)
                        .map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
                        .collect::<Vec<_>>()
                )
            } else {
                String::new()
            };
            let obj_type = dir_data[eoff + 66];
            let start_sector = u32::from_le_bytes([
                dir_data[eoff + 116], dir_data[eoff + 117],
                dir_data[eoff + 118], dir_data[eoff + 119],
            ]);
            let size = u64::from_le_bytes([
                dir_data[eoff + 120], dir_data[eoff + 121],
                dir_data[eoff + 122], dir_data[eoff + 123],
                dir_data[eoff + 124], dir_data[eoff + 125],
                dir_data[eoff + 126], dir_data[eoff + 127],
            ]);

            if obj_type == 1 || obj_type == 2 || obj_type == 5 {
                entries.push((name, start_sector, size, obj_type));
            }
        }

        // Mini-FAT 빌드
        let mut mini_fat = Vec::new();
        if mini_fat_sectors_count > 0 && first_mini_fat_sector != Self::END_OF_CHAIN {
            let mfat_data = Self::read_chain_static(data, &fat, first_mini_fat_sector, sector_size);
            for chunk in mfat_data.chunks(4) {
                if chunk.len() == 4 {
                    mini_fat.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
                }
            }
        }

        // Mini-stream: Root entry의 스트림 데이터
        let mini_stream = if !entries.is_empty() && entries[0].3 == 5 {
            Self::read_chain_static(data, &fat, entries[0].1, sector_size)
        } else {
            Vec::new()
        };

        Ok(LenientCfbReader {
            data: data.to_vec(),
            sector_size,
            entries,
            fat,
            mini_stream,
            mini_fat,
            mini_stream_cutoff,
        })
    }

    fn read_chain_static(data: &[u8], fat: &[u32], start: u32, sector_size: usize) -> Vec<u8> {
        let mut result = Vec::new();
        let mut sid = start;
        let mut visited = std::collections::HashSet::new();
        while sid != Self::END_OF_CHAIN && sid != Self::FREE_SECT {
            if !visited.insert(sid) { break; } // 순환 방지
            let off = 512 + sid as usize * sector_size;
            if off + sector_size > data.len() { break; }
            result.extend_from_slice(&data[off..off + sector_size]);
            if (sid as usize) < fat.len() {
                sid = fat[sid as usize];
            } else {
                break;
            }
        }
        result
    }

    fn read_mini_stream(&self, start: u32, size: u64) -> Vec<u8> {
        let mini_sector_size = 64usize;
        let mut result = Vec::new();
        let mut sid = start;
        let mut visited = std::collections::HashSet::new();
        while sid != Self::END_OF_CHAIN && sid != Self::FREE_SECT {
            if !visited.insert(sid) { break; }
            let off = sid as usize * mini_sector_size;
            if off + mini_sector_size > self.mini_stream.len() { break; }
            result.extend_from_slice(&self.mini_stream[off..off + mini_sector_size]);
            if (sid as usize) < self.mini_fat.len() {
                sid = self.mini_fat[sid as usize];
            } else {
                break;
            }
        }
        result.truncate(size as usize);
        result
    }

    /// 디렉토리 경로로 스트림 내용을 가져온다.
    /// 경로 형식: "FileHeader", "DocInfo", "BodyText/Section0" 등
    fn find_entry_idx(&self, path: &str) -> Option<usize> {
        // 경로 "/" 제거 및 트리 탐색 단순화: 이름으로 검색
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if parts.is_empty() { return None; }

        // 간단한 DFS - directory entries가 Red-Black 트리이므로
        // child_id/sibling을 써야 하지만, 이름 기반 단순 매칭으로 충분
        // (HWP 파일은 스트림이 많지 않음)
        let target_name = if parts.len() == 1 {
            parts[0].to_string()
        } else {
            // 마지막 세그먼트를 이름으로 사용
            parts.last().unwrap().to_string()
        };

        // 정확한 경로 매칭이 필요하면 트리 탐색해야 하지만,
        // HWP에서는 이름이 유일하므로 단순 매칭
        self.entries.iter().position(|(name, _, _, _)| name == &target_name)
    }

    pub fn read_stream(&self, path: &str) -> Result<Vec<u8>, CfbError> {
        let idx = self.find_entry_idx(path)
            .ok_or_else(|| CfbError::StreamNotFound(path.to_string()))?;
        let (_, start, size, obj_type) = &self.entries[idx];
        if *obj_type != 2 {
            return Err(CfbError::StreamError(format!("{}: 스트림이 아님 (type={})", path, obj_type)));
        }

        if *size < self.mini_stream_cutoff as u64 {
            Ok(self.read_mini_stream(*start, *size))
        } else {
            let mut data = Self::read_chain_static(&self.data, &self.fat, *start, self.sector_size);
            data.truncate(*size as usize);
            Ok(data)
        }
    }

    pub fn has_stream(&self, path: &str) -> bool {
        self.find_entry_idx(path).is_some()
    }

    pub fn read_doc_info(&self, compressed: bool) -> Result<Vec<u8>, CfbError> {
        let raw = self.read_stream("DocInfo")?;
        if compressed {
            decompress_stream(&raw)
        } else {
            Ok(raw)
        }
    }

    pub fn read_body_text_section(&self, index: u32, compressed: bool) -> Result<Vec<u8>, CfbError> {
        let name = format!("Section{}", index);
        let raw = self.read_stream(&name)?;
        if compressed {
            decompress_stream(&raw)
        } else {
            Ok(raw)
        }
    }

    pub fn list_entries(&self) -> &[(String, u32, u64, u8)] {
        &self.entries
    }

    /// FileHeader 스트림 읽기 (256바이트, 항상 비압축)
    pub fn read_file_header(&self) -> Result<Vec<u8>, CfbError> {
        self.read_stream("FileHeader")
    }

    /// 본문 섹션 스트림 읽기 (배포용 ViewText 지원)
    pub fn read_body_text_section_full(
        &self,
        index: u32,
        compressed: bool,
        distribution: bool,
    ) -> Result<Vec<u8>, CfbError> {
        if distribution {
            let viewtext_name = format!("Section{}", index);
            // ViewText 하위 스트림 탐색
            if self.has_stream(&viewtext_name) {
                return self.read_stream(&viewtext_name);
            }
        }

        let name = format!("Section{}", index);
        let raw = self.read_stream(&name)?;
        if compressed {
            decompress_stream(&raw)
        } else {
            Ok(raw)
        }
    }

    /// 본문 섹션 수 계산
    pub fn section_count(&self) -> u32 {
        let mut count = 0u32;
        loop {
            let name = format!("Section{}", count);
            if self.entries.iter().any(|(n, _, _, _)| n == &name) {
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}

/// zlib/deflate 압축 해제
///
/// HWP는 raw deflate (wbits=-15) 사용. 실패 시 표준 zlib도 시도.
pub fn decompress_stream(data: &[u8]) -> Result<Vec<u8>, CfbError> {
    // raw deflate (wbits=-15) 시도
    use flate2::read::DeflateDecoder;
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => return Ok(decompressed),
        Err(_) => {}
    }

    // 표준 zlib 시도
    use flate2::read::ZlibDecoder;
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(decompressed),
        Err(e) => Err(CfbError::DecompressError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_empty() {
        // 빈 deflate 스트림 (0x03, 0x00 = final empty block)
        let compressed = [0x03, 0x00];
        let result = decompress_stream(&compressed);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_decompress_invalid_data() {
        let result = decompress_stream(&[0xFF, 0xFF, 0xFF]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_real_data() {
        // flate2로 직접 압축한 데이터 검증
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        let original = b"Hello, HWP World! This is a test string for compression.";
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decompressed = decompress_stream(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_compare_cfb_streams_cli_vs_browser() {
        use std::collections::BTreeMap;

        let cli_path = "samples/20250130-hongbo_saved.hwp";
        let browser_path = "samples/honbo-save.hwp";

        let cli_data = std::fs::read(cli_path)
            .unwrap_or_else(|e| panic!("CLI 파일 읽기 실패: {} - {}", cli_path, e));
        let browser_data = std::fs::read(browser_path)
            .unwrap_or_else(|e| panic!("Browser 파일 읽기 실패: {} - {}", browser_path, e));

        println!("\n{}", "=".repeat(60));
        println!("  CFB Stream Structure Comparison");
        println!("  CLI-saved (works):   {} ({} bytes)", cli_path, cli_data.len());
        println!("  Browser-saved (bad): {} ({} bytes)", browser_path, browser_data.len());
        println!("{}\n", "=".repeat(60));

        let cli_cfb = CfbReader::open(&cli_data).expect("CLI CFB 열기 실패");
        let browser_cfb = CfbReader::open(&browser_data).expect("Browser CFB 열기 실패");

        // Collect all entries with sizes
        let cli_entries: BTreeMap<String, (u64, bool)> = cli_cfb
            .list_all_entries()
            .into_iter()
            .map(|(path, size, is_stream)| (path, (size, is_stream)))
            .collect();

        let browser_entries: BTreeMap<String, (u64, bool)> = browser_cfb
            .list_all_entries()
            .into_iter()
            .map(|(path, size, is_stream)| (path, (size, is_stream)))
            .collect();

        // Print CLI streams
        println!("--- CLI-saved streams (works in Hancom) ---");
        println!("{:<45} {:>10} {:>10}", "Path", "Size", "Type");
        println!("{:-<67}", "");
        for (path, (size, is_stream)) in &cli_entries {
            let type_str = if *is_stream { "stream" } else { "storage" };
            println!("{:<45} {:>10} {:>10}", path, size, type_str);
        }

        println!("\n--- Browser-saved streams (corrupted in Hancom) ---");
        println!("{:<45} {:>10} {:>10}", "Path", "Size", "Type");
        println!("{:-<67}", "");
        for (path, (size, is_stream)) in &browser_entries {
            let type_str = if *is_stream { "stream" } else { "storage" };
            println!("{:<45} {:>10} {:>10}", path, size, type_str);
        }

        // Compare: find differences
        println!("\n--- DIFFERENCES ---");
        println!();

        // Streams only in CLI
        let cli_only: Vec<_> = cli_entries.keys()
            .filter(|k| !browser_entries.contains_key(*k))
            .collect();
        if !cli_only.is_empty() {
            println!("Streams ONLY in CLI-saved (missing from browser-saved):");
            for path in &cli_only {
                let (size, _) = cli_entries[*path];
                println!("  [MISSING] {:<40} (size: {})", path, size);
            }
            println!();
        }

        // Streams only in Browser
        let browser_only: Vec<_> = browser_entries.keys()
            .filter(|k| !cli_entries.contains_key(*k))
            .collect();
        if !browser_only.is_empty() {
            println!("Streams ONLY in Browser-saved (extra, not in CLI-saved):");
            for path in &browser_only {
                let (size, _) = browser_entries[*path];
                println!("  [EXTRA]   {:<40} (size: {})", path, size);
            }
            println!();
        }

        // Streams with different sizes
        println!("Streams with SIZE differences:");
        let mut size_diffs = 0;
        for (path, (cli_size, cli_is_stream)) in &cli_entries {
            if let Some((browser_size, _)) = browser_entries.get(path) {
                if cli_size != browser_size && *cli_is_stream {
                    let diff = *browser_size as i64 - *cli_size as i64;
                    let sign = if diff > 0 { "+" } else { "" };
                    println!("  {:<40} CLI: {:>8}  Browser: {:>8}  ({}{})",
                        path, cli_size, browser_size, sign, diff);
                    size_diffs += 1;
                }
            }
        }
        if size_diffs == 0 {
            println!("  (none)");
        }

        // Specifically analyze BinData streams
        println!("\n--- BinData Stream Analysis ---");
        println!();
        let cli_bins: Vec<_> = cli_entries.keys()
            .filter(|k| k.starts_with("/BinData/"))
            .collect();
        let browser_bins: Vec<_> = browser_entries.keys()
            .filter(|k| k.starts_with("/BinData/"))
            .collect();

        println!("CLI-saved BinData streams ({} total):", cli_bins.len());
        for path in &cli_bins {
            let (size, _) = cli_entries[*path];
            println!("  {:<45} size: {}", path, size);
        }

        println!("\nBrowser-saved BinData streams ({} total):", browser_bins.len());
        for path in &browser_bins {
            let (size, _) = browser_entries[*path];
            println!("  {:<45} size: {}", path, size);
        }

        // Check naming pattern
        println!("\nBinData naming pattern check:");
        for path in cli_bins.iter().chain(browser_bins.iter()) {
            if let Some(name) = path.strip_prefix("/BinData/") {
                if name.starts_with("BIN") {
                    // Extract the numeric part
                    let num_part: String = name.chars()
                        .skip(3)
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    let ext_part: String = name.chars()
                        .skip(3 + num_part.len())
                        .collect();
                    let source = if cli_entries.contains_key(*path) && browser_entries.contains_key(*path) {
                        "BOTH"
                    } else if cli_entries.contains_key(*path) {
                        "CLI-ONLY"
                    } else {
                        "BROWSER-ONLY"
                    };
                    println!("  {} => prefix=BIN, num='{}' (digits={}), ext='{}' [{}]",
                        name, num_part, num_part.len(), ext_part, source);
                }
            }
        }

        // Summary
        println!("\n--- SUMMARY ---");
        println!("CLI-saved:     {} total entries ({} streams)",
            cli_entries.len(),
            cli_entries.values().filter(|(_, is)| *is).count());
        println!("Browser-saved: {} total entries ({} streams)",
            browser_entries.len(),
            browser_entries.values().filter(|(_, is)| *is).count());
        println!("Only in CLI:     {}", cli_only.len());
        println!("Only in Browser: {}", browser_only.len());
        println!("Size differences: {}", size_diffs);

        // Helper functions for record parsing
        fn parse_records(data: &[u8]) -> Vec<(u32, u32, u32, usize)> {
            // Returns: (tag_id, level, size, offset)
            let mut records = Vec::new();
            let mut pos = 0;
            while pos + 4 <= data.len() {
                let header = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                let tag_id = header & 0x3FF;
                let level = (header >> 10) & 0x3FF;
                let size_field = (header >> 20) & 0xFFF;
                let mut size = size_field as u32;
                let mut data_offset = pos + 4;
                if size_field == 0xFFF {
                    if pos + 8 > data.len() { break; }
                    size = u32::from_le_bytes([data[pos+4], data[pos+5], data[pos+6], data[pos+7]]);
                    data_offset = pos + 8;
                }
                records.push((tag_id, level, size, pos));
                pos = data_offset + size as usize;
            }
            records
        }

        fn tag_name(tag_id: u32) -> &'static str {
            match tag_id {
                66 => "PARA_HEADER",
                67 => "PARA_TEXT",
                68 => "PARA_CHAR_SHAPE",
                69 => "PARA_LINE_SEG",
                70 => "PARA_RANGE_TAG",
                71 => "CTRL_HEADER",
                72 => "LIST_HEADER",
                73 => "PAGE_DEF",
                74 => "FOOTNOTE_SHAPE",
                75 => "PAGE_BORDER_FILL",
                76 => "SHAPE_COMPONENT",
                77 => "TABLE",
                78 => "SHAPE_LINE",
                79 => "SHAPE_RECT",
                80 => "SHAPE_ELLIPSE",
                81 => "SHAPE_ARC",
                82 => "SHAPE_POLYGON",
                83 => "SHAPE_CURVE",
                84 => "SHAPE_OLE",
                85 => "SHAPE_PICTURE",
                86 => "SHAPE_CONTAINER",
                87 => "CTRL_DATA",
                88 => "EQEDIT",
                99 => "SHAPE_TEXTART",
                102 => "FORM_OBJECT",
                103 => "MEMO_SHAPE",
                104 => "MEMO_LIST",
                _ => "UNKNOWN",
            }
        }

        // Compare BodyText/Section0 records
        println!("\n--- BodyText/Section0 Record-level Comparison ---");

        let mut cli_cfb3 = CfbReader::open(&cli_data).unwrap();
        let mut browser_cfb3 = CfbReader::open(&browser_data).unwrap();
        let cli_section = cli_cfb3.read_body_text_section(0, true, false).unwrap();
        let browser_section = browser_cfb3.read_body_text_section(0, true, false).unwrap();

        println!("CLI Section0 decompressed size:     {} bytes", cli_section.len());
        println!("Browser Section0 decompressed size: {} bytes", browser_section.len());

        let cli_records = parse_records(&cli_section);
        let browser_records = parse_records(&browser_section);

        println!("\nCLI records: {} total", cli_records.len());
        println!("Browser records: {} total", browser_records.len());

        // Print all records side by side (only show differences and nearby context)
        let max_records = std::cmp::max(cli_records.len(), browser_records.len());
        println!("\n{:<5} {:<30} {:>4} {:>6}  |  {:<30} {:>4} {:>6}  | Match",
            "#", "CLI Tag", "Lvl", "Size", "Browser Tag", "Lvl", "Size");
        println!("{:-<120}", "");

        for i in 0..max_records {
            let cli_str = if i < cli_records.len() {
                let (tag, lvl, sz, _) = cli_records[i];
                format!("{:<30} {:>4} {:>6}", format!("{} ({})", tag_name(tag), tag), lvl, sz)
            } else {
                format!("{:<30} {:>4} {:>6}", "---", "", "")
            };

            let browser_str = if i < browser_records.len() {
                let (tag, lvl, sz, _) = browser_records[i];
                format!("{:<30} {:>4} {:>6}", format!("{} ({})", tag_name(tag), tag), lvl, sz)
            } else {
                format!("{:<30} {:>4} {:>6}", "---", "", "")
            };

            let match_str = if i < cli_records.len() && i < browser_records.len() {
                let (ct, cl, cs, co) = cli_records[i];
                let (bt, bl, bs, bo) = browser_records[i];
                if ct == bt && cl == bl && cs == bs {
                    let cli_data_start = if cs == 0xFFF { co + 8 } else { co + 4 };
                    let browser_data_start = if bs == 0xFFF { bo + 8 } else { bo + 4 };
                    let cli_slice = &cli_section[cli_data_start..cli_data_start + cs as usize];
                    let browser_slice = &browser_section[browser_data_start..browser_data_start + bs as usize];
                    if cli_slice == browser_slice { "OK" } else { "DATA DIFF" }
                } else if ct == bt && cl == bl {
                    "SIZE DIFF"
                } else {
                    "MISMATCH"
                }
            } else {
                "MISSING"
            };

            // Only print non-OK records and some context
            if match_str != "OK" {
                println!("{:<5} {}  |  {}  | {}", i, cli_str, browser_str, match_str);
            }
        }

        // Hex dump of key differing records
        println!("\n--- Hex Dump of Key Differing Records ---");
        for idx in [19usize, 20, 21, 32, 33, 34, 225, 226, 227] {
            if idx >= cli_records.len() && idx >= browser_records.len() { continue; }
            println!("\n=== Record #{} ===", idx);
            if idx < cli_records.len() {
                let (tag, lvl, sz, off) = cli_records[idx];
                let data_start = if sz >= 0xFFF { off + 8 } else { off + 4 };
                let data_end = data_start + sz as usize;
                println!("CLI: tag={} ({}) level={} size={} offset={:#x}",
                    tag_name(tag), tag, lvl, sz, off);
                if data_end <= cli_section.len() {
                    let bytes = &cli_section[data_start..data_end];
                    let show_len = std::cmp::min(bytes.len(), 80);
                    print!("  hex: ");
                    for b in &bytes[..show_len] { print!("{:02x} ", b); }
                    if bytes.len() > show_len { print!("...({} more bytes)", bytes.len() - show_len); }
                    println!();
                }
            }
            if idx < browser_records.len() {
                let (tag, lvl, sz, off) = browser_records[idx];
                let data_start = if sz >= 0xFFF { off + 8 } else { off + 4 };
                let data_end = data_start + sz as usize;
                println!("Browser: tag={} ({}) level={} size={} offset={:#x}",
                    tag_name(tag), tag, lvl, sz, off);
                if data_end <= browser_section.len() {
                    let bytes = &browser_section[data_start..data_end];
                    let show_len = std::cmp::min(bytes.len(), 80);
                    print!("  hex: ");
                    for b in &bytes[..show_len] { print!("{:02x} ", b); }
                    if bytes.len() > show_len { print!("...({} more bytes)", bytes.len() - show_len); }
                    println!();
                }
            }
        }

        // Record count analysis
        println!("\n--- Record Count Analysis ---");
        println!("CLI:     {} records, {} CTRL_HEADER, {} SHAPE_COMPONENT, {} SHAPE_PICTURE, {} CTRL_DATA, {} LIST_HEADER",
            cli_records.len(),
            cli_records.iter().filter(|(t,_,_,_)| *t == 71).count(),
            cli_records.iter().filter(|(t,_,_,_)| *t == 76).count(),
            cli_records.iter().filter(|(t,_,_,_)| *t == 85).count(),
            cli_records.iter().filter(|(t,_,_,_)| *t == 87).count(),
            cli_records.iter().filter(|(t,_,_,_)| *t == 72).count());
        println!("Browser: {} records, {} CTRL_HEADER, {} SHAPE_COMPONENT, {} SHAPE_PICTURE, {} CTRL_DATA, {} LIST_HEADER",
            browser_records.len(),
            browser_records.iter().filter(|(t,_,_,_)| *t == 71).count(),
            browser_records.iter().filter(|(t,_,_,_)| *t == 76).count(),
            browser_records.iter().filter(|(t,_,_,_)| *t == 85).count(),
            browser_records.iter().filter(|(t,_,_,_)| *t == 87).count(),
            browser_records.iter().filter(|(t,_,_,_)| *t == 72).count());

        // CTRL_HEADER sizes
        println!("\n--- All CTRL_HEADER (71) sizes ---");
        let cli_ctrl: Vec<_> = cli_records.iter().enumerate()
            .filter(|(_,(t,_,_,_))| *t == 71).collect();
        let browser_ctrl: Vec<_> = browser_records.iter().enumerate()
            .filter(|(_,(t,_,_,_))| *t == 71).collect();
        println!("CLI:     {:?}", cli_ctrl.iter().map(|(i, (_,l,s,_))| format!("#{}: lvl={} sz={}", i, l, s)).collect::<Vec<_>>());
        println!("Browser: {:?}", browser_ctrl.iter().map(|(i, (_,l,s,_))| format!("#{}: lvl={} sz={}", i, l, s)).collect::<Vec<_>>());

        // SHAPE_COMPONENT sizes
        println!("\n--- All SHAPE_COMPONENT (76) sizes ---");
        let cli_shape: Vec<_> = cli_records.iter().enumerate()
            .filter(|(_,(t,_,_,_))| *t == 76).collect();
        let browser_shape: Vec<_> = browser_records.iter().enumerate()
            .filter(|(_,(t,_,_,_))| *t == 76).collect();
        println!("CLI:     {:?}", cli_shape.iter().map(|(i, (_,l,s,_))| format!("#{}: lvl={} sz={}", i, l, s)).collect::<Vec<_>>());
        println!("Browser: {:?}", browser_shape.iter().map(|(i, (_,l,s,_))| format!("#{}: lvl={} sz={}", i, l, s)).collect::<Vec<_>>());

        // Compare FileHeader bytes
        println!("\n--- FileHeader Comparison ---");
        let mut cli_cfb2 = CfbReader::open(&cli_data).unwrap();
        let mut browser_cfb2 = CfbReader::open(&browser_data).unwrap();

        let cli_header = cli_cfb2.read_file_header().unwrap();
        let browser_header = browser_cfb2.read_file_header().unwrap();

        println!("CLI FileHeader size: {} bytes", cli_header.len());
        println!("Browser FileHeader size: {} bytes", browser_header.len());

        if cli_header == browser_header {
            println!("FileHeaders are IDENTICAL.");
        } else {
            println!("FileHeaders DIFFER at these byte offsets:");
            let max_len = std::cmp::max(cli_header.len(), browser_header.len());
            let mut diff_count = 0;
            for i in 0..max_len {
                let cli_byte = cli_header.get(i).copied();
                let browser_byte = browser_header.get(i).copied();
                if cli_byte != browser_byte {
                    println!("  offset {:#06x}: CLI={:?} Browser={:?}",
                        i, cli_byte.map(|b| format!("{:#04x}", b)),
                        browser_byte.map(|b| format!("{:#04x}", b)));
                    diff_count += 1;
                    if diff_count > 20 {
                        println!("  ... (truncated, more differences exist)");
                        break;
                    }
                }
            }
        }
    }

}
