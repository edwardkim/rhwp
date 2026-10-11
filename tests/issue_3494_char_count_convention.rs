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

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/deferred_exam_scope.rs"
));

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
            if is_deferred_exam_document(path.to_string_lossy().as_ref()) {
                continue;
            }
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
