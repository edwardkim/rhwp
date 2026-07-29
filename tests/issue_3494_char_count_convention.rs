//! Issue #3494: `char_count` 규약이 갈려 `ir-diff` 가 잡음에 묻힌다.
//!
//! 공통 IR 의 `Paragraph::char_count` 는 **문단 종결자를 포함**한다. 모델이 그렇게 못박아
//! 두었다.
//!
//! ```text
//! src/model/paragraph.rs
//!   self.char_count = split_pos + ctrl_code_units + 1; // +1 for paragraph end marker
//! ```
//!
//! `field_query`·`html_table_import`·`table_ops` 도 `+1` 을 붙이고, 유일한 절대 소비처인
//! `append_synthetic_cell_line` 은 `char_count - 1` 을 본문 길이로 쓴다(HWP5
//! `PARA_HEADER.nChars` 와 같은 규약).
//!
//! 그런데 **HWP3 파서와 HTML 임포트 6곳이 종결자를 빠뜨렸다.** 그 결과
//! `ir-diff samples/SO-SUEOP.hwp <저장본>` 이 이 한 가지로 **928문단**을 다르다고 보고했다.
//! 진짜 저장 손실이 그 잡음에 묻힌다 — 실제로 #3492 를 조사할 때 928건을 손으로 걷어내야
//! 컨트롤 차이 3건이 보였다.
//!
//! 규약을 한쪽으로 통일하고, 다시 갈라지지 않도록 고정한다.
#![cfg(not(target_arch = "wasm32"))]

/// 종결자를 세지 않던 HWP 3.0 문서.
const HWP3_SAMPLE: &str = "samples/SO-SUEOP.hwp";

fn sample_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn load(path: &std::path::Path) -> rhwp::model::document::Document {
    let data = std::fs::read(path).expect("파일 읽기");
    rhwp::parser::parse_document(&data).expect("파싱")
}

/// 문단이 실제로 차지하는 UTF-16 코드 유닛 수 (종결자 제외).
/// 탭은 본문에서 한 글자로 보이지만 HWP5 스트림에서는 **8 코드 유닛**을 차지한다
/// (0x0009 + 확장 7). 1로 세면 탭이 든 문단마다 판정이 어긋난다.
fn body_code_units(para: &rhwp::model::paragraph::Paragraph) -> u32 {
    para.text
        .chars()
        .map(|c| {
            if c == '\t' {
                8
            } else {
                c.encode_utf16(&mut [0; 2]).len() as u32
            }
        })
        .sum()
}

/// HWP3 파싱이 문단 종결자를 세는지 — 규약 준수.
#[test]
fn hwp3_char_count_includes_the_paragraph_terminator() {
    let doc = load(&sample_path(HWP3_SAMPLE));
    let mut checked = 0usize;
    let mut violations = Vec::new();
    for section in &doc.sections {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            // 컨트롤이 섞인 문단은 코드 유닛 계산이 복잡해 순수 텍스트 문단만 본다.
            if !para.controls.is_empty() || para.text.is_empty() {
                continue;
            }
            checked += 1;
            let expected = body_code_units(para) + 1;
            if para.char_count != expected {
                if violations.len() < 3 {
                    violations.push(format!(
                        "문단 {pi}: char_count={} 기대={expected} text={:?}",
                        para.char_count,
                        para.text.chars().take(20).collect::<String>()
                    ));
                }
            }
        }
    }
    assert!(
        checked > 100,
        "검사한 문단이 너무 적다({checked}건) — 표본이 바뀌었는지 확인하라"
    );
    assert!(
        violations.is_empty(),
        "HWP3 문단이 종결자를 세지 않는다:\n  {}",
        violations.join("\n  ")
    );
}

/// 규약이 코드에 명시돼 있고, 종결자를 빠뜨린 대입이 다시 들어오지 않는다.
///
/// 소스 가드 — `char_count` 를 텍스트 길이로만 대입하면 규약이 깨진다.
#[test]
fn no_assignment_drops_the_terminator() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            // 테스트 픽스처는 계약 대상이 아니다 — 의도적으로 임의 값을 넣는다.
            if path.to_string_lossy().contains("tests") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for (i, line) in text.lines().enumerate() {
                let t = line.trim();
                // `char_count = <무언가>.count() as u32;` — 종결자 없이 길이만 대입.
                // `<something>.char_count = …` 형태의 **필드 대입**만 본다.
                // `let new_char_count = …` 같은 지역 변수는 대상이 아니다.
                if t.contains(".char_count = ")
                    && t.contains(".count() as u32;")
                    && !t.contains("+ 1")
                {
                    offenders.push(format!("{}:{}", path.display(), i + 1));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "char_count 에 문단 종결자를 빼고 대입한다 ({}건):\n  {}\n\
         공통 IR 의 char_count 는 종결자를 포함한다 (model/paragraph.rs 의 \
         `+1 for paragraph end marker`). 길이 뒤에 `+ 1` 을 붙여라.",
        offenders.len(),
        offenders.join("\n  ")
    );
}
