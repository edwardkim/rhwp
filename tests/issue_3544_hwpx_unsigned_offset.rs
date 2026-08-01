//! [Issue #3544] OWPML XSD 상 `hp:offset` 의 x/y 는 unsigned 좌표 속성이다.
//! 한컴 산출 원본은 음수 오프셋을 u32 wraparound 십진 문자열로 기록하고
//! (예: `x="4294886250"` = -81046), 파서도 `parse_u32 as i32` 로 그 관례를
//! 복호한다. 종전 저장기는 IR 의 signed 값을 그대로 문자열화해 `x="-8974"`,
//! `y="-2"` 같은 XSD 위반을 만들었다 — 복호(파서)만 있고 부호화(저장기)가
//! 없는 인코딩 비대칭이 근인이다. 자체 왕복 --verify 는 파서가 음수도 관대하게
//! 읽어 잡지 못하므로, 방출 문자열 자체를 계약으로 고정한다.

use std::io::Read;

use rhwp::document_core::DocumentCore;

/// 원본에 wraparound `hp:offset` 이 28건 실존하는 실물 코퍼스 샘플 (#3542 와 동일).
const SAMPLE: &str = "samples/hwpx/opengov/36392900_결재문서본문_일일굴착복구공사현황보고.hwpx";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// ZIP 바이트에서 Contents/section*.xml 을 이어붙여 돌려준다.
fn section_xml_of(bytes: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("ZIP 열기 실패");
    let names: Vec<String> = zip.file_names().map(str::to_string).collect();
    let mut xml = String::new();
    for name in names {
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            zip.by_name(&name)
                .expect("section 엔트리")
                .read_to_string(&mut xml)
                .expect("section XML 은 UTF-8 이어야 한다");
        }
    }
    xml
}

/// XML 문자열 안의 모든 `<hp:offset .../>` 태그 원문을 수집한다.
fn offset_tags(xml: &str) -> Vec<&str> {
    let mut tags = Vec::new();
    let mut rest = xml;
    while let Some(pos) = rest.find("<hp:offset ") {
        let tail = &rest[pos..];
        let end = tail.find('>').expect("hp:offset 태그는 닫혀야 한다");
        tags.push(&tail[..=end]);
        rest = &tail[end..];
    }
    tags
}

/// 태그 원문에 u32 상위 절반(>= 2^31) 십진 속성값이 있으면 true — wraparound 부호화 여부.
fn has_wraparound_value(tag: &str) -> bool {
    tag.split('"')
        .filter_map(|tok| tok.parse::<u64>().ok())
        .any(|v| v >= 1 << 31 && v < 1 << 32)
}

#[test]
fn issue_3544_offset_never_negative_and_keeps_wraparound_encoding() {
    let bytes = std::fs::read(sample_path()).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    // 전제: 원본 자체가 wraparound 부호화된 음수 오프셋을 실제로 담고 있어야
    // 이 샘플이 #3544 재현 코퍼스로 유효하다.
    let original = section_xml_of(&bytes);
    let original_wrapped = offset_tags(&original)
        .iter()
        .filter(|t| has_wraparound_value(t))
        .count();
    assert!(
        original_wrapped > 0,
        "샘플 전제 위반: 원본에 wraparound hp:offset 이 없다"
    );
    assert!(
        offset_tags(&original).iter().all(|t| !t.contains("\"-")),
        "샘플 전제 위반: 한컴 원본 hp:offset 에 음수가 있을 수 없다"
    );

    let doc = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"));
    let exported = doc
        .export_hwpx_native()
        .unwrap_or_else(|e| panic!("export {SAMPLE}: {e:?}"));
    let saved = section_xml_of(&exported);
    let saved_tags = offset_tags(&saved);
    assert!(!saved_tags.is_empty(), "저장본에 hp:offset 이 있어야 한다");

    // 계약 1: XSD unsigned — 저장본 hp:offset 에 음수 십진수가 한 건도 없어야 한다.
    let negatives: Vec<&&str> = saved_tags.iter().filter(|t| t.contains("\"-")).collect();
    assert!(
        negatives.is_empty(),
        "hp:offset x/y 는 XSD unsigned — 음수 방출은 스키마 위반 (한컴 관례는 u32 \
         wraparound): {negatives:?}"
    );

    // 계약 2: 클램프가 아니라 부호화 — 원본의 음수 오프셋 정보가 wraparound 형태로
    // 살아 있어야 한다 (0 클램프였다면 wraparound 값이 전멸한다).
    let saved_wrapped = saved_tags
        .iter()
        .filter(|t| has_wraparound_value(t))
        .count();
    assert!(
        saved_wrapped > 0,
        "원본 wraparound {original_wrapped}건이 저장본에서 전부 사라졌다 — 값 소실(클램프) \
         의심: {saved_tags:?}"
    );
}
