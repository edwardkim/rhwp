//! [Issue #8335] HWP↔HWPX 저장이 개체 번호 범주를 바꾸지 않는다.
//!
//! HWP5 개체 공통 속성 attr bit 26-27 은 번호 범주다(#8294). 한컴이 같은 문서를 저장한
//! HWP·HWPX 32쌍의 개체 616개 전부에서 이 두 비트가 HWPX `numberingType` 과 같다 —
//! 개체 종류와 무관하게 `NONE`=0, `PICTURE`=1, `TABLE`=2, `EQUATION`=3.
//! rhwp 는 bit 26 을 GenShape 마다 켜고 bit 27 은 `TABLE` 표에만 켜서, `NONE` 그림을
//! 그림 범주로, `PICTURE` 표를 범주 없음으로 저장했다. 그림 캡션 번호가 밀리거나 모두 1 이
//! 됐다. 기준: `pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf`(64쪽),
//! `pdf/exam_kor-hwp-2020.pdf`(20쪽) — 독립 한컴 Print.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ObjectNumberingType;

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"))
}

fn reopen_as_hwp(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwp_with_adapter().expect("HWP 저장"))
        .expect("HWP 다시 열기")
}

fn reopen_as_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기")
}

/// 문서 순서대로 번호 범주를 갖는 개체(표·그림·도형·수식)의 (종류, 범주, 저장 attr bit 26-27).
fn objects(core: &DocumentCore) -> Vec<(&'static str, ObjectNumberingType, u32)> {
    fn walk(paragraphs: &[Paragraph], out: &mut Vec<(&'static str, ObjectNumberingType, u32)>) {
        for para in paragraphs {
            for control in &para.controls {
                let (kind, common) = match control {
                    Control::Table(table) => ("표", &table.common),
                    Control::Picture(picture) => ("그림", &picture.common),
                    Control::Shape(shape) => ("도형", shape.common()),
                    Control::Equation(equation) => ("수식", &equation.common),
                    _ => continue,
                };
                out.push((
                    kind,
                    common.numbering_type,
                    common.attr & ObjectNumberingType::HWP5_ATTR_MASK,
                ));
                if let Control::Table(table) = control {
                    for cell in &table.cells {
                        walk(&cell.paragraphs, out);
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for section in &core.document().sections {
        walk(&section.paragraphs, &mut out);
    }
    out
}

fn categories(core: &DocumentCore) -> Vec<(&'static str, ObjectNumberingType)> {
    objects(core).into_iter().map(|(k, c, _)| (k, c)).collect()
}

/// 쪽마다 조판된 글(표·캡션 포함) — 캡션 번호와 쪽 소속·순서·누락·중복을 함께 본다.
fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| core.extract_page_text_native(page).expect("쪽 글"))
        .collect()
}

/// 쪽마다 `label` 바로 뒤(공백 허용)에 오는 번호의 (쪽, N).
fn caption_numbers(core: &DocumentCore, label: &str) -> Vec<(usize, u32)> {
    let mut out = Vec::new();
    for (page, text) in page_texts(core).iter().enumerate() {
        for (i, _) in text.match_indices(label) {
            let digits: String = text[i + label.len()..]
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digits.parse() {
                out.push((page, n));
            }
        }
    }
    out
}

fn figure_numbers(core: &DocumentCore) -> Vec<(usize, u32)> {
    caption_numbers(core, "[그림")
}

/// 한컴 PDF 의 `[그림 N]` 순서: 그림 목차의 1‥n, 본문 캡션의 1‥n.
fn hancom_figure_sequence(n: u32) -> Vec<u32> {
    (1..=n).chain(1..=n).collect()
}

fn numbers_only(figures: &[(usize, u32)]) -> Vec<u32> {
    figures.iter().map(|(_, n)| *n).collect()
}

/// 한컴 HWPX 의 `PICTURE` 표 그림 캡션: HWP 저장 뒤에도 1, 2, 3, … (모두 1 이 아니다).
#[test]
fn picture_numbered_tables_keep_category_on_hwp_save() {
    let original = load("samples/issue4599/1790387_policy_progress.hwpx");
    let original_objects = categories(&original);
    assert!(
        original_objects.contains(&("표", ObjectNumberingType::Picture))
            && original_objects.contains(&("그림", ObjectNumberingType::None)),
        "입력은 PICTURE 표와 NONE 그림을 함께 가진다"
    );
    let numbers = figure_numbers(&original);
    assert_eq!(
        numbers_only(&numbers),
        hancom_figure_sequence(16),
        "원본 그림 번호(한컴 PDF 와 같다)"
    );

    let hwp = reopen_as_hwp(&original);
    assert_eq!(categories(&hwp), original_objects, "개체별 번호 범주");
    assert_eq!(figure_numbers(&hwp), numbers, "HWP 저장 뒤 그림 캡션 번호");
    assert_eq!(page_texts(&hwp), page_texts(&original), "쪽별 본문");
}

/// HWP → HWPX → HWP 왕복: 범주 없는 그림이 그림 범주로 바뀌어 번호가 밀리지 않는다.
/// policy_progress 를 HWP 로 저장한 뒤 HWPX → HWP 로 다시 왕복한다.
#[test]
fn none_category_pictures_stay_uncounted_through_hwpx_roundtrip() {
    let original = load("samples/issue4599/1790387_policy_progress.hwpx");
    let hwp = reopen_as_hwp(&original);
    let hwp_objects = objects(&hwp);
    assert!(
        hwp_objects
            .iter()
            .any(|(k, c, bits)| *k == "그림" && *c == ObjectNumberingType::None && *bits == 0),
        "HWP 저장본은 범주 없는 그림을 가진다"
    );

    let round_trip = reopen_as_hwp(&reopen_as_hwpx(&hwp));
    assert_eq!(
        objects(&round_trip),
        hwp_objects,
        "개체별 번호 범주와 저장 비트"
    );
    assert_eq!(
        numbers_only(&figure_numbers(&round_trip)),
        hancom_figure_sequence(16),
        "왕복 뒤 그림 번호(한컴 PDF 와 같다)"
    );
    assert_eq!(
        figure_numbers(&round_trip),
        figure_numbers(&original),
        "그림 번호의 쪽 소속"
    );
    assert_eq!(page_texts(&round_trip), page_texts(&original), "쪽별 본문");
}

/// 한컴 HWP5 의 `NONE` 도형은 HWPX 왕복 뒤에도 범주 없음(bit 26-27 = 0)으로 저장된다.
#[test]
fn hancom_hwp_none_shapes_keep_category_through_hwpx_roundtrip() {
    let original = load("samples/exam_kor.hwp");
    let original_objects = objects(&original);
    assert!(
        original_objects
            .iter()
            .any(|(k, c, _)| *k == "도형" && *c == ObjectNumberingType::None),
        "입력은 NONE 도형을 가진다"
    );
    let round_trip = reopen_as_hwp(&reopen_as_hwpx(&original));
    assert_eq!(
        objects(&round_trip),
        original_objects,
        "개체별 번호 범주와 저장 비트"
    );
    assert_eq!(page_texts(&round_trip), page_texts(&original), "쪽별 본문");
}

/// 한컴 저장 쌍: HWPX 를 rhwp 가 HWP 로 저장한 번호 범주 비트가 한컴의 HWP 와 같다.
/// `ta-pic-001-r` 은 `NONE` 그림과 `PICTURE` 표, `exam_kor` 는 `NONE` 도형과 `PICTURE`
/// 그림·도형·`TABLE` 표, `eq-002` 는 `EQUATION` 수식이다. `k-water-rfp` 는 일반 문서 대조군.
#[test]
fn hwpx_to_hwp_category_bits_match_hancom_hwp() {
    for (hwp_path, hwpx_path, required) in [
        (
            "samples/ta-pic-001-r.hwp",
            "samples/hwpx/ta-pic-001-r.hwpx",
            &[
                ("그림", ObjectNumberingType::None),
                ("표", ObjectNumberingType::Picture),
            ][..],
        ),
        (
            "samples/exam_kor.hwp",
            "samples/hwpx/exam_kor.hwpx",
            &[
                ("도형", ObjectNumberingType::None),
                ("도형", ObjectNumberingType::Picture),
                ("그림", ObjectNumberingType::Picture),
                ("표", ObjectNumberingType::Table),
            ][..],
        ),
        (
            "samples/eq-002.hwp",
            "samples/hwpx/eq-002.hwpx",
            &[("수식", ObjectNumberingType::Equation)][..],
        ),
        (
            "samples/k-water-rfp.hwp",
            "samples/hwpx/k-water-rfp.hwpx",
            &[
                ("그림", ObjectNumberingType::Picture),
                ("표", ObjectNumberingType::Table),
            ][..],
        ),
    ] {
        let hancom = objects(&load(hwp_path));
        for (kind, category) in required {
            assert!(
                hancom.iter().any(|(k, c, bits)| k == kind
                    && c == category
                    && *bits == category.hwp5_attr_bits()),
                "{hwp_path}: 한컴 HWP 에 {kind} {category:?} 개체가 있어야 한다"
            );
        }
        let ours = objects(&reopen_as_hwp(&load(hwpx_path)));
        assert_eq!(ours, hancom, "{hwpx_path} → HWP 번호 범주 비트");
    }
}
