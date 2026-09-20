//! [#7105] 레거시 OLE 수식을 편집기에 `equation` 으로 알려 삭제·속성이 거부됐다.
//!
//! 한/글 5.x·97 계열 수식은 `hwpeq5X` OLE 개체(`Control::Shape(ShapeObject::Ole)`)로
//! 저장된다. `#5725` 경로가 그 스크립트를 수식 렌더러로 그려 렌더 트리에는 `Equation`
//! 노드가 생기고, `getPageControlLayout` 은 그 노드를 `"type":"equation"` 으로 적었다.
//!
//! 편집기(`rhwp-studio`)는 이 종류로 명령을 고른다 — `equation` 이면 삭제는
//! `deleteEquationControl`, 속성은 수식 속성 명령이다. 둘 다 native `Control::Equation` 만
//! 받으므로 **선택 핸들은 뜨는데 지워지지도 속성이 바뀌지도 않았다**(#7105 신고,
//! `실험4 Transistor-MOSFET.hwp` 본문 문단의 OLE 수식 10개 전부).
//!
//! 원본이 본문 문단의 OLE 도형이면 `ole` 로 알려 도형 명령을 쓰게 한다.
//!
//! ## 재현물
//!
//! 저장소에서 레거시 OLE 수식을 담은 문서는 `samples/issue5725/2921145_equation_ole.hwpx`
//! 하나뿐이고, 그 수식은 **표 칸 안**에 있다. 칸 안 수식 노드는 칸 경로(`cellPath`)와 칸 안
//! 컨트롤 번호를 싣지 않아 `ole` 로 알려도 편집기가 대상을 짚을 수 없으므로 이 수정의
//! 범위 밖이다. 그래서 같은 문서의 `<hp:ole>` 요소를 그대로 복제해 **본문 문단**에 하나 더
//! 넣은 사본을 시험 안에서 만든다 — 신고 문서의 형상(본문 문단 OLE 수식)이다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;

const EQ_OLE_SAMPLE: &str = "samples/issue5725/2921145_equation_ole.hwpx";
/// 음성 대조 — native `$eqed` 수식 문서.
const NATIVE_EQ_SAMPLE: &str = "samples/equation-lim.hwp";

fn read_repo(rel: &str) -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// 표 칸 안의 레거시 OLE 수식을 복제해 본문 끝 문단에 하나 더 넣은 HWPX.
fn hwpx_with_body_level_ole_equation() -> Vec<u8> {
    let template = read_repo(EQ_OLE_SAMPLE);
    let mut src = zip::ZipArchive::new(std::io::Cursor::new(template)).expect("template ZIP");
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let deflated = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let names: Vec<String> = src.file_names().map(str::to_string).collect();
    let mut spliced = false;
    for name in names {
        let mut data = Vec::new();
        src.by_name(&name)
            .expect("엔트리")
            .read_to_end(&mut data)
            .expect("엔트리 읽기");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).expect("section0 UTF-8");
            let start = xml.find("<hp:ole ").expect("표본의 OLE 수식 요소");
            let end =
                xml[start..].find("</hp:ole>").expect("OLE 요소 끝") + start + "</hp:ole>".len();
            let ole = xml[start..end].replacen("id=\"1701116216\"", "id=\"1701116217\"", 1);
            let paragraph = format!(
                "<hp:p id=\"3000000000\" paraPrIDRef=\"16\" styleIDRef=\"0\" pageBreak=\"0\" \
                 columnBreak=\"0\" merged=\"0\"><hp:run charPrIDRef=\"10\">{ole}<hp:t/></hp:run></hp:p>"
            );
            let close = xml.rfind("</hs:sec>").expect("구역 끝");
            data = format!("{}{}{}", &xml[..close], paragraph, &xml[close..]).into_bytes();
            spliced = true;
        }
        let options = if name == "mimetype" { stored } else { deflated };
        out.start_file(name.as_str(), options).expect("엔트리 시작");
        out.write_all(&data).expect("엔트리 쓰기");
    }
    assert!(spliced, "section0.xml 을 찾아 고쳐야 한다");
    out.finish().expect("ZIP 마감").into_inner()
}

/// 배치 목록 한 항목 — `(type, secIdx, paraIdx, controlIdx, 칸 안인가)`.
type ControlEntry = (String, usize, usize, usize, bool);

fn controls(core: &DocumentCore) -> Vec<ControlEntry> {
    (0..core.page_count())
        .flat_map(|page| {
            let json = core
                .get_page_control_layout_native(page)
                .expect("control layout");
            let value: serde_json::Value = serde_json::from_str(&json).expect("json");
            value
                .get("controls")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|c| {
            Some((
                c.get("type")?.as_str()?.to_string(),
                c.get("secIdx")?.as_u64()? as usize,
                c.get("paraIdx")?.as_u64()? as usize,
                c.get("controlIdx")?.as_u64()? as usize,
                c.get("cellIdx").is_some(),
            ))
        })
        .collect()
}

fn body_ole(entries: &[ControlEntry]) -> Option<(usize, usize, usize)> {
    entries
        .iter()
        .find(|(kind, .., in_cell)| kind == "ole" && !in_cell)
        .map(|(_, sec, para, ctrl, _)| (*sec, *para, *ctrl))
}

#[test]
fn issue_7105_body_level_ole_equation_is_reported_as_ole() {
    let core = DocumentCore::from_bytes(&hwpx_with_body_level_ole_equation()).expect("open");
    let found = controls(&core);
    assert!(
        body_ole(&found).is_some(),
        "#7105: 본문 문단의 레거시 수식 OLE 는 `ole` 로 알려야 한다 — 수정 전 `equation`: {found:?}"
    );
    assert!(
        !found
            .iter()
            .any(|(kind, .., in_cell)| kind == "equation" && !in_cell),
        "#7105: 이 문서의 본문에는 native 수식이 없으므로 본문 `equation` 은 OLE 를 잘못 알린 것이다: {found:?}"
    );
}

#[test]
fn issue_7105_reported_kind_routes_to_a_command_the_core_accepts() {
    let mut core = DocumentCore::from_bytes(&hwpx_with_body_level_ole_equation()).expect("open");
    let (sec, para, ctrl) = body_ole(&controls(&core)).expect("ole 로 알린 본문 수식 개체");

    // 수정 전 편집기가 고르던 수식 명령은 OLE 를 거부한다 — 이것이 신고된 증상이다.
    assert!(
        core.delete_equation_control_native(sec, para, ctrl)
            .is_err(),
        "수식 명령은 native 수식만 받는다(OLE 거부)"
    );
    // `ole` 이 고르는 도형 명령은 같은 개체를 읽고 지운다.
    assert!(
        core.get_shape_properties_native(sec, para, ctrl).is_ok(),
        "#7105: 도형 속성 조회가 OLE 수식에서 성공해야 한다"
    );
    core.delete_shape_control_native(sec, para, ctrl)
        .expect("#7105: 도형 삭제 명령이 OLE 수식을 지워야 한다");
    assert!(
        body_ole(&controls(&core)).is_none(),
        "삭제 뒤에는 그 본문 OLE 수식이 배치 목록에서 사라져야 한다"
    );
}

#[test]
fn issue_7105_legacy_ole_equation_promotes_to_editable_native_equation_and_survives_hwp_save() {
    let mut core = DocumentCore::from_bytes(&hwpx_with_body_level_ole_equation()).expect("open");
    let (sec, para, ctrl) = body_ole(&controls(&core)).expect("본문 레거시 OLE 수식");

    let promoted: serde_json::Value = serde_json::from_str(
        &core
            .promote_ole_equation_native(sec, para, ctrl)
            .expect("레거시 OLE 수식 전환"),
    )
    .expect("전환 결과 JSON");
    assert_eq!(promoted["ok"], true);
    assert!(
        promoted["script"].as_str().is_some_and(|s| !s.is_empty()),
        "OLE Contents에서 꺼낸 스크립트가 native equation으로 옮겨져야 한다: {promoted}"
    );
    assert!(
        controls(&core)
            .iter()
            .any(|(kind, s, p, c, in_cell)| kind == "equation"
                && !in_cell
                && (*s, *p, *c) == (sec, para, ctrl)),
        "같은 본문 슬롯이 native equation으로 바뀌어야 한다"
    );

    core.set_equation_properties_native(sec, para, ctrl, None, None, r#"{"script":"a over b"}"#)
        .expect("전환 후 일반 수식 편집 경로");
    let saved = core.export_hwp_native().expect("HWP 저장");
    let reopened = DocumentCore::from_bytes(&saved).expect("저장 HWP 재열기");
    let props: serde_json::Value = serde_json::from_str(
        &reopened
            .get_equation_properties_native(sec, para, ctrl, None, None)
            .expect("재열기 후 native equation 속성"),
    )
    .expect("속성 JSON");
    assert_eq!(
        props["script"], "a over b",
        "편집한 수식은 HWP 저장 뒤에도 남아야 한다"
    );
}

#[test]
fn issue_7105_native_equation_is_still_reported_as_equation() {
    // 음성 대조 — native 수식은 종전대로 `equation` 이고 수식 명령이 받는다.
    let mut core = DocumentCore::from_bytes(&read_repo(NATIVE_EQ_SAMPLE)).expect("open");
    let (_, sec, para, ctrl, _) = controls(&core)
        .into_iter()
        .find(|(kind, .., in_cell)| kind == "equation" && !in_cell)
        .expect("#7105: native 수식은 `equation` 으로 알려야 한다");
    core.delete_equation_control_native(sec, para, ctrl)
        .expect("native 수식은 수식 명령이 지운다");
}
