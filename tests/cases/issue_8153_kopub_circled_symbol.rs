//! 관계형: 원문 기호의 공급 프로그램 전진폭과 뒤 글자 소유를 보존한다.
//! 주석 갱신: 2026-10-31T18:41:48+09:00; 직전 파일 커밋: 2026-10-30T19:50:11+09:00 (c417310ca).
//! 두 원문 줄은 run 경계와 무관하게 같은 줄에서 맞닿은 연속 run 의 글로 대조한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 59쪽 Native 100.00% / fresh WASM 100.00%.
//! HWP 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! HWPX 59쪽 Native 100.00% / fresh WASM 100.00%.
//! HWPX 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 Print 및 KoPub 6개 프로그램에서 각 기호의 전진은 900/1000em이다.
//! HWP 5.0 표 33의 글자 크기 1000HU·장평 99%를 원본에서 확인했다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::BoundingBox,
    DocumentCore,
};
fn find(node: &LayerNode, out: &mut Vec<(String, BoundingBox, Option<u32>)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for c in children {
                find(c, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            find(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            for o in ops {
                if let PaintOp::TextRun { bbox, run, .. } = o {
                    out.push((run.text.clone(), *bbox, run.char_shape_id));
                }
            }
        }
    }
}
/// 같은 줄에서 앞 run 의 끝과 뒤 run 의 시작이 맞닿은 연속 run 을 한 글 조각으로 잇는다.
/// run 경계는 글자 슬롯 분류처럼 출력과 무관한 구현 세부로 바뀔 수 있다(`…` 이 앞 기호와
/// 한 run 이어도, 따로 나뉘어도 위치는 같다).
fn segments(
    runs: Vec<(String, BoundingBox, Option<u32>)>,
) -> Vec<Vec<(String, BoundingBox, Option<u32>)>> {
    let mut out: Vec<Vec<(String, BoundingBox, Option<u32>)>> = Vec::new();
    for run in runs {
        let joined = out
            .last()
            .and_then(|s| s.last())
            .is_some_and(|(_, prev, _)| {
                (prev.y - run.1.y).abs() < 0.01 && (prev.x + prev.width - run.1.x).abs() < 0.05
            });
        if joined {
            out.last_mut().unwrap().push(run);
        } else {
            out.push(vec![run]);
        }
    }
    out
}
fn verify(ext: &str) {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut d = DocumentCore::from_bytes(
        &std::fs::read(base.join(format!("samples/2025 행정업무운영 편람(최종).{ext}"))).unwrap(),
    )
    .unwrap();
    d.set_hangul2024_compat(true);
    let tree = d
        .build_page_layer_tree_with_profile(58, RenderProfile::Print)
        .unwrap();
    let mut runs = Vec::new();
    find(&tree.root, &mut runs);
    let got: Vec<_> = segments(runs)
        .into_iter()
        .filter_map(|segment| {
            let text: String = segment.iter().map(|(t, _, _)| t.as_str()).collect();
            if text != "  ①, ②, ③, ④, …" && text != "  ㉮, ㉯, ㉰, ㉱, …" {
                return None;
            }
            for (_, _, char_shape_id) in &segment {
                assert_eq!(*char_shape_id, Some(89), "{text}");
            }
            let first = segment.first().unwrap().1;
            let last = segment.last().unwrap().1;
            Some((text, last.x + last.width - first.x))
        })
        .collect();
    assert_eq!(got.len(), 2, "두 원문 줄의 누락·중복 방지");
    // 독립 Print의 기호 폭 900/1000em, 원본 1000HU·장평 99%로 888HU.
    // 원문 useFontSpace=0은 확인된 반각 496HU, 쉼표는 독립 글꼴 메트릭 308HU.
    // 두 줄 모두 끝의 `…` 하나(888HU)까지 잰다.
    let expected = 6 * 496 + 5 * 888 + 4 * 308;
    for (text, width) in got {
        assert!(
            (width - f64::from(expected) / 75.0).abs() < 1e-7,
            "원본 크기·장평을 적용한 숫자/한글 원 기호의 전진: {text} width={width}"
        );
    }
}
#[test]
fn hwp_circled_symbol_keeps_source_advance() {
    verify("hwp")
}
#[test]
fn hwpx_circled_symbol_keeps_source_advance() {
    verify("hwpx")
}
