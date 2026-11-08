//! 관계형 회귀: 원본 물리 행 조각의 같은 행·가로 비겹침·다음 행 순서·본문 내용 보존. 절대 px 좌표는 고정하지 않는다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! moved-terminal-tab.hwp (api-8549-moved-terminal-tab-hwp): 전 1쪽, Native 최저 99.92003%, fresh WASM 최저 99.92149%.
//! moved-terminal-tab.hwpx (api-8549-moved-terminal-tab-hwpx): 전 1쪽, Native 최저 99.91994%, fresh WASM 최저 99.92141%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형: 원본의 물리 행 조각 소유·본문 순서·내용 보존.
//! 주석 갱신: 2026-11-06T00:51:56+09:00. 신규 파일이므로 직전 파일 변경 커밋은 없다.
//! 측정: 2026-11-06 KST, 한컴 2020 Print, 96dpi PDFium, 보정 전 2px raw 실루엣.
//! moved-terminal-tab.hwp와 .hwpx: 각각 전 1쪽 Native 99.92% / fresh WASM 99.92%.
//! 두 입력 모두 독립 Print와 쪽수가 같으며 전체 쪽 최저값도 각각 99.92% / 99.92%다.
//! 제품 source: a314d7a24d3234305abff43149f575853db31105. 절대 px 좌표를 고정하지 않는다.
//! 근거: mydocs/pr/assets/semanticist21_20261105/validation-r3/pr8549/README.md.
//! 같은 폴더의 원본별 Native/WASM TSV·review/overlay와 독립 PDF 확대 판독을 연결한다.
//! 최소 글꼴 5개 환경에서 수정 전 2 FAIL / 수정 후 2 PASS이며 개인 OTF 설치에 의존하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

fn open(extension: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/semanticist21_integration_review/pr8549")
        .join(format!("moved-terminal-tab.{extension}"));
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn lines<'a>(node: &'a RenderNode, paragraph: usize, found: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(paragraph))
    {
        found.push(node);
    }
    for child in &node.children {
        lines(child, paragraph, found);
    }
}

fn text(node: &RenderNode) -> String {
    match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => node.children.iter().map(text).collect(),
    }
}

#[test]
fn stored_implementation_flags_do_not_stack_declared_row_fragments() {
    let core = open("hwp");
    let paragraph = &core.document().sections[0].paragraphs[2];
    let source = &paragraph.line_segs;
    assert!(source.len() > 1, "원본의 분할 행이 실제 실행된다");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut rendered = Vec::new();
    lines(&tree.root, 2, &mut rendered);
    assert_eq!(
        rendered.len(),
        source.len(),
        "내용 없는 오른쪽 조각의 소유도 보존"
    );
    let mut pairs = 0;
    for (index, row) in source.windows(2).enumerate() {
        if row[0].vertical_pos == row[1].vertical_pos {
            assert!(row[0].is_first_segment() && row[1].is_last_segment());
            assert_eq!(
                rendered[index].bbox.y,
                rendered[index + 1].bbox.y,
                "원본의 같은 물리 행을 별도 줄로 쌓지 않는다"
            );
            assert!(
                rendered[index].bbox.x + rendered[index].bbox.width <= rendered[index + 1].bbox.x,
                "같은 행의 두 조각은 좌우 소유 구간을 지킨다"
            );
            pairs += 1;
        } else {
            assert!(
                rendered[index].bbox.y < rendered[index + 1].bbox.y,
                "다음 물리 행은 앞 행 아래에 놓인다"
            );
        }
    }
    assert!(pairs > 0, "행 조각 검사가 비어 있지 않다");
    assert_eq!(
        rendered.iter().map(|line| text(line)).collect::<String>(),
        paragraph.text,
        "뒤 문단의 내용 순서·누락·중복 보존"
    );
}

#[test]
fn regular_rows_and_following_content_remain_ordered_in_both_formats() {
    let source = open("hwp");
    let mut saved_rows: Vec<_> = source.document().sections[0].paragraphs[2]
        .line_segs
        .iter()
        .map(|row| row.vertical_pos)
        .collect();
    saved_rows.sort_unstable();
    saved_rows.dedup();
    for extension in ["hwp", "hwpx"] {
        let core = open(extension);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut body = Vec::new();
        let mut label = Vec::new();
        let mut tail = Vec::new();
        lines(&tree.root, 0, &mut body);
        lines(&tree.root, 1, &mut label);
        lines(&tree.root, 2, &mut tail);
        assert!(body.len() > 1 && !label.is_empty() && !tail.is_empty());
        assert!(
            body.windows(2).all(|pair| pair[0].bbox.y < pair[1].bbox.y),
            "일반 완결 행은 같은 물리 행으로 합치지 않는다"
        );
        assert!(
            body.last().unwrap().bbox.y < label[0].bbox.y && label[0].bbox.y < tail[0].bbox.y,
            "앞 본문·탭 문단·뒤 본문의 순서"
        );
        let mut actual_rows: Vec<_> = tail.iter().map(|line| line.bbox.y).collect();
        actual_rows.sort_by(f64::total_cmp);
        actual_rows.dedup();
        assert_eq!(
            actual_rows.len(),
            saved_rows.len(),
            "독립 Print와 원본이 공유하는 물리 행 수"
        );
        assert_eq!(
            tail.iter().map(|line| text(line)).collect::<String>(),
            core.document().sections[0].paragraphs[2].text,
            "본문 내용 보존"
        );
    }
}
