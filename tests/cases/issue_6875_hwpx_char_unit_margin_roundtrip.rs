//! [#6875] HWPX 로 쓸 때 문단 여백의 `unit="CHAR"` 표시가 사라져 한/글이 쪽수를 다르게 센다.
//!
//! ## 무엇이 문제였나
//!
//! `paraPr` 의 여백·문단 간격은 `<hp:switch>` 아래 두 벌로 적힌다 — `hp:default`(저장값)와
//! HwpUnitChar `hp:case`(저장값의 절반). 저장값이 **홀수**면 절반이 정수로 안 떨어지는데,
//! 한컴은 그 자리를 `unit="CHAR"` 로 표시해 최하위 자리를 보존한다.
//!
//! rhwp 직렬화기는 단위를 `HWPUNIT` 으로 **고정**하고 값만 `x / 2` 로 적었다. 그래서 한/글이
//! 보는 `case` 값은 `80`(=0.8pt)이 되고, 한컴이 적은 `80 CHAR` 와 다른 뜻이 된다.
//!
//! **한/글은 `case` 를 우선 읽으므로** 그 표시가 없으면 문단 간격을 종전보다 작게 잡는다.
//!
//! rhwp **자신의** 왕복은 종전에도 맞았다 — `#3368` 이 `case × 2` 와 `default` 가 1 이내로
//! 어긋나면 `default` 를 채택하게 해 두었기 때문이다. 그래서 이 결함은 rhwp 안에서는 안
//! 보이고, **파일을 받아 읽는 한/글에서만** 드러난다. 그 점이 이 이슈가 `#5585`(rhwp 자체
//! 조판 쪽수)와 갈리는 지점이다.
//!
//! ```text
//!   07939 소방방재 점검메뉴얼 (1660000-200700020, 코퍼스)
//!     문단 간격   22.02pt → 18.78pt      (paraPr 26 · next 저장값 161)
//!     한/글 쪽수  558 → 545  (−13쪽)
//! ```
//!
//! ## 기대값의 독립 근거 — 한컴 자신의 HWP→HWPX 변환본
//!
//! 같은 원본을 한컴 2024(`13.0.0.3901`)가 직접 HWPX 로 바꾼 산출을 받아 전수로 맞댔다.
//! 규칙에 **예외가 없다**.
//!
//! ```text
//!   margin 항목 2,170개 전수   `저장값이 홀수  ⟺  case 단위가 CHAR`   위반 0
//!   한컴 변환본 단위 분포      next/CHAR 65 · intent/CHAR 4 · left/CHAR 4 · right/CHAR 4
//! ```
//!
//! 한컴 변환본을 PDF 로 바꾸면 **558쪽**이다 — HWPX 포맷이 못 담는 것이 아니라 rhwp 가
//! 못 적은 것이다. rhwp 산출의 그 단위만 고쳐 다시 변환하면 545 → **558쪽**으로 닫힌다.
//!
//! ```text
//!   원본 HWP5                    558
//!   한컴 자체 h2x                558
//!   rhwp h2x (수정 전)           545
//!   rhwp h2x + unit="CHAR"       558
//!   rhwp h2x (이 수정)           558
//! ```
//!
//! ## 이 시험이 잠그는 것
//!
//! 코퍼스 문서는 저장소에 없으므로, 같은 형상을 가진 저장소 실물
//! (`samples/issue5714/…vietnam_labor_report.hwp` — 홀수 여백 20문단 · 홀수 간격 12문단)로
//! **왕복 등식**과 **단위 표기 규칙**을 잠근다. 왕복이 성립하면 한/글이 읽는 `case` 값도
//! 한컴 변환본과 같아진다.
//!
//! ## 잠그지 않는 것
//!
//! `#6875` 가 등재한 16경로 중 이 수정이 닿는 것은 홀수 여백이 있는 문서다. 나머지
//! (예: `07374`·`07821`·`07281`·`01821`)는 `CHAR` 항목이 0개라 **다른 갈래**이고 미해결이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

/// 홀수 여백·간격을 실제로 가진 저장소 실물.
const SAMPLE: &str = "samples/issue5714/1490000-200800034_vietnam_labor_report.hwp";

fn read_sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("정식 원본")
}

/// 내보낸 HWPX 의 `Contents/header.xml`.
fn exported_header(bytes: &[u8]) -> String {
    let doc = HwpDocument::from_bytes(bytes).expect("원본 파스");
    let hwpx = doc.export_hwpx().expect("HWPX 내보내기");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip");
    let mut header = String::new();
    {
        use std::io::Read;
        zip.by_name("Contents/header.xml")
            .expect("header.xml")
            .read_to_string(&mut header)
            .expect("read header");
    }
    header
}

/// `hp:case` 안의 `<hc:*>` 를 (이름, 값, 단위)로, 같은 `paraPr` 의 `hp:default` 값과 함께.
fn case_and_default_margins(header: &str) -> Vec<(String, i64, String, i64)> {
    let mut out = Vec::new();
    for para_pr in header.split("<hh:paraPr ").skip(1) {
        let Some(block) = para_pr.split("</hh:paraPr>").next() else {
            continue;
        };
        let Some(case) = block.split("<hp:case").nth(1).and_then(|rest| {
            rest.split_once('>')
                .and_then(|(_, body)| body.split("</hp:case>").next())
        }) else {
            continue;
        };
        let Some(default) = block
            .split("<hp:default>")
            .nth(1)
            .and_then(|rest| rest.split("</hp:default>").next())
        else {
            continue;
        };
        let parse = |body: &str| -> Vec<(String, i64, String)> {
            body.split("<hc:")
                .skip(1)
                .filter_map(|item| {
                    let name = item.split([' ', '/', '>']).next()?.to_string();
                    let value = item.split("value=\"").nth(1)?.split('"').next()?;
                    let unit = item.split("unit=\"").nth(1)?.split('"').next()?;
                    Some((name, value.parse::<i64>().ok()?, unit.to_string()))
                })
                .collect()
        };
        let defaults = parse(default);
        for (name, value, unit) in parse(case) {
            if let Some((_, def_value, _)) = defaults.iter().find(|(n, _, _)| *n == name) {
                out.push((name, value, unit, *def_value));
            }
        }
    }
    out
}

/// 저장값이 홀수인 자리에만 `unit="CHAR"` 가 붙는다 — 한컴 변환본의 전수 규칙.
///
/// 수정 전에는 단위가 전부 `HWPUNIT` 이라 홀수 자리가 하나도 표시되지 않았다.
#[test]
fn odd_stored_margin_is_marked_with_the_char_unit() {
    let header = exported_header(&read_sample());
    let items = case_and_default_margins(&header);
    assert!(
        !items.is_empty(),
        "paraPr 의 hp:case 여백을 하나도 못 읽었다"
    );

    let odd: Vec<_> = items.iter().filter(|(_, _, _, d)| d % 2 != 0).collect();
    assert!(
        !odd.is_empty(),
        "이 표본에는 홀수 저장값이 있어야 한다 — fixture 가 바뀌었다"
    );

    for (name, value, unit, stored) in &items {
        let expected = if stored % 2 != 0 { "CHAR" } else { "HWPUNIT" };
        assert_eq!(
            unit, expected,
            "hc:{name} 저장값 {stored} → case {value} 의 단위가 {unit} 다. \
             한컴은 홀수 자리를 CHAR 로 표시한다(변환본 2,170항 전수 규칙)"
        );
        let restored = value * 2 + i64::from(stored % 2 != 0);
        assert_eq!(
            restored, *stored,
            "hc:{name}: case {value} + 단위 {unit} 로 저장값을 복원할 수 없다"
        );
    }
}

/// 왕복 등식 — HWPX 로 쓰고 다시 읽으면 문단 여백·간격이 **정확히** 돌아온다.
///
/// 이 검사는 결함을 드러내는 쪽이 **아니다**(수정 전에도 통과한다 — 위 `#3368` 보정).
/// 단위 표기를 바꾸면서 rhwp 자신의 왕복이 깨지지 않는지 잠그는 반대편 가드다.
#[test]
fn paragraph_margins_survive_the_hwpx_roundtrip_exactly() {
    let bytes = read_sample();
    let original = HwpDocument::from_bytes(&bytes).expect("원본 파스");
    let hwpx = original.export_hwpx().expect("HWPX 내보내기");
    let reparsed = HwpDocument::from_bytes(&hwpx).expect("HWPX 파스");

    let margins = |doc: &HwpDocument| -> Vec<(i32, i32, i32, i32, i32)> {
        doc.document()
            .doc_info
            .para_shapes
            .iter()
            .map(|ps| {
                (
                    ps.indent,
                    ps.margin_left,
                    ps.margin_right,
                    ps.spacing_before,
                    ps.spacing_after,
                )
            })
            .collect()
    };
    let before = margins(&original);
    let after = margins(&reparsed);
    assert_eq!(
        before.len(),
        after.len(),
        "문단 모양 수가 왕복에서 달라졌다 — {} → {}",
        before.len(),
        after.len()
    );

    let mismatch: Vec<_> = before
        .iter()
        .zip(&after)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(index, (a, b))| format!("paraPr {index}: {a:?} → {b:?}"))
        .collect();
    assert!(
        mismatch.is_empty(),
        "왕복에서 문단 여백이 바뀌었다({}건). 홀수 저장값이 짝수로 내려앉는 결함이다.\n{}",
        mismatch.len(),
        mismatch
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
