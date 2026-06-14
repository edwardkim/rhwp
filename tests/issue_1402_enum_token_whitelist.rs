//! Task #1402 — 열거 속성 표면 표기 정합 검사.
//!
//! serializer 가 HWPX 로 방출하는 열거 속성 토큰이 **실물 관측 집합 ∪ owpml 표준값**
//! 화이트리스트에 속하는지 검증한다. #1387 numType "FIGURE"(한컴 비실재 표기 →
//! 캡션 번호 미출력) 재발을 봉인하는 것이 목적.
//!
//! 검사 방법: samples/hwpx 전수를 parse→serialize 한 뒤, 방출 XML 에서 속성별
//! 토큰을 추출해 화이트리스트와 대조. 파서가 관용 수용(FIGURE/PICTURE 등)해도
//! serializer 방출은 화이트리스트로 고정된다.
//!
//! 화이트리스트 갱신: 새 열거 속성/토큰을 방출하게 되면, 한컴 실물 또는 owpml
//! 스펙으로 확인 후 아래 WHITELIST 에 추가한다 (추정 금지 — 실물·스펙 교차).

use std::collections::HashMap;

use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

/// (속성명, 허용 토큰 집합). 근거: 실물 관측(samples/hwpx) ∪ owpml/한컴 스펙.
fn whitelist() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();
    // autoNum/newNum numType — #1387: PICTURE(실물), FIGURE 금지. 나머지 owpml 표준.
    m.insert(
        "numType",
        vec![
            "PAGE", "FOOTNOTE", "ENDNOTE", "PICTURE", "TABLE", "EQUATION",
        ],
    );
    // 개체 numberingType
    m.insert(
        "numberingType",
        vec!["NONE", "PICTURE", "TABLE", "EQUATION"],
    );
    // 표 pageBreak (#1393): NONE/CELL/TABLE. (셀 외 pageBreak="0/1"은 다른 의미라 제외)
    m.insert("pageBreak", vec!["NONE", "CELL", "TABLE"]);
    m.insert(
        "textWrap",
        vec![
            "SQUARE",
            "TIGHT",
            "THROUGH",
            "TOP_AND_BOTTOM",
            "BEHIND_TEXT",
            "IN_FRONT_OF_TEXT",
        ],
    );
    m.insert(
        "textFlow",
        vec!["BOTH_SIDES", "LEFT_ONLY", "RIGHT_ONLY", "LARGEST_ONLY"],
    );
    m.insert("vertRelTo", vec!["PAPER", "PAGE", "PARA"]);
    m.insert("horzRelTo", vec!["PAPER", "PAGE", "COLUMN", "PARA"]);
    m.insert(
        "vertAlign",
        vec!["TOP", "CENTER", "BOTTOM", "INSIDE", "OUTSIDE"],
    );
    m.insert(
        "horzAlign",
        vec!["LEFT", "CENTER", "RIGHT", "INSIDE", "OUTSIDE"],
    );
    m.insert("applyPageType", vec!["BOTH", "EVEN", "ODD"]);
    m.insert("lineWrap", vec!["BREAK", "SQUEEZE"]);
    m.insert(
        "familyType",
        vec![
            "FCAT_MYUNGJO",
            "FCAT_GOTHIC",
            "FCAT_SSERIF",
            "FCAT_BRUSHSCRIPT",
            "FCAT_DECORATIVE",
            "FCAT_NONRECTMJ",
            "FCAT_NONRECTGT",
            "FCAT_UNKNOWN",
        ],
    );
    // gutterType (#1388)
    m.insert("gutterType", vec!["LEFT_ONLY", "LEFT_RIGHT", "TOP_BOTTOM"]);
    // 캡션 side (#1387)
    m.insert("side", vec!["LEFT", "RIGHT", "TOP", "BOTTOM"]);
    // circleType: SHAPE_REVERSAL_TIRANGLE 은 한컴 owpml 오타 철자 추정 — 파서·
    // serializer 동일 철자라 roundtrip 정합. owpml 권위 자료 미접근으로 현행 유지.
    m.insert(
        "circleType",
        vec![
            "CHAR",
            "SHAPE_CIRCLE",
            "SHAPE_REVERSAL_CIRCLE",
            "SHAPE_RECTANGLE",
            "SHAPE_REVERSAL_RECTANGLE",
            "SHAPE_TRIANGLE",
            "SHAPE_REVERSAL_TIRANGLE",
        ],
    );
    m.insert("composeType", vec!["OVERLAP", "SPREAD"]);
    m
}

/// XML 에서 `attr="TOKEN"` 형태의 토큰을 추출 (알파벳/언더스코어로 시작하는 값만 —
/// 숫자·소수점 값은 열거 토큰이 아니므로 제외).
fn extract_tokens(xml: &str, attr: &str) -> Vec<String> {
    let needle = format!("{attr}=\"");
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(p) = xml[start..].find(&needle) {
        let vstart = start + p + needle.len();
        if let Some(end) = xml[vstart..].find('"') {
            let val = &xml[vstart..vstart + end];
            // 열거 토큰: 첫 글자가 영문 대문자. (lineWrap/numType 등) 숫자 값 제외.
            if val.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                out.push(val.to_string());
            }
            start = vstart + end + 1;
        } else {
            break;
        }
    }
    out
}

fn all_section_xml(doc: &rhwp::model::document::Document) -> String {
    // serialize_hwpx 는 ZIP 바이트라 직접 XML 비교가 번거롭다. 대신 재직렬화
    // 결과를 재파싱하지 않고, serialize 한 ZIP 에서 section/header XML 을 모은다.
    let bytes = serialize_hwpx(doc).expect("serialize");
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).expect("zip");
    let mut xml = String::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let name = f.name().to_string();
        if name.contains("Contents/") && name.ends_with(".xml") {
            use std::io::Read;
            let mut s = String::new();
            f.read_to_string(&mut s).ok();
            xml.push_str(&s);
        }
    }
    xml
}

#[test]
fn serializer_enum_tokens_within_whitelist() {
    let wl = whitelist();
    let mut violations: Vec<String> = Vec::new();

    for entry in std::fs::read_dir("samples/hwpx").expect("samples") {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "hwpx").unwrap_or(false) {
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let Ok(doc) = parse_hwpx(&bytes) else {
                continue;
            };
            let Ok(_) = std::panic::catch_unwind(|| serialize_hwpx(&doc)).unwrap_or(Err(
                rhwp::serializer::SerializeError::XmlError("skip".into()),
            )) else {
                continue; // SERIALIZE_FAIL 샘플은 건너뜀 (#1384 외 — 현재 없음)
            };
            let xml = all_section_xml(&doc);
            for (attr, allowed) in &wl {
                for tok in extract_tokens(&xml, attr) {
                    if !allowed.contains(&tok.as_str()) {
                        violations.push(format!(
                            "{}: {attr}=\"{tok}\" (화이트리스트 밖)",
                            path.file_name().unwrap().to_string_lossy()
                        ));
                    }
                }
            }
        }
    }
    violations.sort();
    violations.dedup();
    assert!(
        violations.is_empty(),
        "serializer 가 화이트리스트 밖 열거 토큰 방출 ({}건):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

#[test]
fn numtype_figure_regression_guard() {
    // #1387 회귀 봉인 — 어떤 샘플도 serialize 후 numType="FIGURE" 를 방출하면 안 된다.
    for entry in std::fs::read_dir("samples/hwpx").expect("samples") {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "hwpx").unwrap_or(false) {
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let Ok(doc) = parse_hwpx(&bytes) else {
                continue;
            };
            let Ok(out) = serialize_hwpx(&doc) else {
                continue;
            };
            let reader = std::io::Cursor::new(out);
            if let Ok(mut zip) = zip::ZipArchive::new(reader) {
                for i in 0..zip.len() {
                    let mut f = zip.by_index(i).unwrap();
                    if f.name().contains("Contents/") && f.name().ends_with(".xml") {
                        use std::io::Read;
                        let mut s = String::new();
                        f.read_to_string(&mut s).ok();
                        assert!(
                            !s.contains("numType=\"FIGURE\""),
                            "{}: numType=\"FIGURE\" 방출 — #1387 재발 (PICTURE 여야 함)",
                            path.file_name().unwrap().to_string_lossy()
                        );
                    }
                }
            }
        }
    }
}
