//! [#5800] HWP5 원시 한컴 사용자 기호 재현 fixture 생성기.
//!
//! 같은 본문을 HWP 5.1.0.1과 5.0.1.5로 읽는 최소 대조본을 만든다.
//! 5.0.1.5 본문의 U+A832는 한컴 이중 괘선이며, 5.0.1.6 이후에는 유니코드 문자다.
//! 독립 한컴 인쇄본은 `pdf/issue5140-version-boundary/`에 보존한다.
//!
//! Usage:
//!   cargo run --profile release-test --example build_issue_5800_fixture
//!   ./target/release-test/rhwp.exe export-svg samples/issue5800-hancom-symbol.hwp -o output/

use rhwp::document_core::DocumentCore;
use std::{
    fs,
    io::{Cursor, Seek, SeekFrom, Write},
};

const OUT: &str = "samples/issue5800-hancom-symbol.hwp";

const LEGACY: &str = "samples/issue5140-version-boundary/issue5800-v5.0.1.5.hwp";

/// 같은 본문 U+A832 84개 — 태안군 별지 제5호 서식의 제목 아래 이중 밑줄과 같은 개수.
fn double_rule() -> String {
    "\u{A832}".repeat(84)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .map_err(|e| format!("빈 문서 생성 실패: {e:?}"))?;

    let lines = [
        "재난관리기금 수입금 관리대장".to_string(),
        double_rule(),
        "도장란 \u{A12B}    원문자 \u{A289}\u{A28A}    굵은 가로선 \u{A80F}".to_string(),
    ];

    core.begin_batch_native()
        .map_err(|e| format!("배치 시작 실패: {e:?}"))?;
    core.insert_text_native(0, 0, 0, &lines[0])
        .map_err(|e| format!("제목 입력 실패: {e:?}"))?;
    for (i, line) in lines.iter().enumerate().skip(1) {
        core.insert_paragraph_native(0, i)
            .map_err(|e| format!("문단 추가 실패(pi={i}): {e:?}"))?;
        core.insert_text_native(0, i, 0, line)
            .map_err(|e| format!("텍스트 입력 실패(pi={i}): {e:?}"))?;
    }
    core.end_batch_native()
        .map_err(|e| format!("배치 종료 실패: {e:?}"))?;

    let bytes = core
        .export_hwp_native()
        .map_err(|e| format!("HWP 내보내기 실패: {e:?}"))?;
    fs::write(OUT, &bytes)?;
    // HWP5 FileHeader stores revision, build, minor and major starting at byte 32.
    let mut legacy = cfb::CompoundFile::open(Cursor::new(bytes.clone()))?;
    {
        let mut header = legacy.open_stream("/FileHeader")?;
        header.seek(SeekFrom::Start(32))?;
        header.write_all(&[5, 1, 0, 5])?;
    }
    fs::write(LEGACY, legacy.into_inner().into_inner())?;

    // 되읽어 원시 값이 그대로 실렸는지 확인한다(정규화는 렌더 경로의 몫이다).
    let reread = rhwp::parser::parse_hwp(&bytes)?;
    let text: String = reread.sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.clone())
        .collect();
    for (cp, count) in [('\u{A832}', 84), ('\u{A12B}', 1), ('\u{A289}', 1)] {
        let got = text.matches(cp).count();
        assert_eq!(got, count, "U+{:04X} 개수 불일치: {got}", cp as u32);
    }
    println!("OK — {OUT} ({} bytes)", bytes.len());
    Ok(())
}
