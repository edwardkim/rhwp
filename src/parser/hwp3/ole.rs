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
/// 입력은 인식 정보(4바이트)를 제외한 CFB 바이트. 실패 개체는 건너뛴다(읽기 관대).
pub fn extract_ole_payloads(cfb_bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let Ok(mut comp) = cfb::CompoundFile::open(Cursor::new(cfb_bytes.to_vec())) else {
        return out;
    };

    // root 직속 서브 스토리지 = OLE 개체 하나 (이름 = 그림 레코드 참조명)
    let root = std::path::Path::new("/");
    let storages: Vec<std::path::PathBuf> = comp
        .walk()
        .filter(|e| e.is_storage() && !e.is_root())
        .filter(|e| e.path().parent() == Some(root))
        .map(|e| e.path().to_path_buf())
        .collect();

    for storage in storages {
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
        if let Ok(bytes) = crate::serializer::mini_cfb::build_cfb(&refs) {
            out.push((name, bytes));
        }
    }
    out
}
