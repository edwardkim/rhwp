//! [Issue #4098] 레거시 `Contents` 그리드를 구조로 읽는다 — 프로덕션 게이트.
//!
//! #4055 의 프로브는 `src/` 를 건드리지 않고 테스트 안에서만 구조 로케이터를 실증했다.
//! 이 파일은 같은 판정을 **프로덕션 코드**(`rhwp::ole_chart::scan_legacy_grid` /
//! `parse_ole_chart_contents`)에 걸어 코퍼스 전건으로 고정한다.
//!
//! 고정하는 것은 셋이다.
//!
//! 1. **값 필터가 없다** — 소수·0·음수가 그대로 읽힌다(결함 1).
//! 2. **순서를 판정한다** — 계열-major 문서에서 계열·카테고리가 전치되지 않는다(결함 2).
//! 3. **라벨은 이름일 뿐이다** — 숫자가 섞인 라벨이 모양을 흔들지 않는다(결함 3).

#[path = "support/issue_4055_chart_probe.rs"]
mod chart_probe_support;

use chart_probe_support::{
    all_streams, chart_streams, corpus, ground_truth, hwp_nested_cfb, hwp_ole_stream, manifest,
    rebuild_cfb_preserving_clsid, rewrite_hwp,
};

use std::path::{Path, PathBuf};

use rhwp::ole_chart::{parse_ole_chart_contents, scan_legacy_grid, SeriesAxis, SeriesAxisEvidence};
use rhwp::ooxml_chart::OoxmlChart;
use rhwp::parser::ole_container::parse_ole_container;

/// 문서에서 레거시 `Contents` 만 꺼낸다(OOXML 사본이 없는 대조군용).
fn legacy_contents(path: &Path) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{} 읽기: {e}", path.display()));
    let doc = rhwp::parse_document(&bytes).expect("문서 파싱");
    for content in &doc.bin_data_content {
        let raw = content.data.load();
        if !raw.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
            continue;
        }
        if let Some(container) = parse_ole_container(&raw) {
            if let Some(contents) = container.raw_contents {
                return contents;
            }
        }
    }
    panic!("{}: 레거시 Contents 를 찾지 못했다", path.display());
}

fn label(path: &Path) -> String {
    path.strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or(path)
        .display()
        .to_string()
}

/// 코퍼스 28종 × 2포맷.
fn corpus_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for hwpx in corpus() {
        out.push(hwpx.with_extension("hwp"));
        out.push(hwpx);
    }
    out
}

/// 수용 기준 1 — 구조 스캐너가 코퍼스 전건에서 OOXML 정답지와 값이 일치한다.
///
/// 치수는 `VtObject` 가 명시한 것을 쓰고, 값은 셀 인덱스 순서로 읽는다. 어느 단계에서도
/// 값의 크기·부호·정수 여부를 보지 않는다.
#[test]
fn legacy_grid_shape_and_values_match_ooxml_across_corpus() {
    let mut checked = 0usize;
    let mut failures = Vec::new();

    for path in corpus_files() {
        let name = label(&path);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{name} 읽기: {e}"));
        let Some((legacy, ooxml)) = chart_streams(&bytes) else {
            failures.push(format!("{name}: 차트 스트림 추출 실패"));
            continue;
        };
        let Some(series) = ground_truth(&ooxml) else {
            failures.push(format!("{name}: OOXML 정답지 추출 실패"));
            continue;
        };

        let grid = match scan_legacy_grid(&legacy) {
            Ok(grid) => grid,
            Err(error) => {
                failures.push(format!("{name}: 구조 스캔 실패 — {error:?}"));
                continue;
            }
        };

        let expected: usize = series.iter().map(Vec::len).sum();
        if grid.data_rows() * grid.data_cols() != expected {
            failures.push(format!(
                "{name}: 치수 {}x{} 가 정답지 값 {expected}개와 맞지 않는다",
                grid.data_rows(),
                grid.data_cols()
            ));
            continue;
        }

        let located: Vec<(usize, f64)> = grid.value_offsets().collect();
        let values: Vec<f64> = located.iter().map(|(_, value)| *value).collect();
        let series_major: Vec<f64> = series.iter().flatten().copied().collect();
        let cols = series[0].len();
        let category_major: Vec<f64> = (0..cols)
            .flat_map(|c| series.iter().map(move |s| s[c]))
            .collect();
        if values != series_major && values != category_major {
            failures.push(format!(
                "{name}: 값이 어느 순서와도 불일치 — 실측 {values:?} / 정답지 {series:?}"
            ));
            continue;
        }

        // 패치 주소가 실제로 그 값을 담고 있어야 레거시 쓰기(#4100 후속)가 성립한다.
        for (offset, value) in &located {
            let round_trip =
                f64::from_le_bytes(legacy[*offset..*offset + 8].try_into().expect("8바이트"));
            assert_eq!(round_trip, *value, "{name}: 오프셋 {offset} 재독 불일치");
        }
        checked += 1;
    }

    assert!(
        failures.is_empty(),
        "구조 스캔 실패 {}건:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert_eq!(checked, 56, "코퍼스 28종 × 2포맷을 전건 검사해야 한다");
}

/// 수용 기준 2 — `parse_ole_chart_contents` 산출이 코퍼스 전건에서 정답지와 같다.
///
/// 계열별 값 배열을 순서대로 대조하므로 **전치되면 반드시 실패한다.** 결함 2·3 이
/// 남아 있으면 여기서 잡힌다.
#[test]
fn parse_ole_chart_contents_matches_ooxml_across_corpus() {
    let mut checked = 0usize;
    let mut failures = Vec::new();

    for path in corpus_files() {
        let name = label(&path);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{name} 읽기: {e}"));
        let Some((legacy, ooxml)) = chart_streams(&bytes) else {
            failures.push(format!("{name}: 차트 스트림 추출 실패"));
            continue;
        };
        let Some(truth) = OoxmlChart::parse(&ooxml) else {
            failures.push(format!("{name}: OOXML 파싱 실패"));
            continue;
        };
        let expected: Vec<Vec<f64>> = truth
            .series
            .iter()
            .map(|s| s.values.clone())
            .filter(|v| !v.is_empty())
            .collect();

        let chart = match parse_ole_chart_contents(&legacy) {
            Ok(chart) => chart,
            Err(error) => {
                failures.push(format!("{name}: 파싱 실패 — {error}"));
                continue;
            }
        };

        if chart.series.len() != expected.len() {
            failures.push(format!(
                "{name}: 계열 {}개 != 정답지 {}개 (전치 의심)",
                chart.series.len(),
                expected.len()
            ));
            continue;
        }
        for (index, truth_values) in expected.iter().enumerate() {
            let got = &chart.series[index].values;
            if got != truth_values {
                failures.push(format!(
                    "{name}: 계열 {index} 값 {got:?} != 정답지 {truth_values:?}"
                ));
            }
        }
        if chart.categories.len() != expected[0].len() {
            failures.push(format!(
                "{name}: 카테고리 {}개 != 정답지 {}개",
                chart.categories.len(),
                expected[0].len()
            ));
        }
        checked += 1;
    }

    assert!(
        failures.is_empty(),
        "파싱 결과 불일치 {}건:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert_eq!(checked, 56, "코퍼스 28종 × 2포맷을 전건 검사해야 한다");
}

/// 수용 기준 3 — 레거시 단독 대조군은 **열이 계열**이고, #1251 골든이 그대로 나온다.
///
/// `tests/issue_1251_ole_chart_contents.rs` 보다 먼저 실패하도록 방향 판정 계층에서
/// 직접 고정한다.
#[test]
fn control_document_is_column_major() {
    for rel in [
        "samples/143E433F503322BD33.hwp",
        "samples/hwpx/143E433F503322BD33.hwpx",
    ] {
        let contents = legacy_contents(&manifest(rel));
        let chart = parse_ole_chart_contents(&contents).expect("파싱");

        assert_eq!(chart.series_axis, SeriesAxis::Columns, "{rel}");
        assert_eq!(
            chart.series_axis_evidence,
            SeriesAxisEvidence::ColumnLabelsEchoed,
            "{rel}: 머리행이 계열 구간에 재등장하는 것이 판정 근거다"
        );
        assert_eq!(
            chart.categories,
            ["2010년", "2020년", "2030년", "2040년"],
            "{rel}"
        );
        assert_eq!(chart.series.len(), 3, "{rel}");
        assert_eq!(chart.series[0].name.as_deref(), Some("적립금"), "{rel}");
        assert_eq!(
            chart.series[0].values,
            [328.0, 812.0, 1702.0, 1477.0],
            "{rel}"
        );
        assert_eq!(chart.series[1].name.as_deref(), Some("수입"), "{rel}");
        assert_eq!(chart.series[1].values, [50.0, 70.0, 189.0, 191.0], "{rel}");
        assert_eq!(chart.series[2].name.as_deref(), Some("지출"), "{rel}");
        assert_eq!(chart.series[2].values, [11.0, 15.0, 201.0, 289.0], "{rel}");
    }
}

/// 결함 1 회귀 — 소수 값이 그대로 살아 나온다.
///
/// `4.3`·`2.5`·`1.8` 은 옛 `is_plausible_grid_value` 를 통과하지 못했고, 값을 거르는 데
/// 그치지 않고 연속 런을 끊어 이 문서의 파싱 **전체**를 실패시켰다.
#[test]
fn decimal_values_survive_the_removed_filter() {
    for rel in [
        "samples/chart/세로막대형/묶은세로막대형.hwp",
        "samples/chart/세로막대형/묶은세로막대형.hwpx",
    ] {
        let contents = legacy_contents(&manifest(rel));
        let chart = parse_ole_chart_contents(&contents).expect("파싱");

        assert_eq!(chart.series.len(), 3, "{rel}");
        assert_eq!(chart.series[0].values, [4.3, 2.5, 3.5, 4.5], "{rel}");
        assert_eq!(chart.series[1].values, [2.4, 4.4, 1.8, 2.8], "{rel}");
        assert_eq!(chart.series[2].values, [2.0, 2.0, 3.0, 5.0], "{rel}");
        assert_eq!(chart.series_axis, SeriesAxis::Rows, "{rel}");
    }
}

/// 결함 3 회귀 — 숫자가 섞인 라벨이 계열·카테고리를 뒤집지 않는다.
///
/// 분산형은 X 값이 카테고리 자리에 오므로 라벨이 `0.7`·`1.8`·`2.6` 이다. 옛 digit
/// 휴리스틱은 이것도, 계열명 `Y1 값`·`Y2 값` 도 전부 카테고리로 몰아 계열을 0개로 만들었다.
#[test]
fn numeric_labels_are_not_mistaken_for_series() {
    for rel in [
        "samples/chart/분산형/표식만있는분산형.hwp",
        "samples/chart/분산형/표식만있는분산형.hwpx",
    ] {
        let contents = legacy_contents(&manifest(rel));
        let chart = parse_ole_chart_contents(&contents).expect("파싱");

        assert_eq!(chart.categories, ["0.7", "1.8", "2.6"], "{rel}");
        assert_eq!(chart.series.len(), 2, "{rel}");
        assert_eq!(chart.series[0].name.as_deref(), Some("Y1 값"), "{rel}");
        assert_eq!(chart.series[1].name.as_deref(), Some("Y2 값"), "{rel}");
    }
}

/// 가납 조건 고정 — 데이터 행이 1개면 열 라벨 echo 를 증거로 쓰지 않는다.
///
/// 원형 차트의 범례는 계열이 아니라 **슬라이스(=열)** 를 나열하므로 열 라벨이 전부
/// 재등장한다. 가납 조건을 빼면 이 10건이 정확히 반대로 판정된다.
#[test]
fn single_data_row_charts_are_not_transposed() {
    for rel in [
        "samples/chart/원형/2차원원형.hwp",
        "samples/chart/원형/2차원원형.hwpx",
    ] {
        let contents = legacy_contents(&manifest(rel));
        let chart = parse_ole_chart_contents(&contents).expect("파싱");

        assert_eq!(chart.series.len(), 1, "{rel}: 원형은 1계열이다");
        assert_eq!(chart.categories.len(), 4, "{rel}");
        assert_eq!(chart.series_axis, SeriesAxis::Rows, "{rel}");
        assert_eq!(
            chart.series_axis_evidence,
            SeriesAxisEvidence::Inconclusive,
            "{rel}: 관례 폴백이라는 사실이 IR 에 드러나야 한다"
        );
    }
}

/// 정사각 데이터 그리드도 모호하지 않게 풀린다.
///
/// `시가고가저가종가` 는 데이터가 4행 × 4열이라 개수만으로는 축을 못 가른다. echo 판정은
/// 머리열 4/4 · 머리행 0/4 로 갈라낸다.
#[test]
fn square_data_grid_orientation_is_decided() {
    for rel in [
        "samples/chart/기타/시가고가저가종가.hwp",
        "samples/chart/기타/시가고가저가종가.hwpx",
    ] {
        let contents = legacy_contents(&manifest(rel));
        let grid = scan_legacy_grid(&contents).expect("스캔");
        assert_eq!((grid.data_rows(), grid.data_cols()), (4, 4), "{rel}");

        let chart = parse_ole_chart_contents(&contents).expect("파싱");
        assert_eq!(chart.series_axis, SeriesAxis::Rows, "{rel}");
        assert_eq!(
            chart.series_axis_evidence,
            SeriesAxisEvidence::RowLabelsEchoed,
            "{rel}"
        );
        assert_eq!(chart.series.len(), 4, "{rel}");
    }
}

/// 값이 하나뿐인 그리드도 파싱된다.
#[test]
fn degenerate_single_cell_grid_parses() {
    for rel in [
        "samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwp",
        "samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwpx",
    ] {
        let contents = legacy_contents(&manifest(rel));
        let grid = scan_legacy_grid(&contents).expect("스캔");
        assert_eq!((grid.data_rows(), grid.data_cols()), (1, 1), "{rel}");

        let chart = parse_ole_chart_contents(&contents).expect("파싱");
        assert_eq!(chart.series.len(), 1, "{rel}");
        assert_eq!(chart.series[0].values.len(), 1, "{rel}");
        assert_eq!(
            chart.series_axis_evidence,
            SeriesAxisEvidence::Inconclusive,
            "{rel}"
        );
    }
}

/// 육안 확인용 변종 — OOXML 사본만 걷어내 레거시 경로를 강제한다.
///
/// 이 결함들은 **정상 문서에서는 눈에 보이지 않는다.** 렌더러가 OOXML 을 먼저 시도해
/// 성공하면 레거시로 내려가지 않고(`shape_layout.rs` 의 `rendered` 단락), 레거시 단독
/// 대조군은 수용 기준 3 에 따라 결과가 바뀌면 안 되는 문서이기 때문이다.
///
/// 그래서 중첩 CFB 에서 `OOXMLChartContents` 만 빼 레거시 경로를 태운다. 레거시
/// `Contents` 는 손대지 않는다. 산출물을 변경 전/후 바이너리로 각각 렌더하면
///
/// - **변경 전** — 주황 placeholder `OLE 차트 미지원: … labels are incomplete`
/// - **변경 후** — 값 `4.3 / 2.5 / 3.5 / 4.5` 의 막대차트
///
/// ```text
/// cargo test --test issue_4098_legacy_chart_grid -- --ignored generate_legacy_only_variant
/// rhwp export-svg output/issue-4098/legacy-only-묶은세로막대형.hwp -o output/issue-4098/after
/// ```
#[test]
#[ignore = "output/ 에 파일을 쓴다 — 육안 확인 직전에만 실행"]
fn generate_legacy_only_variant() {
    let source = manifest("samples/chart/세로막대형/묶은세로막대형.hwp");
    let bytes = std::fs::read(&source).expect("원본 읽기");

    let ole_path = all_streams(&bytes)
        .into_iter()
        .map(|(name, _)| name)
        .find(|name| name.starts_with("/BinData/") && name.to_ascii_uppercase().ends_with(".OLE"))
        .expect("HWP BinData OLE 스트림");
    let ole_stream = all_streams(&bytes)
        .into_iter()
        .find(|(name, _)| *name == ole_path)
        .map(|(_, data)| data)
        .expect("OLE 스트림 바이트");

    let nested = hwp_nested_cfb(&ole_stream);
    let mut streams = all_streams(&nested);
    let before = streams.len();
    streams.retain(|(name, _)| name != "/OOXMLChartContents");
    assert_eq!(before - streams.len(), 1, "OOXML 사본이 정확히 하나 빠져야 한다");
    assert!(
        streams.iter().any(|(name, _)| name == "/Contents"),
        "레거시 Contents 는 남아 있어야 한다"
    );

    let rebuilt = rebuild_cfb_preserving_clsid(&nested, &streams);
    let variant = rewrite_hwp(&bytes, &[(ole_path, hwp_ole_stream(&rebuilt))]);

    let out_dir = manifest("output/issue-4098");
    std::fs::create_dir_all(&out_dir).expect("output 디렉터리");
    let out = out_dir.join("legacy-only-묶은세로막대형.hwp");
    std::fs::write(&out, &variant).expect("변종 쓰기");

    // 변종이 실제로 레거시 경로를 태우는지, 그리고 값이 살아 있는지 확인한다.
    let doc_streams = all_streams(&variant);
    let nested_again = doc_streams
        .iter()
        .find(|(name, _)| name.to_ascii_uppercase().ends_with(".OLE"))
        .map(|(_, data)| hwp_nested_cfb(data))
        .expect("재작성된 중첩 CFB");
    let legacy = all_streams(&nested_again)
        .into_iter()
        .find(|(name, _)| name == "/Contents")
        .map(|(_, data)| data)
        .expect("레거시 Contents");
    let chart = parse_ole_chart_contents(&legacy).expect("레거시 파싱");
    assert_eq!(chart.series[0].values, [4.3, 2.5, 3.5, 4.5]);

    println!("변종: {}", out.display());
}

/// 구간 제한이 없으면 그리드 밖 `VtDouble`(축 눈금 등)을 주워 온다.
///
/// #4055 Stage 1 실측: 대조군에서 제한 12 · 무제한 14.
#[test]
fn grid_window_bounds_the_value_scan() {
    let contents = legacy_contents(&manifest("samples/143E433F503322BD33.hwp"));
    let grid = scan_legacy_grid(&contents).expect("스캔");
    let bounded = grid.value_offsets().count();
    assert_eq!(bounded, 12, "대조군은 3계열 × 4카테고리 = 12개다");

    // 같은 트레일러를 창 제한 없이 세어 본다.
    let trailer = [0xFFu8, 0xFF, 0x06, 0x00, 0x00, 0x00];
    let anchor = contents
        .windows(b"VtDouble\0".len())
        .position(|w| w == b"VtDouble\0")
        .expect("VtDouble");
    let unbounded = contents
        .windows(trailer.len())
        .enumerate()
        .filter(|(at, w)| *w == trailer && *at >= anchor + 8)
        .count();
    assert!(
        unbounded > bounded,
        "구간 제한이 유효해야 한다 — 무제한 {unbounded} vs 제한 {bounded}"
    );
}
