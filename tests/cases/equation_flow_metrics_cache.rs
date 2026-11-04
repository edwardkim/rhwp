//! 검사 유형: 관계형(합성 공개 API의 캐시 일관성); 절대 페이지 좌표·golden 핀 없음.
//! 주석 갱신: 2026-11-05T04:27:53+09:00; Git 직전 파일 변경: 987b77e94514e66bf868dc5431218905ef635fda 2026-10-27T19:47:54+09:00.
//! 입력: 빈 문서에 HYhwpEQ 한글 수식을 삽입하고 저장 프레임을 유지한 채 script만 편집한다.
//! 기대 관계: 긴 수식은 뒤 캐럿을 전진시키며, 편집 이력이 다른 같은 값의 새 문서와 위치가 같다.
//! 실물 한컴 Print의 배치 정답을 합성 입력으로 대신하지 않는다. 합성 입력의 독립 Print raw는 없다.
//! 실물 성능 문서의 측정·판정은 별도 performance-control-review.md에 기록하며 이 검사의 raw로 쓰지 않는다.
//! 근거: mydocs/pr/assets/semanticist21-20261105/performance-control-review.md.
//! 기존 실제 문서·편집 후 새 스레드 cache 검사는 equation_render_cache_contract.rs에서 별도 유지한다.
//!
//! 수식 흐름 메트릭 캐시는 편집 이력이 아니라 수식 값만으로 결과를 정한다.
//!
//! 한글을 담은 HYhwpEQ 수식은 저장 폭이 아니라 스크립트로 흐름 폭을 계산한다.
//! 저장 폭을 고정한 채 스크립트만 바꿔도 뒤 글자가 밀려야 하고, 같은 값으로
//! 처음부터 만든 수식과 같은 자리에 있어야 한다.

use rhwp::document_core::DocumentCore;

const SHORT: &str = "가";
const LONG: &str = "가나다라마바사";

fn caret_x(core: &DocumentCore, offset: usize) -> f64 {
    let rect: serde_json::Value =
        serde_json::from_str(&core.get_cursor_rect_native(0, 0, offset).unwrap()).unwrap();
    rect["x"].as_f64().expect("캐럿 x")
}

/// 본문 "끝" 앞에 수식을 넣고 저장 크기를 고정한다. 오프셋 1은 "끝" 뒤다.
fn document_with_equation(
    script: &str,
    frame: Option<(u32, u32)>,
) -> (DocumentCore, usize, u32, u32) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "끝").unwrap();
    let inserted: serde_json::Value = serde_json::from_str(
        &core
            .insert_equation_native(0, 0, 0, script, 1000, 0)
            .unwrap(),
    )
    .unwrap();
    let control = inserted["controlIdx"].as_u64().expect("수식 컨트롤") as usize;
    let props: serde_json::Value = serde_json::from_str(
        &core
            .get_equation_properties_native(0, 0, control, None, None)
            .unwrap(),
    )
    .unwrap();
    let (width, height) = frame.unwrap_or((
        props["width"].as_u64().unwrap() as u32,
        props["height"].as_u64().unwrap() as u32,
    ));
    core.set_equation_properties_native(
        0,
        0,
        control,
        None,
        None,
        &format!(r#"{{"width":{width},"height":{height}}}"#),
    )
    .unwrap();
    (core, control, width, height)
}

#[test]
fn script_edit_with_pinned_frame_reflows_like_fresh_equation() {
    let (mut edited, control, width, height) = document_with_equation(SHORT, None);
    let before = caret_x(&edited, 1);

    edited
        .set_equation_properties_native(
            0,
            0,
            control,
            None,
            None,
            &format!(r#"{{"script":"{LONG}","width":{width},"height":{height}}}"#),
        )
        .unwrap();
    let after = caret_x(&edited, 1);
    assert!(
        after > before + 1.0,
        "긴 스크립트가 뒤 글자를 밀어야 한다: {before} -> {after}"
    );

    let (fresh, _, _, _) = document_with_equation(LONG, Some((width, height)));
    assert_eq!(after, caret_x(&fresh, 1));
}
