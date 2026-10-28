//! [#6053] 구조 편집의 계열 정체 보존 계약.
//!
//! rhwp 의 구조 편집은 위치 기반이라 비꼬리 삽입·삭제에서 계열의 이름·값만 밀리고
//! 스타일(`c:spPr`·`c:marker`)은 자리에 남았다 — 저가가 종가의 마커를 물려받은 것이 그
//! 증거였다. 정체 추론 경로는 원본↔목표 계열 대응을 세워 새 계열만 기본 스타일로 끼우고
//! (`insertSeries`), 지운 계열만 들어내며(`removeSeries`), 생존 계열은 자기 스타일을 지킨다.
//!
//! 정답지는 한컴 편집기 산출의 OOXML 모양이다(계획 §2 표) — 한컴을 부르지 않고 스캐너의
//! 스타일 구간(`sp_pr_span`·`symbol_span`)으로 단언한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::queries::chart_extract::collect_charts;
use rhwp::document_core::DocumentCore;
use rhwp::ooxml_chart::data::{scan_chart_values, ChartData};
use rhwp::parser::ole_container::all_ole_streams;

const OOXML_STREAM: &str = "OOXMLChartContents";

fn ohlc(ext: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("samples/chart/기타/시가고가저가종가.{ext}"))
}

fn core_of(path: &Path) -> DocumentCore {
    let bytes = std::fs::read(path).expect("샘플 읽기");
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{}: 코어 {e:?}", path.display()))
}

/// 현재 행렬에서 출발하는 `structure: true` 편집 입력.
fn structure_edits(core: &DocumentCore, mutate: impl FnOnce(&mut serde_json::Value)) -> String {
    let raw = core.get_chart_data_by_index_native(0).expect("차트 읽기");
    let data: serde_json::Value = serde_json::from_str(&raw).expect("JSON");
    let mut e = serde_json::json!({
        "labels": data["labels"],
        "series": data["series"].as_array().unwrap().iter().map(|s| {
            serde_json::json!({ "name": s["name"], "values": s["values"] })
        }).collect::<Vec<_>>(),
        "structure": true,
    });
    mutate(&mut e);
    e.to_string()
}

fn set_chart(core: &mut DocumentCore, edits: &str) -> serde_json::Value {
    let raw = core
        .set_chart_data_by_index_native(0, edits)
        .expect("주소는 유효하다");
    serde_json::from_str(&raw).expect("봉투 JSON")
}

fn ops_with_at(out: &serde_json::Value) -> Vec<(String, u64)> {
    out["changed"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| {
                    v["op"]
                        .as_str()
                        .map(|op| (op.to_string(), v["at"].as_u64().unwrap_or(u64::MAX)))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// ①(있으면)과 ② 의 OOXML — 두 표현이 같은 정체 결과를 실어야 한다.
fn representations(core: &DocumentCore) -> Vec<(&'static str, Vec<u8>)> {
    let chart = collect_charts(core.document())[0].clone();
    let mut out = Vec::new();
    if let Some(i) = chart.zip_part {
        out.push(("①", core.document().bin_data_content[i].data.load()));
    }
    let nested = core.document().bin_data_content[chart.nested_copy.expect("②")]
        .data
        .load();
    let ooxml = all_ole_streams(&nested)
        .expect("CFB")
        .into_iter()
        .find(|(p, _)| p.trim_start_matches('/') == OOXML_STREAM)
        .map(|(_, d)| d)
        .expect("②");
    out.push(("②", ooxml));
    out
}

fn names(data: &ChartData) -> Vec<String> {
    data.series
        .iter()
        .map(|s| s.name.clone().unwrap_or_default())
        .collect()
}

/// 계열 하나의 §2 표 판정 — (marker 가 명시 `none` 인가, 계열 `spPr` 에 `a:ln noFill` 이 있는가).
fn style_of(xml: &str, data: &ChartData, i: usize) -> (bool, bool) {
    let s = &data.series[i];
    let symbol_none = s
        .symbol_span
        .clone()
        .is_some_and(|sp| xml[sp].contains(r#"val="none""#));
    let ln_no_fill = s
        .sp_pr_span
        .clone()
        .is_some_and(|sp| xml[sp].contains("<a:noFill/>"));
    (symbol_none, ln_no_fill)
}

fn assert_idx_sequential(xml: &str, data: &ChartData, ctx: &str) {
    for (i, s) in data.series.iter().enumerate() {
        assert_eq!(
            &xml[s.idx_span.clone().expect("idx")],
            &i.to_string(),
            "{ctx}: 계열 {i} idx"
        );
        assert_eq!(
            &xml[s.order_span.clone().expect("order")],
            &i.to_string(),
            "{ctx}: 계열 {i} order"
        );
    }
}

/// 중간 삽입 — 새 계열만 기본 스타일(Auto 마커·기본 선)이고, 밀려난 저가는 자기 `symbol none`
/// 을 지키며, 재번호가 0..n-1 로 선다. 계획 §2 표 그대로다.
#[test]
fn stock_middle_insert_gives_the_new_series_default_style() {
    for ext in ["hwpx", "hwp"] {
        let path = ohlc(ext);
        let mut core = core_of(&path);
        let e = structure_edits(&core, |e| {
            let n = e["series"][0]["values"].as_array().unwrap().len();
            let arr = e["series"].as_array_mut().unwrap();
            arr.insert(
                1,
                serde_json::json!({ "name": "새 계열", "values": vec!["11"; n] }),
            );
        });
        let out = set_chart(&mut core, &e);
        assert_eq!(out["ok"], true, "{}: {out}", path.display());
        assert!(
            ops_with_at(&out).contains(&("insertSeries".to_string(), 1)),
            "{}: insertSeries at=1 이 없다 — {out}",
            path.display()
        );

        for (rep, xml) in representations(&core) {
            let ctx = format!("{}{rep}", path.display());
            let data = scan_chart_values(&xml).expect("재스캔");
            let xml = std::str::from_utf8(&xml).expect("UTF-8");
            assert_eq!(
                names(&data),
                ["시가", "새 계열", "고가", "저가", "종가"],
                "{ctx}"
            );
            assert_idx_sequential(xml, &data, &ctx);

            // 계획 §2 표 — (marker 명시 none, a:ln noFill).
            // 시가·고가·저가 = (none, noFill) / 새 계열 = (Auto, 없음) / 종가 = (Auto, noFill).
            for i in [0usize, 2, 3] {
                assert_eq!(style_of(xml, &data, i), (true, true), "{ctx}: 계열 {i}");
            }
            assert_eq!(
                data.series[1].sp_pr_span, None,
                "{ctx}: 새 계열에 spPr 이 남았다 — 템플릿 스타일을 물려받았다"
            );
            assert_eq!(
                data.series[1].symbol_span, None,
                "{ctx}: 새 계열에 명시 symbol 이 남았다"
            );
            assert_eq!(style_of(xml, &data, 4), (false, true), "{ctx}: 종가");
        }
    }
}

/// 중간 삭제 — 저가만 사라지고 생존 계열은 자기 스타일(종가의 Auto 마커 포함)을 지킨다.
/// issue5652 원장의 "잔여 계열의 색이 원래 2번째 계열 색일 수 있습니다"가 선언했던 결함이
/// 바로 이 경로에서 사라진다.
#[test]
fn stock_middle_delete_keeps_survivor_styles() {
    for ext in ["hwpx", "hwp"] {
        let path = ohlc(ext);
        let mut core = core_of(&path);
        let e = structure_edits(&core, |e| {
            e["series"].as_array_mut().unwrap().remove(2);
        });
        let out = set_chart(&mut core, &e);
        assert_eq!(out["ok"], true, "{}: {out}", path.display());
        assert!(
            ops_with_at(&out).contains(&("removeSeries".to_string(), 2)),
            "{}: removeSeries at=2 가 없다 — {out}",
            path.display()
        );

        for (rep, xml) in representations(&core) {
            let ctx = format!("{}{rep}", path.display());
            let data = scan_chart_values(&xml).expect("재스캔");
            let xml = std::str::from_utf8(&xml).expect("UTF-8");
            assert_eq!(names(&data), ["시가", "고가", "종가"], "{ctx}");
            assert!(!xml.contains("저가"), "{ctx}: 지운 계열이 남았다");
            assert_idx_sequential(xml, &data, &ctx);
            // 종가가 자기 스타일을 지킨다 — 위치 기반이었다면 저가의 요소(symbol none)가
            // 종가의 값을 실은 채 끝 계열이 됐다.
            assert_eq!(style_of(xml, &data, 0), (true, true), "{ctx}: 시가");
            assert_eq!(style_of(xml, &data, 1), (true, true), "{ctx}: 고가");
            assert_eq!(style_of(xml, &data, 2), (false, true), "{ctx}: 종가");
        }
    }
}

/// 정체 대응이 모호하면(신설 계열과 기존 계열의 개명·값 변경이 동시) 현행 위치 기반으로
/// 폴백한다 — 계획 R1: 모호하면 무조건 폴백. 봉투 op 가 그 경로를 드러낸다.
#[test]
fn ambiguous_mapping_falls_back_to_the_positional_path() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let e = structure_edits(&core, |e| {
        let n = e["series"][0]["values"].as_array().unwrap().len();
        // 계열 2 를 개명하면서 값도 전부 바꾼다 — 이름으로도 값으로도 대응이 안 선다.
        e["series"][1]["name"] = serde_json::json!("다른이름");
        e["series"][1]["values"] = serde_json::json!(vec!["77"; n]);
        // 동시에 중간에 새 계열을 끼운다.
        e["series"].as_array_mut().unwrap().insert(
            1,
            serde_json::json!({ "name": "새 계열", "values": vec!["11"; n] }),
        );
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    let ops: Vec<String> = ops_with_at(&out).into_iter().map(|(op, _)| op).collect();
    assert!(
        ops.iter().any(|op| op == "appendSeries"),
        "폴백은 레거시 꼬리 신설이어야 한다 — {out}"
    );
    assert!(
        !ops.iter().any(|op| op == "insertSeries"),
        "모호한 대응이 정체 경로를 탔다 — {out}"
    );
    // 폴백이어도 목표 행렬은 정확히 실린다(self-check 계약).
    let after: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("읽기")).expect("JSON");
    let names: Vec<&str> = after["series"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["계열 1", "새 계열", "다른이름", "계열 3"]);
}

/// 제자리 개명(값 일치로 생존 판정)과 삽입이 동시면 정체 경로가 선다 — 개명은 대응쌍의
/// `renameSeries` 로, 신설은 `insertSeries` 로 갈린다.
#[test]
fn rename_in_place_with_insert_stays_on_the_identity_path() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/chart/세로막대형/묶은세로막대형.hwpx");
    let mut core = core_of(&path);
    let e = structure_edits(&core, |e| {
        let n = e["series"][0]["values"].as_array().unwrap().len();
        // 값은 그대로 두고 이름만 바꾼다 — 값 일치가 생존을 증명한다.
        e["series"][1]["name"] = serde_json::json!("이름바뀐계열");
        e["series"].as_array_mut().unwrap().insert(
            1,
            serde_json::json!({ "name": "새 계열", "values": vec!["11"; n] }),
        );
    });
    let out = set_chart(&mut core, &e);
    assert_eq!(out["ok"], true, "{out}");
    let ops = ops_with_at(&out);
    assert!(ops.contains(&("insertSeries".to_string(), 1)), "{out}");
    assert!(
        ops.iter().any(|(op, _)| op == "renameSeries"),
        "개명이 대응쌍 편집으로 실리지 않았다 — {out}"
    );
    let after: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).expect("읽기")).expect("JSON");
    let names: Vec<&str> = after["series"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["계열 1", "새 계열", "이름바뀐계열", "계열 3"]);
}
