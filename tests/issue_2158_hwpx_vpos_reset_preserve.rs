//! Issue #2158 — HWPX lineseg 재계산이 저장 쪽-상대 vpos 리셋(쪽나눔 인코딩)을
//! 파괴하던 회귀를 막는 쪽수·본문 소유 검사.
//!
//! HWPX 원본 XML은 HWP5와 동일하게 쪽-상대 vertpos를 저장하며, 직전 문단 대비
//! 급감(예: sample16 pi87 70412 → pi88 568=sb)이 쪽나눔을 인코딩한다. HWPX 로딩은
//! lineSegArray 부재 문단이 있으면 구역 전체 vpos를 재계산하는데, 이때 원본
//! lineseg 문단의 리셋 신호가 누적 좌표로 덮여(568→208008) typeset의 vpos-reset
//! 쪽나눔(#321/#1921)이 무력화 — 동일 문서가 HWP5로는 64쪽(한글 정합), HWPX로는
//! 63쪽이 되던 결함.
//!
//! 과거 COM 쪽수와 달리 같은 원본의 독립 전체 Print는46쪽이다.
//! stage2869 전쪽 Native/fresh WASM 검증과 stage2903 직접 판독을 근거로 한다.

use std::fs;
use std::path::Path;

fn load(rel: &str) -> rhwp::document_core::DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    rhwp::document_core::DocumentCore::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e))
}

fn rendered_text(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> String {
    let svg = doc.render_page_svg_native(page - 1).expect("원본 쪽 출력");
    let xml = roxmltree::Document::parse(&svg).expect("유효한 SVG");
    xml.descendants()
        .filter(|node| node.has_tag_name("text"))
        .flat_map(|node| node.descendants().filter(|child| child.is_text()))
        .filter_map(|node| node.text())
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn assert_content_order(text: &str, anchors: &[&str]) {
    let mut previous = None;
    for anchor in anchors {
        assert_eq!(
            text.matches(anchor).count(),
            1,
            "내용은 한 번만 표시: {anchor}"
        );
        let at = text.find(anchor).expect("독립 PDF의 내용 표시");
        if let Some(previous) = previous {
            assert!(at > previous, "독립 PDF의 내용 순서 보존: {anchor}");
        }
        previous = Some(at);
    }
}

fn partial_range(dump: &str, paragraph: usize) -> (usize, usize) {
    let marker = format!("PartialParagraph  pi={paragraph}  lines=");
    let range = dump
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(&marker))
        .expect("앞뒤 쪽이 같은 문단의 일부를 소유")
        .split_whitespace()
        .next()
        .expect("분할 범위");
    let (start, end) = range.split_once("..").expect("줄 범위 형식");
    (start.parse().expect("시작 줄"), end.parse().expect("끝 줄"))
}

#[test]
fn hwp3_sample16_hwpx_matches_hwp5_and_hangul() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(root.join("samples/hwp3-sample16-hwp5.hwpx")).expect("보존 원본");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("원본 파싱");
    assert_eq!(
        (
            doc.page_count(),
            load("samples/hwp3-sample16-hwp5.hwp").page_count()
        ),
        (64, 64),
        "두 원본의 독립 Print PDF는 각각 64쪽이다"
    );

    // 현재 common frame의 첫 본문은 앞 쪽의 표 뒤 잔여 공간에 끼우지 않는다.
    let page20 = rendered_text(&doc, 20);
    let page21 = rendered_text(&doc, 21);
    let first = "경제성,운영관리효율성을고려한외부IDC센타";
    assert!(
        !page20.contains(first),
        "경제성 문단을 앞 쪽에 남기지 않는다"
    );
    assert!(
        page21.starts_with(&format!("□{first}")),
        "21쪽 첫 본문 소유"
    );
    assert_content_order(
        &page21,
        &[
            first,
            "외부IDC센터운영관리업무범위",
            "RemoteControl관리체계구축",
        ],
    );
    assert!(doc
        .dump_page_items(Some(20))
        .contains("FullParagraph  pi=426"));

    let section = "4.서버통합및원격지재해복구센터시스템구성요건";
    let firewall = "방화벽내․외부간DATA";
    let page22 = rendered_text(&doc, 22);
    let page23 = rendered_text(&doc, 23);
    let page24 = rendered_text(&doc, 24);
    assert!(
        !page21.contains(section),
        "다음 절 제목은 22쪽에서 시작한다"
    );
    assert_content_order(
        &page22,
        &[section, "가.전체시스템구성을위한공통요건", "RDBMS는ORACLE"],
    );
    assert!(!page22.contains(firewall), "방화벽 문단은 23쪽 소유다");
    assert_content_order(
        &page23,
        &[
            firewall,
            "나.주전산센터시스템구성요건",
            "ORACLERAC",
            "HeartBit",
        ],
    );
    assert!(
        !page23.contains("GigaBase"),
        "이어받기를 앞 쪽에 중복하지 않는다"
    );
    assert!(
        page24.starts_with("구성은GigaBase"),
        "문장 이어받기는 다음 쪽 첫 본문이다"
    );
    assert!(
        !page24.contains("HeartBit"),
        "이미 표시한 내용을 중복하지 않는다"
    );
    assert_content_order(
        &page24,
        &[
            "GigaBase",
            "Take-over",
            "다.재해복구센터시스템구성요건",
            "HotSite",
        ],
    );
    let (start, cut) = partial_range(&doc.dump_page_items(Some(22)), 460);
    let (resume, end) = partial_range(&doc.dump_page_items(Some(23)), 460);
    assert_eq!(start, 0, "첫 조각은 문단 시작을 소유한다");
    assert!(
        cut > start && end > resume,
        "두 조각 모두 실제 내용을 소유한다"
    );
    assert_eq!(
        cut, resume,
        "분할 컷과 이어받기가 연결되어야 누락·중복이 없다"
    );

    // 실제 소비한 저장 빈 줄 뒤의 새 절 제목은 다음 쪽에 한 번만 표시한다.
    let page25 = rendered_text(&doc, 25);
    let page26 = rendered_text(&doc, 26);
    assert!(!page25.contains("도입장비내역서"));
    assert_content_order(
        &page26,
        &[
            "도입장비내역서",
            "도입장비내역총괄표",
            "주전산센터서버통합",
            "재해복구센터구축",
            "합계13식",
        ],
    );
    assert!(doc
        .dump_page_items(Some(25))
        .contains("FullParagraph  pi=488"));
}

#[test]
fn onsaemiro_hwpx_page_count_matches_hangul() {
    let doc = load("samples/[2027] 온새미로 1 본교재.hwpx");
    assert_eq!(
        doc.page_count(),
        46,
        "같은 원본의 독립 전체 Print46쪽. 과거47쪽 핀으로 정상 출력을 회귀로 판정하지 않는다."
    );
    let layout: serde_json::Value =
        serde_json::from_str(&doc.get_page_text_layout_native(5).expect("독립 Print6쪽")).unwrap();
    let runs = layout["runs"].as_array().unwrap();
    let original = &doc.document().sections[1].paragraphs[32];
    assert!(original
        .text
        .starts_with("수많은 SF 영화나 소설이 유토피아"));
    let mut next = 0;
    let mut text = String::new();
    for run in runs
        .iter()
        .filter(|run| run["secIdx"] == 1 && run["paraIdx"] == 32)
    {
        assert_eq!(run["charStart"].as_u64(), Some(next), "지문 순서·누락·중복");
        let value = run["text"].as_str().unwrap();
        text.push_str(value);
        next += value.encode_utf16().count() as u64;
    }
    assert_eq!(
        text, original.text,
        "쪽수와 함께 원문 지문의 쪽 소속·전체 내용 보존"
    );
}
