//! [Issue #4680] HWP3 문단이 스타일을 전혀 참조하지 않는다.
//!
//! HWP3 문단 레코드는 `style_index` 를 들고 있고 파서도 그 값을 읽는다. 그런데 IR 로
//! 옮기지 않아 **모든 문단이 0번(바탕글)을 가리켰다** — `DocInfo` 에 스타일을 다
//! 써놓고 아무도 안 쓰는 상태였다. 264쪽 문서 실측에서 3,699문단 전부가 0 이었고,
//! 같은 문서를 한/글이 HWP5 로 저장하면 10종을 쓴다(비-0 문단 439건).
//!
//! # 기대값의 출처
//!
//! `samples/` 에는 같은 문서의 HWP3 원본과 **한/글이 저장한 HWP5 변환본**이 짝으로
//! 들어 있다(`hwp3-sample16.hwp` ↔ `hwp3-sample16-hwp5.hwp`). 두 파일은 포맷만
//! 다른 같은 문서이므로 문단별 스타일 참조도 같아야 한다. 그래서 기대값을 우리
//! 구현이 아니라 **한/글 변환본**에서 가져온다 — 수정 전에는 HWP3 쪽만 전부 0 이라
//! 이 대조가 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use std::collections::BTreeSet;
use std::path::Path;

fn collect<'a>(paragraphs: &'a [Paragraph], out: &mut Vec<&'a Paragraph>) {
    for para in paragraphs {
        out.push(para);
        for ctrl in &para.controls {
            if let Control::Table(table) = ctrl {
                for cell in &table.cells {
                    collect(&cell.paragraphs, out);
                }
            }
        }
    }
}

fn all_paragraphs(doc: &Document) -> Vec<&Paragraph> {
    let mut out = Vec::new();
    for section in &doc.sections {
        collect(&section.paragraphs, &mut out);
    }
    out
}

fn parse(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(path).expect("표본 읽기")).expect("표본 파싱")
}

/// 본문 **최상위** 문단의 style_id 서열.
///
/// 표 칸 안쪽은 세지 않는다 — 우리 HWP3 파스는 글자 없는 도형에 빈 글상자를 붙여
/// 칸 문단 수가 한/글 변환본과 다르다(같은 이슈의 별개 결함). 그 축이 이 대조를
/// 오염시키지 않도록 범위를 본문으로 맞춘다.
fn style_ids(rel: &str) -> Vec<u8> {
    let core = parse(rel);
    core.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| p.style_id)
        .collect()
}

/// (HWP3 원본, 한/글 HWP5 변환본) 짝.
const PAIRS: [(&str, &str); 4] = [
    (
        "samples/hwp3-sample16.hwp",
        "samples/hwp3-sample16-hwp5.hwp",
    ),
    (
        "samples/hwp3-sample11.hwp",
        "samples/hwp3-sample11-hwp5.hwp",
    ),
    (
        "samples/hwp3-sample10.hwp",
        "samples/hwp3-sample10-hwp5.hwp",
    ),
    (
        "samples/hwp3-sample19.hwp",
        "samples/hwp3-sample19-hwp5.hwp",
    ),
];

#[test]
fn hwp3_paragraph_style_ids_match_the_hangul_hwp5_conversion() {
    for (hwp3, hwp5) in PAIRS {
        let a = style_ids(hwp3);
        let b = style_ids(hwp5);
        assert_eq!(
            a.len(),
            b.len(),
            "{hwp3}: 문단 수가 한/글 변환본과 다르다 ({} vs {})",
            a.len(),
            b.len()
        );
        let first_diff = a.iter().zip(b.iter()).position(|(x, y)| x != y);
        assert!(
            first_diff.is_none(),
            "{hwp3}: 문단 {} 의 style_id 가 한/글 변환본과 다르다 ({} vs {}). \
             HWP3 문단이 스타일을 참조하지 않으면 전부 0 이 된다",
            first_diff.unwrap(),
            a[first_diff.unwrap()],
            b[first_diff.unwrap()]
        );
    }
}

#[test]
fn hwp3_paragraphs_reference_more_than_the_default_style() {
    // 결함을 직접 드러낸다 — 수정 전에는 어떤 문서든 `{0}` 하나뿐이었다.
    // `hwp3-sample16` 은 이름 붙은 스타일을 15종 쓰는 문서다.
    let used: BTreeSet<u8> = style_ids("samples/hwp3-sample16.hwp").into_iter().collect();
    assert!(
        used.len() > 1,
        "쓰이는 style_id 가 {used:?} 뿐이다 — 문단이 스타일을 참조하지 않는다"
    );
}

#[test]
fn hwp3_paragraph_style_ids_are_within_the_style_pool() {
    // 반례 — 참조가 일어나되 풀 밖을 가리키면 안 된다.
    for (hwp3, _) in PAIRS {
        let core = parse(hwp3);
        let doc = core.document();
        let n = doc.doc_info.styles.len();
        assert!(n > 0, "{hwp3}: 스타일 풀이 비어 있다");
        for (i, para) in all_paragraphs(doc).iter().enumerate() {
            assert!(
                (para.style_id as usize) < n,
                "{hwp3}: 문단 {i} 의 style_id {} 가 스타일 풀({n}개) 밖이다",
                para.style_id
            );
        }
    }
}
