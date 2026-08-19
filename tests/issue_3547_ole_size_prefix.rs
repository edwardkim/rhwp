//! [Issue #3547] HWPX 파서(`normalize_ole_bytes`)는 내부 OLE(BinData/*.ole)의 선두
//! 4-byte LE size prefix 를 제거한다. HWPX 저장기가 이를 재부착하지 않으면 재저장
//! 파일의 OLE 가 CFB 매직으로 곧바로 시작하고, 한컴오피스는 첫 4바이트를 OLE 개체
//! 크기(~3.75GB)로 오인해 "메모리 부족"을 낸다 — HWP5 저장기에서 #954 로 수정된
//! 결함과 동형. 재저장 내부 OLE 바이트가 원본과 동일함을 고정한다.

use std::io::Read;

use rhwp::document_core::DocumentCore;
use rhwp::parser::ole_container::parse_ole_container;

const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
const SAMPLE: &str = "samples/hwpx/143E433F503322BD33.hwpx";

/// ZIP 바이트에서 확장자가 `.ole`(대소문자 무관)인 엔트리를 (이름, 바이트)로 수집한다.
fn internal_ole_entries(zip_bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).expect("ZIP 열기 실패");
    let names: Vec<String> = zip.file_names().map(str::to_string).collect();
    let mut entries = Vec::new();
    for name in names {
        if name.to_ascii_lowercase().ends_with(".ole") {
            let mut bytes = Vec::new();
            zip.by_name(&name)
                .expect("OLE 엔트리")
                .read_to_end(&mut bytes)
                .expect("OLE 엔트리 읽기");
            entries.push((name, bytes));
        }
    }
    entries
}

fn cfb_with_stream(name: &str, payload: &[u8]) -> Vec<u8> {
    rhwp::serializer::mini_cfb::build_cfb(&[(name, payload)]).expect("테스트 CFB 생성")
}

/// [#5582] HWPX OLE의 u32 LE 길이 프리픽스를 parser 직접 경로도 방어적으로 해석한다.
#[test]
fn issue_5582_length_prefixed_cfb_is_parsed() {
    let cfb = cfb_with_stream("Contents", b"junk-not-a-chart");
    assert!(parse_ole_container(&cfb).is_some(), "무프리픽스 기준선");

    let mut prefixed = (cfb.len() as u32).to_le_bytes().to_vec();
    prefixed.extend_from_slice(&cfb);
    let container = parse_ole_container(&prefixed).expect("프리픽스 CFB 파싱");
    assert!(container.raw_contents.is_some());

    // 길이가 안 맞으면 벗기지 않는다. CFB 매직이 아니므로 열지 않아야 한다.
    let mut wrong = 7u32.to_le_bytes().to_vec();
    wrong.extend_from_slice(&cfb);
    assert!(parse_ole_container(&wrong).is_none());
}

/// [#5582] 한컴 변형의 대문자 `CONTENTS` 스트림도 내용으로 수집한다.
#[test]
fn issue_5582_uppercase_contents_stream_is_collected() {
    let cfb = cfb_with_stream("CONTENTS", b"embedded-object");
    let container = parse_ole_container(&cfb).expect("파싱");
    assert_eq!(
        container.raw_contents.as_deref(),
        Some(b"embedded-object".as_slice())
    );
}

#[test]
fn issue_3547_internal_ole_size_prefix_roundtrips() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    let originals = internal_ole_entries(&bytes);
    assert!(!originals.is_empty(), "샘플에 내부 OLE 가 있어야 한다");
    for (name, ole) in &originals {
        assert!(
            ole.len() >= 12 && ole[4..12] == CFB_MAGIC,
            "{name}: 원본은 [4-byte size][CFB] 형식이어야 한다"
        );
    }

    let doc = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"));
    let exported = doc
        .export_hwpx_native()
        .unwrap_or_else(|e| panic!("export {SAMPLE}: {e:?}"));
    let saved = internal_ole_entries(&exported);
    assert_eq!(
        saved.len(),
        originals.len(),
        "내부 OLE 엔트리 수가 보존되어야 한다"
    );

    for ((oname, obytes), (sname, sbytes)) in originals.iter().zip(saved.iter()) {
        assert!(
            sbytes.len() >= 12 && sbytes[4..12] == CFB_MAGIC,
            "{sname}: 재저장본도 [4-byte size][CFB] 형식이어야 한다 — prefix 미복원이면 \
             CFB 매직으로 곧바로 시작해 한컴이 \"메모리 부족\"을 낸다"
        );
        let size = u32::from_le_bytes(sbytes[..4].try_into().unwrap()) as usize;
        assert_eq!(
            size,
            sbytes.len() - 4,
            "{sname}: prefix 는 CFB 길이를 가리켜야 한다"
        );
        assert_eq!(
            sbytes, obytes,
            "{oname}→{sname}: 내부 OLE 바이트가 원본과 동일해야 한다"
        );
    }
}
