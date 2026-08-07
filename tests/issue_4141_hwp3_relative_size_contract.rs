//! Issue #4141: HWP3 변환본의 글자 상대크기가 0 으로 저장돼 한컴이 전 본문을 ~0.1pt 로 그린다.
//!
//! ## 증상
//!
//! `rhwp convert` 로 만든 HWP3 → HWP5 변환본을 한컴이 **연다**(오류·복구 대화상자 없음).
//! 그런데 사실상 백지다 — `samples/SO-SUEOP.hwp` 46쪽 10,604개 텍스트 span 이 전부 ~0.12pt 로
//! 그려진다(원본은 9.7~50.5pt). 위치는 정상이고 크기만 붕괴한다.
//!
//! 개봉 자체를 거부하는 [#3676] 과는 **실패 부류가 다르다**. 그래서 계약도 따로 둔다.
//!
//! ## 근인
//!
//! `CharShape.relative_sizes` 가 `[0; 7]` 로 직렬화된다. 한컴은 `크기 × 상대크기%` 로 해석하므로
//! 10pt × 0% ≈ 0.1pt 다 — 실측 0.12pt 와 부합한다.
//!
//! OWPML 은 `relSz` 를 `xs:positiveInteger` minInclusive=10 / maxInclusive=250,
//! `default="100"` 으로 정의한다(`mydocs/manual/OWPML SCHEMA/Header XML schema.xml:716-728`).
//! **0 은 타입 수준에서 이미 불법이다.**
//!
//! 같은 문서의 한컴산 HWPX(`samples/SO-SUEOP.hwpx`)는 charPr 51개가 예외 없이 `relSz=100` 이다.
//! 반면 `ratio`(95/90/100/97)와 `spacing`(-1/-2/-3/0)은 편차가 있다 — 그 둘은 HWP3 레코드의
//! 진짜 데이터고, 잘못된 것은 `relSz` 하나뿐이다.
//!
//! ## 이 테스트가 필요한 이유
//!
//! 판정의 정답지는 한컴이지만 CI 에는 한컴이 없다. 그리고 rhwp 자신은 이 결함을 **볼 수 없다** —
//! 렌더러가 `relative_sizes` 를 아예 읽지 않아(`src/renderer/style_resolver.rs:339-361`)
//! `export-text`·자체 렌더·`convert --verify` 가 전부 정상으로 나온다. `hwp5_roundtrip_baseline`
//! 은 HWP3 를 자동 제외하고, `--verify` 의 IR diff 는 CharShape **속성값을 비교하지 않는다**.
//!
//! 그래서 **저장 바이트/XML 에서** 직접 검사한다. 이 테스트가 CI 의 유일한 방어선이다.
//!
//! [#3676]: https://github.com/edwardkim/rhwp/issues/3676
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::{Path, PathBuf};

use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::{detect_format, record::Record, tags, FileFormat};

/// CHAR_SHAPE payload 안에서 `relative_sizes`(7 × u8)가 놓이는 구간.
///
/// 레이아웃 정본은 `src/parser/doc_info.rs:520-577` 이고 라이터
/// `src/serializer/char_shape.rs:12-27` 이 같은 순서로 쓴다:
/// font_ids 0..14 / ratios 14..21 / spacings 21..28 / **relative_sizes 28..35** /
/// char_offsets 35..42 / base_size 42..46 / attr 46..50.
const REL_SIZE_RANGE: std::ops::Range<usize> = 28..35;

/// OWPML `relSz` 유효범위 — `Header XML schema.xml:716-728`.
const REL_SIZE_MIN: u8 = 10;
const REL_SIZE_MAX: u8 = 250;

/// HWP3 유래 CharShape 이 가져야 할 값.
///
/// `Hwp3CharShape`(`src/parser/hwp3/records.rs:193-203`)에는 상대크기 필드 **자체가 없다**.
/// 따라서 HWP3 변환본은 예외 없이 OWPML 기본값이어야 한다 — 범위 검사보다 강한 단언이다.
const HWP3_EXPECTED: u8 = 100;

const SAMPLES_ROOT: &str = "samples";
const SO_SUEOP: &str = "samples/SO-SUEOP.hwp";
const HWP3_SAMPLE: &str = "samples/hwp3-sample.hwp";
/// `<CHARSHAPE>` 가 `FONTID` 만 갖고 `RELSIZE`·`RATIO` 자식이 없는 HML.
/// HML 리더가 채우지 않으면 왕복 저장에 `RELSIZE="0"` 이 나간다.
const HML_WITHOUT_RELSIZE: &str = "tests/fixtures/hml/exambank_math_equations_min.hml";

/// Stage 1 실측(2026-08-07, 분기 기준 `upstream/devel` 0fdac31ba): 수정 전에는 HWP3 표본 15건의
/// CHAR_SHAPE **68,744개 전건**이 relSz=0 이었다. 스윕이 조용히 비면 회귀를 놓치므로 하한을 둔다.
const MIN_SWEPT_SAMPLES: usize = 10;

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// `samples/` 에서 HWP3 서명 파일을 재귀 수집한다 (루트 기준 상대경로, 사전순).
fn hwp3_samples() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, root: &Path, acc: &mut Vec<(PathBuf, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, acc);
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("hwp"))
            {
                let Ok(bytes) = std::fs::read(&path) else {
                    continue;
                };
                if detect_format(&bytes) != FileFormat::Hwp3 {
                    continue;
                }
                let rel = path
                    .strip_prefix(root)
                    .expect("strip_prefix")
                    .to_string_lossy()
                    .replace('\\', "/");
                acc.push((path, rel));
            }
        }
    }
    let root = repo_path(SAMPLES_ROOT);
    let mut acc = Vec::new();
    walk(&root, &root, &mut acc);
    acc.sort_by(|a, b| a.1.cmp(&b.1));
    assert!(
        !acc.is_empty(),
        "samples 에 HWP3 표본이 없다 — 표본 배치를 확인하라"
    );
    acc
}

/// CLI `convert` 와 같은 경로로 HWP3 를 HWP5 바이트로 만든다.
/// (`tests/issue_3676_hwp3_convert_hancom_openable.rs:46-54` 와 같은 조합, 경로만 파라미터화)
fn convert_to_hwp5_bytes(path: &Path) -> Option<Vec<u8>> {
    let raw = std::fs::read(path).expect("표본 읽기");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).ok()?;
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    rhwp::serializer::cfb_writer::serialize_hwp(&doc).ok()
}

/// public 저장 경로 (`DocumentCore::export_hwp_with_adapter`).
fn convert_to_hwp5_bytes_via_document_core(path: &Path) -> Vec<u8> {
    let raw = std::fs::read(path).expect("표본 읽기");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&raw).expect("HWP3 파싱");
    core.export_hwp_with_adapter().expect("HWP5 직렬화")
}

/// 저장된 HWP5 바이트의 DocInfo 에서 CHAR_SHAPE payload 를 꺼낸다.
///
/// `Record::read_all` 이 레코드 순회의 정본이다(`src/parser/record.rs:34`) — 테스트마다
/// `walk_records` 를 손으로 다시 쓰지 않는다. `read_doc_info` 가 압축 해제까지 한다.
fn char_shape_payloads(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut cfb = CfbReader::open(bytes).expect("CFB 열기");
    let file_header = cfb.read_file_header().expect("FileHeader 읽기");
    let compressed = file_header.get(36).is_some_and(|b| b & 0x01 != 0);
    let doc_info = cfb.read_doc_info(compressed).expect("DocInfo 읽기");
    Record::read_all(&doc_info)
        .expect("DocInfo record 파싱")
        .into_iter()
        .filter(|record| record.tag_id == tags::HWPTAG_CHAR_SHAPE)
        .map(|record| record.data)
        .collect()
}

/// HWPX/HML 처럼 텍스트 엔트리를 하나 꺼낸다.
fn zip_text_entry(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("ZIP 열기");
    let mut entry = archive
        .by_name(name)
        .unwrap_or_else(|e| panic!("{name} 찾기: {e}"));
    let mut text = String::new();
    entry.read_to_string(&mut text).expect("엔트리 읽기");
    text
}

/// `<hh:relSz .../>` 나 `<RELSIZE .../>` 같은 태그에서 정수 속성값을 모은다.
fn tag_int_attrs(tag: &str) -> Vec<i64> {
    let mut out = Vec::new();
    let bytes = tag.as_bytes();
    let mut i = 0;
    while let Some(open) = bytes[i..].iter().position(|&b| b == b'"') {
        let start = i + open + 1;
        let Some(close) = bytes[start..].iter().position(|&b| b == b'"') else {
            break;
        };
        if let Ok(v) = tag[start..start + close].parse::<i64>() {
            out.push(v);
        }
        i = start + close + 1;
    }
    out
}

/// 여는 태그 `<name ... />` 를 전부 찾는다 (속성 안에 `>` 가 없다는 전제 — 숫자 속성뿐이다).
fn find_tags<'a>(xml: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!("<{name}");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(at) = rest.find(&open) {
        let after = &rest[at + open.len()..];
        // `<hh:relSz` 가 `<hh:relSzFoo` 를 잡지 않도록 경계 확인
        if !after.starts_with([' ', '/', '>', '\t', '\n', '\r']) {
            rest = &rest[at + open.len()..];
            continue;
        }
        let Some(end) = after.find('>') else { break };
        out.push(&rest[at..at + open.len() + end + 1]);
        rest = &after[end + 1..];
    }
    out
}

/// 유효범위 위반을 `(인덱스, 값)` 으로 모은다.
fn out_of_range(values: &[u8]) -> Vec<(usize, u8)> {
    values
        .iter()
        .enumerate()
        .filter(|(_, &v)| !(REL_SIZE_MIN..=REL_SIZE_MAX).contains(&v))
        .map(|(i, &v)| (i, v))
        .collect()
}

const LANG: [&str; 7] = ["한글", "영어", "한자", "일어", "기타", "기호", "사용자"];

fn why_it_matters() -> &'static str {
    "한컴은 `크기 × 상대크기%` 로 해석하므로 0 이면 10pt 글자가 0.1pt 로 그려져 문서가 \
     백지가 된다 (#4141: SO-SUEOP 46쪽 10,604 span 전수 0.12pt 한컴 PDF 실측). rhwp \
     렌더러는 이 필드를 읽지 않아 자체 검증으로는 드러나지 않는다."
}

/// HWP5 저장 바이트의 CHAR_SHAPE 상대크기를 검사하고 사람이 읽을 실패 사유를 만든다.
fn check_hwp5_relative_sizes(label: &str, hwp5: &[u8]) -> Result<usize, String> {
    let payloads = char_shape_payloads(hwp5);
    if payloads.is_empty() {
        return Err(format!(
            "{label}: CHAR_SHAPE 레코드가 없다 — 저장 경로를 확인하라"
        ));
    }
    let mut violations = Vec::new();
    for (id, payload) in payloads.iter().enumerate() {
        if payload.len() < REL_SIZE_RANGE.end {
            return Err(format!(
                "{label}: CHAR_SHAPE id={id} payload 가 {}바이트로 짧다 (상대크기는 오프셋 \
                 {}..{} 에 있어야 한다)",
                payload.len(),
                REL_SIZE_RANGE.start,
                REL_SIZE_RANGE.end
            ));
        }
        let sizes = &payload[REL_SIZE_RANGE];
        if let Some((slot, &value)) = sizes.iter().enumerate().find(|(_, &v)| v != HWP3_EXPECTED) {
            violations.push((id, slot, value));
        }
    }
    if violations.is_empty() {
        return Ok(payloads.len());
    }
    let (id, slot, value) = violations[0];
    Err(format!(
        "{label}: CHAR_SHAPE {}개 중 {}개의 상대크기가 {HWP3_EXPECTED} 이 아니다 \
         (첫 위반 id={id} {}={value}). HWP3 레코드에는 상대크기 필드가 없으므로 \
         (src/parser/hwp3/records.rs:193) 변환본은 예외 없이 OWPML 기본값 {HWP3_EXPECTED} 이어야 \
         한다 — 유효범위는 {REL_SIZE_MIN}~{REL_SIZE_MAX} 다 (Header XML schema.xml:716-728). {}",
        payloads.len(),
        violations.len(),
        LANG[slot],
        why_it_matters()
    ))
}

/// XML 의 `relSz` 계열 태그가 유효범위 안인지 검사한다.
fn check_xml_rel_sz(
    label: &str,
    xml: &str,
    tag_name: &str,
    schema_ref: &str,
) -> Result<usize, String> {
    let tags = find_tags(xml, tag_name);
    if tags.is_empty() {
        return Err(format!(
            "{label}: <{tag_name}> 이 하나도 없다 — 저장 경로를 확인하라"
        ));
    }
    let mut bad = Vec::new();
    for tag in &tags {
        let values: Vec<u8> = tag_int_attrs(tag)
            .into_iter()
            .map(|v| v.clamp(0, 255) as u8)
            .collect();
        if !out_of_range(&values).is_empty() {
            bad.push(*tag);
        }
    }
    if bad.is_empty() {
        return Ok(tags.len());
    }
    Err(format!(
        "{label}: <{tag_name}> {}개 중 {}개가 유효범위 {REL_SIZE_MIN}~{REL_SIZE_MAX} 밖이다 \
         (첫 위반: `{}`). OWPML 은 상대크기를 positiveInteger 로 정의하므로 0 은 스키마 \
         위반이다 ({schema_ref}). {}",
        tags.len(),
        bad.len(),
        bad[0],
        why_it_matters()
    ))
}

// ── ① HWP5 저장 바이트 — HWP3 표본 전수 ────────────────────────────────────

/// HWP3 표본 전수를 변환해 저장 바이트의 상대크기가 전부 100 인지 본다.
///
/// 단일 표본으로는 부족하다 — `src/parser/hwp3/mod.rs:3566` 이 인덱스 0 자리에
/// `CharShape::default()` 를 넣으므로 **모든** HWP3 문서에 기본값 레코드가 하나씩 생기고,
/// 변환기 한 곳만 고친 부분 수정도 그 자리를 통과시킬 수 있다.
#[test]
fn hwp3_convert_emits_valid_relative_sizes_for_every_sample() {
    let mut failures = Vec::new();
    let mut swept = 0usize;
    let mut total_shapes = 0usize;

    for (path, rel) in hwp3_samples() {
        // 암호 HWP3 는 비밀번호 없이 파싱되지 않는다 — 건너뛴다.
        let Some(hwp5) = convert_to_hwp5_bytes(&path) else {
            continue;
        };
        swept += 1;
        match check_hwp5_relative_sizes(&rel, &hwp5) {
            Ok(n) => total_shapes += n,
            Err(message) => failures.push(format!("  {message}")),
        }
    }

    assert!(
        swept >= MIN_SWEPT_SAMPLES,
        "HWP3 표본 스윕이 {swept}건뿐이다 (하한 {MIN_SWEPT_SAMPLES}). 전부 건너뛰어 조용히 \
         통과하는 것을 막는 가드다 — 표본 배치나 파싱 실패를 확인하라"
    );
    assert!(
        failures.is_empty(),
        "HWP3 표본 {swept}건 중 {}건 실패 (통과분 CHAR_SHAPE {total_shapes}개):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// #4141 의 재현 표본을 이름으로 고정한다 — 한컴 판정의 대상 문서다.
#[test]
fn so_sueop_convert_relative_sizes_are_all_100() {
    let path = repo_path(SO_SUEOP);
    let hwp5 = convert_to_hwp5_bytes(&path).expect("SO-SUEOP HWP3 변환");
    let count = check_hwp5_relative_sizes(SO_SUEOP, &hwp5).unwrap_or_else(|m| panic!("{m}"));
    assert!(
        count > 1000,
        "{SO_SUEOP}: CHAR_SHAPE 가 {count}개다. Stage 1 실측은 2,512개였다 — 표본이나 변환 \
         경로가 바뀌었는지 확인하라"
    );
}

/// public 저장 경로(`DocumentCore::export_hwp_with_adapter`)도 같은 계약을 지킨다.
#[test]
fn public_document_core_export_also_emits_valid_relative_sizes() {
    let hwp5 = convert_to_hwp5_bytes_via_document_core(&repo_path(HWP3_SAMPLE));
    check_hwp5_relative_sizes(HWP3_SAMPLE, &hwp5).unwrap_or_else(|m| panic!("{m}"));
}

// ── ② HWPX 저장 XML ────────────────────────────────────────────────────────

/// `export-hwpx` 는 HWP3 입력을 받는다(`src/main.rs:272` → `:11311` → `parser/mod.rs:1294`).
/// HWPX 라이터(`src/serializer/hwpx/header.rs:638`)에는 가드가 없어 IR 의 0 이 그대로 나간다.
#[test]
fn hwp3_export_hwpx_emits_valid_rel_sz() {
    for rel in [SO_SUEOP, HWP3_SAMPLE] {
        let raw = std::fs::read(repo_path(rel)).expect("표본 읽기");
        let core = rhwp::document_core::DocumentCore::from_bytes(&raw).expect("HWP3 파싱");
        let hwpx = core.export_hwpx_native().expect("HWPX 저장");
        let xml = zip_text_entry(&hwpx, "Contents/header.xml");
        check_xml_rel_sz(rel, &xml, "hh:relSz", "Header XML schema.xml:716-728")
            .unwrap_or_else(|m| panic!("{m}"));
    }
}

// ── ③ HML 저장 XML ─────────────────────────────────────────────────────────

/// HWP3 → HML 은 도달 불가 경로다 — `export_hml_native` 가 HML 출처에만 있는 `hml_metadata` 를
/// 요구한다(`src/document_core/commands/document.rs:1352-1359`). 대신 **HML 왕복**이 같은
/// 결함을 노출한다: `RELSIZE` 자식이 없는 `<CHARSHAPE>` 를 읽으면 IR 이 0 으로 남고
/// (`src/parser/hml/reader.rs:599-605`) 라이터가 가드 없이 방출한다
/// (`src/serializer/hml/head.rs:131`). 이 fixture 로 배포 CLI 에서 재현된다.
#[test]
fn hml_roundtrip_without_relsize_child_emits_valid_relsize() {
    let raw = std::fs::read(repo_path(HML_WITHOUT_RELSIZE)).expect("HML fixture 읽기");
    let xml_in = String::from_utf8_lossy(&raw);
    assert!(
        find_tags(&xml_in, "RELSIZE").is_empty(),
        "{HML_WITHOUT_RELSIZE} 에 RELSIZE 가 생겼다 — 이 fixture 는 '자식이 없을 때'를 \
         재현해야 한다. 다른 fixture 를 쓰거나 회귀 조건을 다시 잡아라"
    );

    let core = rhwp::document_core::DocumentCore::from_bytes(&raw).expect("HML 파싱");
    let out = core.export_hml_native().expect("HML 저장");
    let xml_out = String::from_utf8_lossy(&out);
    check_xml_rel_sz(
        HML_WITHOUT_RELSIZE,
        &xml_out,
        "RELSIZE",
        "Header XML schema.xml:716-728 (HWPML 대응 필드)",
    )
    .unwrap_or_else(|m| panic!("{m}"));
}
