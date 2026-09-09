//! [#6956] 형광펜 표시를 왕복에서 잃지 않는다.
//!
//! **증상.** HWPX 를 rhwp 로 열었다 저장하면 형광펜이 통째로 사라진다. 글자는 남고
//! 강조만 없어져 글자수·쪽수가 안 변하므로 기존 스윕 지표에 안 잡힌다.
//!
//! **근인.** 지원 코드가 아예 없었다 — `grep -rn markpen src/` 가 0건이었다.
//! `<hp:markpenBegin color="#FFFF00"/>`·`<hp:markpenEnd/>` 는 `<hp:t>` 안 글자 사이에
//! 오는 인라인 표지인데 파서의 `_ => {}` 로 떨어지고, 직렬화기는 만들지 않았다.
//!
//! **수정.** `hp:titleMark` 와 같은 부수 채널(`Paragraph::markpen_marks`)로 위치와 색만
//! 보존한다. **글자 축은 건드리지 않는다** — 한컴 원본의 `hp:lineseg/@textpos` 가 이
//! 표지를 세지 않으므로 유닛을 더하면 축이 어긋나 한글이 본문을 버린다(`#5252` 계열).
//!
//! **모수.** 코퍼스 HWPX 3,418건 중 형광펜 문서 **18건 · 표지 59개**. 수정 후 **17/18**
//! 이 개수·색까지 보존된다.
//!
//! **경계.** 남는 1건은 이 재현물의 두 번째 모양이다 — 표지가 `hp:t` 가 아니라 `hp:run`
//! 바로 밑에서 **표를 감싼다**. 글자 축 위 좌표로 표현할 수 없어 이 축이 다루지 않는다.
//!
//! ```xml
//! <hp:run><hp:markpenBegin color="#FFFFFF"/><hp:tbl …/><hp:t><hp:markpenEnd/></hp:t></hp:run>
//! ```
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use zip::ZipArchive;

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6956/3024739-exposure-algorithm-markpen.hwpx")
        .to_string_lossy()
        .into_owned()
}

/// 원본을 읽어 다시 HWPX 로 저장하고 `section0.xml` 을 돌려준다.
fn roundtrip_section() -> String {
    let bytes = std::fs::read(sample()).expect("재현물을 읽어야 한다");
    let doc = rhwp::parser::parse_document(&bytes).expect("HWPX 원본은 열려야 한다");
    let out = rhwp::serializer::hwpx::serialize_hwpx(&doc).expect("HWPX 저장은 성공해야 한다");
    let mut zip = ZipArchive::new(std::io::Cursor::new(out)).expect("산출물은 zip 이다");
    let mut xml = String::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).expect("zip 항목");
        if f.name().to_ascii_lowercase().contains("section0") {
            use std::io::Read;
            f.read_to_string(&mut xml).expect("section0 읽기");
            break;
        }
    }
    assert!(!xml.is_empty(), "section0.xml 을 찾지 못했다");
    xml
}

fn count(hay: &str, needle: &str) -> usize {
    hay.matches(needle).count()
}

#[test]
fn issue_6956_text_level_markpen_survives_the_roundtrip() {
    // **뒤집힘** — `hp:t` 안 표지 2쌍이 색까지 남는다. 종전에는 0개였다.
    let xml = roundtrip_section();
    assert_eq!(
        count(&xml, "<hp:markpenBegin"),
        2,
        "글자 축 위 여는 표지 2개가 남아야 한다"
    );
    assert_eq!(
        count(&xml, "<hp:markpenEnd"),
        2,
        "닫는 표지도 짝만큼 남아야 한다 — 런 **끝**에 오는 형태를 흘리면 안 된다"
    );
    assert_eq!(
        count(&xml, r##"<hp:markpenBegin color="#FFFFFF"/>"##),
        2,
        "색이 원본 그대로여야 한다"
    );
}

#[test]
fn issue_6956_markpen_does_not_consume_the_text_axis() {
    // **음성 대조** — 표지는 글자가 아니다. `text` 에 들어가면 추출·재조판이 어긋나고
    // 유닛을 먹으면 `hp:lineseg/@textpos` 축이 밀려 한글이 본문을 버린다.
    let bytes = std::fs::read(sample()).expect("재현물");
    let doc = rhwp::parser::parse_document(&bytes).expect("원본");
    let marked: Vec<_> = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .filter(|p| !p.markpen_marks.is_empty())
        .collect();
    assert!(!marked.is_empty(), "표지를 읽었어야 한다");
    for para in marked {
        assert!(
            !para.text.contains('\u{0007}'),
            "sentinel 이 본문 텍스트로 새면 안 된다: {:?}",
            para.text
        );
        for mark in &para.markpen_marks {
            assert!(
                mark.char_idx <= para.text.chars().count(),
                "표지 위치가 문단 글자 축 안이어야 한다"
            );
        }
    }
}

#[test]
fn issue_6956_run_level_markpen_around_a_table_is_out_of_scope() {
    // **경계 잠금** — 표를 감싼 run-level 표지는 글자 축 좌표로 표현할 수 없어 이 축이
    // 다루지 않는다. 원본에는 3쌍이 있고 그중 2쌍만 왕복한다는 사실을 못박아, 나중에
    // 이 수가 바뀌면 **의도한 변경**임을 시험이 강제한다.
    let xml = roundtrip_section();
    assert_eq!(
        count(&xml, "<hp:markpenBegin"),
        2,
        "원본 3쌍 중 글자 축 위 2쌍만 왕복한다 (표를 감싼 1쌍은 별도 축)"
    );
}
