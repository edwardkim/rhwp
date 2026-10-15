//! [#7810 후속] HWP3 줄에 한/글 변환기와 같은 들여쓰기 적용 기록(LineSeg bit 20)을 단다.
//!
//! 렌더러는 저장 줄의 `TAG_INDENTATION`(bit 20)이 꺼진 줄에 문단 들여쓰기를 얹지 않는다
//! (#6190). HWP3 파서는 이 비트를 비워 두었으므로, HWP3 문단의 들여쓰기·내어쓰기가 모두
//! 사라졌다. `SO-SUEOP.hwp` 3쪽 `작가가 말하고자…` 줄은 한/글 PDF 보다 6.7px(500 HU) 왼쪽에서
//! 시작했다.
//!
//! 기대값은 한/글 자신의 변환본 `samples/SO-SUEOP.hwpx` 의 줄별 bit 20 기록이다. 두 문서는
//! 문단 순서·문자열·줄 수가 대응하므로, 대응하는 줄마다 비트가 같아야 한다.

use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::parser::parse_document;

fn load(rel: &str) -> rhwp::model::document::Document {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기 실패: {e}"));
    parse_document(&bytes).unwrap_or_else(|e| panic!("{rel} 파싱 실패: {e:?}"))
}

fn indentation_bits(para: &Paragraph) -> Vec<bool> {
    para.line_segs
        .iter()
        .map(|seg| seg.tag & LineSeg::TAG_INDENTATION != 0)
        .collect()
}

#[test]
fn hwp3_line_indentation_flags_match_hangul_conversion() {
    let hwp3 = load("samples/SO-SUEOP.hwp");
    let hancom = load("samples/SO-SUEOP.hwpx");

    let mut compared_lines = 0usize;
    let mut indented_lines = 0usize;
    let mut mismatches = Vec::new();
    for (section_index, (ours, theirs)) in
        hwp3.sections.iter().zip(hancom.sections.iter()).enumerate()
    {
        for (para_index, (a, b)) in ours
            .paragraphs
            .iter()
            .zip(theirs.paragraphs.iter())
            .enumerate()
        {
            // 문자열과 줄 수가 같은 문단만 한/글 기록과 짝짓는다.
            if a.text != b.text || a.line_segs.len() != b.line_segs.len() {
                continue;
            }
            let ours_bits = indentation_bits(a);
            let theirs_bits = indentation_bits(b);
            compared_lines += ours_bits.len();
            indented_lines += theirs_bits.iter().filter(|bit| **bit).count();
            if ours_bits != theirs_bits {
                mismatches.push(format!(
                    "{section_index}.{para_index} {:?}: rhwp {ours_bits:?} / 한/글 {theirs_bits:?}",
                    a.text.chars().take(16).collect::<String>()
                ));
            }
        }
    }

    // 짝지은 줄과 한/글이 bit 20 을 켠 줄이 충분히 있어야 이 비교가 의미를 가진다.
    assert!(
        compared_lines > 500,
        "짝지은 줄이 {compared_lines}개뿐이다 — 표본이 바뀌었다면 대응 방식부터 다시 세워야 한다"
    );
    assert!(
        indented_lines > 40,
        "한/글 변환본에서 bit 20 이 켜진 줄이 {indented_lines}개뿐이다"
    );
    assert!(
        mismatches.is_empty(),
        "줄별 들여쓰기 기록이 한/글 변환본과 다르다 ({}건):\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
