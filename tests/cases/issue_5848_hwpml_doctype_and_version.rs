//! [#5848] DOCTYPE 이 붙은 HWPML 과 `Version="2.1"` 을 연다 — 보안 가드는 그대로.
//!
//! 법제처 국가법령정보센터 배포본은 `<!DOCTYPE HWPML [ <!ENTITY nbsp "&#160;"> ]>` 를
//! 앞에 달고 `Version="2.1"` 로 나온다. 종전 rhwp 는 이것을 네 겹으로 막았다:
//!
//! 1. `has_hwpml_root` 가 `Event::DocType` 을 `_ => false` 로 떨어뜨려 포맷 감지 실패
//!    → "알 수 없는 파일 형식"(실제로는 HWPML 인데 다른 포맷으로 오인)
//! 2. 버전 게이트가 `2.9`/`2.91` 만 통과
//! 3. 본 파서가 `DocType` 을 만나면 `DTD is not allowed` 로 거부
//! 4. `&nbsp;` 가 미리 정의 엔티티가 아니라 `entity &nbsp; is not allowed`
//!
//! 이 시험은 네 겹이 모두 열렸는지와, **열면서 보안 가드가 살아 있는지**를 함께 고정한다.
//! 픽스처는 저장소의 2.91 HML 을 시험 시점에 변형해 만든다 — 새 이진 자산을 넣지
//! 않는 기존 방식(#3707)과 같다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 저장소에 있는 2.91 HWPML 원본.
fn base_hml() -> String {
    let path = repo_root().join("samples/hml/aligns.hml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 루트 앞에 프롤로그를 끼우고 `Version` 을 바꾼 사본을 만든다.
fn variant(prologue: &str, version: Option<&str>, body_patch: Option<(&str, &str)>) -> String {
    let mut xml = base_hml();
    if let Some(v) = version {
        let at = xml.find("<HWPML").expect("HWPML 루트");
        let end = xml[at..].find('>').expect("루트 태그 끝") + at;
        let head = &xml[at..end];
        // 앞의 공백까지 포함해 찾는다 — `SubVersion="` 안에도 `Version="` 이 들어 있어
        // 그냥 찾으면 **SubVersion 을 고치고 Version 은 그대로 둔다**(시험이 엉뚱한
        // 이유로 통과한다). `aligns.hml` 의 루트가 정확히 그 순서다:
        // `<HWPML Style="embed" SubVersion="10.0.0.0" Version="2.91">`
        let needle = " Version=\"";
        let patched = match head.find(needle) {
            Some(vi) => {
                let vs = at + vi + needle.len();
                let ve = vs + xml[vs..].find('"').expect("Version 값 끝");
                format!("{}{}{}", &xml[at..vs], v, &xml[ve..end])
            }
            None => panic!("루트에 Version 속성이 있어야 한다: {head}"),
        };
        xml = format!("{}{}{}", &xml[..at], patched, &xml[end..]);
    }
    if let Some((from, to)) = body_patch {
        xml = xml.replacen(from, to, 1);
    }
    if prologue.is_empty() {
        xml
    } else {
        let at = xml.find("<HWPML").expect("HWPML 루트");
        format!("{}{prologue}\n{}", &xml[..at], &xml[at..])
    }
}

fn open_pages(xml: &str) -> Result<usize, String> {
    let doc =
        rhwp::wasm_api::HwpDocument::from_bytes(xml.as_bytes()).map_err(|e| format!("{e:?}"))?;
    Ok(doc.page_count() as usize)
}

const DOCTYPE_NBSP: &str = "<!DOCTYPE HWPML [\n\t<!ENTITY nbsp\t\"&#160;\">\n]>";

#[test]
fn baseline_2_91_without_doctype_still_opens() {
    let pages = open_pages(&base_hml()).expect("2.91 원본은 종전대로 열려야 한다");
    assert!(pages > 0, "쪽수가 0 이면 안 된다");
}

#[test]
fn doctype_prologue_no_longer_breaks_format_detection() {
    // ①③ — DOCTYPE 만 붙인 판본. 종전에는 "알 수 없는 파일 형식" 이었다.
    let xml = variant(DOCTYPE_NBSP, None, None);
    let pages = open_pages(&xml).unwrap_or_else(|e| panic!("DOCTYPE 이 붙어도 열려야 한다 — {e}"));
    assert!(pages > 0);
}

#[test]
fn hwpml_version_2_1_is_accepted() {
    // ② — 버전만 2.1 로 바꾼 판본. 파서는 버전 값으로 분기하지 않으므로
    // 통과시켜도 해석이 달라지지 않는다.
    let xml = variant("", Some("2.1"), None);
    let pages = open_pages(&xml).unwrap_or_else(|e| panic!("HWPML 2.1 은 열려야 한다 — {e}"));
    assert!(pages > 0);
}

#[test]
fn declared_entity_in_body_resolves() {
    // ④ — 법제처 배포본과 같은 형태: DOCTYPE + 2.1 + 본문에서 `&nbsp;` 사용.
    let xml = variant(
        DOCTYPE_NBSP,
        Some("2.1"),
        Some(("</CHAR>", "&nbsp;</CHAR>")),
    );
    let pages =
        open_pages(&xml).unwrap_or_else(|e| panic!("선언된 엔티티를 쓴 본문도 열려야 한다 — {e}"));
    assert!(pages > 0);
}

#[test]
fn nested_entity_declaration_is_still_refused() {
    // 확장 폭탄(billion laughs) — 값에 다른 엔티티 참조가 있는 선언은 싣지 않으므로
    // 본문에서 그 이름을 부르면 종전대로 거부된다.
    let bomb = "<!DOCTYPE HWPML [\n\
                <!ENTITY a \"AAAAAAAAAA\">\n\
                <!ENTITY b \"&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;\">\n\
                <!ENTITY c \"&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;\">\n\
                ]>";
    let xml = variant(bomb, Some("2.1"), Some(("</CHAR>", "&c;</CHAR>")));
    let err = open_pages(&xml).expect_err("중첩 엔티티는 거부되어야 한다");
    assert!(
        err.contains("&c;") || err.to_lowercase().contains("entity"),
        "엔티티 거부 오류여야 한다: {err}"
    );
}

#[test]
fn external_entity_declaration_is_still_refused() {
    // XXE — `SYSTEM` 선언은 값을 읽지도 않으므로 본문 참조가 거부된다.
    let xxe = "<!DOCTYPE HWPML [\n\
               <!ENTITY xx SYSTEM \"file:///C:/Windows/win.ini\">\n\
               ]>";
    let xml = variant(xxe, Some("2.1"), Some(("</CHAR>", "&xx;</CHAR>")));
    let err = open_pages(&xml).expect_err("외부 엔티티는 거부되어야 한다");
    assert!(
        err.contains("&xx;") || err.to_lowercase().contains("entity"),
        "엔티티 거부 오류여야 한다: {err}"
    );
}

#[test]
fn unknown_hwpml_version_is_still_refused() {
    // 게이트를 없앤 것이 아니라 2.1 을 더한 것이다 — 모르는 버전은 그대로 막힌다.
    let xml = variant("", Some("9.9"), None);
    let err = open_pages(&xml).expect_err("모르는 버전은 거부되어야 한다");
    assert!(
        err.contains("9.9") || err.contains("버전"),
        "버전 거부 오류여야 한다: {err}"
    );
}
