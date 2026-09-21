//! [#7051 조판 축] HFT 한글 전용 face 의 ASCII 를 반각으로 재지 않아 글자가 용지 밖으로 나간다.
//!
//! ## 무엇이 문제였나
//!
//! HWP3 시절 HFT 글꼴(`명조`·`신명 세명조`·`한양신명조` 등)은 **한글 전용**이고 ASCII
//! 글리프를 한글 em 의 절반 폭으로 그린다. rhwp 는 이 글꼴을 렌더 가능한 TTF 로 치환한 뒤
//! (`명조` → `HY견명조`) **치환 글꼴의 비례 폭**을 그대로 써서 ASCII 가 42% 넓어졌다.
//!
//! `samples/hwp3-sample10-hwp5.hwp` 489쪽(문단 `0.17686`)이 그 결과다.
//!
//! ```text
//!   저장 LineSeg   ls[0] ts=0  ls[1] ts=93  ls[2] ts=157   줄폭 sw=42520HU = 566.9px
//!   rhwp 렌더      첫 줄 = 0..93 (저장 끊음을 지켰다)      폭 804.7px  = 자당 0.737 em
//!                  → 단(566.9)을 238.1px 넘고 글자가 용지(793.7) 밖 136.4px 까지
//! ```
//!
//! 끊을 자리가 없어서가 아니다 — **저장 끊음은 지켰고 93자의 폭 측정이 틀렸다.**
//!
//! ## 기대값의 독립 근거
//!
//! 한컴 정본 PDF(MCP engine 2024 / `13.0.0.3901` 변환, 763쪽 — rhwp 쪽수와 동일,
//! sha256 `1b04b75be86572625034f283dc24cbf08a33c4f19dabd8d3b4c062fbdf917bf4`, 90.8MB 라
//! 저장소에 넣지 않는다 — 아래 수치가 그 PDF 에서 읽은 전부다)의 489쪽
//! 낱말 상자를 `pdftotext -bbox` 로 재면 ASCII 전진폭이 em 의 절반이다(9pt → em 12px).
//!
//! ```text
//!   'TABLESPACE(ROLLBACK_DATA),'        26자  156.64px  자당 6.025px = 0.502 em
//!   'TEMPORARY'                          9자   54.13px  자당 6.015px = 0.501 em
//!   'TABLESPACE(TEMPORARY_DATA),USER'   31자  186.70px  자당 6.023px = 0.502 em
//!   ASCII 낱말 자당 중위값  420쪽 6.017 · 198쪽 6.023 · 491쪽 6.021  = 0.502 em
//!   정본 최대 우단          420·198·491쪽 모두 680.0px (단 우단 680.3, 용지 793.7)
//! ```
//!
//! 저장 LineSeg 도 같은 값을 말한다 — 93자가 42520HU(566.9px) 안에 들어가려면 자당
//! 6.10px 이하여야 하고, 반각(6.0px)만이 그 조건을 만족한다(93×6.0 = 558.0px).
//!
//! ## 적용 경계
//!
//! 판정은 글꼴 **치환 경계**로 한다(`style_resolver` 가 이미 가른다).
//!
//! - `FontSubstitutionBoundary::Hft` — 한글 전용 HFT face(`명조`·`가지`·`딸기`·`한양…`).
//!   이 갈래만 ASCII 를 `em/2` 로 전진시킨다.
//! - `FontSubstitutionBoundary::LegacyLatin` — **진짜 영문** HFT/legacy face
//!   (`HCI Poppy`·`Swis721 BT`·`굵은안상수체영문` 등)는 비례 글꼴이라 이 갈래에 안 들어온다.
//! - `ResolvedStyleSet::hwp3_variant` — **HWP3→HWP5 변환본만**. 이 게이트가 없으면
//!   `samples/exam_kor.hwp` 가 깨진다(아래 반례). 같은 HFT 이름이라도 HWP3 시대에 조판된
//!   저장본만 반각이다.
//!
//! ## 비범위
//!
//! HWP3 **원본**(`hwp3-sample10.hwp`) 경로는 이 변경이 닿지 않는다(글꼴 해소가 다른
//! 경로다). 그 문서는 애초에 가로 초과가 0 이라 이 결함이 드러나지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// `#7051` 이 신고한 문서. 489쪽(0-기반 488) 한 줄이 문제였다.
const SAMPLE: &str = "samples/hwp3-sample10-hwp5.hwp";
/// 0-기반 쪽 번호 — 정본 PDF 489쪽.
const PAGE: u32 = 488;
/// 그 줄의 첫 런 머리.
const RUN_HEAD: &str = " TABLESPACE(ROLLBACK_DATA),";
/// 정본 실측 ASCII 전진폭(px) — 9pt(em 12px)의 절반.
const ORACLE_ASCII_ADVANCE_PX: f64 = 6.02;
/// 저장 LineSeg 의 줄폭 42520HU 를 96dpi px 로 옮긴 값 = 단 폭.
const STORED_LINE_WIDTH_PX: f64 = 42520.0 / 7200.0 * 96.0;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 저장 끊음이 배정한 93자가 저장 줄폭 안에 들어간다.
///
/// 수정 전: 폭 804.7px — 줄폭 566.9px 를 238.1px 넘겼다.
#[test]
fn hft_hangul_ascii_run_fits_the_stored_line_width() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(PAGE).expect("page 489");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    let (bbox, text) = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text.starts_with(RUN_HEAD) => {
                Some((n.bbox, r.text.clone()))
            }
            _ => None,
        })
        .expect("489쪽의 TABLESPACE 런");

    assert!(
        bbox.width <= STORED_LINE_WIDTH_PX + 0.5,
        "저장 LineSeg 가 이 {}자를 한 줄(ts=0..93)로 배정했고 그 줄폭은 {:.1}px 다. \
         런 폭 {:.1}px — 수정 전 804.7px(자당 0.737em)",
        text.chars().count(),
        STORED_LINE_WIDTH_PX,
        bbox.width,
    );
}

/// ASCII 자당 전진폭이 정본과 같다 — em 의 절반.
///
/// 폭 총량만 보면 다른 보정으로도 우연히 맞출 수 있으므로 자당 값을 직접 재고,
/// 같은 줄의 **한글**은 전각 그대로인지 함께 확인한다.
#[test]
fn hft_hangul_ascii_advance_matches_the_hancom_oracle() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(PAGE).expect("page 489");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    let (bbox, text) = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text.starts_with(RUN_HEAD) => {
                Some((n.bbox, r.text.clone()))
            }
            _ => None,
        })
        .expect("489쪽의 TABLESPACE 런");

    // 이 런은 전부 ASCII 다(공백 포함) — 반각 하나로 설명돼야 한다.
    assert!(text.is_ascii(), "표본 런이 ASCII 전용이 아니다: {text:?}");
    let per_char = bbox.width / text.chars().count() as f64;
    assert!(
        (per_char - ORACLE_ASCII_ADVANCE_PX).abs() <= 0.15,
        "ASCII 자당 전진폭이 정본({ORACLE_ASCII_ADVANCE_PX:.2}px = em/2)과 다르다. \
         got {per_char:.3}px — 수정 전 8.843px(0.737em)"
    );

    // 같은 줄의 한글 런은 전각이다 — 반각 규칙이 한글까지 눌러선 안 된다.
    // 이 런은 '가' + 말미 공백 두 글자이므로 공백 한 칸(반각)을 빼면 '가' 의 전진폭이다.
    let hangul = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text == "가 " => Some(n.bbox),
            _ => None,
        })
        .expect("같은 줄의 '가 ' 런");
    let hangul_advance = hangul.width - ORACLE_ASCII_ADVANCE_PX;
    assert!(
        (hangul_advance - ORACLE_ASCII_ADVANCE_PX * 2.0).abs() <= 0.3,
        "같은 줄 한글이 전각(ASCII 의 2배 = {:.2}px)이 아니다 — got {hangul_advance:.2}px",
        ORACLE_ASCII_ADVANCE_PX * 2.0,
    );
}

/// 반례 — 같은 HFT 한글 face 라도 **HWP3 시대 저장본이 아니면** 비례 폭이다.
///
/// `samples/exam_kor.hwp` 는 legacy HFT 이름(`신명 견명조`·`한양견명조` 등)을 LATIN 슬롯에
/// 쓰지만 진짜 HWP5 문서다(`HwpSummaryInformation` HWP3 시대 신호 + 스타일 비율 결합 판정에서
/// 변환본이 아니다). 한컴 정본 `pdf/exam_kor-2022.pdf` 6쪽의 큰 숫자 `6` 은 **26.93px**
/// (x 117.12..144.05, font-size 41.74px → 0.645 em)이고 반각 20.87px 이 아니다.
///
/// 이 반례가 없으면 반각 규칙이 문서 시대를 안 보고 발화해 이 문서의 큰 숫자가 25% 좁아진다
/// (`tests/golden_svg/issue-617/exam-kor-page5.svg` 의 `textLength` 27.79 → 20.87).
#[test]
fn modern_hwp5_document_with_legacy_hft_names_keeps_proportional_ascii() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_kor.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(5).expect("page 6");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    // 6쪽 머리의 큰 문항 번호 '6' — 정본 26.93px.
    let six = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            // 이 런은 저장 텍스트가 공백이고 표시 문자가 `6` 이다(문항 번호 치환).
            RenderNodeType::TextRun(r) if r.display_or_text() == "6" && n.bbox.width > 15.0 => {
                Some(n.bbox)
            }
            _ => None,
        })
        .max_by(|a, b| a.width.total_cmp(&b.width))
        .expect("6쪽의 큰 '6' 런");

    assert!(
        six.width > 24.0,
        "HWP3 시대가 아닌 문서에 반각 규칙이 발화했다 — '6' 폭 {:.2}px \
         (정본 26.93px, 반각이면 20.87px)",
        six.width,
    );
}
