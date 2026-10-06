//! [#4680] HWP3 저장 줄높이를 글자 높이로 그대로 써서 줄간격이 그 위에 또 곱해진다.
//!
//! ## 무엇이 문제였나
//!
//! HWP3 파서는 줄마다 `th = linfo.line_height × 4` 를 **글자 높이**(`text_height`)로 적고,
//! 문단의 백분율 줄간격(예: 160%)을 그 값에 곱한다. 그런데 HWP3 가 적어 둔 줄높이는 글자
//! 높이가 아니라 **줄 상자**(글자 + 여유)다. 그래서 10pt 문단이 12pt 글자로 저장되고
//! 줄 전진폭이 16pt 대신 19.2pt 가 된다 — 쪽마다 20%씩 부푼다.
//!
//! 1쪽짜리 별지 서식이 2쪽으로 넘치면 **제출 불가**다. 이슈가 그 패턴을 8건으로 센다.
//!
//! ## 기대값의 독립 근거 — 한컴 자신의 HWP3→HWPX 변환본
//!
//! 한컴 2024(`13.0.0.3901`)에게 같은 HWP3 원본을 HWPX 로 바꾸게 해 전수로 맞댔다.
//! 한컴은 `vertsize` 를 **그 문단의 글자 크기**로 적는다.
//!
//! ```text
//!   코퍼스 04442 (부산 기장군 사무 인계인수 규칙 별지 제23-1호서식) 34문단
//!     한컴  vertsize == charPr height     33 / 34   (나머지 1개는 lineseg 없음)
//!     한컴  spacing  == vertsize × 0.6    (문단 줄간격 160%)
//!
//!     수정 전 rhwp   모든 문단 vertsize=1200 spacing=720  (10pt 문단도 12pt 로)
//!     수정 후 rhwp   한컴과 34 / 34 일치
//!
//!   한/글이 센 쪽수    원본 1쪽 · 한컴 자체 h2x 1쪽 · rhwp h2x 2쪽 → 수정 후 1쪽
//! ```
//!
//! ## 적용 경계 — 좁힌 이유
//!
//! 저장 줄높이가 글자보다 큰 것이 **정당한** 줄도 있다. 줄 안에 더 큰 글자가 있거나
//! 인라인 개체가 있으면 줄 상자가 커야 한다. 그래서 세 조건을 모두 만족할 때만 줄인다.
//!
//! ```text
//!   th > 대표 글자 높이                 저장 줄 상자가 글자보다 크다
//!   문단 안 모든 글자 ≤ 대표 글자 높이   더 큰 글자가 없다
//!   문단에 컨트롤이 없다                 인라인 개체가 없다
//! ```
//!
//! 넓은 판(앞 두 조건 없이)은 `issue_1692`(SO-SUEOP HWP3 1쪽 학교 표시 y)를 깬다 —
//! 그 문단은 저장 줄높이가 큰 것이 맞고 HWPX 기준값이 `912.5` 인데 `847.1` 로 올라간다.
//! `text_overlap_baseline` 도 2구획이 늘었다. 좁힌 판은 저장소 전수에서 회귀가 없다.
//!
//! ## 이 수정이 닫지 않는 것
//!
//! 이슈의 서식 넘침 8건 중 이 축으로 닫히는 것은 `04442` 다. `04640`·`05610` 은 이미
//! `vertsize == charPr` 이라 **다른 원인**이고, `06314` 는 위 경계에 걸리지 않는다.
//! 본문 소실 18건·크래시 8건·`07615` 264쪽→0쪽도 이 수정의 범위가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

/// `#4680` 이 센 "서식 1쪽 → 2쪽" 8건 중 하나의 실물(부산 기장군 사무 인계인수 규칙
/// 별지 제23-1호서식). 한/글은 원본을 1쪽으로, 수정 전 rhwp 산출을 2쪽으로 읽었다.
const SAMPLE: &str = "samples/issue4680/20117321_gijang_handover_bond_ledger_form.hwp";

/// 내보낸 HWPX 에서 `(문단 안 최대 글자 높이, lineseg vertsize, spacing)` 을 모은다.
///
/// 인라인 개체가 있는 문단(`hp:lineseg` 없는 표 host 등)은 제외한다 — 줄 상자가 글자보다
/// 큰 것이 정당한 자리다.
fn exported_line_metrics() -> Vec<(u32, u32, i32)> {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("HWP3 정식 원본");
    let hwpx = HwpDocument::from_bytes(&bytes)
        .expect("HWP3 파스")
        .export_hwpx()
        .expect("HWPX 내보내기");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip");
    let read = |zip: &mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>, name: &str| -> String {
        let mut text = String::new();
        zip.by_name(name)
            .expect(name)
            .read_to_string(&mut text)
            .expect("read");
        text
    };
    let header = read(&mut zip, "Contents/header.xml");
    let section = read(&mut zip, "Contents/section0.xml");

    let mut char_heights = std::collections::HashMap::new();
    for item in header.split("<hh:charPr id=\"").skip(1) {
        let Some((id, rest)) = item.split_once('"') else {
            continue;
        };
        let Some(height) = rest
            .split("height=\"")
            .nth(1)
            .and_then(|h| h.split('"').next())
            .and_then(|h| h.parse::<u32>().ok())
        else {
            continue;
        };
        char_heights.insert(id.to_string(), height);
    }

    let mut out = Vec::new();
    for para in section.split("<hp:p ").skip(1) {
        let body = para.split("</hp:p>").next().unwrap_or(para);
        let heights: Vec<u32> = body
            .split("charPrIDRef=\"")
            .skip(1)
            .filter_map(|item| item.split('"').next())
            .filter_map(|id| char_heights.get(id).copied())
            .collect();
        let Some(&max_height) = heights.iter().max() else {
            continue;
        };
        let Some(seg) = body.split("<hp:lineseg ").nth(1) else {
            continue;
        };
        let field = |name: &str| -> Option<i64> {
            seg.split(&format!("{name}=\""))
                .nth(1)?
                .split('"')
                .next()?
                .parse()
                .ok()
        };
        let (Some(vertsize), Some(spacing)) = (field("vertsize"), field("spacing")) else {
            continue;
        };
        out.push((max_height, vertsize as u32, spacing as i32));
    }
    out
}

/// 인라인 개체가 없는 문단의 `vertsize` 는 그 문단의 글자 높이를 넘지 않는다.
///
/// 수정 전에는 HWP3 저장 줄 상자(12pt)가 10pt 문단에도 그대로 들어가 25문단이 부풀었고,
/// 한/글이 그 산출을 2쪽으로 읽었다.
#[test]
fn exported_line_height_does_not_exceed_the_paragraph_char_size() {
    let metrics = exported_line_metrics();
    assert!(!metrics.is_empty(), "문단 지표를 하나도 못 읽었다");

    let inflated: Vec<_> = metrics
        .iter()
        .filter(|(char_height, vertsize, _)| vertsize > char_height)
        .collect();
    assert!(
        inflated.is_empty(),
        "저장 줄 상자가 글자 높이로 새어 들어갔다 — {}문단 (글자, vertsize, spacing) {:?}. \
         한컴 자신의 HWP3→HWPX 변환본은 vertsize 를 글자 크기로 적는다(이 문서 33/34 일치)",
        inflated.len(),
        inflated.iter().take(5).collect::<Vec<_>>()
    );
}

/// 반례 — 줄이 글자보다 **작아지지는** 않는다.
///
/// 이 수정은 부푼 줄을 글자 크기로 되돌릴 뿐이다. 더 줄이면 줄이 겹친다.
#[test]
fn the_rule_never_shrinks_a_line_below_its_char_size() {
    let metrics = exported_line_metrics();
    let shrunk: Vec<_> = metrics
        .iter()
        .filter(|(char_height, vertsize, _)| vertsize < char_height)
        .collect();
    assert!(
        shrunk.is_empty(),
        "글자보다 작은 줄이 생겼다 — {}문단 {:?}",
        shrunk.len(),
        shrunk.iter().take(5).collect::<Vec<_>>()
    );
}
