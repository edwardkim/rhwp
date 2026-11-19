//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-19 18:07:43 KST (Asia/Seoul).
//! 직전 파일 변경: 2c5461ff240433ec1c5788e9eed4705f52d3dbfd 2026-11-19T07:36:37+09:00.
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
//! 측정일: 2026-11-19. 소스 2c5461ff24(코드), Native release-test native-skia
//! SHA-256 ae3fd109ac31a5e96144806eeebd3d62d96eee05689eb74452559f31e465216f,
//! fresh WASM `rhwp_bg.wasm` SHA-256 dab60baecf026184632f99e840b963eb3894531825811143ed9f08db5d539b2c.
//! 방법: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 색상 경계 보정 전 2px raw.
//! 글꼴: `RHWP_FONT_PATH` 로 이 문서가 쓰는 설치 글꼴만 모은 폴더를 공급(`direct-font-programs`,
//! Native 38개·WASM 34개 파일, Native 글꼴 환경 SHA-256 a978c6dc74e88d18…). 회귀 실행에는 요구하지 않는다.
//! 전체 159쪽(독립 PDF·Native·WASM 같음). 대상 쪽 Native/fresh WASM: 79쪽 99.96/99.95%,
//! 147쪽 99.68/99.94%, 148쪽 99.46/99.91%, 149쪽 99.47/99.86%, 150쪽 99.61/99.94%,
//! 151쪽 99.43/99.87%, 152쪽 99.44/99.79%.
//! 전쪽 최저: Native 90.30%(70쪽), fresh WASM 90.48%(70쪽). 90% 미만 쪽은 없다.
//! 증거: `mydocs/pr/assets/issue_9060_oda/{native,wasm}/oda_silhouette.tsv`,
//! 79·148·152쪽 review·overlay `mydocs/pr/assets/issue_9060_oda/{native,wasm}/oda_{review,overlay}_*.png`.
//!
//! 고정형은 독립 PDF의 전체 쪽수 159다.
//! 관계형은 (1) 2구역 문단 76(빈 호스트, 위 캡션, 문단 기준 세로 오프셋 2668HU의 2×2 표)의 쪽
//! 소유, 본문 위끝·캡션·표 위끝 순서와 거리 대 원본 오프셋, 앞 표(문단 75)와 뒤 문단(77·78)의
//! 쪽 소속이다. 147쪽 바닥은 문단 75 표가 채워 문단 76 표의 첫 행이 `문단 위치 + 오프셋`
//! 자리에 들어가지 않는다. 한/글은 표를 148쪽 본문 위끝 + 바깥 위 여백 + 위 캡션에서 연다.
//! 수정 전에는 오프셋을 새 쪽에서 다시 더해 표가 그만큼 내려가고 문서가 160쪽이 됐다.
//! (2) 문단 80(빈 호스트 표)의 호스트 글줄·빈 문단 81 이 150쪽에 남고, 152쪽 문단 82 가 표 끝
//! 조각 + 원본 바깥 아래 여백에서 시작한다(수정 전 81 이 152쪽에서 9.5px 를 점유).
//! (3) 79쪽 <표 3-23>(글자처럼 취급, 쪽을 채움)의 캡션 위끝 = 본문 위끝 + 저장 첫 줄 vpos(수정 전 0).
//! (4) 70쪽 <표 3-21>은 저장 줄보다 넓어도 바깥 왼쪽 여백을 유지한다. 위의 좁은 표는
//! 남는 폭의 절반을 사용한다. 같은 독립 Print와 보정 코드 b0ce128792의 Native 99.92658%,
//! fresh WASM 99.92923%를 96 DPI raw로 실측했다. 전 159쪽 최저는 각각 96.89%, 96.93%다.
//! 70쪽 판독에서 아래 표의 제목과 왼쪽 괘선이 같은 위치로 돌아왔음을 확인했다.
//! 실물의 절대 px 좌표를 고정하지 않으며 회귀 실행에 설치된 OTF·시스템 글꼴을 요구하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::Alignment;
use rhwp::renderer::hwpunit_to_px;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

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
    body: Option<BoundingBox>,
    /// 표 밖 본문 글줄(문단, 위끝, 아래끝). 표 캡션 글줄도 여기에 든다.
    lines: Vec<(usize, f64, f64)>,
    /// 최상위 표(문단, 배치 영역).
    tables: Vec<(usize, BoundingBox)>,
}

impl Page {
    fn of(core: &DocumentCore, page: u32) -> Self {
        fn walk(node: &RenderNode, in_table: bool, out: &mut Page) {
            let mut inside = in_table;
            match &node.node_type {
                RenderNodeType::Body { .. } if out.body.is_none() => {
                    out.body = Some(node.bbox);
                }
                RenderNodeType::Header | RenderNodeType::Footer => return,
                RenderNodeType::Table(table) => {
                    if let Some(para) = table.para_index.filter(|_| !in_table) {
                        out.tables.push((para, node.bbox));
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

    fn body_vertical_bounds(&self) -> Option<(f64, f64)> {
        self.body.map(|bbox| (bbox.y, bbox.y + bbox.height))
    }

    fn table(&self, para: usize) -> Option<(f64, f64)> {
        self.tables
            .iter()
            .filter(|(p, _)| *p == para)
            .map(|(_, bbox)| (bbox.y, bbox.y + bbox.height))
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
    let (_, previous_body_bottom) = previous.body_vertical_bounds().expect("147쪽 본문 영역");
    assert!(
        previous.table(TABLE_PARA).is_none(),
        "문단 76 표는 147쪽에 조각을 두지 않는다"
    );
    assert!(
        previous_body_bottom - previous_bottom < offset + outer_top,
        "147쪽에 남은 공간은 오프셋 + 바깥 위 여백보다 작다"
    );

    let page = Page::of(&core, TABLE_PAGE);
    let (body_top, body_bottom) = page.body_vertical_bounds().expect("148쪽 본문 영역");
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

/// [#9060] 빈 호스트 표(2구역 문단 80)의 첫 유닛이 위 캡션까지 150쪽에 들어가지 않으면 표만
/// 151쪽에서 열리고, 저장 줄이 150쪽 좌표에 있는 빈 호스트 글줄과 뒤 빈 문단 81 은 150쪽에
/// 남는다. 152쪽에서는 문단 82 가 표 끝 조각 바로 아래(바깥 아래 여백 뒤)에서 시작한다.
#[test]
fn empty_host_successor_stays_with_the_host_and_next_paragraph_follows_the_table_end() {
    const HOST: usize = 80;
    const EMPTY_SUCCESSOR: usize = 81;
    const RESTART: usize = 82;
    let core = document();
    let host = &core.document().sections[SECTION].paragraphs[HOST];
    let [Control::Table(table)] = host.controls.as_slice() else {
        panic!("문단 80 컨트롤: {}", host.controls.len());
    };
    assert!(host.text.is_empty() && table.caption.is_some());
    // 저장 사다리: 문단 81 은 호스트 뒤 같은 쪽 좌표, 문단 82 는 새 쪽으로 되감긴다.
    let vpos =
        |para: usize| core.document().sections[SECTION].paragraphs[para].line_segs[0].vertical_pos;
    assert!(vpos(EMPTY_SUCCESSOR) > vpos(HOST) && vpos(RESTART) < vpos(HOST));
    let outer_bottom = f64::from(table.outer_margin_bottom) * 96.0 / 7200.0;

    let host_page = Page::of(&core, 149);
    assert!(host_page.table(HOST).is_none(), "150쪽에는 표 조각이 없다");
    let (_, body_bottom) = host_page.body_vertical_bounds().expect("150쪽 본문 영역");
    let host_line = host_page
        .lines
        .iter()
        .find(|(p, _, _)| *p == HOST)
        .expect("150쪽에 빈 호스트 글줄이 있다");
    let successor = host_page
        .lines
        .iter()
        .find(|(p, _, _)| *p == EMPTY_SUCCESSOR)
        .expect("150쪽에 빈 문단 81 글줄이 있다");
    assert!(
        host_line.2 <= successor.1 + 0.5,
        "호스트 글줄 뒤에 문단 81 이 온다"
    );
    assert!(
        successor.2 <= body_bottom + 0.5,
        "문단 81 글줄은 150쪽 본문 안이다"
    );

    assert!(
        Page::of(&core, 150).table(HOST).is_some(),
        "표는 151쪽에서 시작한다"
    );
    let end_page = Page::of(&core, 151);
    let (_, table_bottom) = end_page.table(HOST).expect("152쪽에 표 끝 조각이 있다");
    assert!(
        !end_page.has_line(EMPTY_SUCCESSOR),
        "문단 81 은 152쪽 표 뒤를 점유하지 않는다"
    );
    let restart_top = end_page
        .lines
        .iter()
        .filter(|(p, _, _)| *p == RESTART)
        .map(|(_, top, _)| *top)
        .reduce(f64::min)
        .expect("152쪽에 문단 82 가 있다");
    assert!(
        (restart_top - table_bottom - outer_bottom).abs() <= 1.0,
        "문단 82 글줄은 표 끝 조각 + 바깥 아래 여백({outer_bottom:.2}px)에서 시작한다: {:.2}px",
        restart_top - table_bottom
    );
}

/// [#9060] 쪽을 채우는 글자처럼 취급한 표(1구역 문단 808 <표 3-23>)도 단 맨 위에서 저장 첫 줄의
/// 앞 간격(vpos)만큼 내려 시작한다. 다음 문단은 새 쪽 첫머리에서 같은 「쪽 위끝 + 앞 간격」
/// 인코딩으로 다시 시작해 그 원점이 쪽 기준임을 입증한다.
#[test]
fn page_filling_inline_table_keeps_its_stored_page_top_margin() {
    const TAC_PARA: usize = 808;
    let core = document();
    let para = &core.document().sections[0].paragraphs[TAC_PARA];
    let [Control::Table(table)] = para.controls.as_slice() else {
        panic!("문단 808 컨트롤: {}", para.controls.len());
    };
    assert!(table.common.treat_as_char && table.caption.is_some());
    let stored_top = f64::from(para.line_segs[0].vertical_pos) * 96.0 / 7200.0;
    assert!(
        stored_top > 0.0,
        "저장 첫 줄은 쪽 위끝에서 앞 간격만큼 아래다"
    );
    let page = Page::of(&core, 78);
    let (body_top, body_bottom) = page.body_vertical_bounds().expect("79쪽 본문 영역");
    let (top, bottom) = page.table(TAC_PARA).expect("79쪽에 <표 3-23> 이 있다");
    let caption_top = page
        .lines
        .iter()
        .filter(|(_, line_top, _)| *line_top < top)
        .map(|(_, line_top, _)| *line_top)
        .reduce(f64::min)
        .expect("표 위에 위 캡션 글줄이 있다");
    assert!(
        (caption_top - body_top - stored_top).abs() <= 0.5,
        "캡션·표 묶음은 본문 위끝 + 저장 앞 간격({stored_top:.2}px)에서 시작한다: {:.2}px",
        caption_top - body_top
    );
    assert!(bottom <= body_bottom + 0.5, "표는 79쪽 본문 안에서 끝난다");
    assert!(
        Page::of(&core, 79).table(TAC_PARA).is_none(),
        "<표 3-23> 은 80쪽으로 이어지지 않는다"
    );
}

/// <표 3-21>의 바깥 여백을 포함한 폭은 저장 글줄을 넘는다.
/// 표가 왼쪽 여백을 침범하지 않고 위의 좁은 표는 가운데 정렬을 유지한다.
#[test]
fn oversized_centered_inline_table_starts_at_the_stored_line_left() {
    let core = document();
    let page = Page::of(&core, 69);
    let body_left = page.body.expect("70쪽 본문 영역").x;
    let doc = core.document();
    let paragraphs = &doc.sections[0].paragraphs;
    let px = |hu: i32| hwpunit_to_px(hu, 96.0);

    for para_index in [732, 733] {
        let host = &paragraphs[para_index];
        assert_eq!(
            doc.doc_info.para_shapes[usize::from(host.para_shape_id)].alignment,
            Alignment::Center
        );
        let [Control::Table(table)] = host.controls.as_slice() else {
            panic!("문단 {para_index}에 표 하나가 있다");
        };
        assert!(table.common.treat_as_char);
        let [line] = host.line_segs.as_slice() else {
            panic!("문단 {para_index}에 저장 글줄 하나가 있다");
        };
        let occupied = i32::try_from(table.common.width).expect("표의 원본 폭")
            + i32::from(table.outer_margin_left)
            + i32::from(table.outer_margin_right);
        let spare = line.segment_width - occupied;
        let centering = if para_index == 732 {
            assert!(spare > 0, "위 표는 저장 글줄보다 좁다");
            px(spare) / 2.0
        } else {
            assert!(spare < 0, "<표 3-21>은 저장 글줄보다 넓다");
            0.0
        };
        let bbox = page
            .tables
            .iter()
            .find(|(owner, _)| *owner == para_index)
            .map(|(_, bbox)| bbox)
            .expect("70쪽의 해당 본문 표");
        let expected_left =
            body_left + px(line.column_start + i32::from(table.outer_margin_left)) + centering;
        assert!(
            (bbox.x - expected_left).abs() <= px(1),
            "본문 시작·저장 글줄·바깥 왼쪽 여백에 따른 문단 {para_index} 표 위치: 기대 {expected_left:.3}px, 실제 {:.3}px",
            bbox.x
        );
    }
}
