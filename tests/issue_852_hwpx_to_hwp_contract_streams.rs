//! Issue #852: HWPX → HWP 변환 시 한컴 호환 contract 검증.
//!
//! 회귀 가드 — Stage 2.1+2.2+2.4+2.5 에서 해소한 다음 결함이 재발하지 않음을 보장:
//! 1. CFB 9 스트림 contract (Stage 2.1+2.2)
//! 2. Form 컨트롤 BodyText/Section0 직렬화 (CTRL_HEADER 46B + HWPTAG_FORM_OBJECT, Stage 2.4)
//! 3. JavaScript Scripts/DefaultJScript (headerScripts + sourceScripts 결합, Stage 2.5)
//! 4. ClickHere field 의 CTRL_DATA (0x57) 자식 레코드 (Stage 2.5)
//!
//! 정답지: `samples/form-01.hwp` / `samples/form-02.hwp` (한컴 5/5 손상 미판정 확인).

use cfb::CompoundFile;
use flate2::read::DeflateDecoder;
use std::io::{Cursor, Read};
use std::path::Path;

use rhwp::parse_document;
use rhwp::serializer::serialize_hwp;

fn convert(hwpx_name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwpx")
        .join(hwpx_name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = parse_document(&bytes).unwrap_or_else(|e| panic!("parse {hwpx_name}: {e:?}"));
    serialize_hwp(&doc).unwrap_or_else(|e| panic!("serialize {hwpx_name}: {e:?}"))
}

fn read_stream(hwp: &[u8], path: &str) -> Vec<u8> {
    let mut cfb = CompoundFile::open(Cursor::new(hwp)).unwrap_or_else(|e| panic!("open CFB: {e}"));
    let mut stream = cfb
        .open_stream(path)
        .unwrap_or_else(|e| panic!("open stream {path}: {e}"));
    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .unwrap_or_else(|e| panic!("read stream {path}: {e}"));
    buf
}

/// HWP5 BodyText/Section0 압축 여부는 FileHeader flag 로 결정. 변환 결과는
/// HWPX parser 가 `doc.header.compressed = false` 작성 → uncompressed.
/// 그러나 한컴 호환을 위해 양쪽 모두 처리.
fn maybe_decompress(stream: &[u8]) -> Vec<u8> {
    // HWP5 raw record 의 첫 바이트 패턴: tag (10 bit) 가 0x42 (PARA_HEADER) 면 uncompressed.
    // deflate stream 의 첫 바이트는 보통 0x78 (zlib) 또는 임의 (raw deflate).
    // 단순 휴리스틱: 첫 4 bytes 를 HWP5 record header 로 해석 시도.
    if stream.len() >= 4 {
        let h = u32::from_le_bytes([stream[0], stream[1], stream[2], stream[3]]);
        let tag = h & 0x3FF;
        // 합리적 tag 범위 (0x42 ~ 0x5b 가 BodyText 영역)
        if tag >= 0x40 && tag <= 0x80 {
            return stream.to_vec();
        }
    }
    let mut dec = DeflateDecoder::new(stream);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).unwrap_or_else(|e| {
        let head: String = stream.iter().take(20).map(|b| format!("{b:02x}")).collect();
        panic!(
            "raw deflate decompress (len={}): {e}\n  first 20 bytes hex: {head}",
            stream.len()
        )
    });
    out
}

/// HWP5 레코드 헤더 파싱 (tag/level/size 4 bytes + optional 4-byte extended size).
fn walk_records(data: &[u8]) -> Vec<(u16, u16, u32, usize)> {
    let mut records = Vec::new();
    let mut pos = 0;
    while pos + 4 <= data.len() {
        let h = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = (h & 0x3FF) as u16;
        let level = ((h >> 10) & 0x3FF) as u16;
        let mut size = (h >> 20) & 0xFFF;
        pos += 4;
        if size == 0xFFF {
            if pos + 4 > data.len() {
                break;
            }
            size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            pos += 4;
        }
        records.push((tag, level, size, pos));
        pos += size as usize;
    }
    records
}

/// `samples/form-01.hwpx` 변환 결과가 한컴 정답지의 9 CFB 스트림 contract 를 보유.
#[test]
fn form_01_keeps_nine_cfb_streams() {
    let hwp = convert("form-01.hwpx");
    let cfb = CompoundFile::open(Cursor::new(&hwp)).expect("CFB open");
    let actual: std::collections::BTreeSet<String> = cfb
        .walk()
        .filter(|e| e.is_stream())
        // [Task #1775] Windows 는 PathBuf 표시가 `\` 구분자라 기대값(`/`)과 어긋난다.
        .map(|e| e.path().to_string_lossy().replace('\\', "/"))
        .collect();
    let required = [
        "/FileHeader",
        "/DocInfo",
        "/BodyText/Section0",
        "/HwpSummaryInformation",
        "/DocOptions/_LinkDoc",
        "/Scripts/DefaultJScript",
        "/Scripts/JScriptVersion",
        "/PrvImage",
        "/PrvText",
    ];
    for r in required {
        assert!(
            actual.contains(r),
            "missing required stream {r}. Actual streams: {actual:?}"
        );
    }
}

/// `samples/form-01.hwpx` 변환 결과의 BodyText/Section0 가 5 Form CTRL_HEADER + FORM_OBJECT 보유.
///
/// Stage 2.4 회귀 가드: `Control::Form` 직렬화가 미동작하면 모든 form 레코드 누락.
#[test]
fn form_01_body_text_has_five_form_records() {
    let hwp = convert("form-01.hwpx");
    let section0 = read_stream(&hwp, "/BodyText/Section0");
    let raw = maybe_decompress(&section0);
    let recs = walk_records(&raw);

    // CTRL_HEADER (tag 0x47 = 71) with ctrl_id "form" (LE "mrof") + child FORM_OBJECT (0x5b = 91)
    let mut form_pairs = 0;
    for i in 0..recs.len() {
        let (tag, _lvl, size, off) = recs[i];
        if tag == 0x47 && size == 46 {
            let payload = &raw[off..off + 4];
            if payload == b"mrof" {
                // Next record must be FORM_OBJECT
                if i + 1 < recs.len() && recs[i + 1].0 == 0x5b {
                    form_pairs += 1;
                }
            }
        }
    }
    assert_eq!(
        form_pairs, 5,
        "expected 5 Form CTRL_HEADER+FORM_OBJECT pairs, found {form_pairs}"
    );
}

/// `samples/form-01.hwpx` 변환 결과의 Scripts/DefaultJScript 가 정답지와 byte-identical (uncompressed).
///
/// Stage 2.5 회귀 가드: headerScripts + sourceScripts 결합 + raw deflate 미적용 시 JS 핸들러 부재.
#[test]
fn form_01_scripts_default_jscript_matches_golden() {
    let hwp = convert("form-01.hwpx");
    let converted_scripts = read_stream(&hwp, "/Scripts/DefaultJScript");
    let converted_raw = maybe_decompress(&converted_scripts);

    let golden_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/form-01.hwp");
    let golden_bytes = std::fs::read(&golden_path).expect("read golden");
    let golden_scripts = read_stream(&golden_bytes, "/Scripts/DefaultJScript");
    let golden_raw = maybe_decompress(&golden_scripts);

    assert_eq!(
        converted_raw,
        golden_raw,
        "Scripts/DefaultJScript uncompressed bytes must match golden\n\
         golden  ({}B): first 100 hex = {}\n\
         conv    ({}B): first 100 hex = {}",
        golden_raw.len(),
        hex_first(&golden_raw, 100),
        converted_raw.len(),
        hex_first(&converted_raw, 100),
    );
}

/// `samples/form-01.hwpx` 변환 결과의 BodyText 에 `%clk` 컨트롤 (CTRL_HEADER 151B) + CTRL_DATA (0x57, 26B).
///
/// Stage 2.5 회귀 가드: ClickHere field 의 ctrl_id 미설정 / CTRL_DATA 미생성 시 JS 핸들러 reference 끊김.
#[test]
fn form_01_body_text_has_click_here_with_ctrl_data() {
    let hwp = convert("form-01.hwpx");
    let section0 = read_stream(&hwp, "/BodyText/Section0");
    let raw = maybe_decompress(&section0);
    let recs = walk_records(&raw);

    let mut found_clk_with_ctrl_data = false;
    for i in 0..recs.len() {
        let (tag, _lvl, size, off) = recs[i];
        if tag == 0x47 && size == 151 {
            let payload = &raw[off..off + 4];
            if payload == b"klc%" {
                // "%clk" little-endian
                // Next record must be CTRL_DATA (tag 0x57 = 87, size 26)
                if i + 1 < recs.len() && recs[i + 1].0 == 0x57 && recs[i + 1].2 == 26 {
                    found_clk_with_ctrl_data = true;
                    break;
                }
            }
        }
    }
    assert!(
        found_clk_with_ctrl_data,
        "expected %clk (CTRL_HEADER 151B) + CTRL_DATA (0x57, 26B) pair"
    );
}

/// `samples/form-02.hwpx` 도 5 Form + 9 CFB 스트림 contract 보유 (form-01 회귀 가드 대칭).
#[test]
fn form_02_keeps_nine_streams_and_five_forms() {
    let hwp = convert("form-02.hwpx");

    // 9 streams
    let cfb = CompoundFile::open(Cursor::new(&hwp)).expect("CFB open");
    let stream_count = cfb.walk().filter(|e| e.is_stream()).count();
    assert!(
        stream_count >= 9,
        "form-02 expected >= 9 streams, got {stream_count}"
    );

    // 5 Form pairs
    let section0 = read_stream(&hwp, "/BodyText/Section0");
    let raw = maybe_decompress(&section0);
    let recs = walk_records(&raw);
    let form_count = recs
        .iter()
        .filter(|(tag, _lvl, size, off)| {
            *tag == 0x47 && *size == 46 && &raw[*off..*off + 4] == b"mrof"
        })
        .count();
    assert_eq!(form_count, 5, "form-02 expected 5 forms, got {form_count}");
}

fn hex_first(data: &[u8], n: usize) -> String {
    data.iter()
        .take(n)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join("")
}
