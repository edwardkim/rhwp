//! [#4100] B1 엔진축 — 차트 숫자 데이터 편집.
//!
//! 구조 스캐너·최소 diff 패처(Stage 1), 중첩 CFB 스트림 교체(Stage 2), 주소→①② 슬롯
//! 해석과 `get_chart_data_native`(Stage 3), `set_chart_data_native` 의 ①② 동시
//! 기록(Stage 4), 그리고 실사용 문서 회귀(Stage 5)를 검증한다. CLI 계약은
//! `tests/chart_csv_contract.rs` 에 있다. [#5447] B2 구조 변종 스파이크(행·열·라벨
//! 바이트 수술과 한컴 판정 번들)는 Stage 7 이다.
//!
//! 스캐너를 따로 만드는 이유는 `src/ooxml_chart/parser.rs` 가 **손실 파서**이기
//! 때문이다 — `c:pt idx`·`c:f`·`c:externalData`·`extLst` 를 읽지 않아 파싱→재방출로
//! 왕복시키면 모델에 없는 것이 전부 사라진다. 코퍼스 28종 전건이 `c:extLst` 와
//! `ho:hncChartStyle` 을 갖고 있다. 그래서 `c:v` 텍스트 구간만 바꾸는 바이트 수술을 한다.

#[path = "support/issue_4055_chart_probe.rs"]
mod chart_probe_support;

use chart_probe_support::{chart_streams, corpus, manifest, root_clsid};

use rhwp::ooxml_chart::data::{scan_chart_values, SeriesAxis};
use rhwp::ooxml_chart::patch::{apply_value_edits, EditTarget, PatchError, ValueEdit};
use rhwp::ooxml_chart::OoxmlChart;
use rhwp::parser::ole_container::{all_ole_streams, ole_root_clsid};
use rhwp::serializer::ole_container::{replace_ole_stream, OleRepackError};

/// 중첩 CFB 안 OOXML 차트 스트림의 이름.
const OOXML_STREAM: &str = "OOXMLChartContents";

/// 코퍼스 28종 × 2포맷. `samples/chart/` 에 파일을 커밋하면 이 수가 깨진다 —
/// `issue_4055_b1_chart_edit_probe.rs` 의 `checked == 56` 과 같은 고정이다.
const CORPUS_FILES: usize = 56;

/// `(경로, OOXML 차트 XML)` 전건. HWPX 는 `Chart/chartN.xml`, HWP5 는 중첩 CFB 의
/// `OOXMLChartContents` 에서 온다 — `chart_streams` 가 그 차이를 흡수한다.
fn corpus_charts() -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    for hwpx in corpus() {
        for path in [hwpx.with_extension("hwpx"), hwpx.with_extension("hwp")] {
            let bytes = std::fs::read(&path)
                .unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", path.display()));
            let (_legacy, ooxml) = chart_streams(&bytes)
                .unwrap_or_else(|| panic!("{}: 차트 스트림을 꺼내지 못했다", path.display()));
            out.push((path, ooxml));
        }
    }
    assert_eq!(out.len(), CORPUS_FILES, "코퍼스 28종 × 2포맷");
    out
}

/// `(경로, 중첩 CFB 바이트)` 전건.
///
/// IR 의 `bin_data_content` 에는 **접두어 없는 맨 중첩 CFB** 가 들어 있다 — 4바이트 LE
/// 크기 접두어는 직렬화기가 붙이고 파서가 뗀다(`serializer/cfb_writer.rs`).
fn corpus_nested_cfbs() -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    for hwpx in corpus() {
        for path in [hwpx.with_extension("hwpx"), hwpx.with_extension("hwp")] {
            let bytes = std::fs::read(&path).expect("샘플 읽기");
            let doc = rhwp::parse_document(&bytes)
                .unwrap_or_else(|e| panic!("{}: 파싱 {e:?}", path.display()));
            let nested = doc
                .bin_data_content
                .iter()
                .map(|c| c.data.load())
                .find(|b| b.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]))
                .unwrap_or_else(|| panic!("{}: 중첩 CFB 를 못 찾았다", path.display()));
            out.push((path, nested));
        }
    }
    assert_eq!(out.len(), CORPUS_FILES, "코퍼스 28종 × 2포맷");
    out
}

/// 중첩 CFB 에서 스트림 하나를 꺼낸다.
fn stream_of(cfb: &[u8], name: &str) -> Option<Vec<u8>> {
    all_ole_streams(cfb)?
        .into_iter()
        .find(|(p, _)| p.trim_start_matches('/') == name)
        .map(|(_, d)| d)
}

/// 모든 값 점을 **자기 텍스트 그대로** 쓰는 편집 목록. 무편집 왕복의 재료다.
fn identity_edits(data: &rhwp::ooxml_chart::data::ChartData) -> Vec<ValueEdit> {
    let mut edits = Vec::new();
    for (si, series) in data.series.iter().enumerate() {
        for (pi, point) in series.values.iter().enumerate() {
            edits.push(ValueEdit {
                series: si,
                point: pi,
                target: EditTarget::Value,
                text: point.text.clone(),
            });
        }
        if series.axis == SeriesAxis::Scatter {
            for (pi, point) in series.labels.iter().enumerate() {
                edits.push(ValueEdit {
                    series: si,
                    point: pi,
                    target: EditTarget::Label,
                    text: point.text.clone(),
                });
            }
        }
    }
    edits
}

/// 스캐너가 모델 파서와 같은 값을 같은 순서로 본다.
///
/// 오라클을 `OoxmlChart::parse` 로 두는 이유: 그것이 렌더러가 실제로 그리는 값이고,
/// 스캐너와 완전히 다른 경로(SAX 모델 빌드 vs 오프셋 추적)라 공모하지 않는다.
#[test]
fn scanner_agrees_with_the_model_parser_across_the_corpus() {
    let mut checked = 0usize;
    for (path, ooxml) in corpus_charts() {
        let scan = scan_chart_values(&ooxml)
            .unwrap_or_else(|e| panic!("{}: 스캔 실패 {e:?}", path.display()));
        let model = OoxmlChart::parse(&ooxml)
            .unwrap_or_else(|| panic!("{}: 모델 파서가 차트를 못 읽었다", path.display()));

        assert_eq!(
            scan.series.len(),
            model.series.len(),
            "{}: 계열 수",
            path.display()
        );

        for (i, (scanned, modeled)) in scan.series.iter().zip(&model.series).enumerate() {
            let values: Vec<f64> = scanned
                .values
                .iter()
                .map(|p| {
                    p.text
                        .parse::<f64>()
                        .unwrap_or_else(|_| panic!("{}: 계열 {i} 값 `{}`", path.display(), p.text))
                })
                .collect();
            assert_eq!(values, modeled.values, "{}: 계열 {i} 값", path.display());

            if scanned.axis == SeriesAxis::Scatter {
                let xs: Vec<f64> = scanned
                    .labels
                    .iter()
                    .map(|p| p.text.parse::<f64>().expect("분산형 X 는 수치"))
                    .collect();
                assert_eq!(xs, modeled.x_values, "{}: 계열 {i} X", path.display());
            } else {
                assert!(
                    modeled.x_values.is_empty(),
                    "{}: 계열 {i} 는 분산형이 아닌데 모델에 X 가 있다",
                    path.display()
                );
            }
        }
        checked += 1;
    }
    assert_eq!(checked, CORPUS_FILES);
}

/// 구간이 자기 텍스트를 정확히 가리킨다 — 패처가 믿는 유일한 계약이다.
#[test]
fn every_span_slices_back_to_its_own_text() {
    let mut points = 0usize;
    for (path, ooxml) in corpus_charts() {
        let scan = scan_chart_values(&ooxml).expect("스캔");
        for series in &scan.series {
            for point in series.values.iter().chain(series.labels.iter()) {
                let span = point
                    .span
                    .clone()
                    .unwrap_or_else(|| panic!("{}: 코퍼스엔 빈 값이 없다", path.display()));
                assert_eq!(
                    &ooxml[span.clone()],
                    point.text.as_bytes(),
                    "{}: 구간 {span:?} 이 텍스트와 다르다",
                    path.display(),
                );
                points += 1;
            }
        }
    }
    // 계열 68 + 카테고리/X — 코퍼스가 바뀌지 않는 한 0 이 될 수 없다.
    assert!(points > 200, "훑은 점이 너무 적다: {points}");
}

/// **무편집 왕복이 바이트 동일하다** (수용 기준 2 의 XML 층).
///
/// 모든 값을 자기 텍스트로 다시 써도 한 바이트도 달라지지 않아야 한다. 여기서
/// 어긋나면 정규화·재직렬화가 어딘가 섞였다는 뜻이다.
#[test]
fn identity_patch_is_byte_identical_across_the_corpus() {
    let mut checked = 0usize;
    for (path, ooxml) in corpus_charts() {
        let scan = scan_chart_values(&ooxml).expect("스캔");
        let edits = identity_edits(&scan);
        assert!(!edits.is_empty(), "{}: 편집 대상 0건", path.display());

        let patched = apply_value_edits(&ooxml, &scan, &edits)
            .unwrap_or_else(|e| panic!("{}: 패치 실패 {e:?}", path.display()));
        assert_eq!(
            patched,
            ooxml,
            "{}: 무편집 왕복이 바이트를 바꿨다",
            path.display()
        );
        checked += 1;
    }
    assert_eq!(checked, CORPUS_FILES);
}

/// 실제 편집이 그 값 **하나만** 바꾼다 — 최소 diff 의 정의.
#[test]
fn a_single_edit_changes_only_that_value() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let bytes = std::fs::read(&path).expect("샘플 읽기");
    let (_legacy, ooxml) = chart_streams(&bytes).expect("차트 스트림");
    let scan = scan_chart_values(&ooxml).expect("스캔");

    let before = scan.series[0].values[0].text.clone();
    assert_ne!(before, "91.7", "센티널이 원본과 같으면 판정이 공허하다");

    let patched = apply_value_edits(
        &ooxml,
        &scan,
        &[ValueEdit {
            series: 0,
            point: 0,
            target: EditTarget::Value,
            text: "91.7".to_string(),
        }],
    )
    .expect("패치");

    let after = scan_chart_values(&patched).expect("재스캔");
    assert_eq!(after.series[0].values[0].text, "91.7");

    // 나머지 값은 전부 그대로다.
    for (si, (a, b)) in after.series.iter().zip(&scan.series).enumerate() {
        for (pi, (x, y)) in a.values.iter().zip(&b.values).enumerate() {
            if (si, pi) == (0, 0) {
                continue;
            }
            assert_eq!(x.text, y.text, "계열 {si} 점 {pi} 이 함께 바뀌었다");
        }
    }

    // 길이 차이는 텍스트 길이 차이뿐 — 그 외 바이트는 손대지 않았다.
    let delta = patched.len() as isize - ooxml.len() as isize;
    assert_eq!(delta, "91.7".len() as isize - before.len() as isize);
}

/// **M3** — `c:numLit`/`c:strLit` 문서도 캐시형과 같은 경로로 잡힌다.
///
/// 코퍼스에서 리터럴을 쓰는 문서는 이 한 건뿐이고, `c:f`·`c:cat` 참조가 아예 없다.
/// 무편집 왕복의 최난도 케이스라 따로 지목해 둔다.
#[test]
fn numeric_literal_chart_is_scanned_like_a_cached_one() {
    let path = manifest("samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwpx");
    let bytes = std::fs::read(&path).expect("샘플 읽기");
    let (_legacy, ooxml) = chart_streams(&bytes).expect("차트 스트림");

    assert!(
        String::from_utf8_lossy(&ooxml).contains("numLit"),
        "이 샘플은 c:numLit 을 써야 한다 — 아니면 M3 판정이 대상을 잃는다"
    );

    let scan = scan_chart_values(&ooxml).expect("스캔");
    assert_eq!(scan.series.len(), 1);
    assert_eq!(scan.series[0].values.len(), 1);
    assert_eq!(scan.series[0].axis, SeriesAxis::Category);

    let patched = apply_value_edits(&ooxml, &scan, &identity_edits(&scan)).expect("패치");
    assert_eq!(patched, ooxml, "리터럴 문서의 무편집 왕복이 깨졌다");
}

/// **M2** — 분산형의 X 는 편집 대상이고, 코퍼스에서는 계열 간 동일하다.
///
/// 이 동일성은 **코퍼스 성질이지 포맷 보장이 아니다.** OOXML 은 계열마다 다른 X 를
/// 허용하므로 CSV 층(Stage 5)이 `sharedXRequired` 로 거부한다. 여기서는 그 전제가
/// 코퍼스에서 실제로 성립함을 고정한다.
#[test]
fn scatter_series_expose_editable_x_values_shared_across_series() {
    let mut scatter_files = 0usize;
    for (path, ooxml) in corpus_charts() {
        let scan = scan_chart_values(&ooxml).expect("스캔");
        if scan.series.iter().all(|s| s.axis != SeriesAxis::Scatter) {
            continue;
        }
        assert!(
            scan.series.iter().all(|s| s.axis == SeriesAxis::Scatter),
            "{}: 분산형과 카테고리형 계열이 섞였다",
            path.display()
        );

        let first: Vec<&str> = scan.series[0]
            .labels
            .iter()
            .map(|p| p.text.as_str())
            .collect();
        for (i, series) in scan.series.iter().enumerate().skip(1) {
            let xs: Vec<&str> = series.labels.iter().map(|p| p.text.as_str()).collect();
            assert_eq!(
                xs,
                first,
                "{}: 계열 {i} 의 X 가 계열 0 과 다르다",
                path.display()
            );
        }
        scatter_files += 1;
    }
    assert_eq!(scatter_files, 10, "분산형 5종 × 2포맷");
}

/// [#5652] 카테고리 라벨은 **패처 층**에서는 캐시 텍스트 치환으로 적용된다 — B1 의
/// `LabelNotEditable` 은 사라졌다. 허용 여부는 코어의 의도 플래그(`structure`)가 정한다
/// (S3 — `structure_flag_off_keeps_every_b1_refusal` 가 `categoryMismatch` 로 거부를 고정).
#[test]
fn category_labels_patch_at_the_byte_layer() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let bytes = std::fs::read(&path).expect("샘플 읽기");
    let (_legacy, ooxml) = chart_streams(&bytes).expect("차트 스트림");
    let scan = scan_chart_values(&ooxml).expect("스캔");
    assert_eq!(scan.series[0].axis, SeriesAxis::Category);

    let out = apply_value_edits(
        &ooxml,
        &scan,
        &[ValueEdit {
            series: 0,
            point: 0,
            target: EditTarget::Label,
            text: "새 라벨".to_string(),
        }],
    )
    .expect("패처 층은 라벨 치환을 적용한다");
    let text = String::from_utf8(out).expect("UTF-8");
    assert!(text.contains("<c:v>새 라벨</c:v>"), "라벨이 바뀌지 않았다");
    assert!(text.contains("Sheet1!$A$2:$A$5"), "c:f 가 사라졌다");
    let rescan = scan_chart_values(text.as_bytes()).expect("재스캔");
    assert_eq!(rescan.series[0].labels[0].text, "새 라벨");
    assert_eq!(
        rescan.series[1].labels[0].text, "항목 1",
        "다른 계열 라벨은 그대로다"
    );
}

/// 패처는 주소 오류·중복·XML 안전하지 않은 텍스트를 **쓰기 전에** 거부한다.
#[test]
fn patcher_rejects_bad_addresses_duplicates_and_unsafe_text() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let bytes = std::fs::read(&path).expect("샘플 읽기");
    let (_legacy, ooxml) = chart_streams(&bytes).expect("차트 스트림");
    let scan = scan_chart_values(&ooxml).expect("스캔");

    let value = |series, point, text: &str| ValueEdit {
        series,
        point,
        target: EditTarget::Value,
        text: text.to_string(),
    };

    let cases: Vec<(ValueEdit, &str)> = vec![
        (value(99, 0, "1"), "없는 계열"),
        (value(0, 99, "1"), "없는 점"),
        (value(0, 0, "1 < 2"), "XML 특수문자"),
        (value(0, 0, "a&b"), "XML 특수문자"),
    ];
    for (edit, why) in cases {
        assert!(
            apply_value_edits(&ooxml, &scan, &[edit]).is_err(),
            "{why} 는 거부되어야 한다"
        );
    }

    let err = apply_value_edits(&ooxml, &scan, &[value(0, 0, "1"), value(0, 0, "2")])
        .expect_err("같은 점을 두 번 지목하면 거부");
    assert!(matches!(err, PatchError::DuplicateTarget { .. }), "{err:?}");
}

// ---------------------------------------------------------------------------
// Stage 2 — 중첩 CFB 스트림 교체 (②축의 재료)
// ---------------------------------------------------------------------------

/// **재포장이 아는 4종 밖 스트림까지 살린다.**
///
/// `parse_ole_container` 로 재포장하면 나머지가 소실되므로 `all_ole_streams` 전수
/// 열거 위에 선다. 코퍼스는 `Contents`·`\x02OlePres000`·`OOXMLChartContents` 셋인데,
/// 이름을 고정하지 않고 **집합이 보존되는지**로 판정한다.
#[test]
fn repack_preserves_every_stream_and_leaves_the_others_byte_identical() {
    let mut checked = 0usize;
    for (path, nested) in corpus_nested_cfbs() {
        let before = all_ole_streams(&nested)
            .unwrap_or_else(|| panic!("{}: 중첩 CFB 열거 실패", path.display()));
        assert!(
            before
                .iter()
                .any(|(p, _)| p.trim_start_matches('/') == OOXML_STREAM),
            "{}: OOXMLChartContents 가 없다",
            path.display()
        );

        let ooxml = stream_of(&nested, OOXML_STREAM).expect("OOXML");
        let scan = scan_chart_values(&ooxml).expect("스캔");
        let patched = apply_value_edits(
            &ooxml,
            &scan,
            &[ValueEdit {
                series: 0,
                point: 0,
                target: EditTarget::Value,
                text: "91.7".to_string(),
            }],
        )
        .expect("패치");

        let repacked = replace_ole_stream(&nested, OOXML_STREAM, &patched)
            .unwrap_or_else(|e| panic!("{}: 재포장 {e}", path.display()));
        let after = all_ole_streams(&repacked).expect("재포장본 열거");

        let names_before: Vec<&str> = before.iter().map(|(p, _)| p.as_str()).collect();
        let names_after: Vec<&str> = after.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(names_after, names_before, "{}: 스트림 집합", path.display());

        for (name, bytes) in &before {
            if name.trim_start_matches('/') == OOXML_STREAM {
                continue;
            }
            let now = stream_of(&repacked, name.trim_start_matches('/')).expect("스트림");
            assert_eq!(&now, bytes, "{}: `{name}` 이 바뀌었다", path.display());
        }

        assert_eq!(
            stream_of(&repacked, OOXML_STREAM).as_deref(),
            Some(patched.as_slice()),
            "{}: 새 OOXML 이 실리지 않았다",
            path.display()
        );
        checked += 1;
    }
    assert_eq!(checked, CORPUS_FILES);
}

/// **루트 CLSID 가 살아남는다** — 떨구면 한컴이 개체를 알아보지 못해 내용을 비운다(#4097).
///
/// 판정은 `cfb` 크레이트 오라클(`root_clsid`)로 한다. rhwp 의 `ole_root_clsid` 로만
/// 재면 읽기·쓰기가 같은 오프셋 오해를 공유해도 통과해 버린다.
#[test]
fn repack_preserves_the_root_class_id() {
    let mut checked = 0usize;
    for (path, nested) in corpus_nested_cfbs() {
        let original = root_clsid(&nested);
        assert_ne!(
            original,
            [0u8; 16],
            "{}: 원본 CLSID 가 0 이면 판정이 공허하다",
            path.display()
        );

        let ooxml = stream_of(&nested, OOXML_STREAM).expect("OOXML");
        let scan = scan_chart_values(&ooxml).expect("스캔");
        let patched = apply_value_edits(
            &ooxml,
            &scan,
            &[ValueEdit {
                series: 0,
                point: 0,
                target: EditTarget::Value,
                text: "91.7".to_string(),
            }],
        )
        .expect("패치");
        let repacked = replace_ole_stream(&nested, OOXML_STREAM, &patched).expect("재포장");

        assert_eq!(root_clsid(&repacked), original, "{}", path.display());
        assert_eq!(
            ole_root_clsid(&repacked),
            Some(original),
            "{}",
            path.display()
        );
        checked += 1;
    }
    assert_eq!(checked, CORPUS_FILES);
}

/// **바뀐 게 없으면 중첩 CFB 를 다시 쓰지 않는다.**
///
/// 재포장은 섹터 배치가 원본 작성기와 달라 바이트 동일을 보장하지 않는다. 짧은 회로가
/// 없으면 "무편집 왕복 바이트 동일"(수용 기준 2)이 재포장만으로 깨진다.
#[test]
fn unchanged_stream_content_skips_the_repack_entirely() {
    let mut checked = 0usize;
    for (path, nested) in corpus_nested_cfbs() {
        let ooxml = stream_of(&nested, OOXML_STREAM).expect("OOXML");
        let out = replace_ole_stream(&nested, OOXML_STREAM, &ooxml).expect("재포장");
        assert_eq!(
            out,
            nested,
            "{}: 무편집인데 바이트가 바뀌었다",
            path.display()
        );
        checked += 1;
    }
    assert_eq!(checked, CORPUS_FILES);
}

/// 없는 스트림은 새로 만들지 않고 거부한다 — 이름 오타가 조용히 파일을 망치지 않게.
#[test]
fn repack_refuses_to_invent_a_missing_stream() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let bytes = std::fs::read(&path).expect("샘플 읽기");
    let doc = rhwp::parse_document(&bytes).expect("파싱");
    let nested = doc
        .bin_data_content
        .iter()
        .map(|c| c.data.load())
        .find(|b| b.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]))
        .expect("중첩 CFB");

    assert_eq!(
        replace_ole_stream(&nested, "OOXMLChartContent", b"x"),
        Err(OleRepackError::StreamNotFound(
            "OOXMLChartContent".to_string()
        ))
    );
}

// ---------------------------------------------------------------------------
// Stage 3 — 주소 → ①② 슬롯 해석 + get_chart_data_native
// ---------------------------------------------------------------------------

use rhwp::document_core::queries::chart_extract::{chart_xml, collect_charts};
use rhwp::document_core::DocumentCore;

fn core_of(path: &std::path::Path) -> DocumentCore {
    let bytes = std::fs::read(path).expect("샘플 읽기");
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{}: 코어 {e:?}", path.display()))
}

/// 코퍼스 전건에서 차트가 **정확히 하나** 열거되고, 두 표현이 포맷대로 해소된다.
///
/// HWPX 는 ①②가 다 있고, HWP5 는 `Chart/*.xml` 파트가 없어 ②만 있다.
#[test]
fn every_corpus_document_resolves_its_chart_slots() {
    let mut hwpx_seen = 0usize;
    let mut hwp_seen = 0usize;
    for hwpx in corpus() {
        for path in [hwpx.with_extension("hwpx"), hwpx.with_extension("hwp")] {
            let core = core_of(&path);
            let charts = collect_charts(core.document());
            assert_eq!(charts.len(), 1, "{}: 차트 수", path.display());
            let chart = &charts[0];
            assert!(
                chart.is_top_level(),
                "{}: 본문 직속이어야 한다",
                path.display()
            );
            assert!(chart.nested_copy.is_some(), "{}: ② 미해소", path.display());

            if path.extension().is_some_and(|e| e == "hwpx") {
                assert!(chart.zip_part.is_some(), "{}: ① 미해소", path.display());
                hwpx_seen += 1;
            } else {
                assert!(
                    chart.zip_part.is_none(),
                    "{}: HWP5 에 ① 이 있을 수 없다",
                    path.display()
                );
                hwp_seen += 1;
            }

            let (xml, _) = chart_xml(core.document(), chart).expect("차트 XML");
            assert!(scan_chart_values(&xml).is_ok(), "{}: 스캔", path.display());
        }
    }
    assert_eq!((hwpx_seen, hwp_seen), (28, 28));
}

/// **①==②** — 어느 표현에서 읽어도 같은 XML 이다(#4055 의 SHA-256 전건 일치를 코드로 고정).
#[test]
fn both_representations_carry_the_same_xml() {
    let mut checked = 0usize;
    for hwpx in corpus() {
        let core = core_of(&hwpx);
        let charts = collect_charts(core.document());
        let chart = &charts[0];
        let zip = core.document().bin_data_content[chart.zip_part.expect("①")]
            .data
            .load();
        let nested_cfb = core.document().bin_data_content[chart.nested_copy.expect("②")]
            .data
            .load();
        let nested = stream_of(&nested_cfb, OOXML_STREAM).expect("②의 OOXML");
        assert_eq!(zip, nested, "{}: ① 과 ② 가 다르다", hwpx.display());
        checked += 1;
    }
    assert_eq!(checked, 28);
}

/// `get_chart_data_native` 가 모델 파서와 같은 값을 돌려준다.
#[test]
fn get_chart_data_native_matches_the_model_parser() {
    let mut checked = 0usize;
    for hwpx in corpus() {
        for path in [hwpx.with_extension("hwpx"), hwpx.with_extension("hwp")] {
            let core = core_of(&path);
            let chart = &collect_charts(core.document())[0];
            let json: serde_json::Value = serde_json::from_str(
                &core
                    .get_chart_data_native(chart.section, chart.paragraph, chart.control)
                    .unwrap_or_else(|e| panic!("{}: {e:?}", path.display())),
            )
            .expect("JSON");

            assert_eq!(json["ok"], true, "{}", path.display());
            assert_eq!(json["chart"], 1, "{}", path.display());
            assert_eq!(json["labelsShared"], true, "{}", path.display());

            let (xml, _) = chart_xml(core.document(), chart).expect("XML");
            let model = OoxmlChart::parse(&xml).expect("모델");
            let series = json["series"].as_array().expect("series");
            assert_eq!(series.len(), model.series.len(), "{}", path.display());
            for (s, m) in series.iter().zip(&model.series) {
                let values: Vec<f64> = s["values"]
                    .as_array()
                    .expect("values")
                    .iter()
                    .map(|v| v.as_str().expect("문자열").parse().expect("수치"))
                    .collect();
                assert_eq!(values, m.values, "{}", path.display());
            }

            let is_hwpx = path.extension().is_some_and(|e| e == "hwpx");
            assert_eq!(
                json["representations"]["zipPart"],
                is_hwpx,
                "{}",
                path.display()
            );
            assert_eq!(
                json["representations"]["nestedCopy"],
                true,
                "{}",
                path.display()
            );
            assert_eq!(
                json["source"],
                if is_hwpx { "zipPart" } else { "nestedCopy" },
                "{}",
                path.display()
            );
            checked += 1;
        }
    }
    assert_eq!(checked, CORPUS_FILES);
}

/// 값은 **원본 텍스트 그대로** 실린다 — 실수로 파싱했다가 되쓰면 표기가 달라져
/// 무편집 왕복의 바이트 동일이 깨진다.
#[test]
fn values_keep_their_original_spelling() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let core = core_of(&path);
    let chart = &collect_charts(core.document())[0];
    let (xml, _) = chart_xml(core.document(), chart).expect("XML");
    let scan = scan_chart_values(&xml).expect("스캔");

    let json: serde_json::Value = serde_json::from_str(
        &core
            .get_chart_data_native(chart.section, chart.paragraph, chart.control)
            .expect("읽기"),
    )
    .expect("JSON");

    for (si, series) in scan.series.iter().enumerate() {
        for (pi, point) in series.values.iter().enumerate() {
            assert_eq!(
                json["series"][si]["values"][pi].as_str(),
                Some(point.text.as_str())
            );
        }
    }
}

/// 주소 오류만 `Err` 다 — 데이터 문제는 `Ok` + 부정 봉투다.
#[test]
fn only_address_errors_are_err() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let core = core_of(&path);
    let chart = &collect_charts(core.document())[0];

    assert!(core.get_chart_data_native(99, 0, 0).is_err(), "없는 구역");
    assert!(core.get_chart_data_native(0, 9999, 0).is_err(), "없는 문단");
    assert!(
        core.get_chart_data_native(chart.section, chart.paragraph, 9999)
            .is_err(),
        "없는 컨트롤"
    );

    // 차트가 아닌 컨트롤을 지목하면 Err — 같은 문단의 다른 컨트롤을 찾아 시험한다.
    let para = &core.document().sections[chart.section].paragraphs[chart.paragraph];
    if let Some(other) = (0..para.controls.len()).find(|&i| i != chart.control) {
        assert!(
            core.get_chart_data_native(chart.section, chart.paragraph, other)
                .is_err(),
            "차트가 아닌 컨트롤"
        );
    }
}

/// 순번 경로는 컨테이너 안의 차트에도 닿는다 — 3인자 주소가 표현하지 못하는 자리다.
#[test]
fn index_addressing_covers_the_same_chart() {
    let path = manifest("samples/chart/원형/쪼개진원형.hwpx");
    let core = core_of(&path);
    let chart = &collect_charts(core.document())[0];
    let by_addr = core
        .get_chart_data_native(chart.section, chart.paragraph, chart.control)
        .expect("주소");
    let by_index = core.get_chart_data_by_index_native(0).expect("순번");
    assert_eq!(by_addr, by_index);
    assert!(core.get_chart_data_by_index_native(9).is_err(), "없는 순번");
}

// ---------------------------------------------------------------------------
// Stage 4 — set_chart_data_native (①② 동시 기록)
// ---------------------------------------------------------------------------

/// 레거시 `Contents` 스트림 이름.
const LEGACY_STREAM: &str = "Contents";
/// EMF 프리뷰 스트림 이름.
const EMF_STREAM: &str = "\u{2}OlePres000";

/// 현재 값 그대로의 편집 입력 — 무편집 왕복의 재료.
fn edits_from(core: &DocumentCore, index: usize) -> serde_json::Value {
    let read: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(index).expect("읽기"))
            .expect("JSON");
    serde_json::json!({
        "labels": read["labels"],
        "series": read["series"]
            .as_array()
            .expect("series")
            .iter()
            .map(|s| serde_json::json!({"name": s["name"], "values": s["values"]}))
            .collect::<Vec<_>>(),
    })
}

fn slot_bytes(core: &DocumentCore) -> Vec<Vec<u8>> {
    core.document()
        .bin_data_content
        .iter()
        .map(|c| c.data.load())
        .collect()
}

fn set_chart(core: &mut DocumentCore, edits: &serde_json::Value) -> serde_json::Value {
    serde_json::from_str(
        &core
            .set_chart_data_by_index_native(0, &edits.to_string())
            .expect("쓰기"),
    )
    .expect("JSON")
}

/// 실문서의 차트 슬롯에 합성 XML을 주입한다. HWPX는 ①과 ②가 각각 다른 바이트를
/// 가질 수 있으므로 테스트도 두 표현을 독립적으로 바꾼다.
fn replace_chart_representations(core: &mut DocumentCore, zip_xml: &[u8], nested_xml: &[u8]) {
    let chart = collect_charts(core.document())[0].clone();
    let zip_idx = chart.zip_part.expect("합성 경계는 HWPX ①을 쓴다");
    let nested_idx = chart.nested_copy.expect("합성 경계는 HWPX ②을 쓴다");
    let nested_original = core.document().bin_data_content[nested_idx].data.load();
    let nested_new =
        replace_ole_stream(&nested_original, OOXML_STREAM, nested_xml).expect("② 교체");

    core.document_mut().bin_data_content[zip_idx].data = zip_xml.to_vec().into();
    core.document_mut().bin_data_content[nested_idx].data = nested_new.into();
}

const BLANK_VALUE_CHART: &str = concat!(
    r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
    r#"<c:cat><c:strLit><c:pt idx="0"><c:v>A</c:v></c:pt><c:pt idx="1"><c:v>B</c:v></c:pt></c:strLit></c:cat>"#,
    r#"<c:val><c:numLit><c:pt idx="0"><c:v>4.3</c:v></c:pt><c:pt idx="1"><c:v/></c:pt></c:numLit></c:val>"#,
    r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
);

const UNSHARED_CATEGORY_CHART: &str = concat!(
    r#"<c:chartSpace><c:chart><c:plotArea><c:barChart>"#,
    r#"<c:ser><c:tx><c:v>첫째</c:v></c:tx><c:cat><c:strLit>"#,
    r#"<c:pt idx="0"><c:v>A</c:v></c:pt><c:pt idx="1"><c:v>B</c:v></c:pt>"#,
    r#"</c:strLit></c:cat><c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt></c:numLit></c:val></c:ser>"#,
    r#"<c:ser><c:tx><c:v>둘째</c:v></c:tx><c:cat><c:strLit>"#,
    r#"<c:pt idx="0"><c:v>B</c:v></c:pt><c:pt idx="1"><c:v>A</c:v></c:pt>"#,
    r#"</c:strLit></c:cat><c:val><c:numLit><c:pt idx="0"><c:v>3</c:v></c:pt><c:pt idx="1"><c:v>4</c:v></c:pt></c:numLit></c:val></c:ser>"#,
    r#"</c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
);

/// **편집이 ①②에 함께 실린다** (수용 기준 1 의 코어 층).
///
/// HWPX 는 두 표현 모두, HWP5 는 ②가 새 값이다. ③(레거시)·④(EMF)는 바이트 그대로 —
/// B1 은 그것들을 쓰지 않는다.
#[test]
fn an_edit_lands_in_both_representations_and_leaves_the_others_alone() {
    let mut checked = 0usize;
    for hwpx in corpus() {
        for path in [hwpx.with_extension("hwpx"), hwpx.with_extension("hwp")] {
            let mut core = core_of(&path);
            let chart = collect_charts(core.document())[0].clone();
            let nested_idx = chart.nested_copy.expect("②");
            let before_nested = core.document().bin_data_content[nested_idx].data.load();
            let legacy_before = stream_of(&before_nested, LEGACY_STREAM);
            let emf_before = stream_of(&before_nested, EMF_STREAM);

            let mut edits = edits_from(&core, 0);
            edits["series"][0]["values"][0] = serde_json::json!("91.7");
            let out = set_chart(&mut core, &edits);

            assert_eq!(out["ok"], true, "{}: {out}", path.display());
            assert_eq!(out["changedCount"], 1, "{}: {out}", path.display());

            let is_hwpx = path.extension().is_some_and(|e| e == "hwpx");
            let wrote: Vec<&str> = out["wrote"]
                .as_array()
                .expect("wrote")
                .iter()
                .map(|v| v.as_str().expect("문자열"))
                .collect();
            if is_hwpx {
                assert_eq!(wrote, ["zipPart", "nestedCopy"], "{}", path.display());
                let zip = core.document().bin_data_content[chart.zip_part.expect("①")]
                    .data
                    .load();
                assert!(
                    String::from_utf8_lossy(&zip).contains("<c:v>91.7</c:v>"),
                    "{}: ① 에 새 값이 없다",
                    path.display()
                );
            } else {
                assert_eq!(wrote, ["nestedCopy"], "{}", path.display());
            }

            let after_nested = core.document().bin_data_content[nested_idx].data.load();
            let ooxml = stream_of(&after_nested, OOXML_STREAM).expect("②의 OOXML");
            assert!(
                String::from_utf8_lossy(&ooxml).contains("<c:v>91.7</c:v>"),
                "{}: ② 에 새 값이 없다",
                path.display()
            );
            assert_eq!(
                stream_of(&after_nested, LEGACY_STREAM),
                legacy_before,
                "{}: ③ 레거시가 바뀌었다",
                path.display()
            );
            assert_eq!(
                stream_of(&after_nested, EMF_STREAM),
                emf_before,
                "{}: ④ EMF 가 바뀌었다",
                path.display()
            );
            checked += 1;
        }
    }
    assert_eq!(checked, CORPUS_FILES);
}

/// **무편집 왕복이 슬롯 바이트를 건드리지 않는다** (수용 기준 2 의 코어 층).
#[test]
fn writing_the_current_values_back_changes_nothing() {
    let mut checked = 0usize;
    for hwpx in corpus() {
        for path in [hwpx.with_extension("hwpx"), hwpx.with_extension("hwp")] {
            let mut core = core_of(&path);
            let before = slot_bytes(&core);

            let edits = edits_from(&core, 0);
            let out = set_chart(&mut core, &edits);

            assert_eq!(out["ok"], true, "{}", path.display());
            assert_eq!(out["changedCount"], 0, "{}: {out}", path.display());
            assert!(
                out["wrote"].as_array().expect("wrote").is_empty(),
                "{}",
                path.display()
            );
            assert_eq!(
                slot_bytes(&core),
                before,
                "{}: 슬롯 바이트가 바뀌었다",
                path.display()
            );
            checked += 1;
        }
    }
    assert_eq!(checked, CORPUS_FILES);
}

/// ①과 ②는 편집 대상 밖 XML이 달라도 각자 자기 바이트 구간만 고친다.
///
/// 이전 구현은 ①로 만든 전체 XML을 ②에도 넣어 확장 속성·미래 요소를 조용히 잃었다.
#[test]
fn matching_representations_are_patched_independently() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let chart = collect_charts(core.document())[0].clone();
    let original = String::from_utf8(
        core.document().bin_data_content[chart.zip_part.expect("①")]
            .data
            .load(),
    )
    .expect("UTF-8 XML");
    let zip = original.replacen("<c:chartSpace", "<c:chartSpace review=\"zip\"", 1);
    let nested = original.replacen("<c:chartSpace", "<c:chartSpace review=\"nested\"", 1);
    replace_chart_representations(&mut core, zip.as_bytes(), nested.as_bytes());

    let mut edits = edits_from(&core, 0);
    edits["series"][0]["values"][0] = serde_json::json!("91.7");
    let out = set_chart(&mut core, &edits);
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["wrote"], serde_json::json!(["zipPart", "nestedCopy"]));

    let zip_after = String::from_utf8(
        core.document().bin_data_content[chart.zip_part.expect("①")]
            .data
            .load(),
    )
    .expect("UTF-8 XML");
    let nested_after = String::from_utf8(
        stream_of(
            &core.document().bin_data_content[chart.nested_copy.expect("②")]
                .data
                .load(),
            OOXML_STREAM,
        )
        .expect("② XML"),
    )
    .expect("UTF-8 XML");
    assert!(zip_after.contains("review=\"zip\""), "{zip_after}");
    assert!(!zip_after.contains("review=\"nested\""), "{zip_after}");
    assert!(nested_after.contains("review=\"nested\""), "{nested_after}");
    assert!(!nested_after.contains("review=\"zip\""), "{nested_after}");
    assert!(zip_after.contains("<c:v>91.7</c:v>"), "{zip_after}");
    assert!(nested_after.contains("<c:v>91.7</c:v>"), "{nested_after}");
}

/// ①과 ②의 데이터가 이미 다르면 어느 표현을 정본으로 삼아도 다른 쪽을 훼손한다.
/// 읽기와 쓰기 모두 fail-closed하며 슬롯 바이트는 그대로여야 한다.
#[test]
fn mismatched_representations_are_refused_without_writing() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let edits = edits_from(&core, 0);
    let chart = collect_charts(core.document())[0].clone();
    let zip = core.document().bin_data_content[chart.zip_part.expect("①")]
        .data
        .load();
    let nested = String::from_utf8(zip.clone()).expect("UTF-8 XML").replacen(
        "<c:v>4.3</c:v>",
        "<c:v>7.7</c:v>",
        1,
    );
    replace_chart_representations(&mut core, &zip, nested.as_bytes());
    let before = slot_bytes(&core);

    let read: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("주소는 유효"))
            .expect("JSON");
    assert_eq!(read["ok"], false, "{read}");
    assert_eq!(read["invalid"][0]["reason"], "representationMismatch");

    let mut changed = edits;
    changed["series"][0]["values"][0] = serde_json::json!("91.7");
    let out = set_chart(&mut core, &changed);
    assert_eq!(out["ok"], false, "{out}");
    assert_eq!(out["invalid"][0]["reason"], "representationMismatch");
    assert_eq!(slot_bytes(&core), before, "거부했는데 차트 슬롯이 바뀌었다");
}

/// `<c:v/>`는 그 점만 구조 변경 없이는 쓸 수 없다. 다른 점을 바꿀 때는 빈 값을
/// 원형으로 전달해도 전체 행렬을 `notANumber`로 거부해서는 안 된다.
#[test]
fn blank_value_blocks_only_its_own_edit() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    replace_chart_representations(
        &mut core,
        BLANK_VALUE_CHART.as_bytes(),
        BLANK_VALUE_CHART.as_bytes(),
    );

    let edits = serde_json::json!({
        "labels": ["A", "B"],
        "series": [{"values": ["91.7", ""]}],
    });
    let out = set_chart(&mut core, &edits);
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["changedCount"], 1, "{out}");

    let before = slot_bytes(&core);
    let bad = serde_json::json!({
        "labels": ["A", "B"],
        "series": [{"values": ["91.7", "5"]}],
    });
    let out = set_chart(&mut core, &bad);
    assert_eq!(out["ok"], false, "{out}");
    assert_eq!(out["invalid"][0]["reason"], "valueNotPatchable");
    assert_eq!(
        slot_bytes(&core),
        before,
        "빈 값 편집 거부 뒤 바이트가 바뀌었다"
    );
}

/// CSV가 넘기는 한 라벨 열은 모든 계열에서 같은 의미여야 한다. 계열별 범주가 다르면
/// native 호출도 labels를 수락하지 않아 CSV의 행-점 오정렬을 막는다.
#[test]
fn nonshared_category_labels_are_refused_before_writing() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    replace_chart_representations(
        &mut core,
        UNSHARED_CATEGORY_CHART.as_bytes(),
        UNSHARED_CATEGORY_CHART.as_bytes(),
    );
    let before = slot_bytes(&core);

    let read: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("주소는 유효"))
            .expect("JSON");
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(read["labelsShared"], false, "{read}");

    let edits = serde_json::json!({
        "labels": ["A", "B"],
        "series": [
            {"name": "첫째", "values": ["10", "2"]},
            {"name": "둘째", "values": ["3", "4"]},
        ],
    });
    let out = set_chart(&mut core, &edits);
    assert_eq!(out["ok"], false, "{out}");
    assert_eq!(out["invalid"][0]["reason"], "sharedCategoryRequired");
    assert_eq!(
        slot_bytes(&core),
        before,
        "비공유 라벨 거부 뒤 바이트가 바뀌었다"
    );
}

/// 편집이 저장·재파스를 넘어 살아남는다 — ②까지 함께.
#[test]
fn the_edit_survives_save_and_reparse() {
    for name in [
        "세로막대형/묶은세로막대형",
        "원형/쪼개진원형",
        "분산형/직선이있는분산형",
    ] {
        let path = manifest(&format!("samples/chart/{name}.hwpx"));
        let mut core = core_of(&path);
        let mut edits = edits_from(&core, 0);
        edits["series"][0]["values"][0] = serde_json::json!("91.7");
        assert_eq!(set_chart(&mut core, &edits)["ok"], true, "{name}");

        let saved = core.export_hwpx_native().expect("HWPX 저장");
        let reread = DocumentCore::from_bytes(&saved).expect("재파스");
        let json: serde_json::Value =
            serde_json::from_str(&reread.get_chart_data_by_index_native(0).expect("읽기"))
                .expect("JSON");
        assert_eq!(json["series"][0]["values"][0], "91.7", "{name}");

        let chart = collect_charts(reread.document())[0].clone();
        let nested = reread.document().bin_data_content[chart.nested_copy.expect("②")]
            .data
            .load();
        let ooxml = stream_of(&nested, OOXML_STREAM).expect("②");
        assert!(
            String::from_utf8_lossy(&ooxml).contains("<c:v>91.7</c:v>"),
            "{name}: 저장 후 ② 에 새 값이 없다"
        );
    }
}

/// **HWP5 저장에서도 편집이 살아남는다** — 패스스루 면제의 근거 (#2724 가드).
///
/// `issue_2724_passthrough_invalidation_guard` 는 문서 IR 을 바꾸는 `&mut self` 가
/// `section.raw_stream`/`doc_info.raw_stream_dirty` 를 무효화하지 않으면 저장이 원본
/// 바이트를 그대로 돌려줘 **편집이 조용히 사라진다**고 경고한다. 차트 편집은
/// `bin_data_content` 만 바꾸고 그것은 BodyText·DocInfo 스트림이 아니라 BinData
/// 저장소라 패스스루 대상이 아니다 — 그 주장을 말로 두지 않고 저장→재파스로 판정한다.
#[test]
fn the_edit_survives_hwp5_save_despite_stream_passthrough() {
    for name in ["세로막대형/묶은세로막대형", "기타/고가저가종가"] {
        let path = manifest(&format!("samples/chart/{name}.hwp"));
        let mut core = core_of(&path);
        let mut edits = edits_from(&core, 0);
        edits["series"][0]["values"][0] = serde_json::json!("91.7");
        assert_eq!(set_chart(&mut core, &edits)["ok"], true, "{name}");

        let saved = core.export_hwp_native().expect("HWP5 저장");
        let reread = DocumentCore::from_bytes(&saved).expect("재파스");
        let json: serde_json::Value =
            serde_json::from_str(&reread.get_chart_data_by_index_native(0).expect("읽기"))
                .expect("JSON");
        assert_eq!(
            json["series"][0]["values"][0], "91.7",
            "{name}: HWP5 저장에서 편집이 사라졌다"
        );
    }
}

/// **T4 — 편집이 HWPX→HWP5 변환을 넘어 살아남는다** (수용 기준 4).
///
/// B1 이 존재하는 이유 그 자체다. 대조군(①만 고친 문서)이 변환 후 **옛 값**이라는 것이
/// ②를 함께 써야 하는 이유의 살아 있는 근거다.
///
/// #4099 착지 전에는 잴 대상이 없었다 — 그때의 변환은 차트를 통째로 잃어
/// (`bin_data_id=60001` 이 그대로 나가 참조가 끊긴다) 새 값이든 옛 값이든 읽을 차트가
/// 없었다. 그래서 `#[ignore]` 로 잠들어 있었고, 짝으로 둔 회수 게이트가 "변환이 차트를
/// 보존하는가"를 관측해 착지 순간 실패로 깨웠다.
///
/// #4099 는 PR #4499 가 **머지된 게 아니라 CLOSED 되고** 메인테이너가 다른 SHA 로
/// 재착지시켰다(devel `e6a01730d` 기준). 잠들기 전 측정은 `origin/task4099 = e34e6d8b1`
/// 에 대한 것이었으므로 여기 값은 **착지본 기준 재측정**이다.
#[test]
fn the_edit_survives_conversion_to_hwp5() {
    for name in [
        "세로막대형/묶은세로막대형",
        "원형/쪼개진원형",
        "분산형/직선이있는분산형",
    ] {
        let path = manifest(&format!("samples/chart/{name}.hwpx"));

        let original: String = {
            let core = core_of(&path);
            let json: serde_json::Value =
                serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("읽기"))
                    .expect("JSON");
            json["series"][0]["values"][0]
                .as_str()
                .expect("값")
                .to_string()
        };
        assert_ne!(
            original, "91.7",
            "{name}: 센티널이 원본과 같으면 판정이 공허하다"
        );

        // ── 본 시험 — ①② 함께 기록 → 변환 → 새 값이 남는다
        let mut core = core_of(&path);
        let mut edits = edits_from(&core, 0);
        edits["series"][0]["values"][0] = serde_json::json!("91.7");
        assert_eq!(set_chart(&mut core, &edits)["ok"], true, "{name}");

        let hwp = core
            .export_hwp_with_adapter_snapshot()
            .expect("HWP5 변환 저장");
        let converted = DocumentCore::from_bytes(&hwp).expect("변환본 재파스");
        let json: serde_json::Value =
            serde_json::from_str(&converted.get_chart_data_by_index_native(0).expect("읽기"))
                .expect("JSON");
        assert_eq!(
            json["series"][0]["values"][0], "91.7",
            "{name}: 변환 후 새 값이 사라졌다"
        );

        // ── 대조군 — ①만 고치면 변환 후 옛 값이다
        let mut only_zip = core_of(&path);
        {
            let chart = collect_charts(only_zip.document())[0].clone();
            let zip_idx = chart.zip_part.expect("① 은 HWPX 에만 있다");
            let xml = only_zip.document().bin_data_content[zip_idx].data.load();
            let scan = scan_chart_values(&xml).expect("스캔");
            let patched = apply_value_edits(
                &xml,
                &scan,
                &[ValueEdit {
                    series: 0,
                    point: 0,
                    target: EditTarget::Value,
                    text: "91.7".to_string(),
                }],
            )
            .expect("패치");
            only_zip.document_mut().bin_data_content[zip_idx].data = patched.into();
        }
        let hwp = only_zip
            .export_hwp_with_adapter_snapshot()
            .expect("HWP5 변환 저장");
        let converted = DocumentCore::from_bytes(&hwp).expect("변환본 재파스");
        let json: serde_json::Value =
            serde_json::from_str(&converted.get_chart_data_by_index_native(0).expect("읽기"))
                .expect("JSON");
        assert_eq!(
            json["series"][0]["values"][0], original,
            "{name}: 대조군이 새 값이면 ①만 써도 된다는 뜻이라 설계 전제가 무너진다"
        );
    }
}

/// 검증에 걸리면 **한 칸도 쓰지 않는다** — `invalid[]` + `wrote: []`.
#[test]
fn every_refusal_writes_nothing() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let good = edits_from(&core_of(&path), 0);

    let mut cases: Vec<(&str, serde_json::Value)> = Vec::new();

    let mut e = good.clone();
    e["series"].as_array_mut().expect("series").pop();
    cases.push(("seriesCountMismatch", e));

    let mut e = good.clone();
    e["series"][0]["values"]
        .as_array_mut()
        .expect("values")
        .pop();
    cases.push(("valueCountMismatch", e));

    let mut e = good.clone();
    e["series"][0]["values"][0] = serde_json::json!("칠십");
    cases.push(("notANumber", e));

    let mut e = good.clone();
    e["series"][0]["name"] = serde_json::json!("다른 이름");
    cases.push(("seriesNameMismatch", e));

    let mut e = good.clone();
    e["labels"][0] = serde_json::json!("다른 항목");
    cases.push(("categoryMismatch", e));

    for (reason, edits) in cases {
        let mut core = core_of(&path);
        let before = slot_bytes(&core);
        let out = set_chart(&mut core, &edits);

        assert_eq!(out["ok"], false, "{reason}: 거부되어야 한다 — {out}");
        let reasons: Vec<&str> = out["invalid"]
            .as_array()
            .expect("invalid")
            .iter()
            .map(|v| v["reason"].as_str().expect("reason"))
            .collect();
        assert!(reasons.contains(&reason), "{reason} 가 없다: {reasons:?}");
        assert!(
            out["wrote"].as_array().expect("wrote").is_empty(),
            "{reason}"
        );
        assert_eq!(
            slot_bytes(&core),
            before,
            "{reason}: 거부했는데 바이트가 바뀌었다"
        );
    }
}

/// [#4603 리뷰] 비순차 `c:pt idx` 는 읽기·쓰기 모두 fail-closed — 한 칸도 쓰지 않는다.
///
/// 코퍼스는 전건 순차라(계획서 §2 실측 "비순차 0") 실문서 ① 슬롯에 합성 XML 을
/// 주입해 경계를 밟는다. 라벨 idx 0,1,2 + 값 idx 0·2 — 위치 기반 CSV 라면
/// A=10, B=30, C=빈칸으로 **조용히 오정렬**될 형상이다(리뷰 재현 그대로).
#[test]
fn non_sequential_pt_idx_is_refused_and_writes_nothing() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let chart = collect_charts(core.document())[0].clone();
    let zip_idx = chart.zip_part.expect("① 은 HWPX 에만 있다");
    let synthetic = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:cat><c:strRef><c:strCache><c:ptCount val="3"/>"#,
        r#"<c:pt idx="0"><c:v>A</c:v></c:pt><c:pt idx="1"><c:v>B</c:v></c:pt>"#,
        r#"<c:pt idx="2"><c:v>C</c:v></c:pt></c:strCache></c:strRef></c:cat>"#,
        r#"<c:val><c:numRef><c:numCache><c:ptCount val="3"/>"#,
        r#"<c:pt idx="0"><c:v>10</c:v></c:pt><c:pt idx="2"><c:v>30</c:v></c:pt>"#,
        r#"</c:numCache></c:numRef></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    core.document_mut().bin_data_content[zip_idx].data = synthetic.as_bytes().to_vec().into();
    let before = slot_bytes(&core);

    // 읽기 — 주소는 유효하므로 Ok + 부정 봉투. CSV 는 이 봉투를 경유하므로
    // 오정렬 CSV 가 나갈 길이 없다.
    let get: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("주소 유효"))
            .expect("JSON");
    assert_eq!(get["ok"], false, "{get}");
    assert_eq!(
        get["invalid"][0]["reason"], "nonSequentialPointIndex",
        "{get}"
    );

    // 쓰기 — 같은 사유로 거부되고 아무것도 쓰지 않는다. 스캔 거부 봉투에는
    // `wrote` 키가 없으므로 null 허용으로 단언한다.
    let out = set_chart(
        &mut core,
        &serde_json::json!({"series": [{"values": ["10", "20", "30"]}]}),
    );
    assert_eq!(out["ok"], false, "{out}");
    assert_eq!(
        out["invalid"][0]["reason"], "nonSequentialPointIndex",
        "{out}"
    );
    assert!(
        out["wrote"].as_array().is_none_or(|a| a.is_empty()),
        "{out}"
    );
    assert_eq!(
        slot_bytes(&core),
        before,
        "거부했는데 슬롯 바이트가 바뀌었다"
    );
}

/// `dryRun` 은 diff 만 내고 쓰지 않는다.
#[test]
fn dry_run_reports_the_diff_without_writing() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let before = slot_bytes(&core);

    let mut edits = edits_from(&core, 0);
    edits["series"][0]["values"][0] = serde_json::json!("91.7");
    edits["dryRun"] = serde_json::json!(true);

    let out = set_chart(&mut core, &edits);
    assert_eq!(out["ok"], true);
    assert_eq!(out["changedCount"], 1);
    assert_eq!(out["dryRun"], true);
    assert!(out["wrote"].as_array().expect("wrote").is_empty());
    assert_eq!(slot_bytes(&core), before, "dry-run 이 바이트를 바꿨다");
}

/// **T8 — 선행 렌더가 캐시를 채워도, 성공한 편집 뒤 재렌더는 새 값을 그린다.**
///
/// 편집은 `bin_data_content` 의 바이트만 바꾸는데, 렌더된 차트 SVG 는
/// `page_tree_cache` 에 RawSvg 소유값으로 남는다 — 무효화가 빠지면 재렌더가
/// 캐시된 옛 그림을 그대로 돌려준다 (PR #4603 리뷰에서 실측된 회귀).
/// legacy 경로(`render_page_svg_native`)는 캐시를 타지 않으므로 반드시
/// layer 경로로 잰다.
#[test]
fn t8_rerender_after_an_edit_draws_the_new_chart() {
    let hwpx = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    for path in [hwpx.with_extension("hwpx"), hwpx.with_extension("hwp")] {
        let mut core = core_of(&path);
        // 선행 렌더 — 페이지 0 캐시가 채워진다. 차트가 이 페이지에 실제로
        // 렌더됐는지 먼저 가드해야 "캐시 경로를 밟았다"가 성립한다.
        let svg_before = core.render_page_svg_layer_native(0).expect("선행 렌더");
        assert!(
            svg_before.contains("hwp-ooxml-chart"),
            "{}: 페이지 0 에 OOXML 차트 SVG 가 없다",
            path.display()
        );

        let mut edits = edits_from(&core, 0);
        edits["series"][0]["values"][0] = serde_json::json!("91.7");
        let out = set_chart(&mut core, &edits);
        assert_eq!(out["ok"], true, "{}: {out}", path.display());
        assert_eq!(out["changedCount"], 1, "{}: {out}", path.display());

        let svg_after = core.render_page_svg_layer_native(0).expect("재렌더");
        assert_ne!(
            svg_before,
            svg_after,
            "{}: 재렌더가 캐시된 옛 차트를 그대로 돌려줬다",
            path.display()
        );

        // 새 값 반영의 오라클 — 같은 편집을 한 새 코어의 첫(냉간) 렌더와 바이트
        // 동일해야 한다. 값축 눈금은 nice_axis 로 접히므로 "91.7" 리터럴 검색은
        // 성립하지 않는다.
        let mut cold = core_of(&path);
        let mut cold_edits = edits_from(&cold, 0);
        cold_edits["series"][0]["values"][0] = serde_json::json!("91.7");
        assert_eq!(
            set_chart(&mut cold, &cold_edits)["ok"],
            true,
            "{}",
            path.display()
        );
        let svg_cold = cold.render_page_svg_layer_native(0).expect("냉간 렌더");
        assert_eq!(
            svg_after,
            svg_cold,
            "{}: 재렌더가 편집 반영 냉간 렌더와 다르다 — 무효화가 불완전하다",
            path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// Stage 5 — 실사용 문서 회귀 (코퍼스가 못 보여 준 변종)
// ---------------------------------------------------------------------------

/// 실사용 보고서 — 차트 2개, 계열 6개, **한 계열에 `c:cat` 이 없다**.
const REPORT_SAMPLE: &str = "samples/issue2006/1790387_prep_final_report.hwpx";

/// **라벨이 없는 계열이 있어도 값이 사라지지 않는다.**
///
/// `samples/chart/` 코퍼스는 전건이 계열마다 `c:cat` 을 갖고 있어 이 변종을 못 보여 준다.
/// 실사용 문서에서 첫 계열에 `c:cat` 이 없었고, 그 탓에
///
/// - 코어가 `series[0]` 의 라벨(빈 목록)을 문서 전체 라벨로 보고했고
/// - CSV 내보내기가 행 수를 라벨 수로 잡아 **6계열 × 6값 = 36칸이 통째로 사라진** CSV 를 냈다
///
/// 오류도 경고도 없었다. 죽지 않고 틀린 산출이라 눈으로 알아채기 어렵다.
#[test]
fn charts_with_partial_category_labels_keep_every_value() {
    use rhwp::document_core::queries::chart_csv::{from_csv, to_csv};

    let path = manifest(REPORT_SAMPLE);
    let core = core_of(&path);
    let charts = collect_charts(core.document());
    assert_eq!(charts.len(), 2, "이 문서는 차트 2개다");

    let read: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("읽기")).expect("JSON");
    assert_eq!(read["ok"], true);

    let labels: Vec<String> = read["labels"]
        .as_array()
        .expect("labels")
        .iter()
        .map(|v| v.as_str().expect("문자열").to_string())
        .collect();
    let series = read["series"].as_array().expect("series");
    assert_eq!(series.len(), 6, "계열 6개");
    assert!(
        !labels.is_empty(),
        "라벨을 가진 계열이 있는데 빈 목록이 나왔다 — series[0] 만 보고 있다"
    );

    let names: Vec<String> = series
        .iter()
        .map(|s| s["name"].as_str().unwrap_or_default().to_string())
        .collect();
    let values: Vec<Vec<String>> = series
        .iter()
        .map(|s| {
            s["values"]
                .as_array()
                .expect("values")
                .iter()
                .map(|v| v.as_str().expect("문자열").to_string())
                .collect()
        })
        .collect();
    assert!(values.iter().all(|v| v.len() == 6), "계열마다 값 6개");

    let csv = to_csv(&labels, &names, &values, false);
    let back = from_csv(&csv).expect("되읽기");
    assert_eq!(back.values, values, "CSV 왕복에서 값이 사라졌다");
    assert_eq!(back.names, names);
}

/// 그 문서의 무편집 CSV 왕복도 한 칸도 바꾸지 않는다.
#[test]
fn the_real_report_round_trips_without_changes() {
    let mut core = core_of(&manifest(REPORT_SAMPLE));
    let before = slot_bytes(&core);
    let edits = edits_from(&core, 0);
    let out = set_chart(&mut core, &edits);
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["changedCount"], 0, "{out}");
    assert_eq!(slot_bytes(&core), before, "무편집인데 바이트가 바뀌었다");
}

/// 그 문서에서도 편집은 ①② 에 함께 실린다.
#[test]
fn the_real_report_accepts_an_edit_in_both_representations() {
    let mut core = core_of(&manifest(REPORT_SAMPLE));
    let chart = collect_charts(core.document())[0].clone();
    let mut edits = edits_from(&core, 0);
    edits["series"][0]["values"][0] = serde_json::json!("0.999");

    let out = set_chart(&mut core, &edits);
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["changedCount"], 1, "{out}");
    assert_eq!(out["wrote"][0], "zipPart");
    assert_eq!(out["wrote"][1], "nestedCopy");

    let nested = core.document().bin_data_content[chart.nested_copy.expect("②")]
        .data
        .load();
    let ooxml = stream_of(&nested, OOXML_STREAM).expect("②");
    assert!(String::from_utf8_lossy(&ooxml).contains("<c:v>0.999</c:v>"));
}

/// 분산형은 X 도 편집 대상이다 — 계열이 X 를 공유하므로 두 칸이 함께 바뀐다.
#[test]
fn scatter_x_values_are_editable() {
    let path = manifest("samples/chart/분산형/직선이있는분산형.hwpx");
    let mut core = core_of(&path);
    let mut edits = edits_from(&core, 0);
    edits["labels"][0] = serde_json::json!("9.9");

    let out = set_chart(&mut core, &edits);
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["changedCount"], 2, "{out}");

    let json: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("읽기")).expect("JSON");
    assert_eq!(json["labels"][0], "9.9");
}

// ---------------------------------------------------------------------------
// Stage 6 — 한컴 육안 판정 번들
// ---------------------------------------------------------------------------

/// 판정 대상 7종. 폴더마다 하나씩 고르되 **첫 판정 축을 반드시 포함**한다.
///
/// #4055 스파이크는 `묶은세로막대형` 1종으로만 한컴 판정을 받았다. 원형(ofPie 포함)·
/// 분산형·주식형은 이번이 처음이라, 폴더의 알파벳 첫 파일이 아니라 **그 축을 대표하는
/// 변종**을 골랐다.
const JUDGMENT_TARGETS: &[(&str, &str)] = &[
    ("가로막대형", "묶은가로막대형"),
    // 4계열 OHLC — 계열 순서 규약(시/고/저/종)이 걸린 축. 첫 판정.
    ("기타", "시가고가저가종가"),
    ("라인", "표식이있는꺽은선형"),
    // 첫 열이 X 값인 유일한 축. 첫 판정.
    ("분산형", "직선및표식이있는분산형"),
    // 스파이크가 판정한 바로 그 문서 — 회귀 기준선.
    ("세로막대형", "묶은세로막대형"),
    // ofPie(원형대원형) — 보조 플롯이 있는 축. 첫 판정.
    ("원형", "원형대원형"),
    // c:numLit 리터럴 + 단일 계열 — 코퍼스에서 이 문서뿐이다.
    ("특이케이스", "가로막대형_하나만있을떄_단일시리즈제목"),
];

/// 눈에 띄는 센티널 — 원본 최대값의 10배.
///
/// 값을 읽지 않고 **모양만으로** 판정할 수 있어야 한다. 10배면 막대가 차트를 뚫고 솟고,
/// 원형은 그 조각이 원을 거의 다 먹는다. #4055 가 `4.3 → 91.7`(최대 5의 약 18배)로 잡은
/// 것과 같은 취지다.
fn sentinel_for(values: &[String]) -> String {
    let max = values
        .iter()
        .filter_map(|v| v.parse::<f64>().ok())
        .fold(0.0_f64, f64::max);
    let candidate = if max > 0.0 { max * 10.0 } else { 91.7 };
    format!("{candidate:.1}")
}

/// 차트 첫 계열의 값들을 읽는다.
fn first_series_values(core: &DocumentCore) -> Vec<String> {
    let read: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("읽기")).expect("JSON");
    read["series"][0]["values"]
        .as_array()
        .expect("values")
        .iter()
        .map(|v| v.as_str().expect("문자열").to_string())
        .collect()
}

/// Stage 6 — 한컴 육안 판정용 꾸러미를 만든다.
///
/// `output/` 에 파일을 쓰는 부작용이 있어 기본 실행에서 뺀다. 판정 직전에만 돌린다:
///
/// ```text
/// cargo test --profile release-test --test issue_4100_chart_data_edit -- --ignored --nocapture
/// ```
///
/// 각 산출은 내보내기 전에 **rhwp 가 다시 열 수 있는지**와 **의도한 표현에만 센티널이
/// 들어갔는지**를 스스로 확인한다 — 한컴이 못 열었을 때 그게 편집 탓인지 파일 조립 탓인지
/// 헷갈리지 않게 하기 위함이다(#4055 선례).
#[test]
#[ignore = "output/ 에 파일을 쓴다 — 한컴 판정 직전에만 실행"]
fn generate_hancom_judgment_bundle() {
    let out_dir = manifest("output/issue_4100_b1_judgment");
    std::fs::create_dir_all(&out_dir).expect("출력 디렉터리");

    let mut sheet = String::new();
    sheet.push_str("# #4100 B1 — 한컴 육안 판정표\n\n");
    sheet.push_str(
        "차트의 **첫 계열 첫 값**을 원본 최대값의 **10배**로 바꾼 편집본입니다.\n\
         반영되면 그 막대/조각/점이 차트를 압도하므로 **숫자를 읽을 필요가 없습니다.**\n\n",
    );
    sheet.push_str("## 보는 법\n\n");
    sheet.push_str(
        "1. `*-대조군.hwpx` 로 정상 모습을 눈에 익힙니다.\n\
         2. `*-편집.hwpx` / `*-편집.hwp` 를 엽니다. 첫 값이 압도적으로 커야 합니다.\n\
         3. 파일마다 **세 가지**를 함께 봐 주세요:\n   \
         (a) 열 때 오류·복구 대화상자가 뜨는가\n   \
         (b) 차트가 그려지는가 (틀만 나오고 속이 비면 실패입니다)\n   \
         (c) **차트를 더블클릭하면 편집기가 열리는가**\n\n\
         (b) 는 루트 CLSID 축입니다 — 재포장이 그것을 떨구면 한컴이 개체를 못 알아보고 \
         틀만 그립니다(#4097).\n\n",
    );
    sheet.push_str(
        "## 첫 판정 축\n\n\
         #4055 스파이크는 `묶은세로막대형` **1종**으로만 판정했습니다. \
         **원형(ofPie)·분산형·주식형·특이케이스는 이번이 처음**이라 특히 봐 주세요.\n\n",
    );
    sheet.push_str("## 산출물\n\n");
    sheet.push_str("| 파일 | 종류 | 원본 첫 값 | 편집 후 | 쓴 표현 |\n");
    sheet.push_str("|---|---|---|---|---|\n");

    let mut written = 0usize;
    for (folder, stem) in JUDGMENT_TARGETS {
        let hwpx_src = manifest(&format!("samples/chart/{folder}/{stem}.hwpx"));
        let hwp_src = manifest(&format!("samples/chart/{folder}/{stem}.hwp"));

        // 센티널은 두 포맷이 같아야 대조가 선다 (①==② 이므로 값도 같다).
        let values = first_series_values(&core_of(&hwpx_src));
        let before = values[0].clone();
        let sentinel = sentinel_for(&values);
        assert_ne!(
            before, sentinel,
            "{stem}: 센티널이 원본과 같으면 판정이 공허하다"
        );

        // 대조군 — 원본 그대로.
        std::fs::copy(&hwpx_src, out_dir.join(format!("{stem}-대조군.hwpx"))).expect("대조군 복사");
        written += 1;

        for (src, ext, expect_zip) in [(&hwpx_src, "hwpx", true), (&hwp_src, "hwp", false)] {
            let mut core = core_of(src);
            let mut edits = edits_from(&core, 0);
            edits["series"][0]["values"][0] = serde_json::json!(sentinel);
            let result = set_chart(&mut core, &edits);
            assert_eq!(result["ok"], true, "{stem}.{ext}: {result}");
            assert_eq!(result["changedCount"], 1, "{stem}.{ext}: {result}");

            let wrote: Vec<String> = result["wrote"]
                .as_array()
                .expect("wrote")
                .iter()
                .map(|v| v.as_str().expect("문자열").to_string())
                .collect();
            let expected_wrote: Vec<&str> = if expect_zip {
                vec!["zipPart", "nestedCopy"]
            } else {
                vec!["nestedCopy"]
            };
            assert_eq!(wrote, expected_wrote, "{stem}.{ext}");

            let bytes = if expect_zip {
                core.export_hwpx_native().expect("HWPX 저장")
            } else {
                core.export_hwp_native().expect("HWP5 저장")
            };

            // 자기 검증 — rhwp 가 다시 열고, 새 값이 실제로 들어 있다.
            let reread = DocumentCore::from_bytes(&bytes)
                .unwrap_or_else(|e| panic!("{stem}.{ext}: rhwp 가 다시 열지 못한다 — {e:?}"));
            assert_eq!(
                first_series_values(&reread)[0],
                sentinel,
                "{stem}.{ext}: 저장본에 새 값이 없다"
            );

            let name = format!("{stem}-편집.{ext}");
            std::fs::write(out_dir.join(&name), &bytes).expect("산출 쓰기");
            written += 1;

            sheet.push_str(&format!(
                "| `{name}` | {folder} | {before} | **{sentinel}** | {} |\n",
                wrote.join("+")
            ));
        }
        sheet.push_str(&format!(
            "| `{stem}-대조군.hwpx` | {folder} | {before} | (무편집) | — |\n"
        ));
    }

    // 변환 축 — HWPX 를 편집한 뒤 HWP5 로 변환한다. #4099 착지 전에는 만들 수 없었다
    // (변환이 차트를 통째로 잃었다). ①은 변환에서 사라지므로 이 파일이 보여 주는 값은
    // 곧 ②다 — "①만 고치면 안 된다"의 산 증거다.
    {
        let src = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
        let mut core = core_of(&src);
        let sentinel = sentinel_for(&first_series_values(&core));

        let mut edits = edits_from(&core, 0);
        edits["series"][0]["values"][0] = serde_json::json!(sentinel);
        assert_eq!(set_chart(&mut core, &edits)["ok"], true);

        let bytes = core
            .export_hwp_with_adapter_snapshot()
            .expect("HWP5 변환 저장");
        let reread = DocumentCore::from_bytes(&bytes).expect("변환본 재파스");
        assert_eq!(
            first_series_values(&reread)[0],
            sentinel,
            "변환본에 새 값이 없다"
        );

        let name = "묶은세로막대형-편집-HWPX에서변환.hwp";
        std::fs::write(out_dir.join(name), &bytes).expect("변환본 쓰기");
        written += 1;
        sheet.push_str(&format!(
            "| `{name}` | 세로막대형 | — | **{sentinel}** | 변환 후 ② |\n"
        ));
    }

    sheet.push_str(&format!("\n총 {written} 파일.\n\n"));
    sheet.push_str(
        "> **`가로막대형_하나만있을떄_단일시리즈제목` 만 보는 법이 다릅니다.**\n\
         > 값이 **하나뿐**이라 축이 그 값에 맞춰 다시 잡힙니다 — 막대 길이는 그대로이고 \
         **축 눈금 숫자가 바뀝니다**(`0~5` → `0~45`). 이 파일만 축 숫자를 읽어 주세요.\n\n",
    );
    sheet.push_str(
        "## PDF 회신\n\n\
         각 파일을 한컴에서 열어 **같은 폴더에 PDF 로 저장**해 주시면, 대조군과 편집본의 \
         렌더를 144DPI 해시로 갈라 반영 여부를 데이터로 판정하겠습니다(#4055 와 같은 절차).\n\n\
         상세 설계는 `mydocs/plans/task_m100_4100.md`.\n",
    );

    std::fs::write(out_dir.join("PANJEONG.md"), sheet).expect("판정표 쓰기");
    println!("\n  판정 번들: {}", out_dir.display());
    println!("  파일 {written}개 + 판정표 PANJEONG.md");
    assert_eq!(written, 22, "7종 × 3 + 변환본 1");
}

// ---------------------------------------------------------------------------
// Stage 7 — B2 구조 변종 스파이크 (#5447)
// ---------------------------------------------------------------------------
//
// B1 은 개수를 바꾸지 않아 `set_chart_data_native` 로 변종을 만들 수 있었지만,
// B2(행·열·라벨)는 c:pt/c:ser 를 넣고 빼므로 그 경로의 fail-closed 검증
// (seriesCountMismatch·valueCountMismatch·categoryMismatch)을 지나갈 수 없다.
// 그래서 스파이크는 코퍼스 XML 을 **바이트 문자열 수술**로 가공해
// `replace_chart_representations`(HWPX ①②) / `replace_chart_nested_only`(HWP5 ②)로
// 직접 주입한다. 코퍼스 XML 은 한컴이 단일 라인으로 기계 생성한 균일 구조라(28종
// 실측) 문자열 수술이 안전하고, 산출마다 스캔 게이트 재통과를 자가확인한다.
//
// 수술은 #5447 의 정책 3종을 그대로 구현한다 — `c:f` 무갱신(원본 바이트 그대로) /
// `c:ptCount` 항상 재계산 / `c:pt idx` 0..n-1 전수 재번호. ③(레거시 Contents)와
// ④(프리뷰)도 B1 과 같은 불변 유지다.

/// `open`…`close` 블록들의 (시작, 닫는 태그 끝) 오프셋. 중첩 없음 전제(코퍼스 실측).
fn b2_block_ranges(xml: &str, open: &str, close: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(s) = xml[from..].find(open).map(|o| from + o) {
        let e = xml[s..]
            .find(close)
            .map(|o| s + o + close.len())
            .unwrap_or_else(|| panic!("{open} 블록이 닫히지 않는다"));
        out.push((s, e));
        from = e;
    }
    out
}

/// 블록 안 유일한 `<c:ptCount val="…"/>` 를 `delta` 만큼 조정한다 (#5447 §3-2).
fn b2_bump_pt_count(block: &str, delta: i64) -> String {
    const PAT: &str = "<c:ptCount val=\"";
    let s = block.find(PAT).expect("ptCount") + PAT.len();
    let e = s + block[s..].find('"').expect("ptCount 닫는 따옴표");
    assert!(
        !block[e..].contains(PAT),
        "ptCount 가 블록에 두 번 있다 — 래퍼 경계가 틀렸다"
    );
    let n: i64 = block[s..e].parse().expect("ptCount 수치");
    format!("{}{}{}", &block[..s], n + delta, &block[e..])
}

/// 블록 안 모든 `<c:pt idx="…">` 를 등장 순서대로 0..n-1 로 다시 매긴다 (#5447 §3-3).
fn b2_renumber_pt_idx(block: &str) -> String {
    const PAT: &str = "<c:pt idx=\"";
    let mut out = String::with_capacity(block.len());
    let mut rest = block;
    let mut next = 0usize;
    while let Some(p) = rest.find(PAT) {
        out.push_str(&rest[..p + PAT.len()]);
        rest = &rest[p + PAT.len()..];
        let q = rest.find('"').expect("idx 닫는 따옴표");
        out.push_str(&next.to_string());
        next += 1;
        rest = &rest[q..];
    }
    out.push_str(rest);
    out
}

/// 캐시 래퍼(`c:cat`/`c:val`/`c:xVal`/`c:yVal`) 블록마다 점 1개를 끝에 추가한다.
///
/// `texts[i]` 는 i번째 블록(문서 순서 = 계열 순서)의 새 텍스트. `ptCount` 는 +1 로
/// 재계산하고 새 점의 idx 는 기존 개수를 이어받는다. `c:f` 는 손대지 않는다(§3-1).
fn b2_add_point(xml: &str, tag: &str, texts: &[&str]) -> String {
    let ranges = b2_block_ranges(xml, &format!("<{tag}>"), &format!("</{tag}>"));
    assert_eq!(ranges.len(), texts.len(), "{tag}: 블록 수 ≠ 텍스트 수");
    let mut out = xml.to_string();
    for i in (0..ranges.len()).rev() {
        let (s, e) = ranges[i];
        let block = &out[s..e];
        const PAT: &str = "<c:ptCount val=\"";
        let ps = block.find(PAT).expect("ptCount") + PAT.len();
        let pe = ps + block[ps..].find('"').expect("따옴표");
        let count: usize = block[ps..pe].parse().expect("ptCount 수치");

        let bumped = b2_bump_pt_count(block, 1);
        let anchor = [
            "</c:strCache>",
            "</c:numCache>",
            "</c:numLit>",
            "</c:strLit>",
        ]
        .iter()
        .find_map(|t| bumped.find(t))
        .expect("캐시 닫는 태그");
        let point = format!("<c:pt idx=\"{count}\"><c:v>{}</c:v></c:pt>", texts[i]);
        let block_new = format!("{}{}{}", &bumped[..anchor], point, &bumped[anchor..]);
        out.replace_range(s..e, &block_new);
    }
    out
}

/// 캐시 래퍼 블록마다 idx == `remove` 점을 지우고 0..n-1 재번호 + `ptCount` -1.
fn b2_remove_point(xml: &str, tag: &str, remove: usize) -> String {
    let ranges = b2_block_ranges(xml, &format!("<{tag}>"), &format!("</{tag}>"));
    assert!(!ranges.is_empty(), "{tag}: 블록 없음");
    let mut out = xml.to_string();
    for &(s, e) in ranges.iter().rev() {
        let block = &out[s..e];
        let pt_open = format!("<c:pt idx=\"{remove}\">");
        let ps = block
            .find(&pt_open)
            .unwrap_or_else(|| panic!("{tag}: idx {remove} 점이 없다"));
        let pe = ps + block[ps..].find("</c:pt>").expect("점 닫는 태그") + "</c:pt>".len();
        let removed = format!("{}{}", &block[..ps], &block[pe..]);
        let block_new = b2_bump_pt_count(&b2_renumber_pt_idx(&removed), -1);
        out.replace_range(s..e, &block_new);
    }
    out
}

/// `<c:ser>` 블록 범위 목록.
fn b2_ser_ranges(xml: &str) -> Vec<(usize, usize)> {
    b2_block_ranges(xml, "<c:ser>", "</c:ser>")
}

/// 블록 안 첫 `<c:v>…</c:v>` 텍스트를 바꾼다.
fn b2_replace_first_v(block: &str, new_text: &str) -> String {
    let s = block.find("<c:v>").expect("<c:v>") + "<c:v>".len();
    let e = s + block[s..].find("</c:v>").expect("</c:v>");
    format!("{}{}{}", &block[..s], new_text, &block[e..])
}

/// n번째 계열의 이름(`c:tx` 캐시 텍스트)을 바꾼다. `c:f` 는 그대로다.
fn b2_rename_series(xml: &str, nth: usize, new_name: &str) -> String {
    let (s, e) = b2_ser_ranges(xml)[nth];
    let block = &xml[s..e];
    let (ts, te) = b2_block_ranges(block, "<c:tx>", "</c:tx>")[0];
    let tx_new = b2_replace_first_v(&block[ts..te], new_name);
    let mut out = xml.to_string();
    out.replace_range(s + ts..s + te, &tx_new);
    out
}

/// 모든 계열의 카테고리 라벨(`c:cat`) idx 위치 텍스트를 같은 값으로 바꾼다 —
/// 스캐너의 sharedCategoryRequired 를 지키려면 전 계열 동기 수정이 필수다.
fn b2_relabel_category(xml: &str, idx: usize, new_label: &str) -> String {
    let ranges = b2_block_ranges(xml, "<c:cat>", "</c:cat>");
    assert!(!ranges.is_empty(), "c:cat 블록 없음");
    let mut out = xml.to_string();
    for &(s, e) in ranges.iter().rev() {
        let block = &out[s..e];
        let pt_open = format!("<c:pt idx=\"{idx}\">");
        let ps = block.find(&pt_open).expect("라벨 점");
        let pe = ps + block[ps..].find("</c:pt>").expect("닫는 태그") + "</c:pt>".len();
        let pt_new = b2_replace_first_v(&block[ps..pe], new_label);
        out.replace_range(s + ps..s + pe, &pt_new);
    }
    out
}

/// 마지막 계열을 복제해 뒤에 붙인다 — `c:idx`/`c:order` 채번, 이름·값 교체.
///
/// 복제된 `c:f` 참조는 **일부러 원본 그대로** 둔다 — 같은 열을 두 계열이 가리키는
/// 낡은 범위를 한컴이 어떻게 다루는지가 §3-1 실험의 일부다.
fn b2_clone_last_series(xml: &str, new_name: &str, new_values: &[&str]) -> String {
    let ranges = b2_ser_ranges(xml);
    let n = ranges.len();
    let (s, e) = *ranges.last().expect("계열");
    let mut clone = xml[s..e].to_string();

    for tag in ["<c:idx val=\"", "<c:order val=\""] {
        let ps = clone.find(tag).expect("idx/order") + tag.len();
        let pe = ps + clone[ps..].find('"').expect("따옴표");
        clone.replace_range(ps..pe, &n.to_string());
    }

    let (ts, te) = b2_block_ranges(&clone, "<c:tx>", "</c:tx>")[0];
    let tx_new = b2_replace_first_v(&clone[ts..te], new_name);
    clone.replace_range(ts..te, &tx_new);

    let (vs, ve) = b2_block_ranges(&clone, "<c:val>", "</c:val>")[0];
    let mut val_block = clone[vs..ve].to_string();
    assert_eq!(
        val_block.matches("<c:pt idx=").count(),
        new_values.len(),
        "값 점 수 ≠ 새 값 수"
    );
    for (i, text) in new_values.iter().enumerate() {
        let pt_open = format!("<c:pt idx=\"{i}\">");
        let ps = val_block.find(&pt_open).expect("값 점");
        let pe = ps + val_block[ps..].find("</c:pt>").expect("닫는 태그") + "</c:pt>".len();
        let pt_new = b2_replace_first_v(&val_block[ps..pe], text);
        val_block.replace_range(ps..pe, &pt_new);
    }
    clone.replace_range(vs..ve, &val_block);

    let mut out = xml.to_string();
    out.insert_str(e, &clone);
    out
}

/// n번째 계열을 지우고 잔여 `c:idx`/`c:order` 를 0..n-1 로 재번호한다.
fn b2_remove_series(xml: &str, nth: usize) -> String {
    let (s, e) = b2_ser_ranges(xml)[nth];
    let mut out = xml.to_string();
    out.replace_range(s..e, "");
    let remaining = b2_ser_ranges(&out);
    for i in (0..remaining.len()).rev() {
        let (rs, re) = remaining[i];
        let mut block = out[rs..re].to_string();
        for tag in ["<c:idx val=\"", "<c:order val=\""] {
            let ps = block.find(tag).expect("idx/order") + tag.len();
            let pe = ps + block[ps..].find('"').expect("따옴표");
            block.replace_range(ps..pe, &i.to_string());
        }
        out.replace_range(rs..re, &block);
    }
    out
}

/// HWP5 문서(①없음)의 ②만 교체한다 — `replace_chart_representations` 의 HWP5 판.
fn replace_chart_nested_only(core: &mut DocumentCore, nested_xml: &[u8]) {
    let chart = collect_charts(core.document())[0].clone();
    assert!(chart.zip_part.is_none(), "HWP5 전용 경로다");
    let nested_idx = chart.nested_copy.expect("②");
    let nested_original = core.document().bin_data_content[nested_idx].data.load();
    let nested_new =
        replace_ole_stream(&nested_original, OOXML_STREAM, nested_xml).expect("② 교체");
    core.document_mut().bin_data_content[nested_idx].data = nested_new.into();
}

fn b2_envelope(core: &DocumentCore) -> serde_json::Value {
    serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("읽기")).expect("JSON")
}

fn b2_series_names(env: &serde_json::Value) -> Vec<String> {
    env["series"]
        .as_array()
        .expect("series")
        .iter()
        .map(|s| s["name"].as_str().expect("name").to_string())
        .collect()
}

fn b2_labels(env: &serde_json::Value) -> Vec<String> {
    env["labels"]
        .as_array()
        .expect("labels")
        .iter()
        .map(|v| v.as_str().expect("라벨").to_string())
        .collect()
}

fn b2_values(env: &serde_json::Value, si: usize) -> Vec<String> {
    env["series"][si]["values"]
        .as_array()
        .expect("values")
        .iter()
        .map(|v| v.as_str().expect("값").to_string())
        .collect()
}

/// 막대·라인·3D 공통(3계열 × 4카테고리) 행추가 — 새 그룹 「추가항목」에 45/44/43.
///
/// 원본 최대값이 5 라 축을 뚫고 솟는다(#4055 센티널 철학 — 값을 읽지 않고 모양으로 판정).
fn b2_bar_like_row_add(xml: &str) -> String {
    let with_cat = b2_add_point(xml, "c:cat", &["추가항목", "추가항목", "추가항목"]);
    b2_add_point(&with_cat, "c:val", &["45", "44", "43"])
}

fn b2_check_row_add(env: &serde_json::Value, ctx: &str) {
    let labels = b2_labels(env);
    assert_eq!(labels.len(), 5, "{ctx}: 라벨 수");
    assert_eq!(labels.last().map(String::as_str), Some("추가항목"), "{ctx}");
    let values = b2_values(env, 0);
    assert_eq!(values.len(), 5, "{ctx}: 값 수");
    assert_eq!(values.last().map(String::as_str), Some("45"), "{ctx}");
}

/// **B2 수술 왕복** — 행추가·행삭제가 스캔 게이트(비순차 idx·표현 일치)를 지나
/// 저장·재개방되고, 렌더에 실제로 반영된다. #5447 스파이크 하네스의 자기 회귀다.
#[test]
fn b2_category_row_surgery_roundtrips_and_renders() {
    let src = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let src_bytes = std::fs::read(&src).expect("샘플");
    let (_legacy, ooxml) = chart_streams(&src_bytes).expect("차트 스트림");
    let xml = String::from_utf8(ooxml).expect("UTF-8");

    // 행추가 — 라벨·값이 늘고, 새 카테고리 라벨이 실제로 그려진다.
    let added = b2_bar_like_row_add(&xml);
    let mut core = core_of(&src);
    replace_chart_representations(&mut core, added.as_bytes(), added.as_bytes());
    let reread =
        DocumentCore::from_bytes(&core.export_hwpx_native().expect("저장")).expect("재개방");
    let env = b2_envelope(&reread);
    assert_eq!(env["ok"], true, "{env}");
    b2_check_row_add(&env, "행추가");
    let svg = reread.render_page_svg_layer_native(0).expect("렌더");
    assert!(svg.contains("추가항목"), "행추가: 새 라벨이 렌더에 없다");

    // 행삭제 — idx 재번호·ptCount 재계산을 거쳐 스캐너를 통과하고, 지운 라벨이
    // 렌더에서 사라진다.
    let removed = b2_remove_point(&b2_remove_point(&xml, "c:cat", 1), "c:val", 1);
    let mut core = core_of(&src);
    replace_chart_representations(&mut core, removed.as_bytes(), removed.as_bytes());
    let reread =
        DocumentCore::from_bytes(&core.export_hwpx_native().expect("저장")).expect("재개방");
    let env = b2_envelope(&reread);
    assert_eq!(env["ok"], true, "{env}");
    assert_eq!(b2_labels(&env), ["항목 1", "항목 3", "항목 4"]);
    assert_eq!(b2_values(&env, 0), ["4.3", "3.5", "4.5"]);
    let svg = reread.render_page_svg_layer_native(0).expect("렌더");
    assert!(
        !svg.contains("항목 2"),
        "행삭제: 지운 라벨이 여전히 그려진다"
    );
    assert!(svg.contains("항목 3"), "행삭제: 남은 라벨이 렌더에 없다");
}

/// **B2 계열 수술** — 추가는 `c:idx`/`c:order` 채번, 삭제는 전수 재번호를 거쳐
/// 양 포맷에서 재개방된다.
#[test]
fn b2_series_surgery_renumbers_and_reopens() {
    let base = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    for path in [base.with_extension("hwpx"), base.with_extension("hwp")] {
        let bytes = std::fs::read(&path).expect("샘플");
        let (_legacy, ooxml) = chart_streams(&bytes).expect("차트 스트림");
        let xml = String::from_utf8(ooxml).expect("UTF-8");
        let is_hwpx = path.extension().is_some_and(|e| e == "hwpx");

        let cloned = b2_clone_last_series(&xml, "추가계열", &["6", "6", "6", "6"]);
        assert!(cloned.contains(r#"<c:idx val="3"/>"#), "채번이 안 됐다");
        let shrunk = b2_remove_series(&xml, 1);
        assert!(!shrunk.contains(r#"<c:idx val="2"/>"#), "재번호가 안 됐다");

        for (surgery, expect_names) in [
            (cloned, vec!["계열 1", "계열 2", "계열 3", "추가계열"]),
            (shrunk, vec!["계열 1", "계열 3"]),
        ] {
            let mut core = core_of(&path);
            let out = if is_hwpx {
                replace_chart_representations(&mut core, surgery.as_bytes(), surgery.as_bytes());
                core.export_hwpx_native().expect("저장")
            } else {
                replace_chart_nested_only(&mut core, surgery.as_bytes());
                core.export_hwp_native().expect("저장")
            };
            let env = b2_envelope(&DocumentCore::from_bytes(&out).expect("재개방"));
            assert_eq!(env["ok"], true, "{}: {env}", path.display());
            assert_eq!(b2_series_names(&env), expect_names, "{}", path.display());
        }
    }
}

/// B2 판정 변종 1건 — 어느 문서에 어떤 수술을 하고, 재독에서 무엇을 확인하는가.
struct B2Variant {
    folder: &'static str,
    stem: &'static str,
    /// 파일명 접미 (예: 행추가 → `묶은세로막대형-행추가.hwpx`).
    label: &'static str,
    what: &'static str,
    expect_shape: &'static str,
    /// 일부러 낡게 남긴 것 — 판정표에 기록해 한컴 반응의 원인을 가릴 수 있게 한다.
    stale: &'static str,
    surgery: Box<dyn Fn(&str) -> String>,
    check: Box<dyn Fn(&serde_json::Value, &str)>,
}

/// #5447 §5 변종 카탈로그 — 본선 6종(기준 문서) + 경계 2종 + 종류 커버리지 6종.
fn b2_variants() -> Vec<B2Variant> {
    let row_del_check = |env: &serde_json::Value, ctx: &str| {
        assert_eq!(b2_labels(env), ["항목 1", "항목 3", "항목 4"], "{ctx}");
        assert_eq!(b2_values(env, 0), ["4.3", "3.5", "4.5"], "{ctx}");
    };
    vec![
        // -- 본선: 기준 문서 묶은세로막대형 --------------------------------
        B2Variant {
            folder: "세로막대형",
            stem: "묶은세로막대형",
            label: "행추가",
            what: "카테고리 1행 추가 — 전 계열 cat/val 에 c:pt + ptCount 재계산, c:f 그대로",
            expect_shape: "그룹 4→5, 「추가항목」 그룹 막대 3개(45/44/43)가 축을 뚫고 솟음",
            stale: "c:f(4행 범위)·③·④",
            surgery: Box::new(b2_bar_like_row_add),
            check: Box::new(b2_check_row_add),
        },
        B2Variant {
            folder: "세로막대형",
            stem: "묶은세로막대형",
            label: "행삭제",
            what: "카테고리 「항목 2」 삭제 — c:pt 제거 + idx 재번호 + ptCount 재계산",
            expect_shape: "그룹 4→3, 「항목 2」 가 사라짐",
            stale: "c:f(4행 범위)·③·④",
            surgery: Box::new(|xml| b2_remove_point(&b2_remove_point(xml, "c:cat", 1), "c:val", 1)),
            check: Box::new(row_del_check),
        },
        B2Variant {
            folder: "세로막대형",
            stem: "묶은세로막대형",
            label: "계열추가",
            what: "계열 1개 신설 — c:ser 복제 + c:idx/c:order 채번, 이름·값 교체",
            expect_shape: "색 3→4, 새 계열 「추가계열」 이 전 그룹에서 같은 높이(6)",
            stale: "복제 계열의 c:f(원본 열 참조 그대로)·③·④",
            surgery: Box::new(|xml| b2_clone_last_series(xml, "추가계열", &["6", "6", "6", "6"])),
            check: Box::new(|env, ctx| {
                assert_eq!(
                    b2_series_names(env),
                    ["계열 1", "계열 2", "계열 3", "추가계열"],
                    "{ctx}"
                );
                assert_eq!(b2_values(env, 3), ["6", "6", "6", "6"], "{ctx}");
            }),
        },
        B2Variant {
            folder: "세로막대형",
            stem: "묶은세로막대형",
            label: "계열삭제",
            what: "계열 「계열 2」 삭제 + 잔여 c:idx/c:order 재번호",
            expect_shape: "색 3→2, 두 번째 색 막대가 사라짐",
            stale: "잔여 c:f·③·④",
            surgery: Box::new(|xml| b2_remove_series(xml, 1)),
            check: Box::new(|env, ctx| {
                assert_eq!(b2_series_names(env), ["계열 1", "계열 3"], "{ctx}");
            }),
        },
        B2Variant {
            folder: "세로막대형",
            stem: "묶은세로막대형",
            label: "계열명변경",
            what: "계열명 변경 — c:tx 캐시(strCache c:v)만 교체",
            expect_shape: "막대는 그대로, 범례 첫 항목이 「이름바뀐계열」 (글자를 읽어 주세요)",
            stale: "c:f(이름 참조 $B$1)·③·④",
            surgery: Box::new(|xml| b2_rename_series(xml, 0, "이름바뀐계열")),
            check: Box::new(|env, ctx| {
                assert_eq!(
                    b2_series_names(env),
                    ["이름바뀐계열", "계열 2", "계열 3"],
                    "{ctx}"
                );
            }),
        },
        B2Variant {
            folder: "세로막대형",
            stem: "묶은세로막대형",
            label: "라벨변경",
            what: "카테고리 라벨 변경 — 전 계열 c:cat 캐시 동기 교체",
            expect_shape: "막대는 그대로, 첫 축 라벨이 「바뀐항목」 (글자를 읽어 주세요)",
            stale: "c:f(라벨 참조 $A$2:$A$5)·③·④",
            surgery: Box::new(|xml| b2_relabel_category(xml, 0, "바뀐항목")),
            check: Box::new(|env, ctx| {
                assert_eq!(b2_labels(env)[0], "바뀐항목", "{ctx}");
            }),
        },
        // -- 경계: B2 가 거부하려는 편집의 실측 -----------------------------
        B2Variant {
            folder: "원형",
            stem: "원형대원형",
            label: "계열추가",
            what: "(경계) 1계열 고정인 원형에 2번째 계열 신설",
            expect_shape: "한컴이 어떻게 다루는지 자체가 답 — 깨져도 그대로 기록",
            stale: "복제 계열의 c:f·③·④",
            surgery: Box::new(|xml| {
                b2_clone_last_series(xml, "추가계열", &["10", "20", "30", "40"])
            }),
            check: Box::new(|env, ctx| {
                assert_eq!(b2_series_names(env).len(), 2, "{ctx}");
            }),
        },
        B2Variant {
            folder: "기타",
            stem: "시가고가저가종가",
            label: "계열삭제",
            what: "(경계) 주식형 4계열에서 「시가」 삭제 → 3계열 + 재번호",
            expect_shape: "시가고가저가종가가 고·저·종 3계열이 되는가 — 깨져도 그대로 기록",
            stale: "잔여 c:f·③·④",
            surgery: Box::new(|xml| b2_remove_series(xml, 0)),
            check: Box::new(|env, ctx| {
                assert_eq!(b2_series_names(env), ["고가", "저가", "종가"], "{ctx}");
            }),
        },
        // -- 종류 커버리지: 본선 처치(행추가)를 나머지 판정 대상에 ----------
        B2Variant {
            folder: "가로막대형",
            stem: "묶은가로막대형",
            label: "행추가",
            what: "카테고리 1행 추가 (본선과 동일 처치)",
            expect_shape: "그룹 4→5, 「추가항목」 그룹이 가로로 축을 뚫음",
            stale: "c:f(4행 범위)·③·④",
            surgery: Box::new(b2_bar_like_row_add),
            check: Box::new(b2_check_row_add),
        },
        B2Variant {
            folder: "라인",
            stem: "표식이있는꺽은선형",
            label: "행추가",
            what: "카테고리 1행 추가 (본선과 동일 처치)",
            expect_shape: "점 4→5, 마지막에서 세 선이 모두 급등",
            stale: "c:f(4행 범위)·③·④",
            surgery: Box::new(b2_bar_like_row_add),
            check: Box::new(b2_check_row_add),
        },
        B2Variant {
            folder: "분산형",
            stem: "직선및표식이있는분산형",
            label: "점추가",
            what: "점 1개 추가 — c:cat 없는 축이라 xVal/yVal 에 동기 추가",
            expect_shape: "점 3→4, 오른쪽 위 (9, 27)·(9, 10) 새 점",
            stale: "c:f(3점 범위, ser0 은 한컴이 쓴 깨진 범위 $B$2:$A$4)·③·④",
            surgery: Box::new(|xml| {
                let with_x = b2_add_point(xml, "c:xVal", &["9", "9"]);
                b2_add_point(&with_x, "c:yVal", &["27", "10"])
            }),
            check: Box::new(|env, ctx| {
                assert_eq!(b2_series_names(env).len(), 2, "{ctx}");
                let v0 = b2_values(env, 0);
                assert_eq!(v0.len(), 4, "{ctx}: 값 수");
                assert_eq!(v0.last().map(String::as_str), Some("27"), "{ctx}");
            }),
        },
        B2Variant {
            folder: "특이케이스",
            stem: "가로막대형_하나만있을떄_단일시리즈제목",
            label: "점추가",
            what: "1계열 1점(c:numLit/strLit)에 점 1개 추가 — 삭제 하한의 역방향 경계",
            expect_shape: "막대 1→2, 축이 43 에 맞춰 늘어남 (축 숫자를 읽어 주세요)",
            stale: "③·④ (numLit 이라 데이터 c:f 없음)",
            surgery: Box::new(|xml| {
                let with_cat = b2_add_point(xml, "c:cat", &["추가항목"]);
                b2_add_point(&with_cat, "c:val", &["43"])
            }),
            check: Box::new(|env, ctx| {
                assert_eq!(b2_labels(env), ["항목 1", "추가항목"], "{ctx}");
                assert_eq!(b2_values(env, 0), ["4.3", "43"], "{ctx}");
            }),
        },
        B2Variant {
            folder: "세로막대형",
            stem: "누적세로막대형",
            label: "계열삭제",
            what: "누적형에서 「계열 2」 삭제 — 누적 합이 바뀌는 축(#5447 §5)",
            expect_shape: "누적 기둥에서 가운데 색이 빠져 총높이가 낮아짐",
            stale: "잔여 c:f·③·④",
            surgery: Box::new(|xml| b2_remove_series(xml, 1)),
            check: Box::new(|env, ctx| {
                assert_eq!(b2_series_names(env), ["계열 1", "계열 3"], "{ctx}");
            }),
        },
        B2Variant {
            folder: "세로막대형",
            stem: "3차원묶은세로막대형",
            label: "행추가",
            what: "3D(c:view3D)에서 카테고리 1행 추가 (본선과 동일 처치)",
            expect_shape: "3D 그룹 4→5, 「추가항목」 그룹이 축을 뚫음",
            stale: "c:f(4행 범위)·③·④",
            surgery: Box::new(b2_bar_like_row_add),
            check: Box::new(b2_check_row_add),
        },
    ]
}

/// 변종 1건을 양 포맷으로 만들어 자기검증 후 기록한다. 반환은 쓴 파일 수(2).
fn b2_write_variant(out_dir: &std::path::Path, v: &B2Variant, sheet: &mut String) -> usize {
    let mut written = 0usize;
    for ext in ["hwpx", "hwp"] {
        let src = manifest(&format!("samples/chart/{}/{}.{ext}", v.folder, v.stem));
        let src_bytes = std::fs::read(&src).expect("샘플 읽기");
        let (_legacy, ooxml) = chart_streams(&src_bytes).expect("차트 스트림");
        let xml = String::from_utf8(ooxml).expect("UTF-8");
        let name = format!("{}-{}.{ext}", v.stem, v.label);

        let xml_new = (v.surgery)(&xml);
        assert_ne!(xml, xml_new, "{name}: 수술이 아무것도 바꾸지 않았다");
        scan_chart_values(xml_new.as_bytes())
            .unwrap_or_else(|e| panic!("{name}: 변종이 스캐너를 통과하지 못한다 — {e:?}"));

        let mut core = core_of(&src);
        // ③④ 는 손대지 않는다 — 주입 전 바이트를 기억해 뒀다가 저장본에서 대조한다.
        let chart = collect_charts(core.document())[0].clone();
        let nested_before = core.document().bin_data_content[chart.nested_copy.expect("②")]
            .data
            .load();
        let legacy_before = stream_of(&nested_before, LEGACY_STREAM).expect("③");
        let emf_before = stream_of(&nested_before, EMF_STREAM).expect("④");

        let bytes = if ext == "hwpx" {
            replace_chart_representations(&mut core, xml_new.as_bytes(), xml_new.as_bytes());
            core.export_hwpx_native().expect("HWPX 저장")
        } else {
            replace_chart_nested_only(&mut core, xml_new.as_bytes());
            core.export_hwp_native().expect("HWP5 저장")
        };

        // 자기검증 1 — rhwp 재개방 + 스캔 게이트(비순차 idx·표현 일치) 통과 + 구조 확인.
        let reread = DocumentCore::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("{name}: rhwp 가 다시 열지 못한다 — {e:?}"));
        let env = b2_envelope(&reread);
        assert_eq!(env["ok"], true, "{name}: {env}");
        (v.check)(&env, &name);

        // 자기검증 2 — 의도한 표현에만 변종이 실렸고 ③④ 는 바이트 그대로다.
        let chart_after = collect_charts(reread.document())[0].clone();
        let nested_after = reread.document().bin_data_content[chart_after.nested_copy.expect("②")]
            .data
            .load();
        assert_eq!(
            stream_of(&nested_after, OOXML_STREAM).expect("②"),
            xml_new.as_bytes(),
            "{name}: ② 불일치"
        );
        assert_eq!(
            stream_of(&nested_after, LEGACY_STREAM).expect("③"),
            legacy_before,
            "{name}: ③ 이 변했다"
        );
        assert_eq!(
            stream_of(&nested_after, EMF_STREAM).expect("④"),
            emf_before,
            "{name}: ④ 가 변했다"
        );
        if ext == "hwpx" {
            let zip_after = reread.document().bin_data_content[chart_after.zip_part.expect("①")]
                .data
                .load();
            assert_eq!(zip_after, xml_new.as_bytes(), "{name}: ① 불일치");
        }

        std::fs::write(out_dir.join(&name), &bytes).expect("산출 쓰기");
        written += 1;
        sheet.push_str(&format!(
            "| `{name}` | {} | {} | {} | {} |\n",
            v.folder, v.what, v.expect_shape, v.stale
        ));
    }
    written
}

/// Stage 7 — B2 구조 변종 한컴 판정 꾸러미(#5447 §5)를 만든다.
///
/// `output/` 에 파일을 쓰는 부작용이 있어 기본 실행에서 뺀다. 판정 직전에만 돌린다:
///
/// ```text
/// cargo test --profile release-test --test issue_4100_chart_data_edit \
///     generate_b2_structure_judgment_bundle -- --ignored --nocapture
/// ```
///
/// **한컴이 실제로 판정한 산출은 `samples/issue5447/` 에 커밋돼 있다.** 그 38건의 변환
/// PDF 는 `pdf/issue5447/`, 원장은 `samples/issue5447/MANIFEST.json` 이다. 여기서 다시
/// 만든 것과 커밋본이 바이트로 같은지는 `sha256sum` 으로 직접 대조한다 — 어긋나더라도
/// **커밋본이 정본**이다. 판정은 한컴이 연 그 바이트에 대한 관측이기 때문이다.
/// 커밋본과 원장의 정합은 `b2_judgment_assets_match_the_manifest` 가 상시로 지킨다.
#[test]
#[ignore = "output/ 에 파일을 쓴다 — 한컴 판정 직전에만 실행"]
fn generate_b2_structure_judgment_bundle() {
    let out_dir = manifest("output/issue_5447_b2_judgment");
    std::fs::create_dir_all(&out_dir).expect("출력 디렉터리");

    let mut sheet = String::new();
    sheet.push_str("# #5447 B2 스파이크 — 한컴 판정표 (구조 변종)\n\n");
    sheet.push_str(
        "행(카테고리)·열(계열)·라벨을 **구조적으로** 바꾼 변종입니다. B1 과 달리 값이\n\
         아니라 **개수와 글자**가 바뀌므로, 파일마다 「기대 모양」 칸과 대조해 주세요.\n\n\
         `c:f` 참조 범위·③레거시 Contents·④프리뷰는 **일부러 안 고쳤습니다**\n\
         (#5447 §3-1) — 한컴이 낡은 그것들을 어떻게 다루는지가 판정 대상입니다.\n\n",
    );
    sheet.push_str("## 보는 법\n\n");
    sheet.push_str(
        "1. `*-대조군.hwpx` 로 원본 모습을 눈에 익힙니다.\n\
         2. 변종 파일마다 **네 가지**를 봐 주세요:\n   \
         (a) 열 때 오류·복구 대화상자가 뜨는가\n   \
         (b) 차트가 **기대 모양대로** 그려지는가 (틀만 나오고 속이 비면 실패입니다)\n   \
         (c) 차트를 더블클릭하면 편집기가 열리는가\n   \
         (d) **편집기(데이터 편집)의 행·열 수가 기대 모양과 일치하는가**\n\n\
         (d) 가 이번 스파이크의 핵심입니다 — 예: `행추가` 파일에서 편집기가 **4행만**\n\
         보여 주면 한컴 편집기가 낡은 `c:f` 범위를 재해석해 자른 것입니다\n\
         (#5447 S2 — `c:f` 무갱신 정책이 뒤집힐 유일한 지점).\n\n",
    );
    sheet.push_str(
        "## 경계 변종 — 깨져도 그 자체가 답입니다\n\n\
         `원형대원형-계열추가`(원형에 2번째 계열)와 `시가고가저가종가-계열삭제`(4→3계열)는\n\
         B2 가 **거부하려는 편집**의 실측입니다. 오류가 나면 오류대로, 이상하게 그려지면\n\
         그 모양대로 기록해 주세요.\n\n",
    );
    sheet.push_str("## 산출물\n\n");
    sheet.push_str("| 파일 | 종류 | 무엇을 바꿨나 | 기대 모양 | 낡게 남긴 것 |\n");
    sheet.push_str("|---|---|---|---|---|\n");

    let variants = b2_variants();
    let mut written = 0usize;

    // 대조군 — 변종이 쓰는 원본 문서마다 하나씩.
    let mut controls: Vec<(&str, &str)> = Vec::new();
    for v in &variants {
        if !controls.contains(&(v.folder, v.stem)) {
            controls.push((v.folder, v.stem));
        }
    }
    for (folder, stem) in &controls {
        std::fs::copy(
            manifest(&format!("samples/chart/{folder}/{stem}.hwpx")),
            out_dir.join(format!("{stem}-대조군.hwpx")),
        )
        .expect("대조군 복사");
        written += 1;
        sheet.push_str(&format!(
            "| `{stem}-대조군.hwpx` | {folder} | (무편집 원본) | 원본 그대로 | — |\n"
        ));
    }

    for v in &variants {
        written += b2_write_variant(&out_dir, v, &mut sheet);
    }

    // 변환 축 — 행추가 변종을 HWPX 에서 만들고 HWP5 로 변환한다. ①은 변환에서
    // 접히므로 이 파일이 보여 주는 구조는 곧 ②다(B1 변환 축과 같은 취지).
    {
        let src = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
        let src_bytes = std::fs::read(&src).expect("샘플");
        let (_legacy, ooxml) = chart_streams(&src_bytes).expect("차트 스트림");
        let xml_new = b2_bar_like_row_add(&String::from_utf8(ooxml).expect("UTF-8"));

        let mut core = core_of(&src);
        replace_chart_representations(&mut core, xml_new.as_bytes(), xml_new.as_bytes());
        let bytes = core
            .export_hwp_with_adapter_snapshot()
            .expect("HWP5 변환 저장");
        let reread = DocumentCore::from_bytes(&bytes).expect("변환본 재파스");
        let env = b2_envelope(&reread);
        assert_eq!(env["ok"], true, "변환본: {env}");
        b2_check_row_add(&env, "변환본");

        let name = "묶은세로막대형-행추가-HWPX에서변환.hwp";
        std::fs::write(out_dir.join(name), &bytes).expect("변환본 쓰기");
        written += 1;
        sheet.push_str(&format!(
            "| `{name}` | 세로막대형 | 행추가 후 HWPX→HWP5 변환 (구조가 ② 로 접힘) | 그룹 4→5 | c:f·③·④ |\n"
        ));
    }

    sheet.push_str(&format!("\n총 {written} 파일.\n\n"));
    sheet.push_str(
        "## PDF 회신\n\n\
         각 파일을 한컴에서 열어 **같은 폴더에 PDF 로 저장**해 주시면, 대조군과 변종의\n\
         렌더를 144DPI 래스터 해시로 갈라 반영 여부를 데이터로 판정하겠습니다\n\
         (#4100 §4-1 선례 — PDF 스트림 해시는 오판, 반드시 래스터).\n\n\
         (d) 편집기 행·열 수는 해시로 잴 수 없으니 **파일별로 한 줄씩** 남겨 주세요.\n\n\
         상세 설계는 #5447.\n",
    );

    std::fs::write(out_dir.join("PANJEONG.md"), sheet).expect("판정표 쓰기");
    println!("\n  판정 번들: {}", out_dir.display());
    println!("  파일 {written}개 + 판정표 PANJEONG.md");
    assert_eq!(written, 38, "대조군 9 + 변종 14 × 2포맷 + 변환본 1");
}

/// Stage 8 — 판정 원장이 가리키는 자산이 지금도 그 바이트인가.
///
/// PR #5647 이 보류된 이유는 "판정을 재계산할 자산이 저장소에 없다" 였다. 자산을 커밋한
/// 뒤에는 반대 위험이 생긴다 — 원장과 자산 중 **한쪽만 조용히 늙는 것**. 그래서 원장
/// 38행이 가리키는 원본과 한컴 PDF 를 열어 SHA-256 을 다시 잰다.
///
/// 렌더러 의존성이 없어 CI 에서 상시로 돈다. 래스터 재계산과 불변식 재판정은
/// `tools/hancom_chart_judgment_verify.py` 가 로컬에서 맡는다
/// (`scripts/check_e2e_manifest.py` 와 같은 원장 트립와이어 운용).
///
/// 양방향으로 본다 — 원장 → 파일(빠진 자산)뿐 아니라 파일 → 원장(등재되지 않은 자산)도
/// 본다. 뒤쪽을 **이름이 아니라 해시 집합**으로 맞추는 것은 판정 자산이 macOS↔Windows 를
/// 오가며 파일명이 NFC/NFD 로 갈렸던 전례(#5447 보고서 §6-1) 때문이다.
#[test]
fn b2_judgment_assets_match_the_manifest() {
    fn sha256_of(path: &std::path::Path) -> String {
        use sha2::Digest as _;
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut hasher = sha2::Sha256::new();
        hasher.update(&bytes);
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    let ledger_path = manifest("samples/issue5447/MANIFEST.json");
    let ledger: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&ledger_path).expect("판정 원장 읽기"))
            .expect("판정 원장 JSON");
    let entries = ledger["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 38, "대조군 9 + 변종 14 × 2포맷 + 변환본 1");

    // 원장에 등재된 바이트 전부. 아래에서 디렉터리를 거꾸로 훑을 때 대조군이 된다.
    let mut registered: std::collections::BTreeSet<String> = Default::default();
    // 판정 단위(기준문서-변종) → 판정. 포맷 둘이 한 단위이므로 값이 갈리면 원장이 모순이다.
    let mut verdict_of_unit: std::collections::BTreeMap<String, String> = Default::default();

    for entry in entries {
        let name = entry["name"].as_str().expect("name");
        for (path_key, hash_key) in [
            ("original_path", "original_sha256"),
            ("hancom_pdf_path", "hancom_pdf_sha256"),
        ] {
            let rel = entry[path_key]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: {path_key} 가 없다"));
            let recorded = entry[hash_key]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: {hash_key} 가 없다"));
            assert_eq!(
                recorded.len(),
                64,
                "{name}: {hash_key} 가 SHA-256 이 아니다"
            );
            let path = manifest(rel);
            assert!(path.is_file(), "{name}: 원장이 가리키는 {rel} 이 없다");
            assert_eq!(
                sha256_of(&path),
                recorded,
                "{name}: {rel} 의 바이트가 원장과 다르다"
            );
            registered.insert(recorded.to_string());
        }

        if entry["role"] != "control" {
            let unit = if entry["role"] == "conversion" {
                name.rsplit_once('.')
                    .map_or(name, |(stem, _)| stem)
                    .to_string()
            } else {
                format!(
                    "{}-{}",
                    entry["base_document"].as_str().expect("base_document"),
                    entry["variant"].as_str().expect("variant")
                )
            };
            let verdict = entry["verdict"].as_str().expect("verdict").to_string();
            if let Some(seen) = verdict_of_unit.insert(unit.clone(), verdict.clone()) {
                assert_eq!(seen, verdict, "{unit}: 포맷 간 판정이 갈린다");
            }
        }
    }

    // 원장 머리의 집계가 본문에서 다시 세어도 같은가 — 보고서가 인용하는 숫자가 이것이다.
    let counts = &ledger["counts"];
    assert_eq!(
        verdict_of_unit.len() as u64,
        counts["judgment_units"].as_u64().expect("judgment_units"),
        "판정 단위 수가 원장 머리와 다르다"
    );
    let mut tally: std::collections::BTreeMap<String, u64> = Default::default();
    for verdict in verdict_of_unit.values() {
        *tally.entry(verdict.clone()).or_default() += 1;
    }
    assert_eq!(
        counts["tally"].as_object().expect("tally").len(),
        tally.len(),
        "판정 분포의 항목 수가 원장 머리와 다르다"
    );
    for (verdict, count) in &tally {
        assert_eq!(
            counts["tally"][verdict].as_u64(),
            Some(*count),
            "판정 분포 {verdict} 가 원장 머리와 다르다"
        );
    }

    for (dir, ignored) in [
        (
            "samples/issue5447",
            ["MANIFEST.json", "README.md", "PANJEONG.md"].as_slice(),
        ),
        ("pdf/issue5447", ["README.md"].as_slice()),
    ] {
        let mut counted = 0usize;
        for item in std::fs::read_dir(manifest(dir)).expect("판정 자산 디렉터리") {
            let path = item.expect("디렉터리 항목").path();
            if !path.is_file() {
                continue;
            }
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if ignored.contains(&file_name.as_str()) {
                continue;
            }
            assert!(
                registered.contains(&sha256_of(&path)),
                "{dir}/{file_name}: 원장에 등재되지 않은 자산이다"
            );
            counted += 1;
        }
        assert_eq!(counted, 38, "{dir}: 판정 자산이 38건이 아니다");
    }
}

// ---------------------------------------------------------------------------
// Stage 9 — B2 엔진 본구현 (#5652)
// ---------------------------------------------------------------------------
//
// #5447 이 손으로 만든 변종을 한컴이 통과시켰고, 이제 같은 바이트를 **엔진이** 만든다.
// S1 은 스캐너가 구조 좌표(점·계열 요소 구간, ptCount, 삽입 앵커, 계열명·idx·order 구간,
// plot 종류)를 같은 스캔에서 기록하는 것이고, 합성 계약은
// `tests/cases/ooxml_chart_structure_contract.rs` 가 고정한다. 여기서는 코퍼스 56건
// 전건에 대해 그 좌표가 실제 바이트와 정합하는지 — 특히 **선언 ptCount 가 실제 점 수와
// 같다**는 한컴 불변식(#5447 §3-2) — 를 잰다.

/// [#5652 S1] 코퍼스 전건 — 구조 좌표가 입력 바이트로 되읽힌다.
///
/// `&xml[span]` 재슬라이스는 스캐너 자기충족이라(#4100 §9-2 오라클 공모 교훈) 모양
/// 단언을 함께 둔다: 점 요소는 `<c:pt` 로 시작해 `</c:pt>` 로 끝나고, 계열 요소는
/// `<c:ser>`…`</c:ser>`, ptCount 는 십진수 텍스트이며 그 값이 점 수와 같다.
#[test]
fn structure_spans_slice_back_across_the_corpus() {
    use rhwp::ooxml_chart::data::PlotKind;

    let mut points = 0usize;
    let mut series_count = 0usize;
    for (path, xml) in corpus_charts() {
        let name = path.display();
        let text = std::str::from_utf8(&xml).expect("UTF-8");
        let data = scan_chart_values(&xml).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        for (si, series) in data.series.iter().enumerate() {
            series_count += 1;
            let ser = &text[series.element_span.clone()];
            assert!(
                ser.starts_with("<c:ser>") && ser.ends_with("</c:ser>"),
                "{name} ser{si}: {ser:.40}"
            );
            assert_eq!(series.prefix, "c:", "{name} ser{si}");
            assert_ne!(
                series.plot,
                PlotKind::Other,
                "{name} ser{si}: plot 종류를 못 읽었다"
            );
            assert_eq!(
                &text[series
                    .idx_span
                    .clone()
                    .unwrap_or_else(|| panic!("{name} ser{si}: idx 구간"))],
                si.to_string(),
                "{name} ser{si}: c:idx 가 위치와 다르다"
            );
            assert_eq!(
                &text[series
                    .order_span
                    .clone()
                    .unwrap_or_else(|| panic!("{name} ser{si}: order 구간"))],
                si.to_string(),
                "{name} ser{si}: c:order 가 위치와 다르다"
            );
            match (&series.name, &series.name_span) {
                (Some(n), Some(span)) => {
                    assert_eq!(&text[span.clone()], n, "{name} ser{si}: 이름 구간")
                }
                (None, None) => {}
                other => panic!("{name} ser{si}: 이름과 구간이 어긋난다 {other:?}"),
            }

            for (block, pts, shape) in [
                ("labels", &series.labels, &series.labels_shape),
                ("values", &series.values, &series.values_shape),
            ] {
                if pts.is_empty() && shape.is_none() {
                    continue; // c:cat 이 없는 계열(실사용 문서에 있다 — samples/issue2006)
                }
                let shape = shape
                    .as_ref()
                    .unwrap_or_else(|| panic!("{name} ser{si} {block}: 블록 좌표 없음"));
                let pt_count = shape
                    .pt_count
                    .as_ref()
                    .unwrap_or_else(|| panic!("{name} ser{si} {block}: ptCount 없음"));
                assert_eq!(
                    pt_count.value as usize,
                    pts.len(),
                    "{name} ser{si} {block}: 선언 ptCount ≠ 실제 점 수 (한컴 불변식 위반)"
                );
                assert_eq!(&text[pt_count.span.clone()], pt_count.value.to_string());
                let at = shape
                    .insert_at
                    .unwrap_or_else(|| panic!("{name} ser{si} {block}: 삽입 앵커 없음"));
                let last_end = pts
                    .last()
                    .and_then(|p| p.element_span.as_ref())
                    .map(|s| s.end);
                assert_eq!(
                    Some(at),
                    last_end,
                    "{name} ser{si} {block}: 앵커 ≠ 마지막 점 요소 끝"
                );
                assert!(
                    text[at..].starts_with("</c:"),
                    "{name} ser{si} {block}: 앵커 뒤가 캐시 닫는 태그가 아니다: {:.30}",
                    &text[at..]
                );
                for (pi, p) in pts.iter().enumerate() {
                    points += 1;
                    let element = p
                        .element_span
                        .clone()
                        .unwrap_or_else(|| panic!("{name} ser{si} {block}[{pi}]: 요소 구간"));
                    let raw = &text[element.clone()];
                    assert!(
                        raw.starts_with("<c:pt ") && raw.ends_with("</c:pt>"),
                        "{name}: {raw}"
                    );
                    if let Some(span) = &p.span {
                        assert!(element.start < span.start && span.end < element.end);
                        assert_eq!(&text[span.clone()], p.text);
                    }
                }
            }
        }
    }
    assert!(
        series_count >= 56 && points >= 56 * 4,
        "코퍼스가 비었다: {series_count} 계열 / {points} 점"
    );
}

/// [#5652 S2] 엔진 패처의 산출이 #5447 스파이크의 **문자열 수술** 산출과 바이트 동일하다.
///
/// 스파이크 수술(`b2_*`)은 스캐너와 무관한 문자열 탐색 경로라 독립 오라클이 된다(#4100 §9-2
/// 오라클 공모 교훈). 위치 기반 꼬리 증감 모델에서 "중간 행 삭제"는 뒤 행 값을 앞으로 당겨
/// 쓰고 꼬리를 지우는 것인데, 코퍼스 `c:pt` 가 `idx` 와 `c:v` 만 가진 균일 요소라 스파이크의
/// "요소 제거 + idx 재번호"와 같은 바이트가 나온다 — 계획서 §3-1 의 주장을 여기서 잰다.
#[test]
fn engine_patch_matches_the_spike_surgery_byte_for_byte() {
    use rhwp::ooxml_chart::patch::{apply_chart_edits, ChartEdit};

    fn engine(xml: &str, edits: &[ChartEdit]) -> String {
        let data = scan_chart_values(xml.as_bytes()).expect("스캔");
        String::from_utf8(apply_chart_edits(xml.as_bytes(), &data, edits).expect("엔진 패치"))
            .expect("UTF-8")
    }
    fn strs(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    let src = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let (_legacy, ooxml) = chart_streams(&std::fs::read(&src).expect("샘플")).expect("차트 스트림");
    let xml = String::from_utf8(ooxml).expect("UTF-8");
    let data = scan_chart_values(xml.as_bytes()).expect("스캔");
    assert_eq!(data.series.len(), 3);
    assert_eq!(data.series[0].values.len(), 4);

    // 행추가 — 전 계열 cat/val 꼬리에 1점.
    let mut row_add = Vec::new();
    for (s, v) in ["45", "44", "43"].iter().enumerate() {
        row_add.push(ChartEdit::AppendPoints {
            series: s,
            target: EditTarget::Label,
            texts: strs(&["추가항목"]),
        });
        row_add.push(ChartEdit::AppendPoints {
            series: s,
            target: EditTarget::Value,
            texts: strs(&[v]),
        });
    }
    assert_eq!(engine(&xml, &row_add), b2_bar_like_row_add(&xml), "행추가");

    // 행삭제(「항목 2」) — 뒤 행을 앞으로 당겨 쓰고 꼬리 1점 삭제 == 요소 제거 + idx 재번호.
    let mut row_del = Vec::new();
    for (s, series) in data.series.iter().enumerate() {
        for p in 1..3 {
            row_del.push(ChartEdit::Value(ValueEdit {
                series: s,
                point: p,
                target: EditTarget::Label,
                text: series.labels[p + 1].text.clone(),
            }));
            row_del.push(ChartEdit::Value(ValueEdit {
                series: s,
                point: p,
                target: EditTarget::Value,
                text: series.values[p + 1].text.clone(),
            }));
        }
        row_del.push(ChartEdit::TruncatePoints {
            series: s,
            target: EditTarget::Label,
            keep: 3,
        });
        row_del.push(ChartEdit::TruncatePoints {
            series: s,
            target: EditTarget::Value,
            keep: 3,
        });
    }
    assert_eq!(
        engine(&xml, &row_del),
        b2_remove_point(&b2_remove_point(&xml, "c:cat", 1), "c:val", 1),
        "행삭제"
    );

    // 계열추가 — 마지막 계열 복제 + 채번 + 이름·값 교체 (c:f 는 복제분 그대로).
    assert_eq!(
        engine(
            &xml,
            &[ChartEdit::AppendSeries {
                name: Some("추가계열".to_string()),
                labels: None,
                values: strs(&["6", "6", "6", "6"]),
            }]
        ),
        b2_clone_last_series(&xml, "추가계열", &["6", "6", "6", "6"]),
        "계열추가"
    );

    // 계열삭제(마지막) — 꼬리 c:ser 제거.
    assert_eq!(
        engine(&xml, &[ChartEdit::TruncateSeries { keep: 2 }]),
        b2_remove_series(&xml, 2),
        "계열삭제"
    );

    // 계열명변경 — c:tx 캐시 텍스트만.
    assert_eq!(
        engine(
            &xml,
            &[ChartEdit::SeriesName {
                series: 0,
                text: "이름바뀐계열".to_string()
            }]
        ),
        b2_rename_series(&xml, 0, "이름바뀐계열"),
        "계열명변경"
    );

    // 라벨변경 — 전 계열 c:cat 동기 교체.
    let relabel: Vec<ChartEdit> = (0..3)
        .map(|s| {
            ChartEdit::Value(ValueEdit {
                series: s,
                point: 0,
                target: EditTarget::Label,
                text: "바뀐항목".to_string(),
            })
        })
        .collect();
    assert_eq!(
        engine(&xml, &relabel),
        b2_relabel_category(&xml, 0, "바뀐항목"),
        "라벨변경"
    );

    // 분산형 점추가 — xVal/yVal 동기.
    let scatter_src = manifest("samples/chart/분산형/직선및표식이있는분산형.hwpx");
    let (_legacy, ooxml) =
        chart_streams(&std::fs::read(&scatter_src).expect("샘플")).expect("차트 스트림");
    let sxml = String::from_utf8(ooxml).expect("UTF-8");
    let mut point_add = Vec::new();
    for (s, y) in ["27", "10"].iter().enumerate() {
        point_add.push(ChartEdit::AppendPoints {
            series: s,
            target: EditTarget::Label,
            texts: strs(&["9"]),
        });
        point_add.push(ChartEdit::AppendPoints {
            series: s,
            target: EditTarget::Value,
            texts: strs(&[y]),
        });
    }
    let spike = b2_add_point(
        &b2_add_point(&sxml, "c:xVal", &["9", "9"]),
        "c:yVal",
        &["27", "10"],
    );
    assert_eq!(engine(&sxml, &point_add), spike, "분산형 점추가");
}

// ---------------------------------------------------------------------------
// Stage 9 (S3) — 코어: structure 의도 분기 · 위치 기반 plan · 종류별 가드 · self-check
// ---------------------------------------------------------------------------

/// ①(있으면)·② 의 OOXML XML 을 문자열로.
fn b2_representations(core: &DocumentCore) -> (Option<String>, String) {
    let chart = collect_charts(core.document())[0].clone();
    let zip = chart.zip_part.map(|i| {
        String::from_utf8(core.document().bin_data_content[i].data.load()).expect("UTF-8")
    });
    let nested = core.document().bin_data_content[chart.nested_copy.expect("②")]
        .data
        .load();
    let ooxml = String::from_utf8(stream_of(&nested, OOXML_STREAM).expect("②")).expect("UTF-8");
    (zip, ooxml)
}

/// ③④ 바이트.
fn b2_legacy_and_emf(core: &DocumentCore) -> (Vec<u8>, Vec<u8>) {
    let chart = collect_charts(core.document())[0].clone();
    let nested = core.document().bin_data_content[chart.nested_copy.expect("②")]
        .data
        .load();
    (
        stream_of(&nested, LEGACY_STREAM).expect("③"),
        stream_of(&nested, EMF_STREAM).expect("④"),
    )
}

fn b2_reasons(out: &serde_json::Value) -> Vec<String> {
    out["invalid"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v["reason"].as_str().unwrap_or("").to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn b2_ops(out: &serde_json::Value) -> Vec<String> {
    out["changed"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v["op"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// 편집 입력을 "행렬 + structure" 로 만든다 — 현재 값에서 출발해 클로저로 바꾼다.
fn b2_structure_edits(
    core: &DocumentCore,
    mutate: impl FnOnce(&mut serde_json::Value),
) -> serde_json::Value {
    let mut e = edits_from(core, 0);
    e["structure"] = serde_json::json!(true);
    mutate(&mut e);
    e
}

fn b2_push_str(arr: &mut serde_json::Value, s: &str) {
    arr.as_array_mut().expect("배열").push(serde_json::json!(s));
}

/// `structure` 없는 입력은 B1 거부 4종을 그대로 내고, 메시지가 의도 플래그를 안내한다.
#[test]
fn structure_flag_off_keeps_every_b1_refusal() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let mut e = edits_from(&core, 0);
    b2_push_str(&mut e["labels"], "추가항목");
    for s in e["series"].as_array_mut().unwrap() {
        b2_push_str(&mut s["values"], "1");
    }
    let before = slot_bytes(&core);
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], false, "{out}");
    let reasons = b2_reasons(&out);
    assert!(
        reasons.contains(&"valueCountMismatch".to_string()),
        "{reasons:?}"
    );
    let message = out["invalid"][0]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("structure"),
        "의도 플래그 안내가 없다: {message}"
    );
    assert_eq!(slot_bytes(&core), before);
}

/// 행 추가 — ①② 동시 기록, 재독에서 행 5·ptCount 5·idx 0..4, ③④·c:f·hncChartStyle 바이트 불변.
#[test]
fn structure_row_append_writes_both_representations_and_rereads() {
    let base = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    for path in [base.with_extension("hwpx"), base.with_extension("hwp")] {
        let is_hwpx = path.extension().is_some_and(|e| e == "hwpx");
        let mut core = core_of(&path);
        let (legacy_before, emf_before) = b2_legacy_and_emf(&core);
        let (zip_before, nested_before) = b2_representations(&core);
        let e = b2_structure_edits(&core, |e| {
            b2_push_str(&mut e["labels"], "추가항목");
            for (s, v) in e["series"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .zip(["45", "44", "43"])
            {
                b2_push_str(&mut s["values"], v);
            }
        });
        let out = set_chart(&mut core, &e);
        assert_eq!(out["ok"], true, "{}: {out}", path.display());
        let wrote = out["wrote"].as_array().expect("wrote");
        if is_hwpx {
            assert_eq!(wrote.len(), 2, "{out}");
        } else {
            assert_eq!(wrote, &[serde_json::json!("nestedCopy")], "{out}");
        }
        assert!(b2_ops(&out).iter().any(|op| op == "appendPoints"), "{out}");

        // 저장 → 재개방 → 구조 확인.
        let bytes = if is_hwpx {
            core.export_hwpx_native().expect("저장")
        } else {
            core.export_hwp_native().expect("저장")
        };
        let reread = DocumentCore::from_bytes(&bytes).expect("재개방");
        let env = b2_envelope(&reread);
        assert_eq!(env["ok"], true, "{env}");
        b2_check_row_add(&env, "행추가");
        let (zip_after, nested_after) = b2_representations(&reread);
        for (before, after) in [
            (zip_before.as_deref(), zip_after.as_deref()),
            (Some(nested_before.as_str()), Some(nested_after.as_str())),
        ] {
            let (Some(before), Some(after)) = (before, after) else {
                continue;
            };
            let data = scan_chart_values(after.as_bytes()).expect("산출 재스캔");
            for s in &data.series {
                assert_eq!(s.values.len(), 5);
                assert_eq!(s.labels.len(), 5);
                assert_eq!(
                    s.values_shape
                        .as_ref()
                        .unwrap()
                        .pt_count
                        .as_ref()
                        .unwrap()
                        .value,
                    5
                );
                assert_eq!(
                    s.labels_shape
                        .as_ref()
                        .unwrap()
                        .pt_count
                        .as_ref()
                        .unwrap()
                        .value,
                    5
                );
                assert!(s
                    .values
                    .iter()
                    .enumerate()
                    .all(|(i, p)| p.idx as usize == i));
            }
            assert_eq!(
                before.matches("Sheet1!$A$2:$A$5").count(),
                after.matches("Sheet1!$A$2:$A$5").count(),
                "c:f 가 바뀌었다"
            );
            assert_eq!(
                before.matches("ho:hncChartStyle").count(),
                after.matches("ho:hncChartStyle").count()
            );
        }
        if is_hwpx {
            assert_eq!(zip_after, Some(nested_after), "①② 가 다르다");
        }
        let (legacy_after, emf_after) = b2_legacy_and_emf(&reread);
        assert_eq!(legacy_after, legacy_before, "③ 이 변했다");
        assert_eq!(emf_after, emf_before, "④ 가 변했다");
    }
}

/// 중간 행 삭제(「항목 2」) — 엔진 산출 ① 이 스파이크 수술 산출과 바이트 동일.
#[test]
fn structure_middle_row_delete_equals_spike_surgery() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let (zip_before, _) = b2_representations(&core);
    let xml = zip_before.expect("①");
    let e = b2_structure_edits(&core, |e| {
        e["labels"].as_array_mut().unwrap().remove(1);
        for s in e["series"].as_array_mut().unwrap() {
            s["values"].as_array_mut().unwrap().remove(1);
        }
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    assert!(
        b2_ops(&out).iter().any(|op| op == "truncatePoints"),
        "{out}"
    );
    let (zip_after, nested_after) = b2_representations(&core);
    let expected = b2_remove_point(&b2_remove_point(&xml, "c:cat", 1), "c:val", 1);
    assert_eq!(
        zip_after.as_deref(),
        Some(expected.as_str()),
        "① ≠ 스파이크 수술"
    );
    assert_eq!(nested_after, expected, "② ≠ 스파이크 수술");
    let env = b2_envelope(&core);
    assert_eq!(b2_labels(&env), ["항목 1", "항목 3", "항목 4"]);
    assert_eq!(b2_values(&env, 0), ["4.3", "3.5", "4.5"]);
}

/// 계열 추가·삭제·이름 변경·라벨 변경 — 각각 스파이크 수술과 바이트 동일, changed[] 에 op.
#[test]
fn structure_series_append_and_truncate_rename_relabel() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let xml = b2_representations(&core_of(&path)).0.expect("①");

    // 계열 추가
    let mut core = core_of(&path);
    let e = b2_structure_edits(&core, |e| {
        e["series"].as_array_mut().unwrap().push(serde_json::json!({
            "name": "추가계열", "values": ["6", "6", "6", "6"],
        }));
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    assert!(b2_ops(&out).iter().any(|op| op == "appendSeries"), "{out}");
    assert_eq!(
        b2_representations(&core).0.as_deref(),
        Some(b2_clone_last_series(&xml, "추가계열", &["6", "6", "6", "6"]).as_str()),
        "계열추가"
    );
    let env = b2_envelope(&core);
    assert_eq!(
        b2_series_names(&env),
        ["계열 1", "계열 2", "계열 3", "추가계열"]
    );
    assert_eq!(b2_values(&env, 3), ["6", "6", "6", "6"]);

    // 계열 삭제(마지막)
    let mut core = core_of(&path);
    let e = b2_structure_edits(&core, |e| {
        e["series"].as_array_mut().unwrap().pop();
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    assert!(
        b2_ops(&out).iter().any(|op| op == "truncateSeries"),
        "{out}"
    );
    assert_eq!(
        b2_representations(&core).0.as_deref(),
        Some(b2_remove_series(&xml, 2).as_str()),
        "계열삭제"
    );
    assert_eq!(b2_series_names(&b2_envelope(&core)), ["계열 1", "계열 2"]);

    // 계열명 변경
    let mut core = core_of(&path);
    let e = b2_structure_edits(&core, |e| {
        e["series"][0]["name"] = serde_json::json!("이름바뀐계열");
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    assert!(b2_ops(&out).iter().any(|op| op == "renameSeries"), "{out}");
    assert_eq!(
        b2_representations(&core).0.as_deref(),
        Some(b2_rename_series(&xml, 0, "이름바뀐계열").as_str()),
        "계열명변경"
    );

    // 라벨 변경 — 전 계열 동기
    let mut core = core_of(&path);
    let e = b2_structure_edits(&core, |e| {
        e["labels"][0] = serde_json::json!("바뀐항목");
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    assert!(b2_ops(&out).iter().any(|op| op == "relabel"), "{out}");
    assert_eq!(
        b2_representations(&core).0.as_deref(),
        Some(b2_relabel_category(&xml, 0, "바뀐항목").as_str()),
        "라벨변경"
    );
    assert_eq!(b2_labels(&b2_envelope(&core))[0], "바뀐항목");
}

/// dry-run 은 op 를 보고하고 한 바이트도 쓰지 않는다.
#[test]
fn structure_dry_run_reports_ops_and_writes_nothing() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let before = slot_bytes(&core);
    let e = b2_structure_edits(&core, |e| {
        e["dryRun"] = serde_json::json!(true);
        b2_push_str(&mut e["labels"], "추가항목");
        for s in e["series"].as_array_mut().unwrap() {
            b2_push_str(&mut s["values"], "1");
        }
        e["series"].as_array_mut().unwrap().pop();
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["dryRun"], true);
    assert!(out["wrote"].as_array().unwrap().is_empty());
    let ops = b2_ops(&out);
    assert!(
        ops.iter().any(|op| op == "appendPoints") && ops.iter().any(|op| op == "truncateSeries"),
        "{ops:?}"
    );
    assert_eq!(slot_bytes(&core), before, "dry-run 인데 바이트가 바뀌었다");
}

/// 행렬 규칙 — 계열 간 행 수 불일치·라벨 누락·라벨 개수 불일치는 거부(한 바이트도 안 씀).
#[test]
fn structure_matrix_rules_are_enforced() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let cases: Vec<(&str, Box<dyn Fn(&mut serde_json::Value)>)> = vec![
        (
            "rowCountMismatch",
            Box::new(|e| {
                b2_push_str(&mut e["labels"], "추가항목");
                b2_push_str(&mut e["series"][0]["values"], "1");
            }),
        ),
        (
            "labelsRequired",
            Box::new(|e| {
                e["labels"] = serde_json::Value::Null;
                for s in e["series"].as_array_mut().unwrap() {
                    b2_push_str(&mut s["values"], "1");
                }
            }),
        ),
        (
            "labelCountMismatch",
            Box::new(|e| {
                for s in e["series"].as_array_mut().unwrap() {
                    b2_push_str(&mut s["values"], "1");
                }
            }),
        ),
        (
            "notANumber",
            Box::new(|e| {
                b2_push_str(&mut e["labels"], "추가항목");
                for s in e["series"].as_array_mut().unwrap() {
                    b2_push_str(&mut s["values"], "칠십");
                }
            }),
        ),
        (
            "unsafeText",
            Box::new(|e| {
                e["labels"][0] = serde_json::json!("R&D");
            }),
        ),
        (
            "seriesNameRequired",
            Box::new(|e| {
                e["series"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({"values": ["1", "1", "1", "1"]}));
            }),
        ),
    ];
    for (reason, mutate) in cases {
        let mut core = core_of(&path);
        let before = slot_bytes(&core);
        let e = b2_structure_edits(&core, |e| mutate(e));
        let out = set_chart(&mut core, &e);
        assert_eq!(out["ok"], false, "{reason}: {out}");
        let reasons = b2_reasons(&out);
        assert!(
            reasons.contains(&reason.to_string()),
            "{reason} 가 없다: {reasons:?}"
        );
        assert!(out["wrote"].as_array().unwrap().is_empty());
        assert_eq!(
            slot_bytes(&core),
            before,
            "{reason}: 거부했는데 바이트가 바뀌었다"
        );
    }
}

/// 구조 좌표가 없는 블록(`ptCount` 없는 리터럴)의 개수 변경은 `pointsNotInsertable` 로 거부.
#[test]
fn structure_edit_refuses_when_block_not_resizable() {
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    // BLANK_VALUE_CHART 는 strLit/numLit 에 ptCount 가 없다 — 꼬리 삽입 앵커는 있어도 재계산할 ptCount 가 없다.
    replace_chart_representations(
        &mut core,
        BLANK_VALUE_CHART.as_bytes(),
        BLANK_VALUE_CHART.as_bytes(),
    );
    let before = slot_bytes(&core);
    let e = b2_structure_edits(&core, |e| {
        b2_push_str(&mut e["labels"], "C");
        for s in e["series"].as_array_mut().unwrap() {
            b2_push_str(&mut s["values"], "1");
        }
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], false, "{out}");
    assert!(
        b2_reasons(&out).contains(&"pointsNotInsertable".to_string()),
        "{out}"
    );
    assert_eq!(slot_bytes(&core), before);
}

// ---- S3-b 종류별 fail-closed 가드 ------------------------------------------
//
// 근거는 #5447 판정 원장(samples/issue5447/MANIFEST.json): `원형대원형-계열추가` = 미반영
// (한컴이 2번째 계열을 조용히 무시), `시가고가저가종가-계열삭제` = 반영_의미깨짐(c:upDownBars 가
// 남아 고가·저가를 몸통 삼아 틀리게 그림). 한컴이 막지 않으므로 코어가 막는다.

fn b2_refused_on(path: &std::path::Path, reason: &str, mutate: impl Fn(&mut serde_json::Value)) {
    let mut core = core_of(path);
    let before = slot_bytes(&core);
    let e = b2_structure_edits(&core, |e| mutate(e));
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], false, "{}: {out}", path.display());
    let reasons = b2_reasons(&out);
    assert!(
        reasons.contains(&reason.to_string()),
        "{}: {reason} 가 없다 — {reasons:?}",
        path.display()
    );
    assert!(out["wrote"].as_array().unwrap().is_empty());
    assert_eq!(
        slot_bytes(&core),
        before,
        "{}: 거부했는데 바이트가 바뀌었다",
        path.display()
    );
}

/// 원형·3D원형·ofPie 는 계열 수 1 고정 — 계열 추가 거부(양 포맷).
#[test]
fn pie_series_count_is_fixed() {
    for stem in ["원형/원형대원형", "원형/2차원원형", "원형/3차원원형"] {
        let base = manifest(&format!("samples/chart/{stem}.hwpx"));
        for path in [base.with_extension("hwpx"), base.with_extension("hwp")] {
            b2_refused_on(&path, "pieSeriesCountFixed", |e| {
                let n = e["series"][0]["values"].as_array().unwrap().len();
                e["series"].as_array_mut().unwrap().push(serde_json::json!({
                    "name": "추가계열", "values": vec!["1"; n],
                }));
            });
        }
    }
}

/// 주식형은 계열 수가 종류에 묶인다(HLC=3 / OHLC=4) — 개수 변경 거부. 변경은 B3 소관.
#[test]
fn stock_series_count_is_fixed() {
    let ohlc = manifest("samples/chart/기타/시가고가저가종가.hwpx");
    for path in [ohlc.with_extension("hwpx"), ohlc.with_extension("hwp")] {
        b2_refused_on(&path, "stockSeriesCountFixed", |e| {
            e["series"].as_array_mut().unwrap().remove(0);
        });
    }
    let hlc = manifest("samples/chart/기타/고가저가종가.hwpx");
    b2_refused_on(&hlc, "stockSeriesCountFixed", |e| {
        let n = e["series"][0]["values"].as_array().unwrap().len();
        e["series"].as_array_mut().unwrap().push(serde_json::json!({
            "name": "시가", "values": vec!["1"; n],
        }));
    });
}

/// 마지막 1점·1계열은 지울 수 없다 — 특이케이스(1계열 1점, c:numLit)가 그 경계.
#[test]
fn last_point_and_last_series_cannot_be_deleted() {
    let path = manifest("samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwpx");
    b2_refused_on(&path, "lastPointDeleteRefused", |e| {
        e["labels"] = serde_json::json!([]);
        e["series"][0]["values"] = serde_json::json!([]);
    });
    b2_refused_on(&path, "lastSeriesDeleteRefused", |e| {
        e["series"] = serde_json::json!([]);
    });
}

/// 분산형은 행 수가 바뀌면 X(labels)가 같은 개수로 함께 와야 한다 — 한쪽만 바뀌면 거부.
#[test]
fn scatter_rows_require_synchronized_x() {
    let path = manifest("samples/chart/분산형/직선및표식이있는분산형.hwpx");
    b2_refused_on(&path, "scatterXYMismatch", |e| {
        e["labels"] = serde_json::Value::Null;
        for s in e["series"].as_array_mut().unwrap() {
            b2_push_str(&mut s["values"], "1");
        }
    });
    b2_refused_on(&path, "scatterXYMismatch", |e| {
        for s in e["series"].as_array_mut().unwrap() {
            b2_push_str(&mut s["values"], "1");
        }
    });
    // 동기로 오면 통과 — 스파이크 분산형 점추가와 바이트 동일.
    let mut core = core_of(&path);
    let xml = b2_representations(&core).0.expect("①");
    let e = b2_structure_edits(&core, |e| {
        b2_push_str(&mut e["labels"], "9");
        for (s, y) in e["series"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(["27", "10"])
        {
            b2_push_str(&mut s["values"], y);
        }
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    let spike = b2_add_point(
        &b2_add_point(&xml, "c:xVal", &["9", "9"]),
        "c:yVal",
        &["27", "10"],
    );
    assert_eq!(b2_representations(&core).0.as_deref(), Some(spike.as_str()));
}

/// 다층 카테고리(`multiLvlStrRef`)는 구조 편집을 거부한다 — 코퍼스 0건, 합성 주입.
#[test]
fn multi_level_labels_refuse_structure_edits() {
    const MULTI: &str = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser><c:idx val="0"/><c:order val="0"/>"#,
        r#"<c:cat><c:multiLvlStrRef><c:multiLvlStrCache><c:ptCount val="2"/>"#,
        r#"<c:lvl><c:pt idx="0"><c:v>상반기</c:v></c:pt><c:pt idx="1"><c:v>하반기</c:v></c:pt></c:lvl>"#,
        r#"</c:multiLvlStrCache></c:multiLvlStrRef></c:cat>"#,
        r#"<c:val><c:numRef><c:numCache><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt>"#,
        r#"</c:numCache></c:numRef></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );
    let path = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    replace_chart_representations(&mut core, MULTI.as_bytes(), MULTI.as_bytes());
    let before = slot_bytes(&core);
    let e = serde_json::json!({
        "structure": true,
        "series": [{"values": ["1", "2", "3"]}],
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], false, "{out}");
    assert!(
        b2_reasons(&out).contains(&"multiLevelLabelsUnsupported".to_string()),
        "{out}"
    );
    assert_eq!(slot_bytes(&core), before);
    // 값만 바꾸는 편집은 다층이어도 된다(B1 과 같다).
    let out = set_chart(
        &mut core,
        &serde_json::json!({"structure": true, "series": [{"values": ["7", "2"]}]}),
    );
    assert_eq!(out["ok"], true, "{out}");
}

/// 가드는 dry-run 에서도 발화한다.
#[test]
fn guards_fire_in_dry_run_too() {
    let path = manifest("samples/chart/원형/원형대원형.hwpx");
    b2_refused_on(&path, "pieSeriesCountFixed", |e| {
        e["dryRun"] = serde_json::json!(true);
        let n = e["series"][0]["values"].as_array().unwrap().len();
        e["series"].as_array_mut().unwrap().push(serde_json::json!({
            "name": "추가계열", "values": vec!["1"; n],
        }));
    });
}

// ---------------------------------------------------------------------------
// Stage 9 (S5) — 엔진 산출 회귀 + 한컴 판정 번들 (#5652)
// ---------------------------------------------------------------------------
//
// #5447 은 문자열 수술로 만든 변종을 한컴이 판정했다. 본구현은 **엔진이 만든 바이트**를
// 판정해야 한다(이슈 §검증). 변종 카탈로그(`b2_variants`)의 `what`·`expect_shape` 를 그대로
// 쓰되, 수술 클로저 대신 `set_chart_data_by_index_native(…, structure:true)` 로 만든다.
// 경계 2종(원형 계열추가·주식형 계열삭제)은 가드가 막으므로 번들에서 빠지고 거부 테스트가
// 대신한다(`pie_series_count_is_fixed`·`stock_series_count_is_fixed`).

/// 변종 (기준 문서, 라벨) → 엔진 편집 입력. 가드가 막는 경계 변종은 `None`.
fn b2_engine_edits(core: &DocumentCore, stem: &str, label: &str) -> Option<serde_json::Value> {
    let mut e = edits_from(core, 0);
    e["structure"] = serde_json::json!(true);
    match (stem, label) {
        (_, "행추가") => {
            b2_push_str(&mut e["labels"], "추가항목");
            for (s, v) in e["series"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .zip(["45", "44", "43"])
            {
                b2_push_str(&mut s["values"], v);
            }
        }
        (_, "행삭제") => {
            e["labels"].as_array_mut().unwrap().remove(1);
            for s in e["series"].as_array_mut().unwrap() {
                s["values"].as_array_mut().unwrap().remove(1);
            }
        }
        ("묶은세로막대형", "계열추가") => {
            e["series"].as_array_mut().unwrap().push(serde_json::json!({
                "name": "추가계열", "values": ["6", "6", "6", "6"],
            }));
        }
        ("묶은세로막대형", "계열삭제") | ("누적세로막대형", "계열삭제") => {
            // 「계열 2」 삭제 — 위치 기반이라 뒤 계열이 앞으로 당겨지고 꼬리가 지워진다.
            e["series"].as_array_mut().unwrap().remove(1);
        }
        (_, "계열명변경") => {
            e["series"][0]["name"] = serde_json::json!("이름바뀐계열");
        }
        (_, "라벨변경") => {
            e["labels"][0] = serde_json::json!("바뀐항목");
        }
        ("직선및표식이있는분산형", "점추가") => {
            b2_push_str(&mut e["labels"], "9");
            for (s, y) in e["series"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .zip(["27", "10"])
            {
                b2_push_str(&mut s["values"], y);
            }
        }
        ("가로막대형_하나만있을떄_단일시리즈제목", "점추가") => {
            b2_push_str(&mut e["labels"], "추가항목");
            b2_push_str(&mut e["series"][0]["values"], "43");
        }
        // 경계 — 가드가 막는다.
        ("원형대원형", "계열추가") | ("시가고가저가종가", "계열삭제") => {
            return None
        }
        other => panic!("엔진 편집 입력이 정의되지 않은 변종: {other:?}"),
    }
    Some(e)
}

/// 변종 1건을 **엔진으로** 양 포맷으로 만들어 자기검증 후 기록한다. 반환은 쓴 파일 수.
///
/// 자기검증은 #5447 `b2_write_variant` 의 4단 그대로 — 재개방 + 봉투 구조 확인(`v.check`),
/// ①==②(HWPX), ③④ 바이트 불변. `out_dir` 이 `None` 이면 파일을 쓰지 않는다(상시 회귀).
fn b2_engine_write_variant(
    out_dir: Option<&std::path::Path>,
    v: &B2Variant,
    sheet: &mut String,
) -> usize {
    let mut written = 0usize;
    for ext in ["hwpx", "hwp"] {
        let src = manifest(&format!("samples/chart/{}/{}.{ext}", v.folder, v.stem));
        let name = format!("{}-{}.{ext}", v.stem, v.label);
        let mut core = core_of(&src);
        let Some(edits) = b2_engine_edits(&core, v.stem, v.label) else {
            return 0;
        };
        let (legacy_before, emf_before) = b2_legacy_and_emf(&core);

        let out = set_chart(&mut core, &edits);
        assert_eq!(out["ok"], true, "{name}: 엔진이 거부했다 — {out}");
        assert!(
            !out["wrote"].as_array().unwrap().is_empty(),
            "{name}: 쓴 표현이 없다 — {out}"
        );

        let bytes = if ext == "hwpx" {
            core.export_hwpx_native().expect("HWPX 저장")
        } else {
            core.export_hwp_native().expect("HWP5 저장")
        };

        // 자기검증 1 — 재개방 + 스캔 게이트 + 구조 확인.
        let reread = DocumentCore::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("{name}: rhwp 가 다시 열지 못한다 — {e:?}"));
        let env = b2_envelope(&reread);
        assert_eq!(env["ok"], true, "{name}: {env}");
        (v.check)(&env, &name);

        // 자기검증 2 — ①==②, ③④ 불변.
        let (zip_after, nested_after) = b2_representations(&reread);
        if let Some(zip) = zip_after {
            assert_eq!(zip, nested_after, "{name}: ① ≠ ②");
        }
        let (legacy_after, emf_after) = b2_legacy_and_emf(&reread);
        assert_eq!(legacy_after, legacy_before, "{name}: ③ 이 변했다");
        assert_eq!(emf_after, emf_before, "{name}: ④ 가 변했다");

        if let Some(dir) = out_dir {
            std::fs::write(dir.join(&name), &bytes).expect("산출 쓰기");
            written += 1;
            sheet.push_str(&format!(
                "| `{name}` | {} | {} | {} | {} |\n",
                v.folder, v.what, v.expect_shape, v.stale
            ));
        }
    }
    written
}

/// [#5652 S5] 엔진 구조 편집 후 재렌더에 반영된다 — 행추가·행삭제·계열추가.
#[test]
fn b2_engine_row_and_series_edits_render_after_reopen() {
    let src = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");

    // 행추가 — 새 카테고리 라벨이 실제로 그려진다.
    let mut core = core_of(&src);
    let e = b2_engine_edits(&core, "묶은세로막대형", "행추가").unwrap();
    assert_eq!(set_chart(&mut core, &e)["ok"], true);
    let reread =
        DocumentCore::from_bytes(&core.export_hwpx_native().expect("저장")).expect("재개방");
    let svg = reread.render_page_svg_layer_native(0).expect("렌더");
    assert!(svg.contains("추가항목"), "행추가: 새 라벨이 렌더에 없다");

    // 행삭제 — 지운 라벨이 사라진다.
    let mut core = core_of(&src);
    let e = b2_engine_edits(&core, "묶은세로막대형", "행삭제").unwrap();
    assert_eq!(set_chart(&mut core, &e)["ok"], true);
    let reread =
        DocumentCore::from_bytes(&core.export_hwpx_native().expect("저장")).expect("재개방");
    let svg = reread.render_page_svg_layer_native(0).expect("렌더");
    assert!(
        !svg.contains("항목 2"),
        "행삭제: 지운 라벨이 여전히 그려진다"
    );
    assert!(svg.contains("항목 3"), "행삭제: 남은 라벨이 렌더에 없다");

    // 계열추가 — 범례에 새 계열명이 등장한다.
    let mut core = core_of(&src);
    let e = b2_engine_edits(&core, "묶은세로막대형", "계열추가").unwrap();
    assert_eq!(set_chart(&mut core, &e)["ok"], true);
    let reread =
        DocumentCore::from_bytes(&core.export_hwpx_native().expect("저장")).expect("재개방");
    let svg = reread.render_page_svg_layer_native(0).expect("렌더");
    assert!(
        svg.contains("추가계열"),
        "계열추가: 새 계열명이 렌더에 없다"
    );
}

/// [#5652 S5] 변종 12종 × 2포맷의 엔진 산출이 재독·①②·③④ 자기검증을 전건 통과한다 (상시).
#[test]
fn b2_engine_output_passes_the_scanner_for_every_variant() {
    let mut sheet = String::new();
    let mut produced = 0usize;
    for v in b2_variants() {
        if b2_engine_edits(
            &core_of(&manifest(&format!(
                "samples/chart/{}/{}.hwpx",
                v.folder, v.stem
            ))),
            v.stem,
            v.label,
        )
        .is_none()
        {
            continue; // 경계 2종 — 가드 테스트가 대신한다.
        }
        b2_engine_write_variant(None, &v, &mut sheet);
        produced += 1;
    }
    assert_eq!(produced, 12, "경계 2종을 뺀 변종 수");
}

/// Stage 9 — B2 **엔진 산출** 한컴 판정 꾸러미(#5652 S5)를 만든다.
///
/// `output/` 에 파일을 쓰는 부작용이 있어 기본 실행에서 뺀다. 판정 직전에만 돌린다:
///
/// ```text
/// cargo test --profile release-test --test issue_4100_chart_data_edit \
///     generate_b2_engine_judgment_bundle -- --ignored --nocapture
/// ```
///
/// #5447 번들(`generate_b2_structure_judgment_bundle`)과 같은 변종·같은 판정 방법(144DPI 래스터
/// 해시 + 편집기 행·열 수)이되, 바이트는 엔진(`set_chart_data_by_index_native`)이 만든다.
/// 기존 38건 원장(`samples/issue5447/`)은 손대지 않는다 — 판정 결과는 `samples/issue5652/` 에
/// 별도 원장으로 쌓는다.
#[test]
#[ignore = "output/ 에 파일을 쓴다 — 한컴 판정 직전에만 실행"]
fn generate_b2_engine_judgment_bundle() {
    let out_dir = manifest("output/issue_5652_b2_engine_judgment");
    std::fs::create_dir_all(&out_dir).expect("출력 디렉터리");

    let mut sheet = String::new();
    sheet.push_str("# #5652 B2 엔진 — 한컴 판정표 (엔진 산출 구조 변종)\n\n");
    sheet.push_str(
        "#5447 스파이크와 같은 변종을 이번에는 **엔진**(`set_chart_data_by_index_native`,\n\
         `structure:true`)이 만들었습니다. 행·열·라벨이 바뀌므로 파일마다 「기대 모양」 칸과\n\
         대조해 주세요. `c:f`·③레거시 Contents·④프리뷰는 설계대로 **갱신하지 않습니다**(#5447 확정).\n\n\
         경계 2종(원형 계열추가·주식형 계열삭제)은 엔진 가드가 거부하므로 이 번들에 없습니다 —\n\
         거부는 `pie_series_count_is_fixed`·`stock_series_count_is_fixed` 가 상시로 고정합니다.\n\n",
    );
    sheet.push_str("## 보는 법\n\n");
    sheet.push_str(
        "1. `*-대조군.hwpx` 로 원본 모습을 눈에 익힙니다.\n\
         2. 변종 파일마다 **네 가지**를 봐 주세요:\n   \
         (a) 열 때 오류·복구 대화상자가 뜨는가\n   \
         (b) 차트가 **기대 모양대로** 그려지는가\n   \
         (c) 차트를 더블클릭하면 편집기가 열리는가\n   \
         (d) **편집기(데이터 편집)의 행·열 수가 기대 모양과 일치하는가**\n\n\
         계열삭제 변종은 위치 기반(뒤 계열이 앞으로 당겨지고 꼬리가 지워짐)이라 잔여 계열의\n\
         색이 원래 2번째 계열 색일 수 있습니다 — 이름·값이 맞으면 정상입니다.\n\n",
    );
    sheet.push_str("## 산출물\n\n");
    sheet.push_str("| 파일 | 종류 | 무엇을 바꿨나 | 기대 모양 | 낡게 남긴 것 |\n");
    sheet.push_str("|---|---|---|---|---|\n");

    let variants: Vec<B2Variant> = b2_variants()
        .into_iter()
        .filter(|v| {
            b2_engine_edits(
                &core_of(&manifest(&format!(
                    "samples/chart/{}/{}.hwpx",
                    v.folder, v.stem
                ))),
                v.stem,
                v.label,
            )
            .is_some()
        })
        .collect();
    assert_eq!(variants.len(), 12, "경계 2종을 뺀 변종 수");
    let mut written = 0usize;

    let mut controls: Vec<(&str, &str)> = Vec::new();
    for v in &variants {
        if !controls.contains(&(v.folder, v.stem)) {
            controls.push((v.folder, v.stem));
        }
    }
    for (folder, stem) in &controls {
        std::fs::copy(
            manifest(&format!("samples/chart/{folder}/{stem}.hwpx")),
            out_dir.join(format!("{stem}-대조군.hwpx")),
        )
        .expect("대조군 복사");
        written += 1;
        sheet.push_str(&format!(
            "| `{stem}-대조군.hwpx` | {folder} | (무편집 원본) | 원본 그대로 | — |\n"
        ));
    }

    for v in &variants {
        written += b2_engine_write_variant(Some(&out_dir), v, &mut sheet);
    }

    // 변환 축 — 행추가를 HWPX 에서 엔진으로 만들고 HWP5 로 변환한다(①이 ②로 접힌다, #4099).
    {
        let src = manifest("samples/chart/세로막대형/묶은세로막대형.hwpx");
        let mut core = core_of(&src);
        let e = b2_engine_edits(&core, "묶은세로막대형", "행추가").unwrap();
        assert_eq!(set_chart(&mut core, &e)["ok"], true);
        let bytes = core
            .export_hwp_with_adapter_snapshot()
            .expect("HWP5 변환 저장");
        let reread = DocumentCore::from_bytes(&bytes).expect("변환본 재파스");
        let env = b2_envelope(&reread);
        assert_eq!(env["ok"], true, "변환본: {env}");
        b2_check_row_add(&env, "변환본");
        let name = "묶은세로막대형-행추가-HWPX에서변환.hwp";
        std::fs::write(out_dir.join(name), &bytes).expect("변환본 쓰기");
        written += 1;
        sheet.push_str(&format!(
            "| `{name}` | 세로막대형 | 행추가 후 HWPX→HWP5 변환 (구조가 ② 로 접힘) | 그룹 4→5 | c:f·③·④ |\n"
        ));
    }

    sheet.push_str(&format!("\n총 {written} 파일.\n\n"));
    sheet.push_str(
        "## PDF 회신\n\n\
         각 파일을 한컴에서 열어 **같은 폴더에 PDF 로 저장**해 주시면, 대조군과 변종의 렌더를\n\
         144DPI 래스터 해시로 갈라 반영 여부를 데이터로 판정하겠습니다(#5447 절차 —\n\
         `tools/hancom_chart_judgment_verify.py`). (d) 편집기 행·열 수는 **파일별로 한 줄씩**\n\
         남겨 주세요.\n\n상세 설계는 #5652, 전제는 #5447.\n",
    );

    std::fs::write(out_dir.join("PANJEONG.md"), sheet).expect("판정표 쓰기");
    println!("\n  판정 번들: {}", out_dir.display());
    println!("  파일 {written}개 + 판정표 PANJEONG.md");
    assert_eq!(written, 32, "대조군 7 + 변종 12 × 2포맷 + 변환본 1");
}

/// [#5652 S5] **문서 바이트 수준** 동치 — 엔진 경로(`set_chart_data_by_index_native`, structure)와
/// 스파이크 경로(문자열 수술 + 표현 주입)가 같은 현재 라이터로 저장하면, 계열삭제 2종을 뺀
/// 12변종 × 2포맷 전건이 **바이트 동일**하다. 계열삭제는 위치 기반(뒤 계열을 앞으로 당겨 쓰고 꼬리
/// 삭제)이라 바이트는 다르되 논리 데이터(이름·라벨·값)는 같다.
///
/// 이것이 S5 한컴 재판정의 근거다 — 10종은 #5447 이 판정한 것과 같은 차트 XML 을 엔진이 만들고,
/// 계열삭제 2종만 새 바이트다.
#[test]
fn engine_documents_match_spike_documents_except_positional_series_delete() {
    let mut same = 0usize;
    let mut logical_only = 0usize;
    for v in b2_variants() {
        for ext in ["hwpx", "hwp"] {
            let src = manifest(&format!("samples/chart/{}/{}.{ext}", v.folder, v.stem));
            let name = format!("{}-{}.{ext}", v.stem, v.label);
            let is_hwpx = ext == "hwpx";

            // 엔진 경로.
            let mut engine = core_of(&src);
            let Some(edits) = b2_engine_edits(&engine, v.stem, v.label) else {
                continue; // 경계 2종
            };
            assert_eq!(set_chart(&mut engine, &edits)["ok"], true, "{name}");
            let engine_bytes = if is_hwpx {
                engine.export_hwpx_native().expect("저장")
            } else {
                engine.export_hwp_native().expect("저장")
            };

            // 스파이크 경로 — 같은 원본에 문자열 수술을 주입.
            let (_legacy, ooxml) =
                chart_streams(&std::fs::read(&src).expect("샘플")).expect("차트 스트림");
            let xml = String::from_utf8(ooxml).expect("UTF-8");
            let surgery = (v.surgery)(&xml);
            let mut spike = core_of(&src);
            let spike_bytes = if is_hwpx {
                replace_chart_representations(&mut spike, surgery.as_bytes(), surgery.as_bytes());
                spike.export_hwpx_native().expect("저장")
            } else {
                replace_chart_nested_only(&mut spike, surgery.as_bytes());
                spike.export_hwp_native().expect("저장")
            };

            if v.label == "계열삭제" {
                assert_ne!(
                    engine_bytes, spike_bytes,
                    "{name}: 위치 기반 계열삭제는 바이트가 달라야 정상"
                );
                let e = b2_envelope(&DocumentCore::from_bytes(&engine_bytes).expect("재개방"));
                let s = b2_envelope(&DocumentCore::from_bytes(&spike_bytes).expect("재개방"));
                assert_eq!(b2_series_names(&e), b2_series_names(&s), "{name}");
                assert_eq!(b2_labels(&e), b2_labels(&s), "{name}");
                for i in 0..b2_series_names(&e).len() {
                    assert_eq!(b2_values(&e, i), b2_values(&s, i), "{name} 계열 {i}");
                }
                logical_only += 1;
            } else {
                assert_eq!(
                    engine_bytes, spike_bytes,
                    "{name}: 엔진 산출 ≠ 스파이크 산출"
                );
                same += 1;
            }
        }
    }
    assert_eq!(
        (same, logical_only),
        (20, 4),
        "변종 10종 × 2 바이트 동일 + 계열삭제 2종 × 2 논리 동일"
    );
}
