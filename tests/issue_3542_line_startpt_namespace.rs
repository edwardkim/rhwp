//! [Issue #3542] hp:line 의 좌표 자식은 hc:startPt/hc:endPt (XSD LineType — core
//! 네임스페이스) 다. 종전 저장기는 hp: 프리픽스로 방출했고, 최소 사례에서 이 위반만으로
//! 한컴오피스가 문서를 거부했다. hp:connectLine 의 hp:startPt(ConnectPointType 로컬
//! 요소)는 정당하므로, connectLine 이 없는 실물 샘플로 hp:line 방출만 고정한다.
//! 파서는 프리픽스 무관 로컬명 매칭이라 자체 왕복(--verify)으로는 잡히지 않는다.

use std::io::Read;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/hwpx/opengov/36392900_결재문서본문_일일굴착복구공사현황보고.hwpx";

/// 샘플을 저장 경로(export_hwpx_native)로 재직렬화한 뒤 본문 section XML 을 연결해 돌려준다.
fn export_section_xml(sample: &str) -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    let doc = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));
    let exported = doc
        .export_hwpx_native()
        .unwrap_or_else(|e| panic!("export {sample}: {e:?}"));
    let mut zip =
        zip::ZipArchive::new(std::io::Cursor::new(exported)).expect("저장본 ZIP 열기 실패");
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
fn issue_3542_line_start_end_pt_use_core_namespace() {
    let xml = export_section_xml(SAMPLE);
    assert!(xml.contains("<hp:line "), "샘플에 hp:line 이 있어야 한다");
    assert!(
        !xml.contains("<hp:connectLine"),
        "connectLine 없는 샘플이어야 hp:startPt 부재 단언이 유효하다"
    );
    assert!(
        xml.contains("<hc:startPt ") && xml.contains("<hc:endPt "),
        "hp:line 좌표 자식은 hc: 네임스페이스로 방출되어야 한다 (XSD LineType): {}",
        &xml[..xml.len().min(2000)]
    );
    assert!(
        !xml.contains("<hp:startPt") && !xml.contains("<hp:endPt"),
        "hp: 프리픽스 좌표 자식은 XSD 위반 — 한컴오피스가 문서를 거부한다"
    );
}
