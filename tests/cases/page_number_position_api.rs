//! 관계형 및 고정형 회귀: 구역·쪽 번호 컨트롤 소속과 위치 열거값·삽입·제거 계약. 고정 속성값을 화면 좌표로 해석하지 않는다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! hancom-page-number-mid.hwpx (8586-hancom-page-number-mid-hwpx-2022): 전 6쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! hancom-page-number-sections.hwpx (8586-hancom-page-number-sections-hwpx-2022): 전 7쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 쪽 번호 매기기(`pgnp`) 읽기·쓰기 API (#7635).
//!
//! 한/글(12.30 Mac) 쪽 > 쪽 번호 매기기는 캐럿 자리와 상관없이 현재 구역의 쪽 번호 위치를
//! 모두 지우고 구역 첫 문단의 구역·단 정의 바로 뒤에 하나를 넣는다. 쪽 번호 없음도 위치
//! (`pos="NONE"`)로 남긴다.
//!
//! 고정 문서는 한/글로 저장한 여섯 쪽(한 쪽에 한 문단)이다. 쪽 번호 위치(왼쪽 위, 줄표)가
//! 셋째 문단에 있어 한/글도 3쪽부터 번호를 그린다. 한/글에서 이 문서에 쪽 번호 매기기를
//! 하면 셋째 문단의 위치가 지워지고 첫 문단에 하나가 생긴다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const MID: &str = "tests/fixtures/page_number_position/hancom-page-number-mid.hwpx";
/// 한/글 저장본 두 구역. 구역마다 첫 문단 구역·단 정의 뒤에 쪽 번호 위치(가운데 위, 줄표)가 있다.
const TOP: &str = "tests/fixtures/page_number_position/hancom-page-number-top.hwpx";
/// 한/글 저장본 두 구역(1구역 넷째 쪽 뒤 빈 쪽 포함 다섯 쪽, 2구역 두 쪽). 쪽 번호 위치(오른쪽 위,
/// 줄표)는 1구역에만 있고, 한/글 PDF는 2구역 6·7쪽에도 `- 6 -`·`- 7 -`을 그린다.
const SECTIONS: &str = "tests/fixtures/page_number_position/hancom-page-number-sections.hwpx";

fn open(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).expect("open")
}

/// 구역의 (문단, 컨트롤 차례, 위치, 모양, 줄표 문자).
fn positions(core: &DocumentCore, section: usize) -> Vec<(usize, usize, u8, u8, char)> {
    core.document().sections[section]
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(pi, para)| {
            para.controls
                .iter()
                .enumerate()
                .filter_map(move |(ci, control)| match control {
                    Control::PageNumberPos(p) => Some((pi, ci, p.position, p.format, p.dash_char)),
                    _ => None,
                })
        })
        .collect()
}

fn texts(core: &DocumentCore) -> Vec<String> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(|para| para.text.clone())
        .collect()
}

/// 쪽 SVG에서 본문 밖의 글자, 곧 쪽 번호를 차례대로 잇는다. 고정 문서의 본문 첫 줄 기준선은 143.6px이다.
fn page_number(core: &DocumentCore, page: u32) -> String {
    let svg = core.render_page_svg_native(page).expect("svg");
    svg.split("<text")
        .skip(1)
        .filter_map(|chunk| {
            let head = &chunk[..chunk.find('>')?];
            let y: f64 = head
                .split_once(" y=\"")?
                .1
                .split('"')
                .next()?
                .parse()
                .ok()?;
            let body = &chunk[chunk.find('>')? + 1..chunk.find("</text>")?];
            (!(130.0..1000.0).contains(&y)).then(|| body.to_string())
        })
        .collect()
}

#[test]
fn reads_the_section_position() {
    let core = open(&std::fs::read(MID).unwrap());
    let value: serde_json::Value =
        serde_json::from_str(&core.get_page_number_position_native(0).unwrap()).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"ok": true, "exists": true, "position": 1, "format": 0, "dash": true})
    );
    let blank: serde_json::Value = {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        serde_json::from_str(&core.get_page_number_position_native(0).unwrap()).unwrap()
    };
    assert_eq!(blank, serde_json::json!({"ok": true, "exists": false}));
}

#[test]
fn replaces_every_section_position_with_one_at_the_section_start() {
    let mut core = open(&std::fs::read(MID).unwrap());
    assert_eq!(positions(&core, 0), vec![(2, 0, 1, 0, '-')]);
    assert_eq!(page_number(&core, 0), "");
    let before = texts(&core);

    // 지운 자리(문단, 컨트롤 차례)와 넣은 차례를 돌려준다. 셀 캐럿처럼 컨트롤 차례로 가리키는 주소를 옮기는 데 쓴다.
    let edit: serde_json::Value = serde_json::from_str(
        &core
            .set_page_number_position_native(0, 5, 2, false)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        edit,
        serde_json::json!({"ok": true, "controlIdx": 2, "removed": [[2, 0]]})
    );
    // 첫 문단의 구역·단 정의 뒤에 하나만 남고 글은 그대로다.
    assert_eq!(positions(&core, 0), vec![(0, 2, 5, 2, '\0')]);
    assert!(matches!(
        core.document().sections[0].paragraphs[0].controls[..2],
        [Control::SectionDef(_), Control::ColumnDef(_)]
    ));
    assert_eq!(texts(&core), before);
    assert_eq!(page_number(&core, 0), "I");
    assert_eq!(page_number(&core, 3), "IV");

    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = open(&bytes);
        assert_eq!(positions(&reopened, 0), vec![(0, 2, 5, 2, '\0')]);
        assert_eq!(texts(&reopened), before);
        assert_eq!(page_number(&reopened, 1), "II");
    }
}

#[test]
fn inserts_after_the_section_definitions_and_keeps_none() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.set_page_number_position_native(0, 5, 1, true).unwrap();
    assert_eq!(positions(&core, 0), vec![(0, 2, 5, 1, '-')]);
    assert_eq!(page_number(&core, 0), "-①-");

    // 쪽 번호 없음도 위치로 남겨 앞 구역의 번호를 잇지 않게 한다.
    core.set_page_number_position_native(0, 0, 1, false)
        .unwrap();
    assert_eq!(positions(&core, 0), vec![(0, 2, 0, 1, '\0')]);
    assert_eq!(page_number(&core, 0), "");
    let reopened = open(&core.export_hwpx_native().unwrap());
    let value: serde_json::Value =
        serde_json::from_str(&reopened.get_page_number_position_native(0).unwrap()).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"ok": true, "exists": true, "position": 0, "format": 1, "dash": false})
    );
}

#[test]
fn rejects_out_of_range_values_without_changing_the_document() {
    let mut core = open(&std::fs::read(MID).unwrap());
    for (section, position, format) in [(0, 11, 0), (0, 5, 17), (1, 5, 0)] {
        assert!(core
            .set_page_number_position_native(section, position, format, true)
            .is_err());
    }
    assert!(core.get_page_number_position_native(1).is_err());
    assert_eq!(positions(&core, 0), vec![(2, 0, 1, 0, '-')]);
}

#[test]
fn later_sections_without_their_own_position_follow_the_change() {
    let mut core = open(&std::fs::read(SECTIONS).unwrap());
    assert_eq!(core.page_count(), 7);
    assert!(positions(&core, 1).is_empty());
    assert_eq!(page_number(&core, 5), "-6-");

    core.set_page_number_position_native(0, 6, 0, false)
        .unwrap();
    assert_eq!(page_number(&core, 0), "1");
    assert_eq!(page_number(&core, 5), "6");
    assert_eq!(page_number(&core, 6), "7");

    // 뒤 구역에 쪽 번호 없음을 두면 그 구역만 번호가 없다.
    core.set_page_number_position_native(1, 0, 0, false)
        .unwrap();
    assert_eq!(page_number(&core, 4), "5");
    assert_eq!(page_number(&core, 5), "");
}

#[test]
fn replaces_the_position_hancom_put_at_the_section_start_in_place() {
    let mut core = open(&std::fs::read(TOP).unwrap());
    assert_eq!(positions(&core, 0), vec![(0, 2, 2, 0, '-')]);
    let edit: serde_json::Value = serde_json::from_str(
        &core
            .set_page_number_position_native(0, 6, 3, false)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        edit,
        serde_json::json!({"ok": true, "controlIdx": 2, "removed": [[0, 2]]})
    );
    assert_eq!(positions(&core, 0), vec![(0, 2, 6, 3, '\0')]);
    assert_eq!(positions(&core, 1), vec![(0, 2, 2, 0, '-')]);
    assert_eq!(page_number(&core, 0), "i");
    let reopened = open(&core.export_hwp_native().unwrap());
    assert_eq!(positions(&reopened, 0), vec![(0, 2, 6, 3, '\0')]);
}
