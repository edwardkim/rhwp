//! [Issue #4055] B1 스파이크 Stage 1~5 — 차트 표현·CFB 재포장 실현성 프로브.
//!
//! #3683 Track B 착수 순서 3번("레거시 `Contents` f64 in-place 패치의 실현성 확인")의
//! 실측 하네스다. **프로덕션 동작을 바꾸지 않는다** — `src/` 를 건드리지 않고,
//! 기존 파서 자산만 읽기로 사용한다.
//!
//! ## 왜 새 로케이터가 필요한가
//!
//! 현행 `src/ole_chart/parser.rs` 의 `is_plausible_grid_value` 는 **정수·1 이상·100만
//! 이하**만 값으로 인정한다. 그래서 코퍼스의 실제 값(`4.3`, `2.5`, `1.8` …)을 놓치고
//! 개수 불일치로 파싱 전체가 실패한다. 렌더에서는 OOXML 경로가 먼저 이기므로
//! (`renderer/layout/shape_layout.rs`) 지금껏 드러나지 않았다.
//!
//! 편집을 하려면 값이 아니라 **바이트 오프셋**을 알아야 하고, 값 휴리스틱은
//! 사용자가 값을 `0`·음수·소수로 바꾸는 순간 무너진다. 그래서 이 프로브는
//! 값을 보지 않고 **구조만 보는** 로케이터를 쓴다.
//!
//! ## 구조 (코퍼스 실측으로 확정)
//!
//! `VtDataGrid` 구간 안에서 각 셀은 26바이트이고, f64 값 **바로 뒤**에 언제나
//! 같은 6바이트 트레일러가 붙는다.
//!
//! ```text
//! ... 00 "VtDouble\0" 01 00 | <f64 8B> | FF FF 06 00 00 00     ← 첫 셀
//! ... 04 00 00 00 NN 00 00 00 07 00 00 00 | <f64 8B> | FF FF 06 00 00 00
//! ```
//!
//! 따라서 트레일러를 찾아 그 **직전 8바이트**를 읽으면 값이고, 그 위치가 곧
//! in-place 패치 대상 오프셋이다.

#[path = "support/issue_4055_chart_probe.rs"]
mod chart_probe_support;

use chart_probe_support::*;
use std::io::Read as _;
use std::path::Path;

use rhwp::parser::ole_container::parse_ole_container;

/// S1 — 구조 기반 로케이터가 코퍼스 전건에서 정답지와 일치하는가.
///
/// `.hwpx` 와 `.hwp` 는 같은 차트라도 바이트가 다르므로(한컴이 각각 따로 저장)
/// 양쪽을 독립적으로 판정한다.
#[test]
fn legacy_grid_locator_matches_ooxml_ground_truth_across_corpus() {
    let mut checked = 0usize;
    let mut failures = Vec::new();

    for hwpx in corpus() {
        let hwp = hwpx.with_extension("hwp");
        for path in [&hwpx, &hwp] {
            let label = path
                .strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
                .unwrap_or(path)
                .display()
                .to_string();

            let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{label} 읽기: {e}"));
            let Some((legacy, ooxml)) = chart_streams(&bytes) else {
                failures.push(format!("{label}: 차트 스트림 추출 실패"));
                continue;
            };
            let Some(series) = ground_truth(&ooxml) else {
                failures.push(format!("{label}: OOXML 정답지 추출 실패"));
                continue;
            };

            let located = locate_grid_values(&legacy);
            let values: Vec<f64> = located.iter().map(|(_, v)| *v).collect();
            let expected: usize = series.iter().map(Vec::len).sum();

            if values.len() != expected {
                failures.push(format!(
                    "{label}: 값 개수 {} != 정답지 {expected}",
                    values.len()
                ));
                continue;
            }
            if classify(&values, &series).is_none() {
                failures.push(format!(
                    "{label}: 값이 어느 순서와도 불일치 — 실측 {values:?} / 정답지 {series:?}"
                ));
                continue;
            }

            // 패치 대상 오프셋이 실제로 그 값을 담고 있는지 되읽어 확인한다.
            for (offset, value) in &located {
                let round_trip =
                    f64::from_le_bytes(legacy[*offset..*offset + 8].try_into().expect("8바이트"));
                assert_eq!(round_trip, *value, "{label}: 오프셋 {offset} 재독 불일치");
            }
            checked += 1;
        }
    }

    assert!(
        failures.is_empty(),
        "구조 기반 로케이터 실패 {}건:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert_eq!(checked, 56, "코퍼스 28종 × 2포맷을 전건 검사해야 한다");
}

/// S1 곁가지 — 그리드 순서는 문서마다 다르다. 편집기가 `(계열, 카테고리)` 를
/// 셀로 옮기려면 순서를 **판정해야 하고 가정하면 안 된다**는 근거를 고정한다.
#[test]
fn legacy_grid_orientation_is_not_fixed() {
    let modern =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/chart/세로막대형/묶은세로막대형.hwp");
    let bytes = std::fs::read(&modern).expect("모던 샘플 읽기");
    let (legacy, ooxml) = chart_streams(&bytes).expect("차트 스트림");
    let series = ground_truth(&ooxml).expect("정답지");
    let values: Vec<f64> = locate_grid_values(&legacy)
        .into_iter()
        .map(|(_, v)| v)
        .collect();
    assert_eq!(
        classify(&values, &series),
        Some(Orientation::SeriesMajor),
        "모던 코퍼스는 계열-major 로 저장된다"
    );

    // 레거시 `Contents` 단독 문서는 카테고리-major 다.
    let control = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/143E433F503322BD33.hwp");
    let bytes = std::fs::read(&control).expect("대조군 읽기");
    let doc = rhwp::parse_document(&bytes).expect("대조군 파싱");
    let ole = doc
        .bin_data_content
        .iter()
        .find(|c| c.extension == "OLE")
        .expect("대조군 OLE");
    let container = parse_ole_container(&ole.data.load()).expect("중첩 컨테이너");
    assert!(
        container.ooxml_chart.is_none(),
        "대조군은 OOXML 사본이 없는 레거시 단독 문서다"
    );
    let legacy = container.raw_contents.expect("레거시 Contents");
    let values: Vec<f64> = locate_grid_values(&legacy)
        .into_iter()
        .map(|(_, v)| v)
        .collect();

    // #1251 이 고정한 정답: 적립금·수입·지출 3계열 × 4카테고리.
    let series = vec![
        vec![328.0, 812.0, 1702.0, 1477.0],
        vec![50.0, 70.0, 189.0, 191.0],
        vec![11.0, 15.0, 201.0, 289.0],
    ];
    assert_eq!(values.len(), 12, "대조군은 값 12개다");
    assert_eq!(
        classify(&values, &series),
        Some(Orientation::CategoryMajor),
        "대조군은 카테고리-major 로 저장된다 — 순서는 고정이 아니다"
    );
}

/// 로케이터를 `VtDataGrid` 구간으로 묶지 않으면 그리드 밖 `VtDouble` 까지
/// 주워 온다는 것을 고정한다. 창 밖 값을 패치하면 축 눈금 같은 무관한 속성을
/// 덮어쓰게 된다.
#[test]
fn locator_must_be_bounded_to_the_data_grid_window() {
    let control = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/143E433F503322BD33.hwp");
    let bytes = std::fs::read(&control).expect("대조군 읽기");
    let doc = rhwp::parse_document(&bytes).expect("대조군 파싱");
    let ole = doc
        .bin_data_content
        .iter()
        .find(|c| c.extension == "OLE")
        .expect("대조군 OLE");
    let legacy = parse_ole_container(&ole.data.load())
        .expect("중첩 컨테이너")
        .raw_contents
        .expect("레거시 Contents");

    let bounded = locate_grid_values(&legacy).len();

    // 구간 제한 없이 스트림 전체를 훑으면 값이 더 붙는다.
    let anchor = find_from(&legacy, DOUBLE_MARKER, 0).expect("VtDouble 앵커");
    let mut unbounded = 0usize;
    let mut cursor = anchor;
    while let Some(hit) = find_from(&legacy, VALUE_TRAILER, cursor + 1) {
        cursor = hit;
        if hit >= anchor + 8 {
            unbounded += 1;
        }
    }

    assert_eq!(bounded, 12, "그리드 구간 안에서는 12개다");
    assert!(
        unbounded > bounded,
        "구간 제한이 없으면 그리드 밖 VtDouble 까지 잡힌다 (제한 {bounded} / 무제한 {unbounded})"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Stage 2·3 — 변이 생성기와 중첩 CFB 재포장 (S2·S3 재료, S4)
// ─────────────────────────────────────────────────────────────────────────────
/// S4 — 중첩 CFB 를 스트림 단위로 열거해 그대로 다시 싸면 내용이 보존되는가.
///
/// 바이트 동일성이 아니라 **스트림 집합과 각 스트림 바이트의 동일성**을 본다.
/// CFB 컨테이너 레이아웃(섹터 배치)은 빌더마다 달라도 무방하다.
///
/// #4097 이 축을 둘로 넓혔다 — 코퍼스 28종 **× 2포맷(HWPX·HWP)** 전건에서 루트 CLSID 까지
/// 보존되는지 본다. 재포장은 `build_cfb_with_root_clsid` 프로덕션 경로를 탄다.
#[test]
fn nested_cfb_repack_preserves_every_stream() {
    // parse_ole_container 가 아는 스트림 — 이 밖의 것이 있으면 그대로 재포장할 때 소실된다.
    let known = [
        "/Contents",
        "/OOXMLChartContents",
        EMF_STREAM,
        "/\u{1}Ole10Native",
    ];

    let mut checked = 0usize;
    let mut unknown_seen: Vec<String> = Vec::new();

    for hwpx in corpus() {
        // 2포맷 축 — `.hwp` 의 중첩 CFB 도 `bin_data_content` 에서 그대로 얻힌다.
        // HWP5 파서가 BinData 의 4바이트 LE size prefix 를 이미 `drain(..4)` 한다.
        for path in [hwpx.clone(), hwpx.with_extension("hwp")] {
            let label = path.display().to_string();
            let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{label} 읽기: {e}"));
            let doc =
                rhwp::parse_document(&bytes).unwrap_or_else(|e| panic!("{label} 파싱: {e:?}"));
            let nested = doc
                .bin_data_content
                .iter()
                .map(|c| c.data.load())
                .find(|b| b.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]))
                .unwrap_or_else(|| panic!("{label}: 중첩 CFB 가 없다"));

            let before = all_streams(&nested);
            for (name, _) in &before {
                if !known.contains(&name.as_str()) && !unknown_seen.contains(name) {
                    unknown_seen.push(name.clone());
                }
            }

            let rebuilt = rebuild_cfb_preserving_clsid(&nested, &before);
            assert_eq!(
                root_clsid(&rebuilt),
                root_clsid(&nested),
                "{label}: 재포장이 OLE 클래스 ID 를 보존해야 한다"
            );
            let after = all_streams(&rebuilt);
            assert_eq!(
                before.len(),
                after.len(),
                "{label}: 재포장 전후 스트림 개수가 같아야 한다"
            );
            for (name, data) in &before {
                let found = after
                    .iter()
                    .find(|(n, _)| n == name)
                    .unwrap_or_else(|| panic!("{label}: 재포장본에 {name} 없음"));
                assert_eq!(&found.1, data, "{label}: {name} 바이트가 달라졌다");
            }

            // 재포장본이 프로덕션 소비 경로를 그대로 탄다.
            let container = parse_ole_container(&rebuilt).expect("재포장본 컨테이너 파싱");
            assert!(container.ooxml_chart.is_some(), "재포장본 OOXML 유지");
            assert!(container.raw_contents.is_some(), "재포장본 레거시 유지");
            assert!(container.preview_emf.is_some(), "재포장본 EMF 프리뷰 유지");
            checked += 1;
        }
    }

    assert_eq!(
        checked, 56,
        "코퍼스 28종 × 2포맷을 전건 재포장 검사해야 한다"
    );
    assert!(
        unknown_seen.is_empty(),
        "parse_ole_container 가 모르는 스트림이 코퍼스에 있다 — 4종만 뽑아 재포장하면 소실된다: {unknown_seen:?}"
    );
}

/// 중첩 OLE CFB 재포장이 루트 CLSID 를 **보존**하는지 못박는다.
///
/// #4055 스파이크에서는 `mini_cfb_repack_drops_the_ole_class_id` 라는 이름으로 **결함을**
/// 고정하고 있었다. #4097 이 `build_cfb_with_root_clsid` 를 넣으면서 방향을 뒤집었다.
///
/// 중첩 OLE CFB 를 재포장하며 OLE 서버 식별자를 잃으면, 한컴은 개체를 알아보지 못해 틀만
/// 그리고 내용을 비운다(2026-08-05 실측). rhwp 는 `parse_ole_container` 가 스트림 이름으로
/// 판별하므로 이 손실을 감지하지 못한다 — 자체 왕복 검증으로는 절대 안 잡히는 종류다.
#[test]
fn mini_cfb_repack_preserves_the_ole_class_id() {
    let hwpx = std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwpx"))).expect("HWPX 읽기");
    let doc = rhwp::parse_document(&hwpx).expect("HWPX 파싱");
    let nested = doc
        .bin_data_content
        .iter()
        .map(|c| c.data.load())
        .find(|b| b.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]))
        .expect("중첩 CFB");

    // 판정은 `cfb` 크레이트로 한다 — 프로덕션 리더로 판정하면 읽기·쓰기가 같은 오해를
    // 공유해도 통과한다(`root_clsid` helper 주석 참조).
    let original = root_clsid(&nested);
    assert_ne!(
        original, [0u8; 16],
        "코퍼스 차트는 OLE 클래스 ID 를 달고 있다"
    );

    // 인자 없는 `build_cfb` 가 0 을 쓰는 것은 결함이 아니라 **계약**이다 — 바깥 HWP5 CFB 는
    // 원본도 루트 CLSID 가 0 이라 그 호출자(`cfb_writer::write_hwp_cfb`)가 이 쪽을 쓴다.
    let zeroed = rebuild_cfb(&all_streams(&nested));
    assert_eq!(
        root_clsid(&zeroed),
        [0u8; 16],
        "build_cfb 는 CLSID 없음([0u8;16])으로 위임한다 — 이것이 계약이다"
    );

    // 여기가 #4097 이 뒤집은 단언이다. `rebuild_cfb_preserving_clsid` 는 더 이상 테스트
    // 로컬 바이트 수술이 아니라 `ole_root_clsid` + `build_cfb_with_root_clsid` 프로덕션
    // 경로다 — `mini_cfb` 를 되돌리면 이 단언이 red 가 된다.
    let preserved = rebuild_cfb_preserving_clsid(&nested, &all_streams(&nested));
    assert_eq!(
        root_clsid(&preserved),
        original,
        "프로덕션 재포장이 OLE 클래스 ID 를 보존해야 한다 (#4097)"
    );
}

/// 두 패치가 **같은 논리 셀**을 겨냥하는지 확인한다. 그래야 변종 사이 비교가
/// 성립한다.
#[test]
fn both_patches_target_the_same_logical_cell() {
    let bytes = std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwpx"))).expect("HWPX 읽기");
    let (legacy, ooxml) = chart_streams(&bytes).expect("차트 스트림");

    let legacy_first = locate_grid_values(&legacy).first().expect("레거시 첫 값").1;
    let ooxml_first = ground_truth(&ooxml).expect("정답지")[0][0];
    assert_eq!(
        legacy_first, ooxml_first,
        "레거시 첫 그리드 값과 OOXML 첫 값이 같은 셀이어야 한다"
    );

    let patched_legacy = patch_legacy_first_value(&legacy, SENTINEL).expect("레거시 패치");
    assert_eq!(
        patched_legacy.len(),
        legacy.len(),
        "레거시 패치는 길이 불변이어야 한다"
    );
    let changed = legacy
        .iter()
        .zip(&patched_legacy)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        changed <= 8,
        "레거시 패치가 8바이트를 넘게 바꿨다: {changed}"
    );

    let before: Vec<f64> = locate_grid_values(&legacy)
        .into_iter()
        .map(|(_, v)| v)
        .collect();
    let after: Vec<f64> = locate_grid_values(&patched_legacy)
        .into_iter()
        .map(|(_, v)| v)
        .collect();
    assert_eq!(after[0], SENTINEL, "첫 값이 sentinel 로 바뀌어야 한다");
    assert_eq!(after[1..], before[1..], "나머지 값은 그대로여야 한다");

    let patched_ooxml = patch_ooxml_first_value(&ooxml, SENTINEL_TEXT).expect("OOXML 패치");
    let gt = ground_truth(&patched_ooxml).expect("패치본 정답지");
    assert_eq!(gt[0][0], SENTINEL, "OOXML 첫 값이 sentinel 이어야 한다");
    assert_eq!(
        gt[0][1..],
        ground_truth(&ooxml).expect("원본 정답지")[0][1..],
        "OOXML 나머지 값은 그대로여야 한다"
    );
}

/// 편집한 바이트가 rhwp 자체 저장에서 살아나는지 — 한컴에 넘기기 전 자기 검증.
///
/// `bin_data_content` 의 `ooxml_chart` 항목만 갈아끼우면 저장기가 그 바이트를
/// `Chart/chartN.xml` 로 무가공 방출하므로(`serializer/hwpx/mod.rs:164-180`)
/// 직렬화기를 고치지 않고도 편집이 저장된다. 동시에 중첩 CFB 안의 사본은
/// 손대지 않으므로 **4중 표현이 갈리는 지점**이 그대로 드러난다.
#[test]
fn editing_only_the_zip_part_diverges_from_the_nested_copy() {
    use rhwp::document_core::DocumentCore;

    let bytes = std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwpx"))).expect("HWPX 읽기");
    let mut core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    {
        let doc = core.document_mut();
        let slot = doc
            .bin_data_content
            .iter_mut()
            .find(|c| c.extension == "ooxml_chart")
            .expect("ooxml_chart 항목");
        let patched = patch_ooxml_first_value(&slot.data.load(), SENTINEL_TEXT).expect("패치");
        slot.data = patched.into();
    }
    let saved = core.export_hwpx_native().expect("HWPX 저장");

    let reparsed = rhwp::parse_document(&saved).expect("저장본 재파싱");
    let zip_part = reparsed
        .bin_data_content
        .iter()
        .find(|c| c.extension == "ooxml_chart")
        .expect("저장본 ooxml_chart");
    assert_eq!(
        ground_truth(&zip_part.data.load()).expect("zip 파트 정답지")[0][0],
        SENTINEL,
        "편집한 값이 Chart/chartN.xml 로 저장돼야 한다"
    );

    let nested_ooxml = reparsed
        .bin_data_content
        .iter()
        .filter(|c| c.extension != "ooxml_chart")
        .filter_map(|c| parse_ole_container(&c.data.load()))
        .find_map(|c| c.ooxml_chart)
        .expect("중첩 OOXML 사본");
    assert_eq!(
        ground_truth(&nested_ooxml).expect("중첩 사본 정답지")[0][0],
        4.3,
        "중첩 CFB 사본은 손대지 않아 옛 값 그대로다 — 여기서 4중 표현이 갈린다"
    );
}

/// 곁다리 관찰 — HWPX→HWP 변환에서 id 60000+N 차트가 어떻게 되는가.
///
/// `src/document_core/converters/hwpx_to_hwp.rs` 에 `ooxml_chart`·`60000` 전용 처리가
/// **0건**이라 미확인 영역이었다. #4055 스파이크의 질문(S1~S4)에는 없지만, 보고서가
/// 권고한 "편집 시 ①②를 함께 쓴다"의 근거가 여기 걸리므로 **현재 동작만 기록**한다.
///
/// 결함이면 별개 이슈로 분리한다 — B1 정의 밖이다.
#[test]
fn observation_hwpx_to_hwp_conversion_keeps_the_chart() {
    use rhwp::document_core::DocumentCore;

    let bytes = std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwpx"))).expect("HWPX 읽기");
    let before = representations(&bytes);

    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");
    let hwp = core.export_hwp_with_adapter().expect("HWP 로 변환");
    let after = representations(&hwp);

    println!("  HWPX 원본 : {before:?}");
    println!("  HWP  변환 : {after:?}");

    // HWP5 에는 zip 파트가 없으므로 ①이 사라지는 것은 정상이다.
    assert_eq!(after.zip_part, None, "HWP5 에는 zip 차트 파트가 없다");

    // 한컴이 HWP5 에서 읽는 것은 ② 중첩 OOXML 이다(2026-08-05 실측). 변환본에
    // 그것이 남아 있어야 차트가 산다.
    assert_eq!(
        after.nested_ooxml, before.nested_ooxml,
        "변환 후에도 중첩 OOXML 사본이 값 그대로 남아야 한다 — \
         한컴이 HWP5 에서 읽는 표현이다"
    );
    assert_eq!(
        after.legacy, before.legacy,
        "레거시 Contents 도 보존돼야 한다"
    );
    assert!(after.has_emf, "EMF 프리뷰도 보존돼야 한다");
}

/// 보고서 권고 "편집 시 ①②를 함께 쓴다"의 근거를 실측으로 고정한다.
///
/// HWPX 에서 ①`Chart/chartN.xml` 만 고치면 한컴 화면은 맞다(X-A 실측). 그러나
/// HWP 로 저장하면 ①은 사라지고 한컴이 읽는 것은 ②가 되는데, ②는 손대지 않아
/// 옛 값이다 → **편집이 조용히 되돌아간다.**
#[test]
fn editing_only_the_zip_part_is_lost_when_converting_to_hwp() {
    use rhwp::document_core::DocumentCore;

    let bytes = std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwpx"))).expect("HWPX 읽기");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");
    {
        let doc = core.document_mut();
        let slot = doc
            .bin_data_content
            .iter_mut()
            .find(|c| c.extension == "ooxml_chart")
            .expect("ooxml_chart 항목");
        let patched = patch_ooxml_first_value(&slot.data.load(), SENTINEL_TEXT).expect("패치");
        slot.data = patched.into();
    }

    // HWPX 로 저장하면 편집이 살아 있다 — 한컴이 ①을 읽으므로 화면도 맞다.
    let saved_hwpx = core.export_hwpx_native().expect("HWPX 저장");
    assert_eq!(
        representations(&saved_hwpx).zip_part,
        Some(SENTINEL),
        "HWPX 저장본의 zip 파트에는 편집이 남는다"
    );

    // 그러나 HWP 로 변환하면 ①이 사라지고, 한컴이 읽는 ②는 옛 값 그대로다.
    let converted = core.export_hwp_with_adapter().expect("HWP 변환");
    let after = representations(&converted);
    assert_eq!(after.zip_part, None, "HWP5 에는 zip 파트가 없다");
    assert_eq!(
        after.nested_ooxml,
        Some(4.3),
        "①만 고치면 HWP 변환에서 편집이 사라진다 — 그래서 ①② 를 함께 써야 한다"
    );
}

/// Stage 2 — 한컴 육안 판정용 변종 꾸러미를 만든다 (S2·S3).
///
/// `output/` 에 파일을 쓰는 부작용이 있어 기본 실행에서 뺀다. 판정 직전에만 돌린다:
///
/// ```text
/// cargo test --profile release-test --test issue_4055_b1_chart_edit_probe -- --ignored --nocapture
/// ```
///
/// 각 변종은 내보내기 전에 **rhwp 가 다시 열 수 있는지** 스스로 확인한다. 한컴이
/// 못 열었을 때 그게 편집 탓인지 파일 조립 탓인지 헷갈리지 않게 하기 위함이다.
#[test]
#[ignore = "output/ 에 파일을 쓴다 — 한컴 판정 직전에만 실행"]
fn generate_hancom_judgment_bundle() {
    let out_dir = manifest("output/issue_4055_b1_spike");
    std::fs::create_dir_all(&out_dir).expect("출력 디렉터리");

    let hwpx = std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwpx"))).expect("HWPX 원본");
    let hwp = std::fs::read(manifest(&format!("{BASE_SAMPLE}.hwp"))).expect("HWP 원본");

    // ── HWPX 쪽 재료
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx.clone())).expect("zip");
    let chart_part = zip
        .file_names()
        .find(|n| n.starts_with("Chart/") && n.ends_with(".xml"))
        .expect("Chart 파트")
        .to_string();
    let ole_entry = zip
        .file_names()
        .find(|n| n.starts_with("BinData/") && n.to_ascii_lowercase().ends_with(".ole"))
        .expect("BinData ole")
        .to_string();
    let mut chart_xml = Vec::new();
    zip.by_name(&chart_part)
        .expect("Chart 파트 열기")
        .read_to_end(&mut chart_xml)
        .expect("Chart 파트 읽기");
    let mut ole_raw = Vec::new();
    zip.by_name(&ole_entry)
        .expect("ole 열기")
        .read_to_end(&mut ole_raw)
        .expect("ole 읽기");
    let hwpx_nested = if ole_raw.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
        ole_raw.clone()
    } else {
        ole_raw[4..].to_vec()
    };
    let hwpx_prefix = ole_raw.len() - hwpx_nested.len();
    let repack_hwpx_ole = |nested: Vec<u8>| -> Vec<u8> {
        let mut v = Vec::with_capacity(nested.len() + hwpx_prefix);
        if hwpx_prefix == 4 {
            v.extend_from_slice(&(nested.len() as u32).to_le_bytes());
        }
        v.extend_from_slice(&nested);
        v
    };
    let patched_chart =
        patch_ooxml_first_value(&chart_xml, SENTINEL_TEXT).expect("Chart 파트 패치");

    // ── HWP 쪽 재료
    let hwp_ole_path = all_streams(&hwp)
        .into_iter()
        .map(|(n, _)| n)
        .find(|n| n.starts_with("/BinData/") && n.to_ascii_uppercase().ends_with(".OLE"))
        .expect("HWP BinData OLE 스트림");
    let hwp_stream = all_streams(&hwp)
        .into_iter()
        .find(|(n, _)| *n == hwp_ole_path)
        .map(|(_, d)| d)
        .expect("HWP OLE 스트림 바이트");
    let hwp_nested = hwp_nested_cfb(&hwp_stream);

    const OLD: f64 = 4.3;
    let bundle: Vec<(String, Vec<u8>, &str, Representations)> = vec![
        (
            "00-control-원본.hwpx".into(),
            hwpx.clone(),
            "대조군 — 바이트 무수정. 첫 막대가 낮다(4.3)",
            Representations {
                zip_part: Some(OLD),
                nested_ooxml: Some(OLD),
                legacy: Some(OLD),
                has_emf: true,
            },
        ),
        (
            "00-control-원본.hwp".into(),
            hwp.clone(),
            "대조군 — 바이트 무수정. 첫 막대가 낮다(4.3)",
            Representations {
                zip_part: None,
                nested_ooxml: Some(OLD),
                legacy: Some(OLD),
                has_emf: true,
            },
        ),
        (
            "X-A-zip파트만.hwpx".into(),
            rewrite_hwpx(&hwpx, &[(chart_part.clone(), patched_chart.clone())]),
            "① Chart/chart1.xml 만 패치",
            Representations {
                zip_part: Some(SENTINEL),
                nested_ooxml: Some(OLD),
                legacy: Some(OLD),
                has_emf: true,
            },
        ),
        (
            "X-B-레거시만.hwpx".into(),
            rewrite_hwpx(
                &hwpx,
                &[(
                    ole_entry.clone(),
                    repack_hwpx_ole(mutate_nested(&hwpx_nested, false, true, false)),
                )],
            ),
            "③ 레거시 Contents 만 패치",
            Representations {
                zip_part: Some(OLD),
                nested_ooxml: Some(OLD),
                legacy: Some(SENTINEL),
                has_emf: true,
            },
        ),
        (
            "X-C-셋다.hwpx".into(),
            rewrite_hwpx(
                &hwpx,
                &[
                    (chart_part.clone(), patched_chart.clone()),
                    (
                        ole_entry.clone(),
                        repack_hwpx_ole(mutate_nested(&hwpx_nested, true, true, false)),
                    ),
                ],
            ),
            "①②③ 전부 패치, EMF 유지",
            Representations {
                zip_part: Some(SENTINEL),
                nested_ooxml: Some(SENTINEL),
                legacy: Some(SENTINEL),
                has_emf: true,
            },
        ),
        (
            "X-D-셋다+EMF제거.hwpx".into(),
            rewrite_hwpx(
                &hwpx,
                &[
                    (chart_part.clone(), patched_chart.clone()),
                    (
                        ole_entry.clone(),
                        repack_hwpx_ole(mutate_nested(&hwpx_nested, true, true, true)),
                    ),
                ],
            ),
            "①②③ 패치 + ④ EMF 프리뷰 제거",
            Representations {
                zip_part: Some(SENTINEL),
                nested_ooxml: Some(SENTINEL),
                legacy: Some(SENTINEL),
                has_emf: false,
            },
        ),
        (
            // X-A 로 "한컴은 OOXML 표현을 읽는다"가 확인됐으므로, .hwp 에서는
            // 중첩 OOXML 사본과 레거시 중 어느 쪽인지가 남은 질문이다. 이 변종이
            // OOXML 쪽 단독 조건이고 H-B 가 레거시 쪽 단독 조건이다.
            "H-A-중첩OOXML만.hwp".into(),
            rewrite_hwp(
                &hwp,
                &[(
                    hwp_ole_path.clone(),
                    hwp_ole_stream(&mutate_nested(&hwp_nested, true, false, false)),
                )],
            ),
            "② 중첩 OOXMLChartContents 만 패치",
            Representations {
                zip_part: None,
                nested_ooxml: Some(SENTINEL),
                legacy: Some(OLD),
                has_emf: true,
            },
        ),
        (
            "H-B-레거시만.hwp".into(),
            rewrite_hwp(
                &hwp,
                &[(
                    hwp_ole_path.clone(),
                    hwp_ole_stream(&mutate_nested(&hwp_nested, false, true, false)),
                )],
            ),
            "③ 레거시 Contents 만 패치",
            Representations {
                zip_part: None,
                nested_ooxml: Some(OLD),
                legacy: Some(SENTINEL),
                has_emf: true,
            },
        ),
        (
            "H-C-둘다.hwp".into(),
            rewrite_hwp(
                &hwp,
                &[(
                    hwp_ole_path.clone(),
                    hwp_ole_stream(&mutate_nested(&hwp_nested, true, true, false)),
                )],
            ),
            "②③ 패치, EMF 유지",
            Representations {
                zip_part: None,
                nested_ooxml: Some(SENTINEL),
                legacy: Some(SENTINEL),
                has_emf: true,
            },
        ),
        (
            "H-D-둘다+EMF제거.hwp".into(),
            rewrite_hwp(
                &hwp,
                &[(
                    hwp_ole_path.clone(),
                    hwp_ole_stream(&mutate_nested(&hwp_nested, true, true, true)),
                )],
            ),
            "②③ 패치 + ④ EMF 프리뷰 제거",
            Representations {
                zip_part: None,
                nested_ooxml: Some(SENTINEL),
                legacy: Some(SENTINEL),
                has_emf: false,
            },
        ),
    ];

    // 내보내기 전 자기 검증 — rhwp 가 다시 열고 차트를 찾을 수 있어야 한다.
    let mut sheet = String::new();
    sheet.push_str("# #4055 B1 스파이크 — 한컴 육안 판정표\n\n");
    sheet.push_str(
        "원본 `samples/chart/세로막대형/묶은세로막대형` 의 **첫 계열 첫 값**을 \
         `4.3` → `91.7` 로 바꾼 변종들입니다.\n원본 최대값이 `5` 라 반영되면 \
         **첫 막대가 차트를 뚫고 솟습니다.** 확대해서 숫자를 읽을 필요가 없습니다.\n\n",
    );
    sheet.push_str("## 판정 완료 (2026-08-05, 한컴 오피스)\n\n");
    sheet.push_str(
        "전 변종을 열어 PDF 로 저장한 뒤 PDF 페이지 그리기 스트림을 해시로 갈랐습니다\
         (메타데이터·타임스탬프 제외). 렌더 결과는 **딱 두 그룹**이고 그룹 안에서는 \
         바이트 동일합니다.\n\n\
         ```text\n\
         반영   (91,406 B)  X-A · X-C · X-D · H-A · H-C · H-D\n\
         미반영 (87,712 B)  00-control ×2 · X-B · H-B\n\
         ```\n\n\
         1. **한컴은 OOXML 표현을 읽습니다.** ①(zip 파트) 하나만 고쳐도, ②(중첩 사본) \
         하나만 고쳐도 반영됩니다.\n\
         2. **레거시 `Contents` 는 전혀 읽지 않습니다.** `X-B`·`H-B` 는 레거시만 새 값인데 \
         렌더가 대조군과 바이트 동일합니다.\n\
         3. **③을 함께 써도 렌더는 달라지지 않습니다** — `X-C`/`H-C` 가 `X-A`/`H-A` 와 \
         바이트 동일합니다.\n\
         4. **EMF 프리뷰는 필요조건이 아닙니다.** 낡은 채로 둬도 가리지 않고, 제거해도\
         (`X-D`·`H-D`) 정상 개봉·렌더됩니다.\n\n\
         변종 8개 전부 오류·복구 대화상자 없이 열렸습니다.\n\n\
         상세는 `mydocs/report/task_m100_4055_report.md`.\n\n",
    );
    sheet.push_str("## 다시 판정할 때 보는 법\n\n");
    sheet.push_str(
        "1. `00-control-원본` 으로 정상 모습(막대 4개가 고만고만)을 눈에 익힙니다.\n\
         2. 값이 반영되면 첫 막대가 차트를 뚫고 솟습니다 — 확대해서 숫자를 읽을 필요가 없습니다.\n\
         3. 각 파일마다 함께 봐 주세요:\n   \
         (a) 열 때 오류·복구 대화상자가 뜨는가  (b) 차트를 더블클릭하면 편집기가 열리는가\n\n",
    );
    sheet.push_str("## 변종 목록\n\n");
    sheet.push_str("| 파일 | 무엇을 바꿨나 | 한컴 렌더 |\n");
    sheet.push_str("|---|---|---|\n");

    for (name, bytes, note, expected) in &bundle {
        // 자기 검증 ① rhwp 가 다시 연다.
        rhwp::parse_document(bytes)
            .unwrap_or_else(|e| panic!("{name}: rhwp 가 다시 열지 못한다 — {e:?}"));
        // 자기 검증 ② 라벨대로 조립됐다 — 바꾸기로 한 표현만 sentinel 이고 나머지는 원본이다.
        let actual = representations(bytes);
        assert_eq!(
            &actual, expected,
            "{name}: 변종이 라벨과 다르게 조립됐다 — 이대로 판정하면 결론이 뒤집힌다"
        );

        // 한컴이 파일을 열어 둔 채로도 돌 수 있게: 내용이 같으면 건드리지 않고,
        // 잠겨 있으면 내용이 이미 최신인지 확인한 뒤 넘어간다.
        let target = out_dir.join(name);
        let on_disk = std::fs::read(&target).ok();
        let status = if on_disk.as_deref() == Some(bytes.as_slice()) {
            "unchanged"
        } else {
            match std::fs::write(&target, bytes) {
                Ok(()) => "wrote",
                Err(e) => panic!(
                    "{name} 쓰기 실패({e}) — 내용이 디스크와 달라 갱신이 필요하다. \
                     이 파일을 연 프로그램(한컴 등)을 닫고 다시 실행하세요."
                ),
            }
        };
        println!(
            "  {status:<9} {name}  ({} bytes)  zip={:?} nested={:?} legacy={:?} emf={}",
            bytes.len(),
            actual.zip_part,
            actual.nested_ooxml,
            actual.legacy,
            actual.has_emf
        );
        let verdict = if name.contains("control") {
            "기준"
        } else if name.contains("-B-") {
            "**미반영**"
        } else {
            "반영"
        };
        sheet.push_str(&format!("| `{name}` | {note} | {verdict} |\n"));
    }

    sheet.push_str("\n## B1 본구현에 주는 결론\n\n");
    sheet.push_str(
        "| 포맷 | 써야 할 곳 | 필요한 작업 |\n|---|---|---|\n\
         | HWPX | ① `Chart/chartN.xml` | **직렬화기 수정 0** — `bin_data_content` 의 \
         `ooxml_chart`(id=60000+N) 바이트만 교체하면 그대로 방출된다 |\n\
         | HWP5 | ② 중첩 `OOXMLChartContents` | 중첩 CFB 재포장 필요 → **`mini_cfb` 가 루트 \
         CLSID 를 받도록 선행 수정** (안 하면 한컴이 차트를 비워 그린다) |\n\
         | 레거시 단독 문서 | ③ 레거시 `Contents` | 8바이트 제자리 패치. 한컴은 안 읽으므로 후순위 |\n\n\
         **①② 를 함께 쓰기를 권한다.** ①만 고치면 한컴 화면은 맞지만 HWP 로 변환할 때 \
         ①이 사라지고 ②(옛 값)만 남아 편집이 조용히 되돌아간다 — 테스트로 고정했다.\n",
    );

    std::fs::write(out_dir.join("PANJEONG.md"), sheet).expect("판정표 쓰기");
    println!("\n  판정표: {}", out_dir.join("PANJEONG.md").display());
}
