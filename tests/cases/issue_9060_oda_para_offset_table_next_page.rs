//! 검사 유형: 관계형·고정형.
//! 주석 갱신: (측정 후 기록).
//!
//! 입력: `samples/issue9060/oda-1092000-201600017.hwp`(정책연구관리시스템 PRISM 공개 보고서,
//! 국무조정실 「한국 ODA 조달시스템 평가 연구」, 한컴오피스 2010 8.5.8.1485 저장).
//! 입력 SHA-256: 478e77d783cedaccc56a27dc9efbd9eeed2cf7008ecb1b1c6dc71318cd4c5107.
//! 독립 Print: `pdf/issue9060/oda-1092000-201600017-2020.pdf`, 159쪽.
//! PDF SHA-256: 4d61e6a3f511950dec964522c8227aa04fa190b02a81e4e87de361bd3725d448.
//! PDF 출처: 저장 제품 2010 은 MCP engine 2020 버킷이다. engine 2020(HOffice110 11.0.0.9136) Print,
//! `pdf_print_method` 0, `hancom2020_pdf_driver_one_up`, job 871b8b59-3e6f-4e08-98b5-b8226006d3db
//! (첫 job 32c1b42e 는 worker 비정상 종료로 실패해 같은 engine 으로 한 번 다시 실행했다).
//! Creator Hwp 2020. 이슈 첨부 Hwp 2022 12.0.0.4547 Print(159쪽)와 148쪽 표 위 괘선이 같다.
//!
//! 측정일: (측정 후 기록).
//!
//! 고정형은 독립 PDF의 전체 쪽수 159다.
//! 관계형은 2구역 문단 76(빈 호스트, 위 캡션, 문단 기준 세로 오프셋 2668HU의 2×2 표)의 쪽
//! 소유, 본문 위끝·캡션·표 위끝 순서와 거리 대 원본 오프셋, 앞 표(문단 75)와 뒤 문단(77·78)의
//! 쪽 소속이다. 147쪽 바닥은 문단 75 표가 채워 문단 76 표의 첫 행이 `문단 위치 + 오프셋`
//! 자리에 들어가지 않는다. 한/글은 표를 148쪽 본문 위끝 + 바깥 위 여백 + 위 캡션에서 연다.
//! 수정 전에는 오프셋을 새 쪽에서 다시 더해 표가 그만큼 내려가고 첫 조각이 한 줄 짧아져
//! 문서가 160쪽이 됐다.
//! 실물의 절대 px 좌표를 고정하지 않으며 회귀 실행에 설치된 OTF·시스템 글꼴을 요구하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9060/oda-1092000-201600017.hwp";
const SECTION: usize = 2;
const PREVIOUS_TABLE_PARA: usize = 75;
const TABLE_PARA: usize = 76;
/// 표가 처음 나타나는 148쪽(0 기준 147).
const TABLE_PAGE: u32 = 147;

fn document() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

#[derive(Default)]
struct Page {
    body: Option<(f64, f64)>,
    /// 표 밖 본문 글줄(문단, 위끝, 아래끝). 표 캡션 글줄도 여기에 든다.
    lines: Vec<(usize, f64, f64)>,
    /// 최상위 표(문단, 위끝, 아래끝).
    tables: Vec<(usize, f64, f64)>,
}

impl Page {
    fn of(core: &DocumentCore, page: u32) -> Self {
        fn walk(node: &RenderNode, in_table: bool, out: &mut Page) {
            let mut inside = in_table;
            match &node.node_type {
                RenderNodeType::Body { .. } if out.body.is_none() => {
                    out.body = Some((node.bbox.y, node.bbox.y + node.bbox.height));
                }
                RenderNodeType::Header | RenderNodeType::Footer => return,
                RenderNodeType::Table(table) => {
                    if let Some(para) = table.para_index.filter(|_| !in_table) {
                        out.tables
                            .push((para, node.bbox.y, node.bbox.y + node.bbox.height));
                    }
                    inside = true;
                }
                RenderNodeType::TextLine(line) if !in_table => {
                    if let Some(para) = line.para_index {
                        out.lines
                            .push((para, node.bbox.y, node.bbox.y + node.bbox.height));
                    }
                }
                _ => {}
            }
            for child in &node.children {
                walk(child, inside, out);
            }
        }
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page + 1));
        let mut out = Page::default();
        walk(&tree.root, false, &mut out);
        out
    }

    fn table(&self, para: usize) -> Option<(f64, f64)> {
        self.tables
            .iter()
            .filter(|(p, _, _)| *p == para)
            .map(|(_, top, bottom)| (*top, *bottom))
            .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
    }

    fn has_line(&self, para: usize) -> bool {
        self.lines.iter().any(|(p, _, _)| *p == para)
    }
}

/// 원본 표의 문단 기준 세로 오프셋과 바깥 위 여백(px, 96 DPI).
fn offset_and_outer_top_px(core: &DocumentCore) -> (f64, f64) {
    let host = &core.document().sections[SECTION].paragraphs[TABLE_PARA];
    assert!(host.text.is_empty(), "문단 76 은 빈 호스트다");
    let [Control::Table(table)] = host.controls.as_slice() else {
        panic!("문단 76 컨트롤: {}", host.controls.len());
    };
    assert!(!table.common.treat_as_char, "자리차지 개체");
    assert!(table.caption.is_some(), "위 캡션이 있다");
    let offset = table.common.vertical_offset as i32;
    assert_eq!(offset, 2668, "원본 문단 기준 세로 오프셋");
    let px = |hu: i32| f64::from(hu) * 96.0 / 7200.0;
    (px(offset), px(i32::from(table.outer_margin_top)))
}

#[test]
fn page_count_matches_hancom_print() {
    assert_eq!(
        document().page_count(),
        159,
        "첨부 독립 한컴 Print PDF는 159쪽"
    );
}

/// [#9060] 앞 표가 147쪽 바닥을 채우면 문단 76 표는 148쪽에서 오프셋 없이 시작한다.
#[test]
fn table_opens_next_page_at_body_top_without_paragraph_offset() {
    let core = document();
    let (offset, outer_top) = offset_and_outer_top_px(&core);

    let previous = Page::of(&core, TABLE_PAGE - 1);
    let (_, previous_bottom) = previous
        .table(PREVIOUS_TABLE_PARA)
        .expect("147쪽에 문단 75 표의 마지막 조각이 있다");
    let (_, previous_body_bottom) = previous.body.expect("147쪽 본문 영역");
    assert!(
        previous.table(TABLE_PARA).is_none(),
        "문단 76 표는 147쪽에 조각을 두지 않는다"
    );
    assert!(
        previous_body_bottom - previous_bottom < offset + outer_top,
        "147쪽에 남은 공간은 오프셋 + 바깥 위 여백보다 작다"
    );

    let page = Page::of(&core, TABLE_PAGE);
    let (body_top, body_bottom) = page.body.expect("148쪽 본문 영역");
    let (top, bottom) = page
        .table(TABLE_PARA)
        .expect("148쪽에 문단 76 표의 첫 조각이 있다");
    assert!(
        page.table(PREVIOUS_TABLE_PARA).is_none(),
        "문단 75 표는 148쪽으로 이어지지 않는다"
    );
    // 표 위에는 캡션 글줄만 있다. 앵커 문단은 빈 호스트라 148쪽 표 위를 차지하지 않는다.
    let above: Vec<_> = page
        .lines
        .iter()
        .filter(|(_, line_top, _)| *line_top < top)
        .collect();
    assert_eq!(
        above.len(),
        1,
        "148쪽 표 위의 글줄은 위 캡션 하나다: {above:?}"
    );
    let (_, caption_top, caption_bottom) = *above[0];
    assert!(
        caption_top >= body_top + outer_top - 0.5,
        "캡션은 본문 위끝 + 바깥 위 여백 아래에서 시작한다"
    );
    assert!(caption_bottom <= top + 0.5, "캡션은 표 위에서 끝난다");
    assert!(
        caption_top - body_top < offset,
        "캡션·표 묶음은 문단 오프셋을 다시 더하지 않는다: 본문 위끝에서 {:.2}px, 오프셋 {offset:.2}px",
        caption_top - body_top
    );
    assert!(
        bottom <= body_bottom + 0.5,
        "148쪽 표 조각은 본문 안에서 끝난다"
    );
}

/// 뒤 문단의 쪽 소속: 표는 148·149쪽에 걸치고 문단 77·78 은 149쪽에서 표 뒤에 시작한다.
#[test]
fn table_continuation_and_following_paragraphs_keep_page_ownership() {
    let core = document();
    let next = Page::of(&core, TABLE_PAGE + 1);
    let (_, rest_bottom) = next
        .table(TABLE_PARA)
        .expect("149쪽에 문단 76 표의 나머지 조각이 있다");
    for para in [77, 78] {
        assert!(next.has_line(para), "문단 {para} 는 149쪽에 있다");
        assert!(
            next.lines
                .iter()
                .filter(|(p, _, _)| *p == para)
                .all(|(_, line_top, _)| *line_top >= rest_bottom - 0.5),
            "문단 {para} 글줄은 표 나머지 조각 아래에 있다"
        );
    }
    for page in [TABLE_PAGE - 1, TABLE_PAGE] {
        let owner = Page::of(&core, page);
        assert!(
            !owner.has_line(77) && !owner.has_line(78),
            "문단 77·78 은 {}쪽에 없다",
            page + 1
        );
    }
    assert!(
        Page::of(&core, TABLE_PAGE + 2).table(TABLE_PARA).is_none(),
        "문단 76 표는 150쪽까지 이어지지 않는다"
    );
}
