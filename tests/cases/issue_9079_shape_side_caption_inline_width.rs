//! [#9079] 관계형·고정형 계약 — 글자처럼 취급 도형의 옆 캡션은 글줄이 예약하는 폭에 들어간다.
//!
//! 주석 갱신: 2026-11-15 01:50 KST(+09:00). 새 파일.
//!
//! 유형: 쪽수(6쪽)와 캡션 줄 수(3줄)는 고정형(독립 한컴 Print·원본 저장 줄 기대값). 개체·캡션이
//! 본문 단 안에 있고 합친 상자가 줄 가운데에 놓인다는 검사는 관계형이다. 절대 px 좌표는 고정하지
//! 않는다.
//!
//! 대상: `samples/issue9079/black_stork_restoration_press_release.hwpx`(기후에너지환경부 공개
//! 보도자료, korea.kr fileId 198526926) 3쪽. 가운데 정렬 문단의 글자처럼 취급 묶음 개체(폭
//! 29260HU)에 `caption side="RIGHT" width="17007" gap="850"` 이 있다. 한컴은 개체 + 간격 + 캡션
//! (47117HU)을 한 글자처럼 줄 폭 48188HU 안에서 가운데에 놓는다. 캡션 문단의 저장 줄은
//! `textpos` 0·21·37 세 줄인데, 숨김 변경 추적 투영으로 지워져 캡션 폭으로 다시 조판된다.
//!
//! 수정 전에는 줄 구성이 개체 폭만 예약해 개체가 가운데(한컴보다 약 119px 오른쪽)에 놓이고 캡션이
//! 본문 오른쪽 끝을 넘었다. 도형 캡션은 캡션 폭 재조판 대상이 아니어서 두 줄로 눌려 겹쳤다.
//!
//! 시각 측정(2026-11-15, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`,
//! `silhouette_raw_match_percent`, 글꼴은 한컴 PDF 가 내장한 face(KoPub돋움체·한컴 고딕·HY헤드라인M·
//! 바탕·굴림) 디렉터리를 첫 항목으로, 이어서 `ttfs/hwp`·`ttfs/windows`. 기준
//! `pdf/issue9079/black_stork_restoration_press_release-print-2022.pdf` — 한컴오피스 2022
//! 12.0.0.4547 Print → Hancom PDF, 저장 제품 hancom-office-2018):
//! - 6쪽 = 한컴 6쪽. Native 99.55·99.56·100.00·90.88·99.99·100.00(최저 90.88), fresh WASM
//!   99.98·99.91·100.00·91.17·99.99·100.00(최저 91.17). 수정 전 Native 3쪽 84.10.
//! - KoPub돋움체가 글꼴 목록에 없으면 fresh WASM 4쪽이 명조 대체로 78.19 까지 내려간다(배치가
//!   아니라 글꼴 공급 차이). 위 수치는 그 face 를 공급하고 잰 값이다.
//! - 1·2쪽은 #9073(머리 표의 저장 줄 전진) 위에서 잰 값이다.
//! - 증거: `mydocs/pr/assets/issue_9079/`(전쪽 Native·fresh WASM TSV, 대표 review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::shape::{CaptionDirection, ShapeObject};
use rhwp::renderer::render_tree::{CaptionControlKind, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::path::PathBuf;

const SOURCE: &str = "samples/issue9079/black_stork_restoration_press_release.hwpx";

fn collect<'a>(
    node: &'a RenderNode,
    pick: &dyn Fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if pick(node) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, pick, out);
    }
}

struct Placement {
    column_left: f64,
    column_right: f64,
    object_left: f64,
    caption_left: f64,
    caption_right: f64,
    /// 캡션 줄의 (x, y, width).
    caption_lines: Vec<(f64, f64, f64)>,
}

fn placement() -> Placement {
    let bytes =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).expect(SOURCE);
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 6, "한컴 2022 Print 의 전체 쪽수");

    let paragraphs = &core.document().sections[0].paragraphs;
    let (host, control_index, object_width, caption_width, gap) = paragraphs
        .iter()
        .enumerate()
        .find_map(|(index, paragraph)| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .find_map(|(ci, control)| match control {
                    Control::Shape(shape) => match shape.as_ref() {
                        ShapeObject::Group(group) if group.common.treat_as_char => {
                            let caption = group.caption.as_ref()?;
                            (caption.direction == CaptionDirection::Right).then(|| {
                                (
                                    index,
                                    ci,
                                    group.common.width as i32,
                                    caption.width as i32,
                                    i32::from(caption.spacing),
                                )
                            })
                        }
                        _ => None,
                    },
                    _ => None,
                })
        })
        .expect("오른쪽 캡션을 가진 글자처럼 취급 묶음 개체");
    let px = |hu: i32| rhwp::renderer::hwpunit_to_px(hu, 96.0);

    let tree = core.build_page_render_tree(2).expect("3쪽");
    let mut columns = Vec::new();
    collect(
        &tree.root,
        &|node| matches!(node.node_type, RenderNodeType::Column(_)),
        &mut columns,
    );
    let column = columns.first().expect("본문 단").bbox;

    let mut groups = Vec::new();
    collect(
        &tree.root,
        &|node| {
            matches!(&node.node_type, RenderNodeType::Group(group)
                if group.section_index == Some(0)
                    && group.para_index == Some(host)
                    && group.control_index == Some(control_index))
        },
        &mut groups,
    );
    let object = groups.first().expect("3쪽의 묶음 개체").bbox;

    let mut lines = Vec::new();
    collect(
        &tree.root,
        &|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.caption_owner.is_some_and(|owner| {
                owner.sec_idx == 0
                    && owner.para_idx == host
                    && owner.control_idx == control_index
                    && owner.control_kind == CaptionControlKind::Shape
            }))
        },
        &mut lines,
    );
    let caption_left = object.x + px(object_width) + px(gap);
    Placement {
        column_left: column.x,
        column_right: column.x + column.width,
        object_left: object.x,
        caption_left,
        caption_right: caption_left + px(caption_width),
        caption_lines: lines
            .iter()
            .map(|line| (line.bbox.x, line.bbox.y, line.bbox.width))
            .collect(),
    }
}

/// 개체 + 간격 + 캡션을 합친 상자가 본문 단 안에서 가운데에 놓인다.
///
/// 관계: 개체 왼쪽 여백 = 캡션 상자 오른쪽 여백(둘 다 0 이상). 폭은 원본 저장값(개체 폭·캡션
/// 폭·간격)에서 계산한다. 수정 전에는 개체만 가운데에 놓여 캡션 상자가 본문 오른쪽 끝을 넘었다.
#[test]
fn inline_group_and_its_right_caption_are_centered_as_one_box() {
    let p = placement();
    let left_slack = p.object_left - p.column_left;
    let right_slack = p.column_right - p.caption_right;
    assert!(
        left_slack >= -0.5 && right_slack >= -0.5,
        "개체와 캡션은 본문 단 안에 있다: left={left_slack:.2}px right={right_slack:.2}px"
    );
    assert!(
        (left_slack - right_slack).abs() <= 1.0,
        "개체 + 간격 + 캡션이 줄 가운데에 놓인다: left={left_slack:.2}px right={right_slack:.2}px"
    );
}

/// 저장 줄이 없는 도형 캡션은 캡션 폭으로 조판된다.
///
/// 관계: 캡션은 한컴 저장 줄(textpos 0·21·37)과 같이 세 줄이고, 각 줄은 캡션 상자 안에서 서로
/// 다른 줄 위치에 놓인다. 수정 전에는 두 줄로 눌려 겹쳤다.
#[test]
fn unstored_shape_caption_wraps_inside_the_caption_box() {
    let p = placement();
    assert_eq!(
        p.caption_lines.len(),
        3,
        "캡션은 한컴 저장 줄과 같이 세 줄이다"
    );
    for &(x, _, width) in &p.caption_lines {
        assert!(
            x + 0.5 >= p.caption_left && x + width <= p.caption_right + 0.5,
            "캡션 줄은 캡션 상자 안에 있다: x={x:.2}..{:.2} box={:.2}..{:.2}",
            x + width,
            p.caption_left,
            p.caption_right
        );
    }
    let mut tops: Vec<f64> = p.caption_lines.iter().map(|&(_, y, _)| y).collect();
    tops.sort_by(f64::total_cmp);
    assert!(
        tops.windows(2).all(|pair| pair[1] - pair[0] > 1.0),
        "캡션 세 줄은 서로 다른 줄 위치에 놓인다(겹치지 않음)"
    );
}
