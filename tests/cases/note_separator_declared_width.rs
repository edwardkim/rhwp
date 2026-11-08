//! 관계형 회귀: 원본 각주 구분선 종류·선 굵기와 최종 논리 선의 소유·길이·수평 관계. 절대 배치 좌표는 고정하지 않는다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! note-formats.hwp (8536-note-formats-hwp-2022): 전 19쪽, Native 최저 99.98307%, fresh WASM 최저 99.98279%.
//! note-formats.hwpx (8536-note-formats-hwpx-2022): 전 19쪽, Native 최저 99.98307%, fresh WASM 최저 99.98279%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형: 원본 각주 선 종류·선 굵기와 최종 논리 선의 소유·길이·수평 관계.
//! 주석 갱신: 2026-11-05T23:03:12+09:00. 신규 파일이므로 직전 파일 변경 커밋은 없다.
//! 측정: 2026-11-05 KST, 독립 한컴 Mac 12.30 Print, 96dpi, 보정 전 2px raw 실루엣.
//! 원본별 HWP/HWPX 전 19쪽: Native 최저 99.98%, fresh WASM 최저 99.98%, PDF 쪽수 일치.
//! 대상 8·9·10·11쪽은 두 형식·두 backend 모두 100.00%; 절대 배치 좌표를 고정하지 않는다.
//! 제품 source: dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b. 시스템 글꼴 공급에 의존하지 않는다.
//! 정식 근거: mydocs/pr/assets/semanticist21_20261105/rebase-ece9-note-repair/README.md.
//! TSV: 같은 폴더 hwp-native.tsv, hwp-wasm.tsv, hwpx-native.tsv, hwpx-wasm.tsv.
//! 독립 PDF: pdf/number_shapes/note-formats-hancom-mac-12.30.pdf.
//! 각주 10·11쪽 review·overlay와 확대 판독은 같은 증적 폴더에 연결한다.
//! 수정 전 R1 최종 트리는 복선 2개·삼중선 3개로 분해하며, 수정 후는 선언 폭을 소유한 한 선이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::RenderNodeType;
use rhwp::renderer::LineRenderType;

fn check_compound_separators(name: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/number_shapes")
        .join(name);
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut checked = 0;
    for (section_index, section) in core.document().sections.iter().enumerate() {
        let shape = &section.section_def.footnote_shape;
        let expected_type = match shape.separator_line_type {
            8 => LineRenderType::Double,
            9 => LineRenderType::ThinThickDouble,
            10 => LineRenderType::ThickThinDouble,
            11 => LineRenderType::ThinThickThinTriple,
            _ => continue,
        };
        let tree = core.build_page_render_tree(section_index as u32).unwrap();
        let area = tree
            .root
            .children
            .iter()
            .find(|node| matches!(node.node_type, RenderNodeType::FootnoteArea))
            .expect("원본 구역의 각주 영역");
        let lines: Vec<_> = area
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::Line(line) => Some(line),
                _ => None,
            })
            .collect();
        assert_eq!(
            lines.len(),
            1,
            "{name} 구역 {section_index}: 각주 구분선은 선언 굵기를 소유한 하나의 논리 선"
        );
        let line = lines[0];
        assert_eq!(line.style.line_type, expected_type, "원본 선 종류 보존");
        // HWP5 표 25·133: 원본 굵기 인덱스 7은 0.5mm다. 독립 Print는
        // 600dpi pen 격자를 사용한다. 제품의 폭 변환 helper를 기대값에 재사용하지 않는다.
        assert_eq!(shape.separator_line_width, 7, "검증 원본의 선언 굵기");
        let source_hwp_units = (0.5_f64 * 7200.0 / 25.4).round();
        let print_pen_units = (source_hwp_units / 12.0).round();
        let declared_width = print_pen_units * 96.0 / 600.0;
        assert!(
            (line.style.width - declared_width).abs() < 1e-9,
            "{name} 구역 {section_index}: 표 테두리 최소 밴드로 각주 굵기를 확대하지 않는다"
        );
        assert!(line.x2 > line.x1, "구분선은 양의 길이를 가진다");
        assert_eq!(line.y1, line.y2, "각주 구분선의 수평 관계");
        checked += 1;
    }
    assert_eq!(checked, 4, "복선·가는/굵은 복선·삼중선 모두 실행");
}

#[test]
fn hwp_note_compound_lines_keep_the_source_declared_width() {
    check_compound_separators("note-formats.hwp");
}

#[test]
fn hwpx_note_compound_lines_keep_the_source_declared_width() {
    check_compound_separators("note-formats.hwpx");
}
