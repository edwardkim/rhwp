//! [#5800] HWP5 원시 한컴 사용자 기호 재현 fixture 생성기.
//!
//! 이슈 본문의 대표 재현 문서(지방 조례 별지 서식)는 저작권/배포 문제로 저장소에
//! 담지 않는다. 대신 같은 증상을 내는 최소 문서를 rhwp 자체 HWP5 writer 로 만든다 —
//! HWP5 는 한컴 사용자 정의 기호를 **BMP 단일 유닛 `0xA000 | X`** 로 싣기 때문에,
//! 그 값을 그대로 넣으면 원본과 같은 바이트 표현이 된다.
//!
//! Usage:
//!   cargo run --profile release-test --example build_issue_5800_fixture
//!   ./target/release-test/rhwp.exe export-svg samples/issue5800-hancom-symbol.hwp -o output/

use rhwp::document_core::DocumentCore;
use std::fs;

const OUT: &str = "samples/issue5800-hancom-symbol.hwp";

/// 0xA832(═) 84 개 — 태안군 별지 제5호 서식의 제목 아래 이중 밑줄과 같은 개수.
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
