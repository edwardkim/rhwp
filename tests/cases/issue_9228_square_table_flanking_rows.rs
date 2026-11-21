//! 관계형·고정형 — [#9228] 어울림 표 양옆 차선에 이어진 뒤 문단은 표 옆에서 자기 줄만 소비한다.
//! 주석 갱신: 2026-11-20 16:36 KST (새 파일)
//!
//! 입력: 공개 보도자료(산업통상자원부 「산업융합 규제샌드박스 500건 돌파」, HWPX, 21쪽)
//! `samples/issue9228/regulatory-sandbox-press-release-flanking-box.hwpx`.
//! 기준: 한컴 Print PDF `pdf/issue9228/regulatory-sandbox-press-release-flanking-box-2020.pdf`(21쪽).
//! 6쪽 `pi=76·78·80·82` 는 글자 없는 문단에 어울림 1×1 표(글자처럼 취급 아님, 세로 문단 기준
//! 280~800HU, 가로 문단 기준 3480~3540HU)를 가진다. host 와 다음 문단(`pi=77·79·81·83`)의 저장
//! 줄은 같은 `vertical_pos` 에 두 조각(왼쪽 차선 `cs=0, sw=가로 오프셋`, 오른쪽 차선
//! `cs=오프셋 + 바깥 여백 + 표 폭`)이고, 저장 사다리는 host 줄만큼만 전진한다(`pi=76` vpos
//! 45493, lh 400, ls 120 → `pi=77` vpos 46013). 수정 전 rhwp 는 뒤 문단을 표 아래로 밀고
//! (`pi=79·81` 은 차선 폭도 잃음) 줄이 겹쳤으며 전체 22쪽이었다.
//! - 고정형: 전체 쪽수 21(= PDF).
//! - 관계형: 6쪽에서 각 host 표의 다음 문단 첫 줄이 표와 세로로 겹치고, 그 줄의 조각이 표
//!   왼쪽·오른쪽에 하나씩 있으며 표와 가로로 겹치는 조각이 없다. 네 문단의 줄은 문서 순서대로
//!   서로 겹치지 않고, 뒤 제목 문단(`pi=85`)은 마지막 표 아래에 있다.
//!
//! 측정(2026-11-20, 이 파일과 같은 커밋의 코드, base = upstream/devel `0a437a7eef`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 21쪽 / 21쪽, 6쪽 99.76, 전쪽 최저 96.43(5쪽). base 는 22쪽(쪽수 불일치), 6쪽 76.43.
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 21쪽 / 21쪽, 6쪽 99.88, 전쪽 최저 96.42(5쪽)
//! - 증거: `mydocs/pr/assets/issue_9228/`(native·wasm 전쪽 TSV, 5·6쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9228/regulatory-sandbox-press-release-flanking-box.hwpx";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 21;
/// 어울림 표의 host 문단. 각 host 의 다음 문단이 표 양옆 차선을 흐른다.
const HOST_PARAS: [usize; 4] = [76, 78, 80, 82];
/// 마지막 표 아래 전폭으로 돌아온 제목 문단.
const TITLE_PARA: usize = 85;

/// 본문 직속 표와 글줄만 모은다(표 안의 줄은 칸 기준 문단 번호라 섞지 않는다).
fn collect_body<'a>(
    node: &'a RenderNode,
    tables: &mut Vec<(usize, BoundingBox)>,
    lines: &mut Vec<(usize, BoundingBox)>,
) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index {
                tables.push((para, node.bbox));
            }
            return;
        }
        RenderNodeType::TextLine(line) => {
            if let Some(para) = line.para_index {
                lines.push((para, node.bbox));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect_body(child, tables, lines);
    }
}

#[test]
fn successor_rows_flow_on_both_sides_of_the_square_table() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    let page = core.build_page_render_tree(5).expect("6쪽");
    let (mut tables, mut lines) = (Vec::new(), Vec::new());
    collect_body(&page.root, &mut tables, &mut lines);
    let lines_of = |para: usize| -> Vec<BoundingBox> {
        lines
            .iter()
            .filter(|(owner, bbox)| *owner == para && bbox.width > 0.5)
            .map(|(_, bbox)| *bbox)
            .collect()
    };

    let mut previous_bottom = f64::NEG_INFINITY;
    let mut last_table_bottom = f64::NEG_INFINITY;
    for host in HOST_PARAS {
        let table = tables
            .iter()
            .find(|(para, _)| *para == host)
            .map(|(_, bbox)| *bbox)
            .unwrap_or_else(|| panic!("6쪽에 pi={host} 의 표가 있어야 한다"));
        let (table_right, table_bottom) = (table.x + table.width, table.y + table.height);
        let successor = lines_of(host + 1);
        assert!(
            !successor.is_empty(),
            "pi={} 의 글줄이 6쪽에 있어야 한다",
            host + 1
        );
        let first_top = successor.iter().map(|b| b.y).fold(f64::INFINITY, f64::min);
        let first_row: Vec<_> = successor
            .iter()
            .filter(|b| (b.y - first_top).abs() < 0.5)
            .collect();
        let first_bottom = first_row[0].y + first_row[0].height;
        assert!(
            first_top < table_bottom && first_bottom > table.y,
            "pi={} 첫 줄 {first_top:.2}~{first_bottom:.2} 은 표 {:.2}~{table_bottom:.2} 와 세로로 겹쳐야 한다",
            host + 1,
            table.y
        );
        assert!(
            first_row.iter().any(|b| b.x + b.width <= table.x + 0.5),
            "pi={} 첫 줄에 표 왼쪽 차선 조각이 있어야 한다",
            host + 1
        );
        assert!(
            first_row.iter().any(|b| b.x + 0.5 >= table_right),
            "pi={} 첫 줄에 표 오른쪽 차선 조각이 있어야 한다",
            host + 1
        );
        for row in &first_row {
            assert!(
                row.x + row.width <= table.x + 0.5 || row.x + 0.5 >= table_right,
                "pi={} 첫 줄 조각 {:.2}~{:.2} 이 표 {:.2}~{table_right:.2} 와 가로로 겹친다",
                host + 1,
                row.x,
                row.x + row.width,
                table.x
            );
        }
        assert!(
            first_top + 0.5 >= previous_bottom,
            "pi={} 첫 줄 위 {first_top:.2} 이 앞 문단 줄 아래 {previous_bottom:.2} 보다 위다",
            host + 1
        );
        previous_bottom = successor
            .iter()
            .map(|b| b.y + b.height)
            .fold(f64::NEG_INFINITY, f64::max);
        last_table_bottom = table_bottom;
    }

    let title = lines_of(TITLE_PARA);
    assert!(
        !title.is_empty(),
        "pi={TITLE_PARA} 제목 줄이 6쪽에 있어야 한다"
    );
    assert!(
        title[0].y + 0.5 >= last_table_bottom && title[0].y + 0.5 >= previous_bottom,
        "제목 줄 위 {:.2} 은 마지막 표 아래 {last_table_bottom:.2} 와 앞 줄 아래 {previous_bottom:.2} 보다 아래여야 한다",
        title[0].y
    );
}
