//! Issue #4161: `CharShape.ratios` 기본값 0 이 OWPML 유효범위 [50,200] 밖 — 렌더러가
//! 장평 0 으로 소비한다.
//!
//! ## 증상
//!
//! `RATIO` 자식이 없는 HML 을 배포 CLI 로 왕복시키면 장평이 0 으로 나간다:
//!
//! ```bash
//! rhwp export-hml tests/fixtures/hml/exambank_math_equations_min.hml -o /tmp/rt.hml
//! # → <RATIO Hangul="0" Latin="0" Hanja="0" Japanese="0" Other="0" Symbol="0" User="0"/>
//! ```
//!
//! ## 근인
//!
//! `impl Default for CharShape`(`src/model/style.rs`)의 `ratios` 가 `[0; 7]` 이다. OWPML 은
//! `ratio` 를 `xs:positiveInteger` minInclusive=50 / maxInclusive=200, `default="100"` 으로
//! 정의한다(`mydocs/manual/OWPML SCHEMA/Header XML schema.xml:590-611`) — **0 은 타입
//! 수준에서 이미 불법이다.** HWP5 CHAR_SHAPE 파서는 이미 100 을 폴백하므로
//! (`src/parser/doc_info.rs:528-532`) 모델 기본값만 스펙과 어긋나 있다.
//!
//! `relative_sizes`(#4141)와 달리 **렌더러가 이 값을 읽는다** —
//! `src/renderer/style_resolver.rs:355` 가 `/100.0` 으로 나눠 장평 배율을 만든다.
//! 장평 0 = 글자 폭 0. HML 은 렌더 경로이기도 하다(`export-svg`/`export-pdf` 가 `.hml` 을 받는다).
//!
//! ## 이 테스트가 필요한 이유
//!
//! rhwp 자신은 이 결함을 **가려서 본다** — 폭 계산 경로가 전부 `ratio > 0.0` 폴백을 갖고
//! 있어(`src/renderer/layout/text_measurement.rs:71` 등) 자체 렌더는 정상으로 나온다. 그러나
//! 저장 바이트/XML 에는 스키마 불법값 0 이 그대로 실리고, 그 값을 신뢰하는 소비자(한컴,
//! 외부 도구)는 장평 0 을 받는다. HWPX 라운드트립 검증기는 char_shapes 를 개수만 비교하고
//! HML preflight 비교 목록에도 `ratios` 가 없어 기존 게이트로는 잡히지 않는다.
//! 그래서 #4141 과 같은 방식으로 **저장 바이트/XML 에서** 직접 검사한다.
//!
//! ## #4141 과 단언 형태가 다른 이유
//!
//! `relative_sizes` 는 HWP3 레코드에 필드 자체가 없어 `==100` 전수 단언이 가능했지만,
//! `ratios` 는 레코드의 진짜 데이터다(`src/parser/hwp3/records.rs:196` — SO-SUEOP 한컴산
//! HWPX 실측 95/90/100/97 편차). 따라서 여기서는 유효범위 [50,200] **소속**을 단언한다.
//!
//! HWPX `<hh:ratio>` 자식 부재 유닛 케이스는 따로 두지 않는다 — 그 경로는
//! `CharShape::default()` 로 채워지고 기본값 자체는 `src/model/style.rs` 의 잠금 유닛
//! 테스트가 고정하며, 기본값 레코드의 방출은 아래 ①·② 가 HWP3 인덱스 0 placeholder
//! (`src/parser/hwp3/mod.rs:3763` — 모든 HWP3 문서에 1개) 경유로 관통한다.
#![cfg(not(target_arch = "wasm32"))]

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/deferred_exam_scope.rs"
));

use std::io::Read;
use std::path::{Path, PathBuf};

use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::{detect_format, record::Record, tags, FileFormat};

/// CHAR_SHAPE payload 안에서 `ratios`(7 × u8)가 놓이는 구간.
///
/// 레이아웃 정본은 `src/parser/doc_info.rs:520-577` 이고 라이터
/// `src/serializer/char_shape.rs:12-27` 이 같은 순서로 쓴다:
/// font_ids 0..14 / **ratios 14..21** / spacings 21..28 / relative_sizes 28..35 /
/// char_offsets 35..42 / base_size 42..46 / attr 46..50.
const RATIO_RANGE: std::ops::Range<usize> = 14..21;

/// OWPML `ratio` 유효범위 — `Header XML schema.xml:590-611`.
const RATIO_MIN: u8 = 50;
const RATIO_MAX: u8 = 200;

const SAMPLES_ROOT: &str = "samples";

const HWP3_SAMPLE: &str = "samples/hwp3-sample.hwp";

/// #4141 Stage 1 실측: HWP3 표본 15건 전부에 인덱스 0 placeholder 가 정확히 1개씩 있다.
/// 스윕이 조용히 비면 회귀를 놓치므로 하한을 둔다.
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
            if is_deferred_exam_document(path.to_string_lossy().as_ref()) {
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

/// 유효범위 위반을 `(인덱스, 값)` 으로 모은다.
fn out_of_range(values: &[u8], min: u8, max: u8) -> Vec<(usize, u8)> {
    values
        .iter()
        .enumerate()
        .filter(|(_, &v)| !(min..=max).contains(&v))
        .map(|(i, &v)| (i, v))
        .collect()
}

const LANG: [&str; 7] = ["한글", "영어", "한자", "일어", "기타", "기호", "사용자"];

fn why_it_matters() -> &'static str {
    "OWPML ratio 는 positiveInteger [50,200] 이라 0 은 스키마 위반이고, 이 값을 신뢰하는 \
     소비자는 장평 0 = 글자 폭 0 을 받는다. rhwp 자체 렌더는 ratio>0 폴백으로 결함을 \
     가리므로 저장 바이트/XML 검사가 유일한 방어선이다 (#4161)."
}

/// HWP5 저장 바이트의 CHAR_SHAPE 장평이 유효범위 안인지 검사한다.
fn check_hwp5_ratios(label: &str, hwp5: &[u8]) -> Result<usize, String> {
    let payloads = char_shape_payloads(hwp5);
    if payloads.is_empty() {
        return Err(format!(
            "{label}: CHAR_SHAPE 레코드가 없다 — 저장 경로를 확인하라"
        ));
    }
    let mut violations = Vec::new();
    for (id, payload) in payloads.iter().enumerate() {
        if payload.len() < RATIO_RANGE.end {
            return Err(format!(
                "{label}: CHAR_SHAPE id={id} payload 가 {}바이트로 짧다 (장평은 오프셋 \
                 {}..{} 에 있어야 한다)",
                payload.len(),
                RATIO_RANGE.start,
                RATIO_RANGE.end
            ));
        }
        for (slot, value) in out_of_range(&payload[RATIO_RANGE], RATIO_MIN, RATIO_MAX) {
            violations.push((id, slot, value));
        }
    }
    if violations.is_empty() {
        return Ok(payloads.len());
    }
    let (id, slot, value) = violations[0];
    Err(format!(
        "{label}: CHAR_SHAPE {}개 중 위반 {}건 — 첫 위반 id={id} {}={value}. OWPML ratio 는 \
         positiveInteger {RATIO_MIN}~{RATIO_MAX} 다 (Header XML schema.xml:590-611). {}",
        payloads.len(),
        violations.len(),
        LANG[slot],
        why_it_matters()
    ))
}

// ── ① HWP5 저장 바이트 — HWP3 표본 전수 ────────────────────────────────────

/// HWP3 표본 전수를 변환해 저장 바이트의 장평이 전부 유효범위 안인지 본다.
///
/// 단일 표본으로는 부족하다 — `src/parser/hwp3/mod.rs:3763` 이 인덱스 0 자리에
/// `CharShape::default()` 를 넣으므로 **모든** HWP3 문서에 기본값 레코드가 하나씩 생긴다.
/// 실데이터(`convert_char_shape` 가 레코드에서 복사)는 이미 유효범위 안이고, 위반은
/// placeholder 의 기본값에서만 나온다.
#[test]
fn hwp3_convert_emits_valid_ratios_for_every_sample() {
    let mut failures = Vec::new();
    let mut swept = 0usize;
    let mut total_shapes = 0usize;

    for (path, rel) in hwp3_samples() {
        // 암호 HWP3 는 비밀번호 없이 파싱되지 않는다 — 건너뛴다.
        let Some(hwp5) = convert_to_hwp5_bytes(&path) else {
            continue;
        };
        swept += 1;
        match check_hwp5_ratios(&rel, &hwp5) {
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
fn public_document_core_export_also_emits_valid_ratios() {
    let hwp5 = convert_to_hwp5_bytes_via_document_core(&repo_path(HWP3_SAMPLE));
    check_hwp5_ratios(HWP3_SAMPLE, &hwp5).unwrap_or_else(|m| panic!("{m}"));
}

// ── ② HWPX 저장 XML ────────────────────────────────────────────────────────

// ── ③ HML 저장 XML ─────────────────────────────────────────────────────────
