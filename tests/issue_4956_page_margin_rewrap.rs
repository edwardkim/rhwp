//! [#4956] 쪽 여백을 바꾸면 본문 문단이 새 본문 폭으로 다시 접힌다.
//!
//! 결함 형태: 저장 line_segs 는 파일이 기록해 둔 "그 용지 폭에서의 줄 나눔"인데, 본문 재래핑
//! 게이트(`paragraph_layout`/`typeset`/`height_measurer` 의 `para.line_segs.is_empty()`)가
//! 저장 분할이 있으면 무조건 신뢰한다. 그래서 `set_page_def_native` 로 여백을 좁혀도 줄은
//! 그대로 두고 글자만 눌러 짜여, 텍스트가 새 본문 상자 밖으로 넘쳤다.
//! (실측: 본문 폭 657.7→438.6 인데 3쪽 텍스트가 x 709.6 까지, TextRun 수 109 불변)
//!
//! 판정은 절대좌표가 아니라 "텍스트가 본문 상자 안에 있는가" 라는 관계다 — 폰트 메트릭
//! 환경에 강건하다(#3458 의 교훈).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;

/// 초과폭 허용치 (px). 재래핑이 돌면 초과는 기준과 같은 수준(구두점 내밀기)에 머물고,
/// 안 돌면 여백 변화량(여기서는 219px)만큼 벌어진다 — 그 사이는 넉넉히 비어 있다.
const TOLERANCE_PX: f64 = 20.0;

/// 폭이 본문 상자에서 오는 TextRun 의 오른쪽 끝 최대값 (px).
///
/// 표와 글상자는 제외한다 — 폭을 스스로 정하는 개체라 본문이 좁아지면 정당하게 넘친다.
/// 머리말·꼬리말은 **포함한다**: 그 폭은 본문과 같은 content_left..content_right 라서
/// (model/page.rs) 본문과 똑같이 다시 접혀야 한다. 빈 run 이 본문 폭 전체를 차지해 판정을
/// 가리던 문제는 글자 유무로 거른다.
fn max_text_right(node: &serde_json::Value, max_right: &mut f64) {
    match node.get("type").and_then(|t| t.as_str()) {
        Some("Table") | Some("TextBox") => return,
        Some("TextRun") => {
            let has_text = node
                .get("text")
                .and_then(|t| t.as_str())
                .is_some_and(|t| !t.trim().is_empty());
            if has_text {
                if let Some(bbox) = node.get("bbox") {
                    let x = bbox.get("x").and_then(|v| v.as_f64());
                    let w = bbox.get("w").and_then(|v| v.as_f64());
                    if let (Some(x), Some(w)) = (x, w) {
                        *max_right = max_right.max(x + w);
                    }
                }
            }
        }
        _ => {}
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    {
        max_text_right(child, max_right);
    }
}

/// 모든 쪽을 통틀어 TextRun 이 도달하는 가장 오른쪽 x (px).
///
/// 쪽 번호를 고르지 않는다 — 여백을 바꾸면 쪽 나눔 자체가 달라져서 "몇 쪽을 보는가" 가
/// 판정을 흔든다. 계약은 "어느 쪽에서도 본문 밖으로 나가지 않는다" 이므로 전 쪽 최대값으로
/// 본다.
fn max_text_right_all_pages(core: &DocumentCore) -> f64 {
    let mut max_right = f64::NEG_INFINITY;
    for page_num in 0..core.page_count() {
        let page = core
            .build_page_render_tree(page_num as u32)
            .expect("render tree 를 얻을 수 있어야 한다");
        let tree: serde_json::Value =
            serde_json::from_str(&page.root.to_json()).expect("render tree JSON");
        max_text_right(&tree, &mut max_right);
    }
    max_right
}

/// 구역 0 의 본문 오른쪽 끝 (px).
fn body_right_px(core: &DocumentCore) -> f64 {
    let pd = &core.document().sections[0].section_def.page_def;
    rhwp::renderer::hwpunit_to_px((pd.width - pd.margin_right) as i32, 96.0)
}

/// 구역 0 본문 문단들의 저장 분할 지문 — 줄마다 (시작 오프셋, 줄 높이).
fn body_line_seg_fingerprint(core: &DocumentCore) -> Vec<Vec<(u32, i32)>> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| {
            p.line_segs
                .iter()
                .map(|s| (s.text_start, s.line_height))
                .collect()
        })
        .collect()
}

/// 글이 있는 머리말·꼬리말 문단의 저장 분할 줄 수.
fn header_footer_line_seg_counts(core: &DocumentCore) -> Vec<usize> {
    use rhwp::model::control::Control;
    let mut out = Vec::new();
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                let nested = match ctrl {
                    Control::Header(header) => &header.paragraphs,
                    Control::Footer(footer) => &footer.paragraphs,
                    _ => continue,
                };
                for nested_para in nested {
                    if !nested_para.text.trim().is_empty() {
                        out.push(nested_para.line_segs.len());
                    }
                }
            }
        }
    }
    out
}

/// 글자가 있는데 저장 분할이 없는 문단 수 — 조판이 NO_LS 로 취급하는 계급이다.
fn no_lineseg_text_paragraphs(core: &DocumentCore) -> usize {
    core.document().sections[0]
        .paragraphs
        .iter()
        .filter(|p| p.line_segs.is_empty() && !p.text.is_empty())
        .count()
}

#[test]
fn body_rewraps_when_right_margin_grows() {
    let bytes = std::fs::read("rhwp-studio/public/samples/biz_plan.hwp")
        .expect("fixture 를 읽을 수 있어야 한다");
    let mut core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");

    let before_right = max_text_right_all_pages(&core);
    let before_body = body_right_px(&core);
    assert!(before_right.is_finite(), "픽스처에 TextRun 이 있어야 한다");
    // 기준 상태에도 본문을 조금 넘는 줄이 있다(구두점 내밀기·장평 보정, 실측 11.3px).
    // 그래서 절대 경계가 아니라 "여백을 바꿔도 이 초과폭이 커지지 않는가" 로 판정한다.
    let before_overflow = before_right - before_body;

    // 오른쪽 여백을 약 3배로 (5102 → 21529 HWPUNIT). 본문 폭 657.7 → 438.6 px.
    core.set_page_def_native(0, r#"{"marginRight":21529}"#)
        .expect("쪽 여백 변경");

    let after_right = max_text_right_all_pages(&core);
    let after_body = body_right_px(&core);
    assert!(
        after_body < before_body - 100.0,
        "본문 상자가 좁아지지 않았다 ({:.1} → {:.1}) — 전제 확인",
        before_body,
        after_body
    );
    let after_overflow = after_right - after_body;
    assert!(
        after_overflow <= before_overflow + TOLERANCE_PX,
        "여백을 넓혔는데 본문이 다시 접히지 않았다 — 본문 오른쪽 끝 {:.1} 인데 텍스트가 \
         x {:.1} 까지 간다(초과 {:.1}px, 기준 {:.1}px). 결함 시 텍스트는 옛 줄 폭 그대로 \
         {:.1} 부근에 머문다: 저장 line_segs 를 그대로 신뢰해 줄 나눔이 재계산되지 않는다",
        after_body,
        after_right,
        after_overflow,
        before_overflow,
        before_right
    );
}

#[test]
fn body_rewraps_when_left_margin_grows() {
    // 왼쪽 여백은 본문 원점이 함께 움직여 글이 따라가는 것처럼 보이지만, 줄 폭이 옛 값이면
    // 오른쪽으로 그만큼 넘친다 — 오른쪽 여백과 같은 결함의 다른 얼굴이다.
    let bytes = std::fs::read("rhwp-studio/public/samples/biz_plan.hwp")
        .expect("fixture 를 읽을 수 있어야 한다");
    let mut core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");

    let before_overflow = max_text_right_all_pages(&core) - body_right_px(&core);
    assert!(
        before_overflow.is_finite(),
        "픽스처에 TextRun 이 있어야 한다"
    );

    core.set_page_def_native(0, r#"{"marginLeft":21529}"#)
        .expect("쪽 여백 변경");

    // 왼쪽 여백은 본문 오른쪽 끝을 옮기지 않는다. 줄이 다시 접히지 않으면 옛 폭 그대로
    // 오른쪽으로 밀려 그만큼 넘친다.
    let right = max_text_right_all_pages(&core);
    let body = body_right_px(&core);
    assert!(
        right - body <= before_overflow + TOLERANCE_PX,
        "왼쪽 여백을 넓혔는데 줄 폭이 옛 값 그대로다 — 텍스트 {:.1}, 본문 오른쪽 끝 {:.1} \
         (초과 {:.1}px, 기준 {:.1}px)",
        right,
        body,
        right - body,
        before_overflow
    );
}

#[test]
fn header_stays_inside_the_body_box_without_touching_its_line_segs() {
    // 머리말·꼬리말의 폭은 본문과 같은 content_left..content_right 다(model/page.rs). 그래서
    // "본문만 다시 접으면 머리말이 옛 폭으로 남아 넘친다" 는 추론이 자연스럽지만, 실제로는
    // 합성 경로가 저장 분할과 무관하게 영역 폭으로 다시 접는다. 그 사실이 재래핑 범위를
    // 본문 문단으로 한정한 근거이므로 여기에 박아 둔다 — 합성 경로가 바뀌어 이 전제가
    // 깨지면 이 테스트가 먼저 알려준다.
    //
    // biz_plan 은 머리말이 표뿐이라(표는 자기 폭을 정한다) 이 성질을 못 본다. 꼬리말에 접힐
    // 글이 60자 한 줄로 들어 있는 픽스처로 본다.
    let bytes =
        std::fs::read("samples/hwp3-sample19-hwp5.hwp").expect("fixture 를 읽을 수 있어야 한다");
    let mut core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");

    let before_overflow = max_text_right_all_pages(&core) - body_right_px(&core);
    assert!(
        before_overflow.is_finite(),
        "픽스처에 TextRun 이 있어야 한다"
    );
    let segs_before = header_footer_line_seg_counts(&core);
    assert!(
        !segs_before.is_empty(),
        "픽스처의 머리말/꼬리말에 글이 있어야 한다"
    );

    // 본문 폭을 절반으로 (오른쪽 여백 = 원래 오른쪽 여백 + 본문 폭/2)
    let narrowed = {
        let pd = &core.document().sections[0].section_def.page_def;
        pd.margin_right + (pd.width - pd.margin_left - pd.margin_right) / 2
    };
    core.set_page_def_native(0, &format!("{{\"marginRight\":{narrowed}}}"))
        .expect("쪽 여백 변경");

    assert_eq!(
        header_footer_line_seg_counts(&core),
        segs_before,
        "재래핑이 머리말/꼬리말의 저장 분할까지 건드렸다 — 범위는 본문 문단이어야 한다"
    );

    let right = max_text_right_all_pages(&core);
    let body = body_right_px(&core);
    // 기준 초과폭이 음수면(원래 폭에서는 어떤 줄도 끝까지 안 참) 0 을 바닥으로 둔다 —
    // 넘침은 음수로 내려갈 수 없으므로 음수를 그대로 더하면 허용치가 좁아진다.
    assert!(
        right - body <= before_overflow.max(0.0) + TOLERANCE_PX,
        "본문을 좁혔더니 글이 본문 상자를 넘었다 — 텍스트 {:.1}, 본문 오른쪽 끝 {:.1} \
         (초과 {:.1}px, 기준 {:.1}px). 머리말/꼬리말을 영역 폭으로 다시 접던 합성 경로가 \
         바뀌었다면 재래핑 범위를 다시 판단해야 한다",
        right,
        body,
        right - body,
        before_overflow
    );
}

#[test]
fn page_height_alone_does_not_rewrap() {
    // 재래핑은 공짜가 아니다 — 저장 분할을 버리고 새로 접는 것은 실측 광역 회귀가 있는
    // 동작이라(commands/document.rs 의 92셋 전수: 88→76) 본문 가로 폭이 실제로 바뀐
    // 구역에만 걸어야 한다. 용지 높이는 줄바꿈과 무관하므로 아무것도 건드리면 안 된다.
    let bytes = std::fs::read("rhwp-studio/public/samples/biz_plan.hwp")
        .expect("fixture 를 읽을 수 있어야 한다");
    let mut core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");

    // 줄 수가 아니라 분할 내용(각 줄의 시작 오프셋과 세로 좌표)을 본다. 같은 폭으로 다시
    // 접으면 줄 수는 그대로여도 저장 분할과 값이 달라진다 — rhwp 의 재계산이 한글이 파일에
    // 적어 둔 분할과 글자 단위로 같지는 않기 때문이다.
    let segs_before = body_line_seg_fingerprint(&core);

    let taller = core.document().sections[0].section_def.page_def.height + 10_000;
    core.set_page_def_native(0, &format!("{{\"height\":{taller}}}"))
        .expect("용지 높이 변경");

    assert_eq!(
        body_line_seg_fingerprint(&core),
        segs_before,
        "용지 높이만 바꿨는데 줄 나눔을 다시 계산했다 — 발동 조건이 본문 가로 폭이 아니라 \
         필드 나열이면 이렇게 샌다"
    );
}

#[test]
fn rewrapped_paragraphs_keep_their_line_segs() {
    // 다시 접는 것과 비우는 것은 다르다. 비우기만 하면 문단이 조판의 NO_LS 계급이 되는데,
    // 그 계급은 쪽 나눔에서 문단 위 간격을 0 으로 세고(renderer/typeset.rs 의
    // `para.line_segs.is_empty()` 분기) 렌더는 그대로 그린다 — 여백을 1 HWPUNIT 만 건드려도
    // 쪽 나눔과 그리기가 문단마다 spacing_before 만큼 어긋나, 본문 끝이 본문 상자 아래로
    // 흘러내리고 쪽 수가 모자라게 계산된다.
    let bytes = std::fs::read("rhwp-studio/public/samples/biz_plan.hwp")
        .expect("fixture 를 읽을 수 있어야 한다");
    let mut core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");

    let before = no_lineseg_text_paragraphs(&core);

    core.set_page_def_native(0, r#"{"marginRight":21529}"#)
        .expect("쪽 여백 변경");

    let after = no_lineseg_text_paragraphs(&core);
    assert_eq!(
        after, before,
        "여백 변경이 본문 문단을 NO_LS 계급으로 떨어뜨렸다 — 글자 있는 문단 중 저장 분할이 \
         없는 것이 {before}개에서 {after}개로 늘었다. 비운 뒤 다시 접지 않으면 쪽 나눔이 \
         문단 위 간격을 세지 않는다"
    );
}
