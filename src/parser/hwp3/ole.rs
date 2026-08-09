//! HWP3 OLE 개체 파싱
//!
//! 문서 내에 삽입된 OLE(Object Linking and Embedding) 개체 정보를 파싱한다.
//! 외부 애플리케이션에서 생성된 데이터 구조를 안전하게 읽고 무시하거나 추출할 수 있게 한다.

use byteorder::{LittleEndian, ReadBytesExt};
use snafu::Snafu;
use std::io::{self, Cursor, Read, Seek};

#[derive(Debug, Snafu)]
pub enum Hwp3OleError {
    #[snafu(display("입출력 오류가 발생했습니다: {source}"))]
    IoError { source: io::Error },
    #[snafu(display("알 수 없는 OLE 서명입니다: {signature:#X}"))]
    UnknownSignature { signature: u32 },
}

impl From<io::Error> for Hwp3OleError {
    fn from(error: io::Error) -> Self {
        Hwp3OleError::IoError { source: error }
    }
}

/// OLE 추가 정보 블록 내용
#[derive(Debug)]
pub struct Hwp3OleInfo {
    pub signature: u32,
    pub storage_data: Vec<u8>,
}

impl Hwp3OleInfo {
    pub fn read<R: Read>(mut reader: R, total_length: u32) -> Result<Self, Hwp3OleError> {
        if total_length < 4 {
            return Err(Hwp3OleError::IoError {
                source: io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "OLE Info length is too short",
                ),
            });
        }
        let signature = reader.read_u32::<LittleEndian>()?;

        let mut storage_data = super::alloc_record_buf((total_length - 4) as usize)?;
        reader.read_exact(&mut storage_data)?;

        // 0xF8995567 (한글 3.0 ~ 3.0a - ILockBytes)
        // 0xF8995568 (한글 3.0b 이상 - StgCreateDocfile)
        if signature != 0xF8995567 && signature != 0xF8995568 {
            return Err(Hwp3OleError::UnknownSignature { signature });
        }

        Ok(Hwp3OleInfo {
            signature,
            storage_data,
        })
    }
}

/// 자체 관리 정보 (.inf 스트림에 저장)
#[derive(Debug)]
pub struct Hwp3OleStreamInfo {
    pub width: u32,  // HIMETRIC 단위
    pub height: u32, // HIMETRIC 단위
    pub aspect: u32, // DVASPECT_CONTENT 또는 DVASPECT_ICON
    pub reserved: [u8; 116],
}

impl Default for Hwp3OleStreamInfo {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            aspect: 0,
            reserved: [0; 116],
        }
    }
}

impl Hwp3OleStreamInfo {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let width = reader.read_u32::<LittleEndian>()?;
        let height = reader.read_u32::<LittleEndian>()?;
        let aspect = reader.read_u32::<LittleEndian>()?;
        let mut reserved = [0u8; 116];
        reader.read_exact(&mut reserved)?;

        Ok(Hwp3OleStreamInfo {
            width,
            height,
            aspect,
            reserved,
        })
    }
}

/// 차트 연결 정보 (HWPChart.Info 스트림에 저장)
#[derive(Debug)]
pub struct Hwp3ChartConnectionInfo {
    pub linked: u16, // 비트 0 = 연결 여부, 비트 1-15 = 예약
    pub tblid: u16,  // 표ID
    pub entire: u32, // 비트 0 = 전체 표 여부
    pub startcol: u32,
    pub startrow: u32,
    pub endcol: u32,
    pub endrow: u32,
    pub chsize: u32, // 표 내용 데이터 길이
    pub reserved: [u8; 100],
}

impl Default for Hwp3ChartConnectionInfo {
    fn default() -> Self {
        Self {
            linked: 0,
            tblid: 0,
            entire: 0,
            startcol: 0,
            startrow: 0,
            endcol: 0,
            endrow: 0,
            chsize: 0,
            reserved: [0; 100],
        }
    }
}

impl Hwp3ChartConnectionInfo {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let linked = reader.read_u16::<LittleEndian>()?;
        let tblid = reader.read_u16::<LittleEndian>()?;
        let entire = reader.read_u32::<LittleEndian>()?;
        let startcol = reader.read_u32::<LittleEndian>()?;
        let startrow = reader.read_u32::<LittleEndian>()?;
        let endcol = reader.read_u32::<LittleEndian>()?;
        let endrow = reader.read_u32::<LittleEndian>()?;
        let chsize = reader.read_u32::<LittleEndian>()?;
        let mut reserved = [0u8; 100];
        reader.read_exact(&mut reserved)?;

        Ok(Hwp3ChartConnectionInfo {
            linked,
            tblid,
            entire,
            startcol,
            startrow,
            endcol,
            endrow,
            chsize,
            reserved,
        })
    }
}

/// [#3363] 추가 정보 블록 id=2(OLE 정보, 스펙 표 82)의 스토리지에서 개체별 payload를
/// 분해한다.
///
/// 스펙 12.1절: 모든 OLE 개체는 하나의 CFB 스토리지에 모여 저장되고, 그림 코드에는
/// 이름(= root 서브 스토리지명, 예: `00000000.OOO`)만 존재한다. 한컴의 HWPX 변환은
/// 개체 서브 스토리지를 root로 승격한 standalone CFB를 `BinData/*.ole`로 방출한다
/// (SO-SUEOP 실측: 스트림 md5 완전 동일). 여기서도 같은 재포장을 수행해 HWPX와 동일한
/// 소비 경로(`parse_ole_container`)에 태운다.
///
/// 재포장에는 `cfb::CompoundFile::create()` 대신 자체 `mini_cfb` 빌더를 쓴다 —
/// create()는 `SystemTime::now()`를 호출해 wasm32 타겟에서 panic한다.
///
/// 승격할 때 서브 스토리지의 **CLSID 도 새 루트로 옮긴다** — OLE 개체는 그 값으로 서버를
/// 식별하므로, 비우면 한컴이 개체를 알아보지 못해 내용을 비워 그린다 (#4097).
///
/// 입력은 인식 정보(4바이트)를 제외한 CFB 바이트. 실패 개체는 건너뛴다(읽기 관대).
pub fn extract_ole_payloads(cfb_bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let Ok(mut comp) = cfb::CompoundFile::open(Cursor::new(cfb_bytes.to_vec())) else {
        return out;
    };

    // root 직속 서브 스토리지 = OLE 개체 하나 (이름 = 그림 레코드 참조명)
    let root = std::path::Path::new("/");
    let storages: Vec<(std::path::PathBuf, [u8; 16])> = comp
        .walk()
        .filter(|e| e.is_storage() && !e.is_root())
        .filter(|e| e.path().parent() == Some(root))
        // [#4097] 서브 스토리지의 CLSID 를 함께 들고 간다 — 승격 후 새 루트에 실어야 한다.
        // `cfb` 는 CLSID 를 (LE u32, LE u16, LE u16, 8B) 로 읽어 `Uuid::from_fields` 하므로
        // (`cfb/internal/direntry.rs:88`), `to_bytes_le()` 가 파일 원시 16바이트를 복원한다.
        .map(|e| (e.path().to_path_buf(), e.clsid().to_bytes_le()))
        .collect();

    for (storage, storage_clsid) in storages {
        let Some(name) = storage
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .filter(|n| !n.is_empty())
        else {
            continue;
        };

        // 개체 스토리지 직속 스트림 수집 (실존 OLE 개체 스토리지는 평탄 구조)
        let stream_paths: Vec<(String, std::path::PathBuf)> = comp
            .walk()
            .filter(|e| e.is_stream())
            .filter(|e| e.path().parent() == Some(storage.as_path()))
            .filter_map(|e| {
                e.path()
                    .file_name()
                    .map(|n| (n.to_string_lossy().to_string(), e.path().to_path_buf()))
            })
            .collect();

        let mut named: Vec<(String, Vec<u8>)> = Vec::new();
        for (stream_name, path) in stream_paths {
            let Ok(mut stream) = comp.open_stream(&path) else {
                continue;
            };
            let mut buf = Vec::new();
            if stream.read_to_end(&mut buf).is_ok() {
                named.push((stream_name, buf));
            }
        }
        if named.is_empty() {
            continue;
        }

        let refs: Vec<(&str, &[u8])> = named
            .iter()
            .map(|(n, d)| (n.as_str(), d.as_slice()))
            .collect();
        // [#4097] 승격 재포장은 반드시 CLSID 를 실어야 한다. 비우면 한컴이 개체를 알아보지
        // 못해 틀과 선택 핸들만 그리고 내용을 비운다.
        if let Ok(bytes) =
            crate::serializer::mini_cfb::build_cfb_with_root_clsid(&refs, storage_clsid)
        {
            out.push((name, bytes));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// mini_cfb 출력에서 이름으로 디렉터리 엔트리를 찾아 CLSID(+80)를 스탬프한다.
    ///
    /// `cfb` 크레이트의 `set_storage_clsid` 를 쓰지 않는 이유는 `Uuid` 타입 이름이 필요해
    /// `uuid` 를 dev-dependency 로 끌어들이기 때문이다. mini_cfb 는 sector_shift=9,
    /// first dir sector=0 고정이라 엔트리가 파일 오프셋 512 부터 128바이트씩 이어진다.
    fn stamp_named_entry_clsid(cfb: &mut [u8], target: &str, clsid: [u8; 16]) {
        let mut at = 512;
        while at + 128 <= cfb.len() {
            let name_len = u16::from_le_bytes([cfb[at + 64], cfb[at + 65]]) as usize;
            if name_len >= 2 && name_len <= 64 {
                let units: Vec<u16> = cfb[at..at + name_len - 2]
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                if String::from_utf16_lossy(&units) == target {
                    cfb[at + 80..at + 96].copy_from_slice(&clsid);
                    return;
                }
            }
            at += 128;
        }
        panic!("디렉터리 엔트리 '{target}' 를 찾지 못했다");
    }

    /// [#4097] 서브 스토리지를 루트로 승격할 때 CLSID 도 함께 옮겨야 한다.
    ///
    /// SO-SUEOP 실측값은 `{00044214-0000-0000-C000-000000000046}` 이지만(글맵시 서버 클래스,
    /// `mydocs/working/task_m100_4097_stage1.md` §2.2), 여기서는 특정 GUID 를 하드코딩하지 않고
    /// **배선 자체**를 증명한다 — 원본 서브 스토리지에 있던 값이 출력 루트에 나타나는가.
    #[test]
    fn task4097_promoted_sub_storage_clsid_becomes_the_new_root_clsid() {
        const CLSID: [u8; 16] = [
            0x14, 0x42, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x46,
        ];
        let mut src = crate::serializer::mini_cfb::build_cfb(&[
            ("/00000000.OOO/Contents", &[1u8; 64][..]),
            ("/00000000.OOO/\u{2}OlePres000", &[2u8; 64][..]),
        ])
        .expect("합성 CFB");
        stamp_named_entry_clsid(&mut src, "00000000.OOO", CLSID);

        let out = extract_ole_payloads(&src);
        assert_eq!(out.len(), 1, "root 직속 서브 스토리지 1건이 승격되어야 함");
        assert_eq!(out[0].0, "00000000.OOO");
        assert_eq!(
            crate::parser::cfb_reader::root_clsid(&out[0].1),
            Some(CLSID),
            "승격된 서브 스토리지의 CLSID 가 새 루트로 옮겨져야 한다 (#4097)"
        );

        // 스트림 내용도 그대로 넘어가는지 — 재포장이 CLSID 만 보고 나머지를 흘리면 안 된다.
        let container = crate::parser::ole_container::parse_ole_container(&out[0].1)
            .expect("재포장본을 parse_ole_container 가 열 수 있어야 함");
        assert!(container.raw_contents.is_some(), "Contents 유지");
    }

    /// CLSID 가 없는(0) 서브 스토리지는 0 인 채로 승격된다 — 없는 값을 지어내지 않는다.
    #[test]
    fn task4097_zero_sub_storage_clsid_stays_zero() {
        let src = crate::serializer::mini_cfb::build_cfb(&[("/OBJ/Contents", &[3u8; 32][..])])
            .expect("합성 CFB");
        let out = extract_ole_payloads(&src);
        assert_eq!(out.len(), 1);
        assert_eq!(
            crate::parser::cfb_reader::root_clsid(&out[0].1),
            Some([0u8; 16])
        );
    }
}
