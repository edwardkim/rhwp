//! 관계형: PR #8409의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:46:04+09:00 (`8e4f05c63a789b6b1988190016ede401c3a636f5`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/2025 행정업무운영 편람(최종).hwp`; 독립 Print `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`.
//! 전체 383쪽, 전쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 입력 `samples/2025 행정업무운영 편람(최종).hwpx`; 독립 Print `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`.
//! 전체 383쪽, 전쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8409_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
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
