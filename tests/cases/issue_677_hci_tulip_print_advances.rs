//! 관계형: HCI Tulip 표 영문 런의 그린 폭과 원본 글자 크기·독립 Print 전진폭의 비례 관계.
//! 주석 갱신: 2026-11-19T13:15:02+09:00 (새 파일).
//! 측정일 2026-11-19, 제품 코드 upstream/devel `c526dd402b` + 이 변경(HCI Tulip 전진폭 표).
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only --compat 2022`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣,
//! `threshold_boundary_color_support_v1`, 색 허용 32, PDF raster pdfium).
//! 입력 `samples/issue677/1480000-200900112-hci-tulip-table.hwp`;
//! 독립 Print `pdf/issue677/1480000-200900112-hci-tulip-table-2022.pdf`(한컴 2022 12.0.0.4547).
//! 전체 16쪽(독립 Print 16쪽), 전쪽 최저 Native 91.75% / fresh WASM 91.81%(둘 다 7쪽).
//! 주장 7쪽 raw: Native 91.75% / fresh WASM 91.81%. 수정 전 devel `fbf8081db5` Native 83.53%.
//! 증거: `mydocs/pr/assets/issue_677/{native,wasm,native-base}/*_silhouette.tsv`,
//! 7쪽 review/overlay `mydocs/pr/assets/issue_677/{native,wasm}/{review,overlay}/`.
//! 7쪽 남은 차이는 HY견명조 표시 글리프 모양과 그림 3-1 자르기 배율이며 이 검사의 범위 밖이다.
//! 절대 px 좌표는 고정하지 않는다(7쪽 raw 98% 미만).
//!
//! 원인: [표 3-1] 셀 영문(charPr 27·29~33)의 LATIN face 는 HFT `HCI Tulip` 이다. rhwp 는
//! 표시를 HY견명조로 대체하고, 원본 전진폭 표(`font_metrics_overlays`)에 숫자·괄호·N·O 등
//! 17자만 있어 나머지 영문은 HY견명조의 폭(M 0.96em 등)으로 쟀다. 그래서 칸 영문이 한컴보다
//! 약 1.3배 넓게 그려지고 7쪽 raw 가 83.53% 였다.
//!
//! 독립 기대값: 한컴 2022 Print `pdf/issue677/1480000-200900112-hci-tulip-table-2022.pdf`
//! 7쪽 `/T8`(obj 132, Type3, FontMatrix 0.001) 의 ToUnicode·/Widths(= CharProc d1) 이다.
//! 아래 `PRINT_WIDTHS` 는 그 PDF 에서 직접 읽은 값이며 구현의 표를 재인용하지 않는다.
//! 이 문서에서 T8 을 쓰는 글자모양은 LATIN face 8(`HCI Tulip`, HFT) 뿐이고 모두 보통체다.
//!
//! 검사: 7쪽 표의 charPr 27(자간 0·장평 100) 한 단어 셀 런마다
//! `그린 폭 = Σ Print 전진폭/1000 × 글자 크기` 의 비례 관계를 확인한다(±0.5px).
//! 수정 전(devel `c526dd402b`)에는 HY견명조 폭이라 `Increasing` 이 기대 49.82px 대신
//! 68.1px, `STABILIZATION` 이 90.49px 대신 110.2px 로 그려져 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue677/1480000-200900112-hci-tulip-table.hwp";

/// 독립 Print `/T8` 의 전진폭(1/1000 em). 검사 단어가 쓰는 글자만 둔다.
const PRINT_WIDTHS: &[(char, u16)] = &[
    ('A', 722),
    ('B', 667),
    ('C', 667),
    ('D', 722),
    ('E', 609),
    ('F', 554),
    ('G', 722),
    ('H', 722),
    ('I', 332),
    ('L', 609),
    ('M', 888),
    ('N', 722),
    ('O', 722),
    ('P', 554),
    ('R', 667),
    ('S', 554),
    ('T', 609),
    ('U', 722),
    ('Y', 722),
    ('Z', 609),
    ('a', 443),
    ('c', 443),
    ('d', 498),
    ('e', 443),
    ('g', 498),
    ('h', 498),
    ('i', 277),
    ('n', 498),
    ('o', 498),
    ('q', 498),
    ('r', 332),
    ('s', 388),
    ('t', 277),
    ('u', 498),
    ('y', 498),
];

/// charPr 27 의 한 단어 셀(공백 없음 — 공백은 문서의 반각 계약이 소유한다).
const WORDS: &[&str] = &[
    "STABILIZATION",
    "CURING",
    "BIOLOGY",
    "Increasing",
    "Decreasing",
    "AERATION",
    "TEMPERATURE",
    "Frequency",
    "High",
    "Moderate",
    "None",
];

fn print_em(word: &str) -> f64 {
    word.chars()
        .map(|c| {
            let (_, w) = PRINT_WIDTHS
                .iter()
                .find(|(ch, _)| *ch == c)
                .unwrap_or_else(|| panic!("독립 Print 폭이 없는 글자 {c:?}"));
            f64::from(*w) / 1000.0
        })
        .sum()
}

fn tulip_runs<'a>(node: &'a RenderNode, shape: u32, out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.char_shape_id == Some(shape) && run.cell_context.is_some() {
            out.push(node);
        }
    }
    for child in &node.children {
        tulip_runs(child, shape, out);
    }
}

#[test]
fn hci_tulip_table_runs_use_print_advances() {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 16, "독립 Print 전체 쪽수");

    // 전제(원본 저장값): charPr 27 의 LATIN face 는 HFT HCI Tulip, 보통체, 자간 0, 장평 100.
    let doc_info = &core.document().doc_info;
    let shape = &doc_info.char_shapes[27];
    let latin = &doc_info.font_faces[1][usize::from(shape.font_ids[1])];
    assert_eq!(latin.name, "HCI Tulip");
    assert_eq!(latin.alt_type, 2, "구형 글꼴 선언");
    assert!(!shape.bold && !shape.italic);
    assert_eq!((shape.spacings[1], shape.ratios[1]), (0, 100));

    let tree = core.build_page_render_tree(6).expect("7쪽 render tree");
    let mut runs = Vec::new();
    tulip_runs(&tree.root, 27, &mut runs);
    for word in WORDS {
        let matched: Vec<_> = runs
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if run.text.trim_end() == *word => {
                    Some((node.bbox.width, run.style.font_size))
                }
                _ => None,
            })
            .collect();
        assert!(!matched.is_empty(), "7쪽 표에서 {word:?} 런을 찾지 못함");
        for (width, font_size) in matched {
            let expected = print_em(word) * font_size;
            assert!(
                (width - expected).abs() <= 0.5,
                "{word:?}: 그린 폭 {width:.2}px, 독립 Print 전진폭 합 {expected:.2}px (글자 {font_size}px)"
            );
        }
    }
}
