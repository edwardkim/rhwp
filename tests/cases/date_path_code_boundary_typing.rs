//! 관계형 회귀: 날짜·경로 필드 경계 입력의 소속·필드 값·왕복 내용 보존. 실물 절대 배치 핀은 없다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! date-path-context-before.hwp (8564-.result.9d14e579-4810-4dc8-b9db-a0cf0bb4bd23.partial-hwp-2022): 전 1쪽, Native 최저 91.81401%, fresh WASM 최저 91.17272%.
//! date-path-context-edited.hwp (api-8564-context-after): 전 1쪽, Native 최저 92.01038%, fresh WASM 최저 92.28612%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 날짜·파일 경로 코드 경계에서 친 글자는 코드 밖에 둔다.
//!
//! 한/글(12.30 Mac)은 날짜 코드(`%dte`)와 파일 경로 코드(`%pat`)의 글을 문서를 열 때와
//! 다른 이름으로 저장할 때 다시 쓴다. 코드 바로 앞·뒤에서 친 글자가 코드 안에 들어가면
//! 그때 사라진다. 코드 가운데에서 친 글자는 한/글처럼 코드의 글이다.
//!
//! 고정 문서는 한/글에서 만든 두 문단이다: `A` + 날짜 코드(`2026년 11월 4일`) + `B`,
//! 파일 이름 코드(`date-codes.hwpx`) + `C`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::FieldType;

const FIXTURE: &str = "tests/fixtures/date_path_codes/hancom-date-path-codes.hwpx";
const DATE: &str = "2026년 11월 4일";
const PATH: &str = "date-codes.hwpx";

fn open() -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(FIXTURE).unwrap()).unwrap()
}

/// 문단 글과 코드(종류·글).
type Paragraphs = Vec<String>;
type Codes = Vec<(FieldType, String)>;

fn state(core: &DocumentCore) -> (Paragraphs, Codes) {
    (
        core.document().sections[0]
            .paragraphs
            .iter()
            .map(|para| para.text.clone())
            .collect(),
        core.collect_all_fields()
            .into_iter()
            .map(|info| (info.field.field_type, info.value))
            .collect(),
    )
}

/// 편집한 문서와 그 HWP·HWPX 저장본이 같은 글과 코드를 가진다.
fn check(core: DocumentCore, paragraphs: &[&str], codes: &[(FieldType, &str)]) {
    let expected = (
        paragraphs.iter().map(|text| text.to_string()).collect(),
        codes
            .iter()
            .map(|(ty, text)| (*ty, text.to_string()))
            .collect(),
    );
    let saved = [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ];
    let reopened = saved
        .iter()
        .map(|bytes| DocumentCore::from_bytes(bytes).unwrap());
    for (format, result) in ["memory", "hwp", "hwpx"]
        .into_iter()
        .zip(std::iter::once(core).chain(reopened))
    {
        assert_eq!(state(&result), expected, "{format}");
    }
}

#[test]
fn hancom_codes_load_as_date_and_path_fields() {
    let core = open();
    assert_eq!(
        state(&core),
        (
            vec![format!("A{DATE}B"), format!("{PATH}C")],
            vec![
                (FieldType::Date, DATE.to_string()),
                (FieldType::Path, PATH.to_string())
            ],
        )
    );
}

#[test]
fn typing_at_code_boundaries_stays_outside_the_code() {
    let mut core = open();
    // 날짜 코드 뒤(문단 가운데)와 앞, 파일 이름 코드 앞(문단 맨 앞)과 뒤.
    core.insert_text_native(0, 0, 13, "뒤").unwrap();
    core.insert_text_native(0, 0, 1, "앞").unwrap();
    core.insert_text_native(0, 1, 15, "뒤").unwrap();
    core.insert_text_native(0, 1, 0, "앞").unwrap();
    check(
        core,
        &[&format!("A앞{DATE}뒤B"), &format!("앞{PATH}뒤C")],
        &[(FieldType::Date, DATE), (FieldType::Path, PATH)],
    );
}

#[test]
fn typing_after_code_at_paragraph_end_stays_outside() {
    let mut core = open();
    core.delete_text_native(0, 1, 15, 1).unwrap();
    core.insert_text_native(0, 1, 15, "끝").unwrap();
    check(
        core,
        &[&format!("A{DATE}B"), &format!("{PATH}끝")],
        &[(FieldType::Date, DATE), (FieldType::Path, PATH)],
    );
}

#[test]
fn typing_inside_code_stays_in_the_code() {
    let mut core = open();
    core.insert_text_native(0, 0, 5, "!").unwrap();
    check(
        core,
        &[&format!("A2026!년 11월 4일B"), &format!("{PATH}C")],
        &[
            (FieldType::Date, "2026!년 11월 4일"),
            (FieldType::Path, PATH),
        ],
    );
}

#[test]
fn typing_beside_code_in_clickhere_paragraph_stays_outside() {
    // 누름틀이 있는 문단은 원시 슬롯을 지키는 다른 입력 경로를 쓴다.
    let mut core = open();
    core.insert_click_here_field_at(0, 0, 14, "안내", "", "", false)
        .unwrap();
    core.insert_text_native(0, 0, 13, "뒤").unwrap();
    core.insert_text_native(0, 0, 1, "앞").unwrap();
    check(
        core,
        &[&format!("A앞{DATE}뒤B"), &format!("{PATH}C")],
        &[
            (FieldType::Date, DATE),
            (FieldType::ClickHere, ""),
            (FieldType::Path, PATH),
        ],
    );
}

#[test]
fn typing_into_emptied_code_fills_the_code() {
    // 글을 모두 지운 코드에 친 글자는 그 코드의 글이다(누름틀 문단의 첫 빈 필드 규칙과 같다).
    let mut core = open();
    core.delete_text_native(0, 0, 1, 12).unwrap();
    core.insert_text_native(0, 0, 1, "X").unwrap();
    check(
        core,
        &["AXB", &format!("{PATH}C")],
        &[(FieldType::Date, "X"), (FieldType::Path, PATH)],
    );
}
