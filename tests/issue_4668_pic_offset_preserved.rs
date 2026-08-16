//! [Issue #4668] HWPX 저장 시 `hp:pic` 의 `<hp:offset>` 이 원본 값 대신 `<hp:pos>`
//! 의 vertOffset/horzOffset 에서 재유도되어, 저장만 해도 그림 offset 이 바뀌던
//! 문제의 회귀 계약. 음수(u32 wraparound) offset 으로 쪽 밖에 둔 그림이 무편집
//! 저장 후 쪽 안(문단 앵커 위치)에 노출되는 표시 변화를 한글 실측으로 확인한
//! 결함이다 — pic 전용 writer(`picture.rs write_offset`)만 pos 유래로 방출했고,
//! 도형·OLE·컨테이너 공용 writer(`shape.rs`, #3544)는 이미 원문을 보존한다.

use std::io::Read;

use rhwp::document_core::DocumentCore;

/// 원본 pic 에 `<hp:offset x="5250" y="4294962964"/>`(y = −4332 wraparound)가
/// 실존하고, 그 pic 의 `<hp:pos>` 는 vertOffset=0/horzOffset=0 인 실물 샘플.
const SAMPLE: &str = "samples/ta-pic-cell-center-pos-bottom.hwpx";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

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

#[test]
fn issue_4668_pic_offset_is_not_rewritten_from_pos() {
    let bytes = std::fs::read(sample_path()).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    // 전제: 원본에 wraparound offset 과 pos(vertOffset=10436) 가 함께 있어야
    // "pos 유래 재작성" 회귀를 이 샘플로 판별할 수 있다.
    let original = section_xml_of(&bytes);
    assert!(
        original.contains(r#"<hp:offset x="5250" y="4294962964"/>"#),
        "샘플 전제 위반: 원본 wraparound hp:offset 이 없다"
    );
    assert!(
        original.contains(r#"vertOffset="10436""#),
        "샘플 전제 위반: 재작성 판별용 pos vertOffset 이 없다"
    );

    let doc = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"));
    let exported = doc
        .export_hwpx_native()
        .unwrap_or_else(|e| panic!("export {SAMPLE}: {e:?}"));
    let saved = section_xml_of(&exported);

    // 계약: 원본 offset 원문이 그대로 살아 있어야 한다.
    assert!(
        saved.contains(r#"<hp:offset x="5250" y="4294962964"/>"#),
        "hp:pic offset 이 원문 보존되지 않았다 — pos 유래 재작성(#4668) 재발 의심"
    );
    // 회귀 신호: 종전 결함의 산출 형태(pos vertOffset 을 offset y 로 복사).
    assert!(
        !saved.contains(r#"<hp:offset x="0" y="10436"/>"#),
        "hp:pic offset 이 pos vertOffset 으로 재작성됐다(#4668 재발)"
    );
}
