//! [#5875] 셀 안 중첩 표의 **글자 캡션**도 그려야 한다.
//!
//! `nested_table_text_caption.hwp`(2181727 방호장치 안전인증 고시 별표 1의2) 7·8쪽은
//! 바깥 표의 한 칸 안에 시험조건 중첩 표 7개가 들어 있고, 한글은 각 표 위에
//! `<표 1> 공급전압 차단` 같은 캡션 제목을 그린다.
//!
//! 수정 전 rhwp 는 중첩 표(depth >= 1) 캡션을 "캡션 안에 위/아래 그림이 있을 때"만
//! 그려서(#1585 게이트) 글자만 있는 캡션 5개(`<표 1>`·`<표 2>`·`<표 3>`·`<표 5>`·
//! `<표 7>`)가 렌더·텍스트 추출 양쪽에서 통째로 사라졌다. 살아남았던
//! `<표 4>`·`<표 6>`·`<표 8>` 은 캡션이 아니라 셀 안 일반 문단이다.
//!
//! 높이 측정기는 처음부터 캡션 높이를 깊이와 무관하게 표 총 높이에 넣으므로,
//! 캡션을 안 그리면 그 자리가 표 아래 빈 띠로 남는다(표3 하단→`라.` 문단 간격
//! 한글 12.2px ↔ 수정 전 rhwp 60.6px). 여기서는 텍스트 존재와 쪽수 불변만 고정한다
//! — 기하 정합은 한글 2022 정답지 PDF 대조로 이슈에 기록했다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5875/nested_table_text_caption.hwp";

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse sample")
}

fn page_text(doc: &DocumentCore, page: u32) -> String {
    doc.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 텍스트 추출: {e}", page + 1))
}

#[test]
fn issue_5875_nested_table_text_captions_render_on_page7() {
    let doc = load_doc();
    let page7 = page_text(&doc, 6);
    // 캡션 컨트롤 3개 — 수정 전에는 전부 소실.
    for title in ["<표 1> 공급전압 차단", "<표 2>", "<표 3>"] {
        assert!(
            page7.contains(title),
            "#5875: 7쪽 중첩 표 글자 캡션 {title:?} 이 그려져야 한다\n--- 7쪽 ---\n{page7}"
        );
    }
    // 일반 문단 제목 — 수정 전에도 살아 있었다 (회귀 가드).
    assert!(
        page7.contains("<표 4>"),
        "#5875 전제: 7쪽 일반 문단 제목 <표 4> 는 원래부터 있어야 한다\n--- 7쪽 ---\n{page7}"
    );
}

#[test]
fn issue_5875_nested_table_text_captions_render_on_page8() {
    let doc = load_doc();
    let page8 = page_text(&doc, 7);
    // 캡션 컨트롤 2개 — 수정 전에는 소실.
    for title in ["<표 5>", "<표 7>"] {
        assert!(
            page8.contains(title),
            "#5875: 8쪽 중첩 표 글자 캡션 {title:?} 이 그려져야 한다\n--- 8쪽 ---\n{page8}"
        );
    }
    // 일반 문단 제목 2개 (회귀 가드).
    for title in ["<표 6>", "<표 8>"] {
        assert!(
            page8.contains(title),
            "#5875 전제: 8쪽 일반 문단 제목 {title:?} 은 원래부터 있어야 한다\n--- 8쪽 ---\n{page8}"
        );
    }
}

#[test]
fn issue_5875_page_count_stays_12() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        12,
        "#5875: 캡션 렌더는 저장 사다리 흐름을 바꾸지 않으므로 쪽수 12 는 그대로여야 한다"
    );
}
