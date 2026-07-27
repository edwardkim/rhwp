//! synam-001.hwp p30 "7. [필수]" 행 회귀 가드.
//!
//! 증상: 선행 자리차지(TopAndBottom, vert=Para) float 표들이 만든
//! `visible_float_exclusions` 존을 피해 현재 문단의 표 시작 y가 아래로
//! 밀렸을 때, 그 문단의 host 텍스트("7. [필수]") 줄 높이를 표 앞에
//! 예약하는 로직(layout.rs `table_y_start` 3단계 보정)이 밀리기 *전*
//! 원시 `para_y_for_table` 를 기준으로 자체 exclusion 판정을 다시 수행해,
//! 이미 밀려난 위치와 어긋나 예약분(host_line_px)이 사실상 사라졌다.
//! 결과: host 문단 텍스트와 표의 첫 줄이 같은 y에 겹쳐 그려짐(가독 불가).
//!
//! 수정: title_flow_y 계산의 기준을 이미 exclusion 보정이 반영된
//! `table_y_start`로 바꿔, host 줄 높이 예약이 항상 실제 표 시작 위치
//! 위에 정확히 얹히도록 한다.
//!
//! 이 테스트는 실제 문서 p30(0-based sec para 229)의 SVG 출력에서
//! host label "7"(굵게, 좌측 마진에서 시작)의 y 좌표와, 뒤따르는 표
//! 셀 첫 줄 텍스트의 y 좌표가 최소 한 줄 높이(8px) 이상 떨어져 있는지
//! 확인한다 — 겹치면 그 차이가 1px 미만으로 떨어진다(수정 전 실측 0.75px).

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/synam-001.hwp";

fn render_page_svg(page_num: u32) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse synam-001.hwp");
    core.render_page_svg_native(page_num)
        .unwrap_or_else(|e| panic!("render page {page_num} svg: {e}"))
}

/// `<text ... y="123.45" ...>CHAR<` 패턴에서 속성 y 값을 모두 뽑아
/// (attrs, text) 쌍으로 반환한다 (아주 단순한 스캐너 — 테스트 전용).
fn extract_text_elements(svg: &str) -> Vec<(String, f64, String)> {
    let mut out = Vec::new();
    let bytes = svg.as_bytes();
    let mut i = 0;
    while let Some(rel) = svg[i..].find("<text ") {
        let start = i + rel;
        let Some(close_rel) = svg[start..].find('>') else {
            break;
        };
        let tag_end = start + close_rel;
        let attrs = &svg[start..tag_end];
        let text_start = tag_end + 1;
        let Some(lt_rel) = svg[text_start..].find('<') else {
            break;
        };
        let text = &svg[text_start..text_start + lt_rel];
        if let Some(y) = extract_attr(attrs, "y") {
            out.push((attrs.to_string(), y, text.to_string()));
        }
        i = text_start + lt_rel;
        if i >= bytes.len() {
            break;
        }
    }
    out
}

fn extract_attr(attrs: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = attrs.find(&needle)? + needle.len();
    let end = attrs[start..].find('"')? + start;
    attrs[start..end].parse::<f64>().ok()
}

#[test]
fn host_title_line_does_not_overlap_visible_float_table_first_line() {
    // 0-based 페이지 인덱스: PDF/화면상 "30페이지"(1-based) = index 29.
    let svg = render_page_svg(29);
    let elements = extract_text_elements(&svg);
    assert!(
        !elements.is_empty(),
        "p30 SVG에 텍스트 요소가 있어야 함 (렌더 회귀로 텍스트 자체가 비어있으면 안 됨)"
    );

    // host label "7" : 굵게(font-weight="bold"), 좌측 본문 마진(x≈37.8)에서 시작.
    let host_y = elements
        .iter()
        .filter(|(attrs, _, text)| {
            attrs.contains("font-weight=\"bold\"") && attrs.contains("x=\"37.78") && text == "7"
        })
        .map(|(_, y, _)| *y)
        .next()
        .expect("host label '7' 글리프가 좌측 마진에서 렌더되어야 함");

    // 표 셀 첫 줄 텍스트: "본인은 위 1~6호..." 문단의 첫 글자 "본".
    // 굵지 않고(font-weight 속성 없음), 같은 표 영역에서 host label과 가장 가까운 "본"이다.
    // `y > host_y`를 먼저 걸면 표가 host 위로 올라와 겹친 회귀를 결과 집합에서 숨기게 된다.
    let cell_y = elements
        .iter()
        .filter(|(attrs, _, text)| {
            !attrs.contains("font-weight=\"bold\"")
                && text == "본"
                && extract_attr(attrs, "x").map(|x| x > 40.0).unwrap_or(false)
        })
        .map(|(_, y, _)| *y)
        .min_by(|a, b| {
            (a - host_y)
                .abs()
                .partial_cmp(&(b - host_y).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("표 셀 첫 줄 '본' 글리프가 렌더되어야 함 (내용 소실 회귀 가드 겸함)");
    assert!(
        cell_y > host_y,
        "표 셀 첫 줄이 host 줄 아래에 있어야 함: host_y={host_y:.2}, cell_y={cell_y:.2}"
    );

    let gap = cell_y - host_y;
    assert!(
        gap >= 8.0,
        "host 줄('7. [필수]', y={host_y:.2})과 표 첫 줄('본인은...', y={cell_y:.2})이 \
         겹침 — gap={gap:.2}px (최소 8px 기대). \
         선행 float exclusion 보정 후 title_flow_y 기준이 어긋난 회귀."
    );
    assert!(
        gap <= 19.0,
        "host 줄과 표 첫 줄 사이가 과도하게 벌어짐 — gap={gap:.2}px. \
         #2439 exclusion에서 복원한 outer-top을 host 줄 보정에서 다시 더한 회귀."
    );
}
