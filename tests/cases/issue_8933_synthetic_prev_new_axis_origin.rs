//! 관계형·고정형 — [#8933] 저장 줄 없는 합성 문단 뒤 새 원본 축의 단 원점과 빈 줄 끝 간격.
//! 주석 갱신: 2026-11-11 14:47 KST (새 파일)
//!
//! 입력: 공개 결재문서 36495398(한컴오피스 2020 저장 HWPX)의 글꼴 대체본. 이 환경에 없는
//! `HY신명조`·`HY중고딕` 두 face 를 문서가 이미 쓰는 `HY견명조`·`HY견고딕` 으로 바꿨다
//! (`Contents/header.xml` 의 `hh:font face` 14곳, 그 밖의 저장 정보는 원본 그대로).
//! 구역 1 의 4쪽 첫 문단 `pi=21` 은 저장 줄이 없어 로드 때 줄을 합성하며 그 vpos 는 앞 쪽
//! 누적값이다. 다음 표 문단 `pi=22` 는 저장 vpos 3720 으로 새 원본 축을 연다. 빈 문단
//! `pi=23`·`pi=24` 와 `pi=25` 는 저장 사다리(vpos + lh + ls = 다음 vpos)로 이어진다.
//!
//! 기준: 대체본을 한컴 engine 2020 으로 Print 한 PDF
//! `pdf/issue8933/36495398-synthetic-prev-new-axis-2020.pdf`(5쪽). 4쪽 글자 기준선은 원본의
//! engine 2020 PDF 와 `pi=28` 둘째 줄까지 같다(`pi=25` 495.8px).
//! - 고정형: 전체 쪽수 5.
//! - 관계형: 4쪽 본문 글줄 `pi=23`→`pi=24`→`pi=25` 의 위끝 차이 = 각 저장 vpos 차이.
//!   표 `pi=22` 는 `pi=23` 보다 위에 있고 겹치지 않는다.
//!
//! 측정(2026-11-11, 코드 커밋 60c35ae73f(사슬 #8928 cd57cc84cf 위),
//! `scripts/visual_sweep.py --silhouette-only --dpi 96`, raw = `silhouette_raw_match_percent`,
//! 문서 face 만 모은 글꼴 폴더를 `RHWP_FONT_PATH`·`--font-path` 로 공급):
//! - Native: 5/5쪽, 1쪽 100.00 · 2쪽 99.93 · 3쪽 99.98 · 4쪽 100.00 · 5쪽 99.68, 전쪽 최저 99.68.
//! - fresh WASM(wasm sha256 a283edcd…): 5/5쪽, 1쪽 100.00 · 2쪽 100.00 · 3쪽 97.14 · 4쪽 95.68 ·
//!   5쪽 100.00, 전쪽 최저 95.68. 4쪽 `pi=32` 이후 글자는 render tree 가 Native 와 같고 Canvas
//!   글리프만 4px 위에 그려진다(수정 전 WASM 에도 같은 4px, 이 변경과 무관).
//! - 수정 전(base cd57cc84cf) 4쪽: Native 62.71, fresh WASM(sha256 6df776c1…) 65.28 —
//!   `pi=24` 이후 전부 7.2px(`pi=23` 끝 줄간격 540HU) 아래.
//! - 증거: `mydocs/pr/assets/issue_8933/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8933/36495398-synthetic-prev-new-axis.hwpx";
const SECTION: usize = 1;
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn document() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

/// 표 안으로 내려가지 않고 본문 글줄(문단 번호, 줄 위끝)과 본문 표(문단 번호, 위끝, 아래끝)를 모은다.
fn collect(node: &RenderNode, lines: &mut Vec<(usize, f64)>, tables: &mut Vec<(usize, f64, f64)>) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if table.section_index == Some(SECTION) {
                if let Some(para) = table.para_index {
                    tables.push((para, node.bbox.y, node.bbox.y + node.bbox.height));
                }
            }
            return;
        }
        RenderNodeType::TextLine(line) if line.section_index == Some(SECTION) => {
            if let Some(para) = line.para_index {
                lines.push((para, node.bbox.y));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, lines, tables);
    }
}

#[test]
fn stored_axis_after_synthetic_page_head_keeps_empty_line_spacing() {
    let core = document();
    assert_eq!(core.page_count(), 5, "한컴 engine 2020 Print PDF 는 5쪽");

    let paragraphs = &core.document().sections[SECTION].paragraphs;
    let first = |pi: usize| -> &LineSeg {
        paragraphs[pi]
            .line_segs
            .first()
            .unwrap_or_else(|| panic!("pi={pi} 줄"))
    };
    assert_ne!(
        first(21).tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY,
        0,
        "원본 전제: pi=21 은 저장 줄 없는 합성 문단"
    );
    assert!(
        first(22).vertical_pos < first(21).vertical_pos,
        "원본 전제: pi=22 는 합성 누적 축보다 작은 저장 vpos 로 새 축을 연다"
    );
    for pi in 23..=24 {
        let last = paragraphs[pi].line_segs.last().expect("저장 줄");
        assert_eq!(
            last.vertical_pos + last.line_height + last.line_spacing,
            first(pi + 1).vertical_pos,
            "원본 전제: pi={pi} 끝에서 pi={} 저장 vpos 로 이어진다",
            pi + 1
        );
    }

    let tree = core.build_page_render_tree(3).expect("4쪽");
    let (mut lines, mut tables) = (Vec::new(), Vec::new());
    collect(&tree.root, &mut lines, &mut tables);
    let top = |pi: usize| {
        lines
            .iter()
            .filter(|(p, _)| *p == pi)
            .map(|(_, y)| *y)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or_else(|| panic!("4쪽 pi={pi} 글줄: {lines:?}"))
    };
    let (_, _, table_bottom) = tables
        .iter()
        .find(|(p, _, _)| *p == 22)
        .copied()
        .unwrap_or_else(|| panic!("4쪽 pi=22 표: {tables:?}"));
    assert!(
        table_bottom <= top(23),
        "표 pi=22 아래끝 {table_bottom:.2} 는 pi=23 위끝 {:.2} 보다 위",
        top(23)
    );
    for pi in 23..=24 {
        let gap = top(pi + 1) - top(pi);
        let stored = hu_to_px(first(pi + 1).vertical_pos - first(pi).vertical_pos);
        assert!(
            (gap - stored).abs() <= TOLERANCE_PX,
            "pi={pi}→pi={} 위끝 차이 {gap:.2}px 는 저장 vpos 차이 {stored:.2}px 와 같아야 한다\
             (끝 줄간격을 한 번만 소비)",
            pi + 1
        );
    }
}
