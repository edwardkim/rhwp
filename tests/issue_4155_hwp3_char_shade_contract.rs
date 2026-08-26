//! Issue #4155: HWP3 변환본의 글자 음영이 검정으로 저장돼 한컴이 본문 전체를 검정 막대로 덮는다.
//!
//! ## 증상
//!
//! `rhwp convert` 로 만든 HWP3 → HWP5 변환본을 한컴으로 열면 **본문 전체가 검정 막대**다.
//! 글자는 제 위치·제 크기로 그려지는데 그 위에 순검정 사각형이 칠해진다
//! (`samples/SO-SUEOP.hwp` 변환본 한컴 PDF 3쪽 실측: 줄 크기 검정 fill 65개.
//! 같은 쪽 원본은 글리프 크기 fill 35개 + 밑줄 1개).
//!
//! ## 근인
//!
//! HWP3 글자 음영은 팔레트 인덱스(글자 모양 offset 23, 0~7)와 음영 비율(offset 25, 0~100%)
//! **조합**인데, `convert_char_shape` 가 비율을 무시하고 인덱스만 읽었다. 비율 0 은 음영
//! 없음이고 실문서 다수인데 — `SO-SUEOP.hwp` 는 2,511건 전건이 0 이다 — 인덱스 0(검정)만
//! 보고 `0x00000000` 을 썼다.
//!
//! ## 이 테스트가 필요한 이유
//!
//! 판정의 정답지는 한컴이지만 CI 에는 한컴이 없다. 그리고 rhwp 자신은 이 결함을 **볼 수 없다**
//! — 렌더러가 검정을 "음영 없음" sentinel 로 쓰므로(`src/renderer/svg.rs` 형광펜 조건)
//! `export-svg`·자체 렌더·`convert --verify` 가 전부 정상으로 나온다. 과거 HWPX 라이터는
//! `shade_color == 0 → "none"` 반창고로 비율 0 표본만 우연히 보호했지만, 실제 100% 검정 음영도
//! 잃었다. 이제 HWPX도 `0xFFFFFFFF` sentinel만 `"none"`으로 내보내며, **HWP5 바이너리 축**의
//! 검정 막대 결함과 HWPX의 검정 음영 보존 축을 각각 저장 바이트/XML에서 검증한다.
//!
//! 그래서 **저장 바이트/XML 에서** 직접 검사한다. 이 테스트가 CI 의 유일한 방어선이다.
//!
//! ## 계약의 두 축
//!
//! 1. 저장 바이트에 검정 음영이 없다 (한컴산 HWP5 코퍼스 380건 전수에서 0건인 값이다).
//! 2. 음영이 **있는** 표본은 한컴 저장본과 같은 회색을 낸다 — 이쪽이 합성 공식의 반올림
//!    방향을 고정한다. 절상으로 구현하면 `0xd9d9d9`·`0xf0f0f0` 이 나와 red 가 된다.
//!
//! [#4141]: https://github.com/edwardkim/rhwp/issues/4141
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::{Path, PathBuf};

use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::{detect_format, record::Record, tags, FileFormat};

/// CHAR_SHAPE payload 안에서 `shade_color`(ColorRef, LE u32)가 놓이는 구간.
///
/// 레이아웃 정본은 `src/parser/doc_info.rs` 이고 라이터 `src/serializer/char_shape.rs` 가
/// 같은 순서로 쓴다: font_ids 0..14 / ratios 14..21 / spacings 21..28 /
/// relative_sizes 28..35 / char_offsets 35..42 / base_size 42..46 / attr 46..50 /
/// shadow_offset_x,y 50..52 / text_color 52..56 / underline_color 56..60 /
/// **shade_color 60..64** / shadow_color 64..68.
const SHADE_RANGE: std::ops::Range<usize> = 60..64;

/// 한컴이 "음영 없음"에 쓰는 값. 코퍼스 380건의 CHAR_SHAPE 에서 22,189회.
const NO_SHADE: u32 = 0xFFFF_FFFF;

/// 절대 나오면 안 되는 값. 한컴산 HWP5 코퍼스 380건 전수에서 **0회**다.
const BLACK_SHADE: u32 = 0x0000_0000;

const SAMPLES_ROOT: &str = "samples";
const SO_SUEOP: &str = "samples/SO-SUEOP.hwp";
const HWP3_SAMPLE: &str = "samples/hwp3-sample.hwp";

/// `<CHARSHAPE>` 에 `ShadeColor` 속성이 없는 HML. 리더가 채우지 않으면 왕복 저장에
/// `ShadeColor="0"`(검정)이 나간다. ([#4141] 이 `RELSIZE` 부재에 쓴 것과 같은 fixture)
const HML_WITHOUT_SHADECOLOR: &str = "tests/fixtures/hml/exambank_math_equations_min.hml";

/// 스윕이 조용히 비면 회귀를 놓치므로 하한을 둔다 ([#4141] 과 같은 가드).
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
/// `Record::read_all` 이 레코드 순회의 정본이다 — 테스트마다 손으로 다시 쓰지 않는다.
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

/// 저장 바이트의 CHAR_SHAPE 음영색을 id 순으로 뽑는다.
fn shade_colors(label: &str, hwp5: &[u8]) -> Vec<u32> {
    let payloads = char_shape_payloads(hwp5);
    assert!(
        !payloads.is_empty(),
        "{label}: CHAR_SHAPE 레코드가 없다 — 저장 경로를 확인하라"
    );
    payloads
        .into_iter()
        .enumerate()
        .map(|(id, payload)| {
            assert!(
                payload.len() >= SHADE_RANGE.end,
                "{label}: CHAR_SHAPE id={id} payload 가 {}바이트로 짧다 \
                 (음영색은 오프셋 {}..{} 에 있어야 한다)",
                payload.len(),
                SHADE_RANGE.start,
                SHADE_RANGE.end
            );
            u32::from_le_bytes(payload[SHADE_RANGE].try_into().expect("4바이트"))
        })
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

fn why_it_matters() -> &'static str {
    "한컴은 검정 음영을 글자마다 순검정 사각형으로 칠하므로 본문 전체가 검정 막대가 된다 \
     (#4155: SO-SUEOP 변환본 한컴 PDF 3쪽에 줄 크기 검정 fill 65개). rhwp 렌더러는 검정을 \
     \"음영 없음\" sentinel 로 읽어(renderer/svg.rs) 자체 검증으로는 드러나지 않는다."
}

// ── ① HWP3 표본 전수 — 저장 바이트에 검정 음영이 없다 ─────────────────────────

/// 단일 표본으로는 부족하다 — 변환기 한 곳만 고친 부분 수정도 특정 표본은 통과시킬 수 있다.
#[test]
fn hwp3_convert_never_emits_black_char_shade() {
    let mut failures = Vec::new();
    let mut swept = 0usize;
    let mut total_shapes = 0usize;

    for (path, rel) in hwp3_samples() {
        // 암호 HWP3 는 비밀번호 없이 파싱되지 않는다 — 건너뛴다.
        let Some(hwp5) = convert_to_hwp5_bytes(&path) else {
            continue;
        };
        swept += 1;
        let shades = shade_colors(&rel, &hwp5);
        total_shapes += shades.len();

        let black: Vec<usize> = shades
            .iter()
            .enumerate()
            .filter(|(_, &c)| c == BLACK_SHADE)
            .map(|(id, _)| id)
            .collect();
        if !black.is_empty() {
            failures.push(format!(
                "  {rel}: CHAR_SHAPE {}개 중 {}개가 검정 음영이다 (첫 위반 id={}). \
                 HWP3 글자 음영은 팔레트 인덱스 × 음영 비율이고 비율 0 은 음영 없음이다 \
                 (src/parser/hwp3/mod.rs hwp3_char_shade_color)",
                shades.len(),
                black.len(),
                black[0]
            ));
        }
    }

    assert!(
        swept >= MIN_SWEPT_SAMPLES,
        "HWP3 표본 스윕이 {swept}건뿐이다 (하한 {MIN_SWEPT_SAMPLES}). 전부 건너뛰어 조용히 \
         통과하는 것을 막는 가드다 — 표본 배치나 파싱 실패를 확인하라"
    );
    assert!(
        failures.is_empty(),
        "HWP3 표본 {swept}건 중 {}건 실패 (통과분 CHAR_SHAPE {total_shapes}개):\n{}\n{}",
        failures.len(),
        failures.join("\n"),
        why_it_matters()
    );
}

// ── ② 재현 표본 고정 ───────────────────────────────────────────────────────

/// `SO-SUEOP.hwp` 는 HWP3 원본의 글자 음영 비율이 2,511건 전건 0 이다.
/// 따라서 저장본의 CHAR_SHAPE 음영색은 예외 없이 "음영 없음" sentinel 이어야 한다.
#[test]
fn so_sueop_char_shades_are_all_no_shade() {
    let hwp5 = convert_to_hwp5_bytes(&repo_path(SO_SUEOP)).expect("SO-SUEOP HWP3 변환");
    let shades = shade_colors(SO_SUEOP, &hwp5);

    assert!(
        shades.len() > 1000,
        "{SO_SUEOP}: CHAR_SHAPE 가 {}개다. Stage 1 실측은 2,512개였다 — 표본이나 변환 \
         경로가 바뀌었는지 확인하라",
        shades.len()
    );

    let deviant: Vec<(usize, u32)> = shades
        .iter()
        .enumerate()
        .filter(|(_, &c)| c != NO_SHADE)
        .map(|(id, &c)| (id, c))
        .collect();
    assert!(
        deviant.is_empty(),
        "{SO_SUEOP}: CHAR_SHAPE {}개 중 {}개가 음영 없음이 아니다 \
         (첫 위반 id={} 값=0x{:08x}). 원본은 음영 비율이 2,511건 전건 0 이다. {}",
        shades.len(),
        deviant.len(),
        deviant[0].0,
        deviant[0].1,
        why_it_matters()
    );
}

// ── ③ 한컴 실측 정합 — 합성 공식의 반올림 방향을 고정한다 ──────────────────────

/// 음영이 **있는** 표본이 한컴 저장본과 같은 회색을 내는지 본다.
///
/// 기대값은 이슈 #4155 가 같은 원본의 한컴 저장본에서 잰 값이다. 흰 바탕 lerp 를 절상으로
/// 구현하면 `0xd9d9d9`·`0xf0f0f0` 이 나와 이 테스트가 red 가 된다 — 검정 여부만 보는 ①
/// 로는 못 잡는 축이다.
#[test]
fn hwp3_shaded_samples_match_hancom_gray() {
    // (표본, 한컴 저장본 음영색, HWP3 원본의 팔레트 × 비율)
    let cases: &[(&str, u32, &str)] = &[
        ("samples/hwp3-sample16.hwp", 0x00D8_D8D8, "0 × 15%"),
        ("samples/hwp3-sample5.hwp", 0x00EF_EFEF, "0 × 6%"),
        ("samples/hwp3-sample11.hwp", 0x0099_9999, "0 × 40%"),
        ("samples/hwp3-sample11.hwp", 0x00D8_D8D8, "0 × 15%"),
    ];

    let mut failures = Vec::new();
    for (rel, expected, origin) in cases {
        let path = repo_path(rel);
        if !path.exists() {
            failures.push(format!("  {rel}: 표본이 없다 — 배치를 확인하라"));
            continue;
        }
        let Some(hwp5) = convert_to_hwp5_bytes(&path) else {
            failures.push(format!("  {rel}: HWP3 파싱/저장 실패"));
            continue;
        };
        let shades = shade_colors(rel, &hwp5);
        if !shades.contains(expected) {
            let mut seen: Vec<u32> = shades.iter().copied().filter(|&c| c != NO_SHADE).collect();
            seen.sort_unstable();
            seen.dedup();
            failures.push(format!(
                "  {rel}: 한컴 저장본의 0x{expected:08x} ({origin})이 없다. \
                 음영 없음이 아닌 값들: {seen:08x?}. 흰 바탕 lerp 는 정수 **절하** 여야 \
                 한다 — (c*r + 255*(100-r))/100. 절상(255-(255-c)*r/100)이면 15%/6% 가 \
                 1씩 커진다 (src/parser/hwp3/mod.rs hwp3_char_shade_color)"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "한컴 실측 불일치:\n{}",
        failures.join("\n")
    );
}

// ── ③-2 한컴 자기 변환본을 정답지로 쓴다 ─────────────────────────────────────

/// `samples/hwp3-sampleN-hwp5.hwp` 는 **한컴이 같은 원본을 직접 변환한** 산출물이다.
/// 저장소 안에 정답지가 있으므로 CI 에서 한컴 없이도 값 정합을 판정할 수 있다.
///
/// 개수는 맞출 수 없다 — 한컴은 CHAR_SHAPE 를 중복 제거해 수십 개만 남기고 우리는
/// 문단마다 쌓아 수천 개를 낸다(별개 사안). 대신 **값 집합**을 본다: 우리가 내는 음영색은
/// 한컴이 같은 문서에서 낸 값들 안에 있어야 한다.
///
/// 수정 전에는 `hwp3-sample16` 에서 `0x00000000`×6,516 와 `0x0000ff00`×4 가 나왔는데
/// 한컴의 값은 `0xffffffff`×162 와 `0x00d8d8d8`×1 이었다 — 교집합이 0 이었다.
#[test]
fn shade_values_are_a_subset_of_hancom_own_conversion() {
    // (HWP3 원본, 한컴이 변환한 같은 문서)
    let pairs: &[(&str, &str)] = &[
        ("samples/hwp3-sample.hwp", "samples/hwp3-sample-hwp5.hwp"),
        ("samples/hwp3-sample4.hwp", "samples/hwp3-sample4-hwp5.hwp"),
        (
            "samples/hwp3-sample10.hwp",
            "samples/hwp3-sample10-hwp5.hwp",
        ),
        (
            "samples/hwp3-sample11.hwp",
            "samples/hwp3-sample11-hwp5.hwp",
        ),
        (
            "samples/hwp3-sample13.hwp",
            "samples/hwp3-sample13-hwp5.hwp",
        ),
        (
            "samples/hwp3-sample14.hwp",
            "samples/hwp3-sample14-hwp5.hwp",
        ),
        (
            "samples/hwp3-sample16.hwp",
            "samples/hwp3-sample16-hwp5.hwp",
        ),
        (
            "samples/hwp3-sample19.hwp",
            "samples/hwp3-sample19-hwp5.hwp",
        ),
    ];

    let mut compared = 0usize;
    let mut failures = Vec::new();
    for (src, oracle) in pairs {
        let (src_path, oracle_path) = (repo_path(src), repo_path(oracle));
        if !src_path.exists() || !oracle_path.exists() {
            continue;
        }
        let Some(hwp5) = convert_to_hwp5_bytes(&src_path) else {
            continue;
        };
        compared += 1;

        let mut hancom: Vec<u32> =
            shade_colors(oracle, &std::fs::read(&oracle_path).expect("읽기"));
        hancom.sort_unstable();
        hancom.dedup();

        let mut extra: Vec<u32> = shade_colors(src, &hwp5)
            .into_iter()
            .filter(|v| !hancom.contains(v))
            .collect();
        extra.sort_unstable();
        extra.dedup();

        if !extra.is_empty() {
            failures.push(format!(
                "  {src}: 한컴이 내지 않는 음영색 {extra:08x?} 를 냈다 \
                 (한컴 값 집합: {hancom:08x?})"
            ));
        }
    }

    assert!(
        compared >= 6,
        "한컴 변환본 짝이 {compared}건뿐이다 — 표본 배치를 확인하라"
    );
    assert!(
        failures.is_empty(),
        "한컴 자기 변환본과 값이 어긋난다 ({compared}쌍 중 {}쌍):\n{}\n{}",
        failures.len(),
        failures.join("\n"),
        why_it_matters()
    );
}

// ── ④ public 저장 경로 ─────────────────────────────────────────────────────

/// `DocumentCore::export_hwp_with_adapter` 도 같은 계약을 지킨다.
#[test]
fn public_document_core_export_also_avoids_black_char_shade() {
    let hwp5 = convert_to_hwp5_bytes_via_document_core(&repo_path(HWP3_SAMPLE));
    let shades = shade_colors(HWP3_SAMPLE, &hwp5);
    let black = shades.iter().filter(|&&c| c == BLACK_SHADE).count();
    assert_eq!(
        black,
        0,
        "{HWP3_SAMPLE}: public 저장 경로에서 CHAR_SHAPE {}개 중 {black}개가 검정 음영이다. {}",
        shades.len(),
        why_it_matters()
    );
}

// ── ⑤ HWPX 축 무회귀 ──────────────────────────────────────────────────────

/// 종전에도 정상이던 축이 이 변경으로 깨지지 않는지 본다.
///
/// HWPX 라이터는 IR sentinel 을 `color_hex` 로 `"none"` 에 매핑한다. SO-SUEOP 은 전수
/// `"none"`, sample11 은 음영 2건이 그대로 `#D8D8D8`·`#999999` 로 나가야 한다.
#[test]
fn hwp3_export_hwpx_keeps_shade_color_contract() {
    let so_sueop = hwpx_header_xml(SO_SUEOP);
    let char_pr = so_sueop.matches("<hh:charPr ").count();
    let none = so_sueop.matches(r#"shadeColor="none""#).count();
    assert!(
        char_pr > 0,
        "{SO_SUEOP}: charPr 이 없다 — 저장 경로를 확인하라"
    );
    assert_eq!(
        none, char_pr,
        "{SO_SUEOP}: charPr {char_pr}개 중 shadeColor=\"none\" 은 {none}개다. \
         원본은 음영 비율이 전건 0 이므로 전수 \"none\" 이어야 한다"
    );

    let sample11 = hwpx_header_xml("samples/hwp3-sample11.hwp");
    for expected in [r##"shadeColor="#D8D8D8""##, r##"shadeColor="#999999""##] {
        assert!(
            sample11.contains(expected),
            "samples/hwp3-sample11.hwp: HWPX 에 {expected} 가 없다. 실제 음영이 있는 \
             글자 모양은 HWPX 축에도 그대로 나가야 한다"
        );
    }
}

fn hwpx_header_xml(rel: &str) -> String {
    let raw = std::fs::read(repo_path(rel)).expect("표본 읽기");
    let core = rhwp::document_core::DocumentCore::from_bytes(&raw).expect("HWP3 파싱");
    let hwpx = core.export_hwpx_native().expect("HWPX 저장");
    zip_text_entry(&hwpx, "Contents/header.xml")
}

// ── ⑥ HML 축 ──────────────────────────────────────────────────────────────

/// `ShadeColor` 속성이 없는 `<CHARSHAPE>` 를 왕복해도 검정이 나가면 안 된다.
///
/// 한/글이 산출한 HML 은 음영 없음을 `4294967295`(= `0xFFFFFFFF`)로 명시한다
/// (`samples/hml/*.hml` 2/2). 리더가 속성 부재를 0 으로 채우면 라이터가 `ShadeColor="0"`
/// 을 내보내 같은 검정 계약이 HML 축에도 새어 나간다.
#[test]
fn hml_roundtrip_without_shadecolor_emits_no_shade_sentinel() {
    let raw = std::fs::read(repo_path(HML_WITHOUT_SHADECOLOR)).expect("HML fixture 읽기");
    let xml_in = String::from_utf8_lossy(&raw);
    assert!(
        !xml_in.contains("ShadeColor"),
        "{HML_WITHOUT_SHADECOLOR} 에 ShadeColor 가 생겼다 — 이 fixture 는 '속성이 없을 때'를 \
         재현해야 한다. 다른 fixture 를 쓰거나 회귀 조건을 다시 잡아라"
    );

    let core = rhwp::document_core::DocumentCore::from_bytes(&raw).expect("HML 파싱");
    let out = core.export_hml_native().expect("HML 저장");
    let xml_out = String::from_utf8_lossy(&out);

    let emitted = xml_out.matches("ShadeColor=").count();
    assert!(
        emitted > 0,
        "{HML_WITHOUT_SHADECOLOR}: 왕복 저장에 ShadeColor 가 하나도 없다 — 저장 경로를 확인하라"
    );
    assert_eq!(
        xml_out.matches(r#"ShadeColor="4294967295""#).count(),
        emitted,
        "{HML_WITHOUT_SHADECOLOR}: ShadeColor {emitted}개 중 음영 없음 sentinel 이 아닌 것이 \
         있다. 한/글 산출 HML 은 4294967295 를 쓴다 (samples/hml/*.hml 2/2)"
    );
    assert!(
        !xml_out.contains(r#"ShadeColor="0""#),
        "{HML_WITHOUT_SHADECOLOR}: ShadeColor=\"0\"(검정)이 나갔다. {}",
        why_it_matters()
    );
}
