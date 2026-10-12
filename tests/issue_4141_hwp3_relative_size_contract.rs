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

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/deferred_exam_scope.rs"
));
mod exam_fixtures {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/exam_fixtures.rs"
    ));
}

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
const REL_SIZE_MIN: u8 = 10;
const REL_SIZE_MAX: u8 = 250;

/// HWP3 유래 CharShape 이 가져야 할 값.
///
/// `Hwp3CharShape`(`src/parser/hwp3/records.rs:193-203`)에는 상대크기 필드 **자체가 없다**.
/// 따라서 HWP3 변환본은 예외 없이 OWPML 기본값이어야 한다 — 범위 검사보다 강한 단언이다.
const HWP3_EXPECTED: u8 = 100;

const SAMPLES_ROOT: &str = "samples";

const HWP3_SAMPLE: &str = "samples/hwp3-sample.hwp";

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
            if is_deferred_exam_document(path.to_string_lossy().as_ref())
                || exam_fixtures::is_exam_fixture(&path)
            {
                continue;
            }
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

/// public 저장 경로(`DocumentCore::export_hwp_with_adapter`)도 같은 계약을 지킨다.
#[test]
fn public_document_core_export_also_emits_valid_relative_sizes() {
    let hwp5 = convert_to_hwp5_bytes_via_document_core(&repo_path(HWP3_SAMPLE));
    check_hwp5_relative_sizes(HWP3_SAMPLE, &hwp5).unwrap_or_else(|m| panic!("{m}"));
}

// ── ② HWPX 저장 XML ────────────────────────────────────────────────────────

// ── ③ HML 저장 XML ─────────────────────────────────────────────────────────
