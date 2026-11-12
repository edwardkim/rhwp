//! 관계형 — [#8931] 칸 안 어울림(Square) 그림 옆에 들어가지 못하는 TAC 표 줄은 그림 아래에서 시작한다.
//! 주석 갱신: 2026-11-11 13:40 KST (새 파일)
//!
//! 입력: 공개 결재문서 36341965(한컴오피스 2020 저장) 원본. 1쪽 문단 0 의 2×1 자리차지 표 아래 칸
//! (세로 가운데 정렬)은 cp0 = 공백 줄 + 어울림 그림(로고, 11498×5016HU, 칸 오른쪽에 붙음),
//! cp1 = 글자처럼 취급 1×1 제목 표(폭 47903HU, 바깥 여백 283HU)다. 표가 그림 왼쪽 틈(약 36771HU)에
//! 들어가지 않으므로 한/글은 cp1 줄을 그림 아래끝에서 시작해 저장한다(cp1 저장 vpos 5016 = 그림 높이).
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8931/36341965-cell-square-picture-tac-title-2020.pdf`(8쪽).
//! - 관계형: 제목 표 위끝 − 그림 아래끝 = 표 위 바깥 여백 283HU(PDF 제목 상자 위끝 약 430px).
//! - 관계형: 제목 표 아래끝 < 뒤 문단 「2026. 6.」 글줄 위끝(내용 순서 보존).
//!
//! 반례: 같은 원본에서 제목 표 폭만 20000HU 로 줄인 변형. 표가 그림 왼쪽 틈에 들어가므로 그림
//! 아래로 내려가지 않고 그림 세로 범위 안(그림 옆)에서 시작하며, 그림과 가로로 겹치지 않는다.
//!
//! 측정(2026-11-11, 코드 head 9b90d28f54, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
//! raw = `silhouette_raw_match_percent`):
//! - Native: 8/8쪽, p1 100.00 · p2 99.82 · p3 99.98 · p4 99.89 · p5 99.91 · p6 99.96 · p7 99.93 ·
//!   p8 99.90, 전쪽 최저 99.82.
//! - fresh WASM: 8/8쪽, p1 100.00 · p2 99.10 · p3 99.96 · p4 99.92 · p5 99.92 · p6 99.94 · p7 99.87 ·
//!   p8 99.91, 전쪽 최저 99.10.
//! - 수정 전(base cd57cc84cf 코드): Native p1 62.56 · fresh WASM p1 62.46, 나머지 쪽 동일.
//! - 증거: `mydocs/pr/assets/issue_8931/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8931/36341965-cell-square-picture-tac-title.hwpx";
const TOLERANCE_PX: f64 = 0.5;
const TITLE_TABLE_SZ: &str = r#"<hp:sz width="47903" widthRelTo="ABSOLUTE" height="7031""#;
const TITLE_CELL_SZ: &str = r#"<hp:cellSz width="47903" height="7031"/>"#;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// 제목 표(와 그 칸) 폭만 20000HU 로 줄여 그림 왼쪽 틈에 들어가게 한 HWPX.
fn with_narrow_title_table() -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(sample_bytes())).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        let name = entry.name().expect("name").to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).expect("utf-8");
            assert_eq!(xml.matches(TITLE_TABLE_SZ).count(), 1, "원본 제목 표 크기");
            assert_eq!(
                xml.matches(TITLE_CELL_SZ).count(),
                1,
                "원본 제목 표 칸 크기"
            );
            data = xml
                .replace(TITLE_TABLE_SZ, &TITLE_TABLE_SZ.replace("47903", "20000"))
                .replace(TITLE_CELL_SZ, &TITLE_CELL_SZ.replace("47903", "20000"))
                .into_bytes();
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        out.start_file(name, options).expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

fn text_of(node: &RenderNode) -> String {
    nodes(node)
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

struct TitleLayout {
    picture: BoundingBox,
    title: BoundingBox,
    date_line_top: f64,
}

fn title_layout(bytes: &[u8], label: &str) -> TitleLayout {
    let core = DocumentCore::from_bytes(bytes).unwrap_or_else(|e| panic!("open {label}: {e:?}"));
    let page = core.build_page_render_tree(0).expect("1쪽");
    let all = nodes(&page.root);
    let pictures: Vec<_> = all
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        .collect();
    assert_eq!(pictures.len(), 1, "{label}: 1쪽 그림은 칸 안 로고 하나");
    let titles: Vec<_> = all
        .iter()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::Table(_))
                && text_of(n).starts_with("2026년 새활용")
        })
        .collect();
    assert_eq!(titles.len(), 1, "{label}: 제목 표 하나");
    let date_line_top = all
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextLine(_) if text_of(n).trim() == "2026. 6." => Some(n.bbox.y),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{label}: 「2026. 6.」 글줄"));
    TitleLayout {
        picture: pictures[0].bbox,
        title: titles[0].bbox,
        date_line_top,
    }
}

#[test]
fn tac_title_table_that_does_not_fit_beside_square_picture_starts_below_it() {
    let layout = title_layout(&sample_bytes(), "원본");
    let picture_bottom = layout.picture.y + layout.picture.height;
    let gap = layout.title.y - picture_bottom;
    let margin = hu_to_px(283);
    assert!(
        (gap - margin).abs() <= TOLERANCE_PX,
        "제목 표 위끝이 그림 아래끝에서 {gap:.2}px 떨어졌다 — 표 위 바깥 여백 {margin:.2}px 이어야 한다"
    );
    assert!(
        layout.title.y + layout.title.height < layout.date_line_top,
        "제목 표 아래끝 {:.2} 은 뒤 문단 「2026. 6.」 위끝 {:.2} 보다 위여야 한다",
        layout.title.y + layout.title.height,
        layout.date_line_top
    );
}

#[test]
fn tac_title_table_that_fits_beside_square_picture_stays_beside_it() {
    let layout = title_layout(&with_narrow_title_table(), "좁은 제목 표 변형");
    let picture_bottom = layout.picture.y + layout.picture.height;
    assert!(
        layout.title.y < picture_bottom - TOLERANCE_PX,
        "그림 왼쪽 틈에 들어가는 표 위끝 {:.2} 이 그림 아래끝 {picture_bottom:.2} 으로 내려갔다",
        layout.title.y
    );
    assert!(
        layout.title.x + layout.title.width <= layout.picture.x + TOLERANCE_PX,
        "그림 옆에 남은 표 오른끝 {:.2} 이 그림 왼끝 {:.2} 을 넘는다",
        layout.title.x + layout.title.width,
        layout.picture.x
    );
}
