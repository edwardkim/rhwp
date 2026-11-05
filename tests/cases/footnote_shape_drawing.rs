//! 관계형 및 고정형 회귀: 각주 번호 크기 비율·선 종류와 독립 Print 4·5·6쪽의 구분선·번호 끝 좌표. 기존 오차는 0.5 CSSpx다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! shape.hwp (8538-shape-hwp-2022): 전 7쪽, Native 최저 92.35136%, fresh WASM 최저 92.4024%.
//! 고정 4쪽: Native 100.0%, fresh WASM 100.0%; 같은 영역의 독립 Print 확대 판독 완료.
//! 고정 5쪽: Native 100.0%, fresh WASM 100.0%; 같은 영역의 독립 Print 확대 판독 완료.
//! 고정 6쪽: Native 100.0%, fresh WASM 100.0%; 같은 영역의 독립 Print 확대 판독 완료.
//! shape.hwpx (8538-shape-hwpx-2022): 전 7쪽, Native 최저 92.35136%, fresh WASM 최저 92.4024%.
//! 고정 4쪽: Native 100.0%, fresh WASM 100.0%; 같은 영역의 독립 Print 확대 판독 완료.
//! 고정 5쪽: Native 100.0%, fresh WASM 100.0%; 같은 영역의 독립 Print 확대 판독 완료.
//! 고정 6쪽: Native 100.0%, fresh WASM 100.0%; 같은 영역의 독립 Print 확대 판독 완료.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 각주 모양의 그리기: 각주 세로 위치(본문 아래), 각주 내용 번호 속성(작게), 구분선 종류.
//!
//! 시험 문서는 한/글 12.30(Mac)이 연 뒤 HWP·HWPX로 저장한 7구역 문서다(구역마다 한 쪽).
//! 한/글 인쇄 PDF(`pdf/footnote_drawing/`)에서 잰 값을 쓴다(1pt = 4/3px).
//! - 4쪽(본문 아래): 구분선이 본문 마지막 글줄 하단(157.2pt) + 구분선 위 8.5pt = 165.7pt에
//!   있다. 꼬리말 바로 위(5·6쪽)는 쪽 아래에 둔다.
//! - 5쪽(작게): 각주 내용 번호 `1)`이 본문의 각주 표시처럼 글자 크기의 3/4이고 90.95pt에서
//!   끝난다(보통인 6쪽은 92.88pt).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::StrokeDash;

const DIR: &str = "tests/fixtures/footnote_drawing";
const PX_PER_PT: f64 = 96.0 / 72.0;

fn read(name: &str) -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(DIR).join(name)).unwrap()
}

fn footnote_area(core: &DocumentCore, page: u32) -> RenderNode {
    let tree = core.build_page_render_tree(page).expect("render tree");
    tree.root
        .children
        .into_iter()
        .find(|node| matches!(node.node_type, RenderNodeType::FootnoteArea))
        .expect("footnote area")
}

fn separator(area: &RenderNode) -> (f64, StrokeDash) {
    area.children
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Line(line) => Some((line.y1, line.style.dash)),
            _ => None,
        })
        .expect("separator")
}

/// 첫 각주 글줄의 글자 run (글자, 끝 x, 글자 크기)
fn first_line_runs(area: &RenderNode) -> Vec<(String, f64, f64)> {
    let line = area
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .expect("text line");
    line.children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some((
                run.display_or_text().trim().to_string(),
                node.bbox.x + node.bbox.width,
                run.style.font_size,
            )),
            _ => None,
        })
        .filter(|(text, _, _)| !text.is_empty())
        .collect()
}

#[test]
fn footnotes_below_text_start_under_the_last_body_line() {
    for name in ["shape.hwp", "shape.hwpx"] {
        let core = DocumentCore::from_bytes(&read(name)).expect("open");
        let (below_text, _) = separator(&footnote_area(&core, 3));
        assert!(
            (below_text - 165.7 * PX_PER_PT).abs() < 0.5,
            "{name} 본문 아래 구분선 y={below_text}"
        );
        // 꼬리말 바로 위는 그대로 쪽 아래에 둔다.
        let (above_footer, _) = separator(&footnote_area(&core, 5));
        assert!(
            above_footer > 900.0,
            "{name} 꼬리말 위 구분선 y={above_footer}"
        );
    }
}

#[test]
fn small_content_numbers_match_the_body_reference_size() {
    for name in ["shape.hwp", "shape.hwpx"] {
        let core = DocumentCore::from_bytes(&read(name)).expect("open");
        let small = first_line_runs(&footnote_area(&core, 4));
        let (number, number_end, number_size) = &small[0];
        let (_, _, text_size) = &small[1];
        assert_eq!(number, "1)");
        assert!(
            (number_size - text_size * 0.75).abs() < 0.01,
            "{name} 작게 번호 크기 {number_size} / 글자 {text_size}"
        );
        assert!(
            (number_end - 90.95 * PX_PER_PT).abs() < 0.5,
            "{name} 작게 번호 끝 x={number_end}"
        );
        let normal = first_line_runs(&footnote_area(&core, 5));
        assert!((normal[0].2 - normal[1].2).abs() < 0.01, "{name} 보통 번호");
        assert!(
            (normal[0].1 - 92.88 * PX_PER_PT).abs() < 0.5,
            "{name} 보통 번호 끝 x={}",
            normal[0].1
        );
    }
}

#[test]
fn footnote_separator_draws_its_line_type() {
    use std::io::{Read, Write};
    let mut input = zip::ZipArchive::new(std::io::Cursor::new(read("shape.hwpx"))).unwrap();
    let mut output = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let name = file.name().expect("entry name").to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        if name == "Contents/section3.xml" {
            // 4쪽 각주 구분선을 점선으로 바꾼다.
            let xml = String::from_utf8(data).unwrap();
            let at = xml.find("<hp:footNotePr>").unwrap();
            let at = at + xml[at..].find(r#"type="SOLID""#).unwrap();
            data = format!("{}type=\"DASH\"{}", &xml[..at], &xml[at + 12..]).into_bytes();
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(file.compression());
        output.start_file(name, options).unwrap();
        output.write_all(&data).unwrap();
    }
    let core =
        DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).expect("open edited");
    let (_, dash) = separator(&footnote_area(&core, 3));
    assert_ne!(dash, StrokeDash::Solid);
    let (_, solid) = separator(&footnote_area(&core, 4));
    assert_eq!(solid, StrokeDash::Solid);
}
