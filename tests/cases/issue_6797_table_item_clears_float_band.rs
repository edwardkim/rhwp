//! [Issue #6797] 렌더의 자리차지 배제 밴드 소비가 `item_is_paragraph` 전용이라,
//! **빈 host 에 표만 달린 항목**(`PageItem::Table`)이 앞 문단 float 표의 밴드를
//! 그대로 통과했다.
//!
//! `156160455`(사회적 농장 양돈 소득) 7쪽 실측:
//!
//! ```text
//!   문단 0.70  글자 host       ls[0] vpos=3252  ls[1] vpos=5204
//!              표 3x4  vert=문단(4764 = +63.5px)  633.1 x 113.4px
//!   문단 0.71  빈 문단         ls[0] vpos=16306  ( = 절대 296.77px )
//!              표 1x2  vert=문단(0)               635.0 x 165.7px
//!
//!   rhwp(수정 전)   pi=70 표 181.5..294.9      pi=71 표 174.8..340.5
//!   저장 사다리     pi=70 밴드 181.5..296.8    pi=71 앵커 296.77  ← 밴드 바닥
//! ```
//!
//! ⚠ **세 수치를 구분한다**(PR #6798 지적).
//!
//! | 값 | 뜻 |
//! |---|---|
//! | `113.4px` | `pi=70` 표의 **높이**이자, 두 표의 세로 **겹침 높이**(294.9 − 174.8 을 상자 교집합으로 자른 값) |
//! | `159.1px` | 수정 hunk 를 제거한 **음성 대조**에서 검토자가 관측한 최악 중첩 — 이때는 `pi=71` 이 위쪽 다른 표와도 겹쳐 더 크다 |
//! | `165.7px` | `pi=71` 표 자신의 높이 |
//!
//! ⭐⭐ **판정은 저장 사다리가 한다 — 크기 문턱이 없다.**
//!
//! ```text
//!                      host 글   저장 vpos(절대)   밴드 바닥   판정
//!   156160455 pi=71     없음         296.77         296.8     옮긴다
//!   synam-001 pi=229    있음         930.16         930.2     안 옮긴다
//! ```
//!
//! 두 문서 모두 저장 사다리가 문단을 **밴드 바닥 정확히 그 자리**에 둔다(0.04px 차).
//! 갈리는 것은 **host 문단에 보이는 글이 있는가** 하나다 — `synam-001 pi=229` 는
//! host 줄(`vpos=64094`)이 그 스냅을 이미 지고 있어 문단 경로가 제자리에 놓는다.
//! 표 항목까지 따로 옮기면 **이중 적용**이라 host 줄과 표 사이가 벌어진다
//! (`issue_3521_synam001` 핀). 빈 host 는 그 스냅을 질 줄이 없으므로 표 항목
//! 자신이 비켜야 한다.
//!
//! ⚠ 초판은 `MIN_BAND_INTRUSION_PX = 64.0`(24px 에서 synam001 이 깨지고 이 건은
//! 159px 침범하므로 그 사이) 이라는 경험적 문턱을 썼다. 구조에서 유도된 값이 아니고,
//! "밴드 안 0.1px 시작이면 점프 / 밴드 위 63.9px 침범이면 불변" 이라는 불연속도
//! 생겼다. PR #6798 지적에 따라 제거했다 — 새 판정에는 상수도 표 높이 계산도 없다
//! (따라서 `control_index` 로 어느 표 높이를 쓸지 고르는 문제 자체가 사라졌다).
//!
//! ⚠ 여기서 `retain` 으로 밴드를 지우면 안 된다 — 뒤따르는 형제 float 이 아직 그
//! 밴드를 봐야 한다(`#2439` 의 zero-offset 첫 표). 읽기만 한다.
//!
//! 결과: 겹침 1 → **0**, `pi=71` 표가 `174.8 → 296.8`. 쪽수 11 유지.
//! 남는 `overflow` 8건은 이 축과 무관하다(수정 전후 동일).
//!
//! ⚠ 이 문서는 `hancom-office-2010` 저장본이라 저장소 정책상 기준 엔진은 **2020**
//! 이다. 그 기준 PDF 는 아직 없다(작업 PC 에 MCP `.env.local` 부재) — fixture README
//! 참조. 다만 이 축의 판정은 **저장 사다리**가 주므로 버전과 무관하다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6797/156160455-social-pig-farm-income.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6797 정식 HWP fixture 읽기")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

/// 이 쪽 최상위(중첩 아님) 표를 `pi`/`ci` 로 직접 집는다.
///
/// ⚠ 종전 시험은 "표가 둘 이상이고 인접 상자가 안 겹친다" 만 보아, **대상 표가
/// 사라지거나 다른 쪽으로 옮겨가도** 나머지 표 둘로 통과할 수 있었다(PR #6798 지적).
fn top_level_table<'a>(
    column: &'a RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<&'a RenderNode> {
    column.children.iter().find(|c| match &c.node_type {
        RenderNodeType::Table(t) => {
            t.cell_context.is_none()
                && t.para_index == Some(para_index)
                && t.control_index == Some(control_index)
        }
        _ => false,
    })
}

fn page_column(core: &DocumentCore, page: u32) -> RenderNode {
    let tree = core.build_page_render_tree(page).expect("render tree");
    let body = find_body(&tree.root).expect("Body 노드").clone();
    body.children
        .into_iter()
        .find(|c| matches!(c.node_type, RenderNodeType::Column(_)))
        .expect("Column 노드")
}

/// 7쪽 `pi=70 ci=0` 자리차지 표와 `pi=71 ci=0` 표가 **세로로 겹치지 않는다**.
///
/// 두 표를 `pi`/`ci` 로 직접 집어 존재·쪽 귀속·상자 관계를 함께 고정한다.
/// 수정 전: `pi=70` 표 `181.5..294.9` 와 `pi=71` 표 `174.8..340.5` 가 **113.4px** 겹쳤다
/// (겹침 높이 — 두 상자의 세로 교집합).
#[test]
fn table_item_clears_the_previous_float_band() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 11, "쪽수 핀 — 본 수정은 조판 불변");

    let column = page_column(&core, 6);
    let band_owner = top_level_table(&column, 70, 0).expect("7쪽에 pi=70 ci=0 표가 있어야 한다");
    let follower = top_level_table(&column, 71, 0).expect("7쪽에 pi=71 ci=0 표가 있어야 한다");

    let owner_bottom = band_owner.bbox.y + band_owner.bbox.height;
    let overlap = owner_bottom - follower.bbox.y;

    assert!(
        overlap <= 0.5,
        "pi=71 표가 pi=70 자리차지 표의 밴드를 비켜야 한다 — #6797 회귀          (겹침 {overlap:.1}px; pi=70 {:.1}..{:.1}, pi=71 {:.1}..{:.1}; 수정 전 113.4px)",
        band_owner.bbox.y,
        owner_bottom,
        follower.bbox.y,
        follower.bbox.y + follower.bbox.height
    );
}

/// 반대 방향 — **host 에 글이 있는 문단은 옮기지 않는다**.
///
/// `synam-001` 30쪽 `pi=229` 는 host 줄(`vpos=64094` → 절대 930.2)이 저장 스냅을 이미
/// 지고 있고, 그 값이 `pi=228` 자리차지 표가 만든 밴드 바닥(930.2)과 **같다**. 표 항목을
/// 따로 옮기면 이중 적용이라 host 줄과 표 사이가 벌어진다.
///
/// ```text
///                      host 글   저장 vpos(절대)   밴드 바닥   판정
///   156160455 pi=71     없음         296.77         296.8     옮긴다
///   synam-001 pi=229    있음         930.16         930.2     안 옮긴다
/// ```
#[test]
fn a_host_with_text_is_not_moved_by_the_band() {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/synam-001.hwp"))
        .expect("synam-001 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 35, "#3521 전제: synam-001 원본 35쪽");

    let column = page_column(&core, 29);
    let host_line_bottom = {
        let mut bottom = None;
        fn walk(node: &RenderNode, pi: usize, out: &mut Option<f64>) {
            if let RenderNodeType::TextLine(line) = &node.node_type {
                if line.para_index == Some(pi) {
                    let b = node.bbox.y + node.bbox.height;
                    *out = Some(out.map_or(b, |cur: f64| cur.max(b)));
                }
            }
            for child in &node.children {
                walk(child, pi, out);
            }
        }
        walk(&column, 229, &mut bottom);
        bottom.expect("30쪽에 pi=229 host 줄이 있어야 한다")
    };
    let table = top_level_table(&column, 229, 0).expect("30쪽에 pi=229 ci=0 표가 있어야 한다");

    let gap = table.bbox.y - host_line_bottom;
    assert!(
        (0.0..=19.0).contains(&gap),
        "host 줄과 표 사이 간격이 유지돼야 한다 — 밴드 점프 이중 적용 회귀          (간격 {gap:.2}px, host 바닥 {host_line_bottom:.1}, 표 상단 {:.1};           64px 문턱 판에서는 24px 로 낮추자 20.71px 로 벌어졌다)",
        table.bbox.y
    );
}
