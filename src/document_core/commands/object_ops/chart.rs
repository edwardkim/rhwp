//! [#4100] 차트 숫자 데이터 native 명령 (B1 엔진축).
//!
//! 주소 지정은 기존 개체 API 와 동형이다 — `(section_idx, parent_para_idx, control_idx)`
//! 3인자(`picture.rs` 선례). **주소·타입 오류만 `Err`** 이고, 데이터 문제는 `Ok` + 부정
//! 봉투(`{"ok":false,"invalid":[…]}`)로 돌려준다. 검증기를 코어와 CLI 두 곳으로 가르지
//! 않기 위해서다 — CLI 는 이 봉투를 그대로 실어 나른다.

use serde::Deserialize;

use crate::document_core::queries::chart_extract::{
    collect_charts, nested_chart_xml, zip_chart_xml, ChartRef, ChartSource,
};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::ooxml_chart::data::{
    scan_chart_values, ChartData, ChartScanError, ChartSeries, PlotKind, SeriesAxis,
};
use crate::ooxml_chart::patch::{
    apply_chart_edits, is_safe_text, ChartEdit, EditTarget, ValueEdit,
};
use crate::serializer::ole_container::replace_ole_stream;

/// 중첩 CFB 안 OOXML 차트 스트림 이름.
const OOXML_STREAM: &str = "OOXMLChartContents";

/// 부정 봉투 한 건. CLI `invalid[]` 와 같은 모양(`reason` + `message`)이다.
fn invalid(reason: &str, message: String) -> serde_json::Value {
    serde_json::json!({ "reason": reason, "message": message })
}

fn refused(reason: &str, message: String) -> String {
    serde_json::json!({ "ok": false, "invalid": [invalid(reason, message)] }).to_string()
}

/// 스캔 거부의 reason 매핑.
///
/// `chartScan` 은 "XML 자체가 깨졌다"(비 UTF-8·파싱 실패·계열 없음)이고, 비순차
/// `c:pt idx` 는 **정상 XML 의 미지원 형상**이라 별도 reason 으로 가른다 — 판정은
/// 데이터이므로 에이전트가 메시지 파싱 없이 reason 으로 분기할 수 있어야 한다.
fn scan_refusal(e: &ChartScanError) -> String {
    let reason = match e {
        ChartScanError::NonSequentialPointIndex { .. } => "nonSequentialPointIndex",
        _ => "chartScan",
    };
    refused(reason, e.to_string())
}

/// 두 표현을 같은 논리 차트로 볼 수 있는가.
///
/// 바이트 동일성은 요구하지 않는다. OOXML은 확장 속성·요소 순서 같은 편집 밖 바이트가
/// 다를 수 있으므로, B1이 주소로 삼는 계열/축/점의 `idx`와 텍스트만 비교한다. 이 계약이
/// 깨진 문서는 어느 사본을 기준으로 고쳐도 다른 사본의 의미를 덮어쓸 수 있으므로 쓴다.
///
/// [#5652] 논리 필드만 명시 비교한다 — 구조 좌표(`span`·`element_span`·`*_shape`)는 ①②의
/// 바이트 오프셋이라 같을 이유가 없고, `PartialEq` 파생을 쓰면 포맷 차이만으로 어긋난다.
/// plot 종류는 종류별 가드의 입력이라 논리 필드로 본다.
fn same_chart_data(left: &ChartData, right: &ChartData) -> bool {
    left.series.len() == right.series.len()
        && left.series.iter().zip(&right.series).all(|(left, right)| {
            left.name == right.name
                && left.axis == right.axis
                && left.plot == right.plot
                && left.labels_multi_level == right.labels_multi_level
                && left.labels.len() == right.labels.len()
                && left.values.len() == right.values.len()
                && left
                    .labels
                    .iter()
                    .zip(&right.labels)
                    .all(|(left, right)| left.idx == right.idx && left.text == right.text)
                && left
                    .values
                    .iter()
                    .zip(&right.values)
                    .all(|(left, right)| left.idx == right.idx && left.text == right.text)
        })
}

fn representation_mismatch() -> String {
    refused(
        "representationMismatch",
        "Chart/chartN.xml(①)과 OOXMLChartContents(②)의 계열·축·라벨·값이 다릅니다. \
         어느 한쪽을 기준으로 다른 사본을 덮어쓰지 않도록 아무것도 기록하지 않습니다."
            .to_string(),
    )
}

/// ①과 ②를 각각 스캔한 결과.
///
/// HWP5는 ②만 가진다. HWPX는 두 표현이 모두 있을 때만 ①을 기록할 수 있으며, 패치는
/// 각 표현의 원본 XML과 그 원본에서 얻은 byte span을 짝으로 유지한다.
struct ChartRepresentations {
    zip: Option<(Vec<u8>, ChartData)>,
    nested: Option<(Vec<u8>, ChartData)>,
}

impl ChartRepresentations {
    fn primary(&self) -> Option<(&ChartData, ChartSource)> {
        self.zip
            .as_ref()
            .map(|(_, data)| (data, ChartSource::ZipPart))
            .or_else(|| {
                self.nested
                    .as_ref()
                    .map(|(_, data)| (data, ChartSource::NestedCopy))
            })
    }

    fn nested_for_write(&self) -> Result<(&[u8], &ChartData), String> {
        self.nested
            .as_ref()
            .map(|(xml, data)| (xml.as_slice(), data))
            .ok_or_else(|| {
                refused(
                    "nestedCopyNotFound",
                    "중첩 CFB 사본을 특정하지 못했습니다 — <hp:switch> 의 fallback OLE 가 없습니다. \
                     ①만 기록하면 HWP 변환에서 편집이 사라지므로 아무것도 쓰지 않습니다."
                        .to_string(),
                )
            })
    }
}

/// 두 차트 표현을 독립적으로 읽고, 모두 존재하면 논리 데이터의 동일성을 확인한다.
fn scan_chart_representations(
    document: &crate::model::document::Document,
    chart: &ChartRef,
) -> Result<ChartRepresentations, String> {
    let zip = if chart.zip_part.is_some() {
        let xml = zip_chart_xml(document, chart).ok_or_else(|| {
            refused(
                "zipPartMissing",
                "Chart/chartN.xml(①) 슬롯이 비어 있거나 읽을 수 없습니다. ②를 ①에 복제하지 않고 \
                 아무것도 기록합니다."
                    .to_string(),
            )
        })?;
        let data = scan_chart_values(&xml).map_err(|e| scan_refusal(&e))?;
        Some((xml, data))
    } else {
        None
    };

    let nested = if chart.nested_copy.is_some() {
        let xml = nested_chart_xml(document, chart).ok_or_else(|| {
            refused(
                "nestedCopyNotFound",
                "중첩 CFB의 OOXMLChartContents(②)를 읽을 수 없습니다. ①만 기록하면 HWP 변환에서 \
                 편집이 사라지므로 아무것도 기록하지 않습니다."
                    .to_string(),
            )
        })?;
        let data = scan_chart_values(&xml).map_err(|e| scan_refusal(&e))?;
        Some((xml, data))
    } else {
        None
    };

    if let (Some((_, zip_data)), Some((_, nested_data))) = (&zip, &nested) {
        if !same_chart_data(zip_data, nested_data) {
            return Err(representation_mismatch());
        }
    }

    if zip.is_none() && nested.is_none() {
        return Err(refused(
            "chartStreamMissing",
            "차트 XML 을 읽을 수 없습니다 — 두 표현 모두 비어 있습니다.".to_string(),
        ));
    }

    Ok(ChartRepresentations { zip, nested })
}

/// 라벨을 **가진** 첫 계열의 라벨.
///
/// `series[0]` 을 무조건 보면 안 된다 — `c:cat` 이 일부 계열에만 있는 실사용 문서가 있다
/// (`samples/issue2006/1790387_prep_final_report.hwpx`: 6계열 중 5계열만 `c:cat` 보유).
/// 첫 계열을 맹신하면 라벨이 통째로 비어 CSV 가 값을 잃는다.
fn label_texts(data: &ChartData) -> Vec<&str> {
    data.series
        .iter()
        .find(|s| !s.labels.is_empty())
        .map(|s| s.labels.iter().map(|p| p.text.as_str()).collect())
        .unwrap_or_default()
}

/// 라벨(카테고리 또는 분산형 X)이 **가진 계열들 사이에서** 같은가.
///
/// OOXML 은 계열마다 다른 라벨/X 를 허용하지만 CSV 는 한 열로 표현한다. 코퍼스는 전건
/// 일치하지만 **포맷의 보장이 아니라서** 표지를 실어 CSV 층이 거부할 수 있게 한다.
/// 라벨이 아예 없는 계열은 판정에서 뺀다 — 없는 것과 다른 것은 다르다.
fn labels_shared(data: &ChartData) -> bool {
    let head = label_texts(data);
    data.series
        .iter()
        .filter(|s| !s.labels.is_empty())
        .all(|s| {
            s.labels
                .iter()
                .map(|p| p.text.as_str())
                .eq(head.iter().copied())
        })
}

fn chart_data_json(chart: &ChartRef, data: &ChartData, source: ChartSource) -> serde_json::Value {
    let axis = match data.series.first().map(|s| s.axis) {
        Some(SeriesAxis::Scatter) => "scatter",
        _ => "category",
    };
    let labels = label_texts(data);

    serde_json::json!({
        "ok": true,
        "chart": chart.index + 1,
        "axis": axis,
        "source": match source {
            ChartSource::ZipPart => "zipPart",
            ChartSource::NestedCopy => "nestedCopy",
        },
        "representations": {
            "zipPart": chart.zip_part.is_some(),
            "nestedCopy": chart.nested_copy.is_some(),
        },
        "labelsShared": labels_shared(data),
        "labelsMultiLevel": data.series.iter().any(|s| s.labels_multi_level),
        "labels": labels,
        "series": data
            .series
            .iter()
            .map(|s| serde_json::json!({
                "name": s.name,
                "values": s.values.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
    })
}

// ---------------------------------------------------------------------------
// 쓰기 — 입력 형태와 검증
// ---------------------------------------------------------------------------

/// `set_chart_data_native` 입력.
///
/// **행렬 형태**다 — CSV 한 장과 같은 모양이라 `csv-to-chart` 가 얇아지고, 행·열 수
/// 대조 같은 검증이 코어 한 곳에만 남는다(`agent_surface_playbook` 규칙 2).
///
/// 값은 **문자열**로만 받는다. JSON 숫자로 받으면 `4.3` 이 `4.30` 으로 되쓰일 수 있어
/// 무편집 왕복의 바이트 동일이 깨진다.
///
/// [#5652] `structure: true` 면 행렬은 **목표 상태**다 — 행(점)·열(계열) 수와 계열명·라벨이
/// 차트와 달라도 되고, 치수 차이는 **위치 기반 꼬리 증감**으로 적용된다(행이 늘면 꼬리에
/// 점 추가, 줄면 꼬리 점 삭제; 계열은 마지막 계열 복제/꼬리 삭제). `false`(기본)면 B1 그대로 —
/// 개수·이름·라벨이 다르면 거부한다. 의도 없이 개수가 어긋난 입력은 여전히 사고다.
#[derive(Debug, Deserialize)]
struct ChartEdits {
    /// 카테고리 라벨(분산형이면 X). `structure: false` 면 대조만 하고 분산형에서만 기록한다.
    /// `structure: true` 면 목표 라벨이다 — 행 수가 바뀌면 필수.
    #[serde(default)]
    labels: Option<Vec<String>>,
    series: Vec<SeriesEdits>,
    /// 검증과 diff 만 하고 쓰지 않는다.
    #[serde(default, rename = "dryRun")]
    dry_run: bool,
    /// [#5652] 구조 편집 의도.
    #[serde(default)]
    structure: bool,
}

#[derive(Debug, Deserialize)]
struct SeriesEdits {
    /// `structure: false` 면 주면 대조만 한다. `structure: true` 면 목표 계열명이다 —
    /// 기존 계열은 다르면 바꾸고(`c:tx` 가 없으면 거부), 신설 계열은 템플릿에 이름이 있으면 필수.
    #[serde(default)]
    name: Option<String>,
    values: Vec<String>,
}

/// `<c:v>` 에 넣어도 되는 수치 표기인가.
///
/// 비유한(`inf`/`NaN`)과 빈 문자열은 거부한다. 빈 값은 결측 표현이라 요소 자체를 다시
/// 써야 하고 그건 구조 변경이다.
fn is_number(text: &str) -> bool {
    !text.is_empty() && text.parse::<f64>().is_ok_and(f64::is_finite)
}

/// 검증 결과 — 하나라도 있으면 **한 칸도 쓰지 않는다**.
///
/// [#5652] `structure` 로 갈린다 — 없으면 B1 의 네 거부(개수·이름·라벨)가 그대로 서고,
/// 있으면 [`validate_structure`] 가 목표 행렬의 규칙과 종류별 가드를 본다.
fn validate(data: &ChartData, edits: &ChartEdits) -> Vec<serde_json::Value> {
    if edits.structure {
        validate_structure(data, edits)
    } else {
        validate_values(data, edits)
    }
}

/// B1 검증 — 구조는 바꾸지 않는다.
fn validate_values(data: &ChartData, edits: &ChartEdits) -> Vec<serde_json::Value> {
    let mut out = Vec::new();

    if edits.series.len() != data.series.len() {
        out.push(serde_json::json!({
            "reason": "seriesCountMismatch",
            "expected": data.series.len(),
            "actual": edits.series.len(),
            "message": format!(
                "계열 수 {} 가 차트의 계열 수 {} 와 다릅니다 — 계열 신설·삭제는 structure:true 로만 합니다.",
                edits.series.len(), data.series.len()
            ),
        }));
        return out; // 계열 수가 다르면 이후 대조가 무의미하다.
    }

    let scatter = data.series.first().map(|s| s.axis) == Some(SeriesAxis::Scatter);

    for (i, (want, have)) in edits.series.iter().zip(&data.series).enumerate() {
        if want.values.len() != have.values.len() {
            out.push(serde_json::json!({
                "reason": "valueCountMismatch",
                "series": i,
                "expected": have.values.len(),
                "actual": want.values.len(),
                "message": format!(
                    "계열 {} 의 값 개수 {} 가 차트의 {} 와 다릅니다 — 점 신설·삭제는 structure:true 로만 합니다.",
                    i, want.values.len(), have.values.len()
                ),
            }));
            continue;
        }
        if let Some(name) = &want.name {
            // CSV는 `c:tx` 부재(None)와 빈 계열명(Some(""))을 구분해 표현할 수 없다.
            // B1은 계열명을 쓰지 않으므로 이 비교에서만 둘을 같은 무편집 값으로 본다.
            if have.name.as_deref().unwrap_or_default() != name {
                out.push(serde_json::json!({
                    "reason": "seriesNameMismatch",
                    "series": i,
                    "expected": have.name,
                    "actual": name,
                    "message": format!("계열 {} 의 이름이 다릅니다 — 계열명 변경은 structure:true 로만 합니다.", i),
                }));
            }
        }
        for (p, (text, point)) in want.values.iter().zip(&have.values).enumerate() {
            if text != &point.text && !is_number(text) {
                out.push(serde_json::json!({
                    "reason": "notANumber", "series": i, "point": p, "value": text,
                    "message": format!("계열 {} 점 {} 의 값 `{}` 이 수치가 아닙니다.", i, p, text),
                }));
            } else if text != &point.text && point.span.is_none() {
                out.push(serde_json::json!({
                    "reason": "valueNotPatchable", "series": i, "point": p,
                    "message": format!(
                        "계열 {} 점 {} 은 빈 값(<c:v/>)이라 제자리 치환 대상이 아닙니다.", i, p
                    ),
                }));
            }
        }
    }

    if let Some(labels) = &edits.labels {
        validate_labels(data, labels, scatter, &mut out);
    }
    out
}

fn validate_labels(
    data: &ChartData,
    labels: &[String],
    scatter: bool,
    out: &mut Vec<serde_json::Value>,
) {
    // CSV 첫 열은 카테고리와 분산형 X 모두 하나뿐이다. 계열마다 다르면 같은 행 번호가
    // 다른 뜻을 가리켜 조용한 오편집이 된다. 값만 주소로 편집하는 native 호출은 labels를
    // 생략할 수 있지만, CSV가 넘긴 labels는 반드시 공유됨을 증명해야 한다.
    if !labels_shared(data) {
        out.push(shared_labels_refusal(scatter));
        return;
    }

    // 라벨이 없는 계열은 대조에서 뺀다 — 없는 것과 다른 것은 다르다.
    for (i, series) in data
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.labels.is_empty())
    {
        if labels.len() != series.labels.len() {
            out.push(serde_json::json!({
                "reason": if scatter { "valueCountMismatch" } else { "categoryMismatch" },
                "series": i,
                "expected": series.labels.len(),
                "actual": labels.len(),
                "message": format!(
                    "라벨 개수 {} 가 계열 {} 의 {} 와 다릅니다 — 행 신설·삭제는 structure:true 로만 합니다.",
                    labels.len(), i, series.labels.len()
                ),
            }));
            continue;
        }
        for (p, (text, point)) in labels.iter().zip(&series.labels).enumerate() {
            if scatter {
                if text != &point.text && !is_number(text) {
                    out.push(serde_json::json!({
                        "reason": "notANumber", "series": i, "point": p, "value": text,
                        "message": format!("분산형 X `{}` 이 수치가 아닙니다.", text),
                    }));
                } else if text != &point.text && point.span.is_none() {
                    out.push(serde_json::json!({
                        "reason": "valueNotPatchable", "series": i, "point": p,
                        "message": format!("계열 {} X {} 는 빈 값이라 치환 대상이 아닙니다.", i, p),
                    }));
                }
            } else if text != &point.text {
                out.push(serde_json::json!({
                    "reason": "categoryMismatch", "point": p,
                    "expected": point.text, "actual": text,
                    "message": format!(
                        "카테고리 라벨 {} 이 다릅니다 — 라벨 변경은 structure:true 로만 합니다.", p
                    ),
                }));
                break; // 라벨은 한 열이라 첫 불일치로 충분하다.
            }
        }
        if !scatter {
            break; // 카테고리 대조는 계열 0 하나면 된다.
        }
    }
}

fn shared_labels_refusal(scatter: bool) -> serde_json::Value {
    serde_json::json!({
        "reason": if scatter { "sharedXRequired" } else { "sharedCategoryRequired" },
        "message": if scatter {
            "계열마다 X 값이 달라 CSV 의 X 한 열로 표현할 수 없습니다."
        } else {
            "계열마다 카테고리 라벨이 달라 CSV 의 한 라벨 열로 표현할 수 없습니다."
        },
    })
}

// ---------------------------------------------------------------------------
// [#5652] 구조 편집 — 목표 행렬의 규칙과 종류별 가드
// ---------------------------------------------------------------------------

/// 계열이 라벨 블록(`c:cat`/`c:xVal`)을 가졌는가 — 점이 0개인 빈 캐시도 블록은 블록이다.
fn has_labels(series: &ChartSeries) -> bool {
    !series.labels.is_empty() || series.labels_shape.is_some()
}

/// 블록의 개수를 바꿀 수 있는가 — 삽입 앵커·`ptCount`·점 요소 구간이 있어야 한다.
fn block_resizable(series: &ChartSeries, target: EditTarget) -> bool {
    let (shape, points) = match target {
        EditTarget::Value => (&series.values_shape, &series.values),
        EditTarget::Label => (&series.labels_shape, &series.labels),
    };
    shape
        .as_ref()
        .is_some_and(|s| s.insert_at.is_some() && s.pt_count.is_some())
        && points.iter().all(|p| p.element_span.is_some())
}

fn block_name(scatter: bool, target: EditTarget) -> &'static str {
    match (scatter, target) {
        (false, EditTarget::Value) => "val",
        (false, EditTarget::Label) => "cat",
        (true, EditTarget::Value) => "yVal",
        (true, EditTarget::Label) => "xVal",
    }
}

/// [#5652] 구조 편집 검증 — 목표 행렬이 차트에 적용 가능한가.
///
/// 순서: 행렬 모양 → 삭제 하한 → 종류별 가드 → 라벨 규칙 → 값 → 이름 → 삽입 가능성.
/// 한컴은 잘못된 구조(원형 2계열, 주식형 3계열)를 막지 않고 조용히 무시하거나 틀리게
/// 그리므로(#5447 §3-4) 여기서 fail-closed 로 막는다.
fn validate_structure(data: &ChartData, edits: &ChartEdits) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let k = data.series.len();
    let k_new = edits.series.len();
    let scatter = data.series.first().map(|s| s.axis) == Some(SeriesAxis::Scatter);

    // 1) 행렬 모양 — 직사각형이어야 한다(CSV 동형).
    if k_new == 0 {
        out.push(serde_json::json!({
            "reason": "lastSeriesDeleteRefused", "expected": 1, "actual": 0,
            "message": "마지막 계열은 지울 수 없습니다 — 계열 0개인 차트는 다시 읽을 수 없습니다.",
        }));
        return out;
    }
    let rows = edits.series[0].values.len();
    for (i, s) in edits.series.iter().enumerate().skip(1) {
        if s.values.len() != rows {
            out.push(serde_json::json!({
                "reason": "rowCountMismatch", "series": i, "expected": rows, "actual": s.values.len(),
                "message": format!(
                    "계열 {} 의 값 개수 {} 가 계열 0 의 {} 와 다릅니다 — 행렬은 직사각형이어야 합니다.",
                    i, s.values.len(), rows
                ),
            }));
        }
    }
    if !out.is_empty() {
        return out;
    }
    if rows == 0 {
        out.push(serde_json::json!({
            "reason": "lastPointDeleteRefused", "expected": 1, "actual": 0,
            "message": "마지막 점은 지울 수 없습니다 — 점 0개인 블록은 다시 읽을 수 없습니다.",
        }));
        return out;
    }

    // 2) 종류별 가드 — 계열 수 변경.
    if k_new != k {
        if data
            .series
            .iter()
            .any(|s| matches!(s.plot, PlotKind::Pie | PlotKind::OfPie))
        {
            out.push(serde_json::json!({
                "reason": "pieSeriesCountFixed", "expected": k, "actual": k_new,
                "message": "원형·3D원형·원형대원형 차트는 계열 수가 1 로 고정입니다 — 한컴은 2번째 계열을 조용히 무시합니다(#5447).",
            }));
        }
        if data.series.iter().any(|s| s.plot == PlotKind::Stock) {
            out.push(serde_json::json!({
                "reason": "stockSeriesCountFixed", "expected": k, "actual": k_new,
                "message": "주식형 차트는 계열 수가 종류에 묶입니다(HLC=3 / OHLC=4) — 변경은 종류 변환(B3)으로만 합니다. 한컴은 캔들 장치를 남긴 채 틀리게 그립니다(#5447).",
            }));
        }
    }
    // 다층 카테고리 — 행 수 변화나 라벨 지정은 거부, 값만 바꾸는 건 허용.
    let row_change_anywhere = data
        .series
        .iter()
        .take(k_new)
        .any(|s| s.values.len() != rows);
    if data.series.iter().any(|s| s.labels_multi_level)
        && (row_change_anywhere || edits.labels.is_some())
    {
        out.push(serde_json::json!({
            "reason": "multiLevelLabelsUnsupported",
            "message": "다층 카테고리(multiLvlStrRef) 차트의 행·라벨 구조 편집은 지원하지 않습니다 — 값만 바꿀 수 있습니다.",
        }));
    }
    if !out.is_empty() {
        return out;
    }

    // 3) 라벨 규칙.
    let labeled_rows_change = data
        .series
        .iter()
        .take(k_new)
        .any(|s| has_labels(s) && s.values.len() != rows);
    match &edits.labels {
        None => {
            if labeled_rows_change {
                out.push(serde_json::json!({
                    "reason": if scatter { "scatterXYMismatch" } else { "labelsRequired" },
                    "expected": rows,
                    "message": if scatter {
                        "분산형에서 행 수를 바꾸려면 X(labels)를 같은 개수로 함께 주어야 합니다 — xVal/yVal 은 동기로만 바뀝니다."
                    } else {
                        "행 수를 바꾸려면 카테고리 라벨(labels)을 목표 행 수만큼 함께 주어야 합니다."
                    },
                }));
            }
        }
        Some(labels) => {
            if labels.len() != rows {
                out.push(serde_json::json!({
                    "reason": if scatter { "scatterXYMismatch" } else { "labelCountMismatch" },
                    "expected": rows, "actual": labels.len(),
                    "message": format!(
                        "라벨 개수 {} 가 목표 행 수 {} 와 다릅니다.", labels.len(), rows
                    ),
                }));
            } else if !labels_shared(data) {
                out.push(shared_labels_refusal(scatter));
            } else {
                // 라벨 텍스트 — 바뀌는 칸·새 칸만 본다.
                let have = label_texts(data);
                for (p, text) in labels.iter().enumerate() {
                    let changed = have.get(p) != Some(&text.as_str());
                    if !changed {
                        continue;
                    }
                    if scatter && !is_number(text) {
                        out.push(serde_json::json!({
                            "reason": "notANumber", "point": p, "value": text,
                            "message": format!("분산형 X `{}` 이 수치가 아닙니다.", text),
                        }));
                    } else if !is_safe_text(text) {
                        out.push(serde_json::json!({
                            "reason": "unsafeText", "point": p, "value": text,
                            "message": format!("라벨 `{}` 에 XML 특수문자(<, >, &, 제어문자)가 있습니다 — 이스케이프하지 않고 거부합니다.", text),
                        }));
                    } else if p < have.len() {
                        // 기존 점 — 라벨을 가진 계열마다 구간이 있어야 한다.
                        for (i, s) in data.series.iter().take(k_new).enumerate() {
                            if let Some(point) = s.labels.get(p) {
                                if point.text != *text && point.span.is_none() {
                                    out.push(serde_json::json!({
                                        "reason": "valueNotPatchable", "series": i, "point": p,
                                        "message": format!("계열 {} 라벨 {} 은 빈 값이라 치환 대상이 아닙니다.", i, p),
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 4) 기존 계열 — 값·이름.
    for (i, (want, have)) in edits.series.iter().zip(&data.series).enumerate() {
        for (p, text) in want.values.iter().enumerate() {
            match have.values.get(p) {
                Some(point) if point.text == *text => {}
                Some(point) => {
                    if !is_number(text) {
                        out.push(serde_json::json!({
                            "reason": "notANumber", "series": i, "point": p, "value": text,
                            "message": format!("계열 {} 점 {} 의 값 `{}` 이 수치가 아닙니다.", i, p, text),
                        }));
                    } else if point.span.is_none() {
                        out.push(serde_json::json!({
                            "reason": "valueNotPatchable", "series": i, "point": p,
                            "message": format!("계열 {} 점 {} 은 빈 값(<c:v/>)이라 제자리 치환 대상이 아닙니다.", i, p),
                        }));
                    }
                }
                None => {
                    if !is_number(text) {
                        out.push(serde_json::json!({
                            "reason": "notANumber", "series": i, "point": p, "value": text,
                            "message": format!("계열 {} 새 점 {} 의 값 `{}` 이 수치가 아닙니다.", i, p, text),
                        }));
                    }
                }
            }
        }
        if let Some(name) = &want.name {
            match &have.name {
                Some(h) if h == name => {}
                Some(_) => {
                    if !is_safe_text(name) {
                        out.push(serde_json::json!({
                            "reason": "unsafeText", "series": i, "field": "name", "value": name,
                            "message": format!("계열 {} 의 새 이름에 XML 특수문자가 있습니다.", i),
                        }));
                    } else if have.name_span.is_none() {
                        out.push(serde_json::json!({
                            "reason": "seriesNameNotPatchable", "series": i,
                            "message": format!("계열 {} 의 c:tx 계열명 캐시 구간을 특정하지 못했습니다.", i),
                        }));
                    }
                }
                None => {
                    // c:tx 가 없는 계열 — 빈 이름은 무편집, 그 밖은 넣을 자리가 없다.
                    if !name.is_empty() {
                        out.push(serde_json::json!({
                            "reason": "seriesNameNotPatchable", "series": i,
                            "message": format!("계열 {} 은 c:tx 계열명이 없어 이름을 넣을 자리가 없습니다.", i),
                        }));
                    }
                }
            }
        }
        // 행 수 변화 — 블록이 개수를 바꿀 수 있어야 한다.
        if have.values.len() != rows {
            if !block_resizable(have, EditTarget::Value) {
                out.push(serde_json::json!({
                    "reason": "pointsNotInsertable", "series": i, "block": block_name(scatter, EditTarget::Value),
                    "message": format!("계열 {} 의 값 블록은 캐시 구조 좌표(ptCount·삽입 앵커)가 없어 개수를 바꿀 수 없습니다.", i),
                }));
            }
            if has_labels(have) && !block_resizable(have, EditTarget::Label) {
                out.push(serde_json::json!({
                    "reason": "pointsNotInsertable", "series": i, "block": block_name(scatter, EditTarget::Label),
                    "message": format!("계열 {} 의 라벨 블록은 캐시 구조 좌표(ptCount·삽입 앵커)가 없어 개수를 바꿀 수 없습니다.", i),
                }));
            }
        }
    }

    // 5) 신설 계열 — 템플릿은 마지막 계열.
    if k_new > k {
        let template = &data.series[k - 1];
        if template.idx_span.is_none() || template.order_span.is_none() {
            out.push(serde_json::json!({
                "reason": "seriesNotClonable", "series": k,
                "message": "마지막 계열의 c:idx/c:order 구간을 특정하지 못해 복제할 수 없습니다.",
            }));
        }
        for (i, want) in edits.series.iter().enumerate().skip(k) {
            match &want.name {
                Some(name) if !is_safe_text(name) => {
                    out.push(serde_json::json!({
                        "reason": "unsafeText", "series": i, "field": "name", "value": name,
                        "message": format!("신설 계열 {} 의 이름에 XML 특수문자가 있습니다.", i),
                    }));
                }
                Some(_) if template.name_span.is_none() => {
                    out.push(serde_json::json!({
                        "reason": "seriesNameNotPatchable", "series": i,
                        "message": format!("신설 계열 {} 에 이름을 줬지만 템플릿(마지막 계열)에 c:tx 가 없어 넣을 자리가 없습니다.", i),
                    }));
                }
                None if template.name_span.is_some() => {
                    out.push(serde_json::json!({
                        "reason": "seriesNameRequired", "series": i,
                        "message": format!("신설 계열 {} 의 이름(name)이 필요합니다 — 템플릿 계열에 이름이 있습니다.", i),
                    }));
                }
                _ => {}
            }
            for (p, text) in want.values.iter().enumerate() {
                if !is_number(text) {
                    out.push(serde_json::json!({
                        "reason": "notANumber", "series": i, "point": p, "value": text,
                        "message": format!("신설 계열 {} 점 {} 의 값 `{}` 이 수치가 아닙니다.", i, p, text),
                    }));
                }
            }
        }
    }

    out
}

/// 바뀐 칸·구조를 편집 목록과 `changed[]` 봉투로 만든다.
///
/// [#5652] `structure: false` 는 B1 계획(값 치환, 분산형 X)이고, `true` 는 목표 행렬과의
/// 위치 기반 diff — 겹치는 칸은 치환, 남는 행·열은 꼬리 증감, 라벨은 라벨 보유 전 계열에 동기.
fn plan_edits(
    data: &ChartData,
    edits: &ChartEdits,
    scatter: bool,
) -> (Vec<ChartEdit>, Vec<serde_json::Value>) {
    let mut plan = Vec::new();
    let mut changed = Vec::new();

    let k = data.series.len();
    let k_new = edits.series.len();
    let overlap = k.min(k_new);

    for (i, (want, have)) in edits.series.iter().zip(&data.series).enumerate() {
        let keep = want.values.len().min(have.values.len());
        for (p, (text, point)) in want.values.iter().zip(&have.values).enumerate().take(keep) {
            if text != &point.text {
                plan.push(ChartEdit::Value(ValueEdit {
                    series: i,
                    point: p,
                    target: EditTarget::Value,
                    text: text.clone(),
                }));
                changed.push(serde_json::json!({
                    "series": i, "point": p, "from": point.text, "to": text,
                }));
            }
        }
        if edits.structure {
            if let Some(name) = &want.name {
                if let Some(h) = &have.name {
                    if h != name {
                        plan.push(ChartEdit::SeriesName {
                            series: i,
                            text: name.clone(),
                        });
                        changed.push(serde_json::json!({
                            "op": "renameSeries", "series": i, "from": h, "to": name,
                        }));
                    }
                }
            }
            if want.values.len() > have.values.len() {
                let texts: Vec<String> = want.values[have.values.len()..].to_vec();
                changed.push(serde_json::json!({
                    "op": "appendPoints", "series": i, "block": block_name(scatter, EditTarget::Value),
                    "from": have.values.len(), "to": want.values.len(),
                }));
                plan.push(ChartEdit::AppendPoints {
                    series: i,
                    target: EditTarget::Value,
                    texts,
                });
            } else if want.values.len() < have.values.len() {
                changed.push(serde_json::json!({
                    "op": "truncatePoints", "series": i, "block": block_name(scatter, EditTarget::Value),
                    "from": have.values.len(), "to": want.values.len(),
                }));
                plan.push(ChartEdit::TruncatePoints {
                    series: i,
                    target: EditTarget::Value,
                    keep: want.values.len(),
                });
            }
        }
    }

    if let Some(labels) = &edits.labels {
        // B1(분산형 X)과 B2(카테고리 라벨·X) 모두 — 라벨을 가진 계열마다 동기 적용.
        let apply_labels = scatter || edits.structure;
        if apply_labels {
            for (i, series) in data.series.iter().enumerate().take(overlap) {
                if !has_labels(series) {
                    continue;
                }
                let keep = labels.len().min(series.labels.len());
                for (p, (text, point)) in labels.iter().zip(&series.labels).enumerate().take(keep) {
                    if text != &point.text {
                        plan.push(ChartEdit::Value(ValueEdit {
                            series: i,
                            point: p,
                            target: EditTarget::Label,
                            text: text.clone(),
                        }));
                        changed.push(if scatter {
                            serde_json::json!({ "series": i, "x": p, "from": point.text, "to": text })
                        } else {
                            serde_json::json!({ "op": "relabel", "series": i, "point": p, "from": point.text, "to": text })
                        });
                    }
                }
                if edits.structure {
                    if labels.len() > series.labels.len() {
                        changed.push(serde_json::json!({
                            "op": "appendPoints", "series": i, "block": block_name(scatter, EditTarget::Label),
                            "from": series.labels.len(), "to": labels.len(),
                        }));
                        plan.push(ChartEdit::AppendPoints {
                            series: i,
                            target: EditTarget::Label,
                            texts: labels[series.labels.len()..].to_vec(),
                        });
                    } else if labels.len() < series.labels.len() {
                        changed.push(serde_json::json!({
                            "op": "truncatePoints", "series": i, "block": block_name(scatter, EditTarget::Label),
                            "from": series.labels.len(), "to": labels.len(),
                        }));
                        plan.push(ChartEdit::TruncatePoints {
                            series: i,
                            target: EditTarget::Label,
                            keep: labels.len(),
                        });
                    }
                }
            }
        }
    }

    if edits.structure {
        if k_new > k {
            for (i, want) in edits.series.iter().enumerate().skip(k) {
                changed.push(serde_json::json!({
                    "op": "appendSeries", "series": i, "name": want.name,
                }));
                plan.push(ChartEdit::AppendSeries {
                    name: want.name.clone(),
                    labels: edits.labels.clone(),
                    values: want.values.clone(),
                });
            }
        } else if k_new < k {
            changed.push(serde_json::json!({
                "op": "truncateSeries", "from": k, "to": k_new,
            }));
            plan.push(ChartEdit::TruncateSeries { keep: k_new });
        }
    }

    (plan, changed)
}

/// [#5652] 패치 산출을 다시 읽어 목표 행렬과 같은지 본다 — "rhwp 가 자기 산출을 다시 읽을 수
/// 있다"(수용 기준 2)를 쓰기 전에 코드로 강제한다. 다르면 한 바이트도 쓰지 않는다.
fn rescan_matches(rescan: &ChartData, edits: &ChartEdits, scatter: bool) -> Result<(), String> {
    if rescan.series.len() != edits.series.len() {
        return Err(format!(
            "계열 수 {} ≠ 목표 {}",
            rescan.series.len(),
            edits.series.len()
        ));
    }
    for (i, (want, have)) in edits.series.iter().zip(&rescan.series).enumerate() {
        let values: Vec<&str> = have.values.iter().map(|p| p.text.as_str()).collect();
        let want_values: Vec<&str> = want.values.iter().map(String::as_str).collect();
        if values != want_values {
            return Err(format!("계열 {i} 값 {values:?} ≠ 목표 {want_values:?}"));
        }
        if let Some(name) = &want.name {
            let have_name = have.name.as_deref().unwrap_or_default();
            if have_name != name {
                return Err(format!("계열 {i} 이름 {have_name:?} ≠ 목표 {name:?}"));
            }
        }
    }
    if let Some(labels) = &edits.labels {
        let want: Vec<&str> = labels.iter().map(String::as_str).collect();
        for (i, series) in rescan.series.iter().enumerate() {
            if !has_labels(series) {
                continue;
            }
            let have: Vec<&str> = series.labels.iter().map(|p| p.text.as_str()).collect();
            if have != want {
                return Err(format!(
                    "계열 {i} {} {have:?} ≠ 목표 {want:?}",
                    if scatter { "X" } else { "라벨" }
                ));
            }
        }
    }
    Ok(())
}

impl DocumentCore {
    /// 주소가 가리키는 **본문 직속** 차트를 찾는다.
    ///
    /// 컨테이너(글상자·머리말·표 셀) 안의 차트는 이 3인자 주소로 표현할 수 없다 —
    /// 그쪽은 문서 순번(`collect_charts` 의 `index`)으로 지목한다. 그림 API 와 같은
    /// 한계이며, 편집 자체는 슬롯 바이트만 건드리므로 순번 경로로는 문제없이 된다.
    fn resolve_chart_ref(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<ChartRef, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {} 범위 초과",
                control_idx
            )));
        }

        collect_charts(&self.document)
            .into_iter()
            .find(|c| {
                c.is_top_level()
                    && c.section == section_idx
                    && c.paragraph == parent_para_idx
                    && c.control == control_idx
            })
            .ok_or_else(|| HwpError::RenderError("지정된 컨트롤이 차트가 아닙니다".to_string()))
    }

    /// 차트의 숫자 데이터를 JSON 으로 읽는다.
    ///
    /// 값은 **원본 텍스트 그대로** 싣는다(`"4.3"`) — 실수로 파싱했다가 되쓰면 표기가
    /// 달라져 무편집 왕복의 바이트 동일이 깨진다.
    pub fn get_chart_data_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let chart = self.resolve_chart_ref(section_idx, parent_para_idx, control_idx)?;
        Ok(self.chart_data_at(&chart))
    }

    /// 문서 순번(0-based)으로 차트 데이터를 읽는다 — CLI `--chart N` 의 뒷면.
    pub fn get_chart_data_by_index_native(&self, index: usize) -> Result<String, HwpError> {
        let charts = collect_charts(&self.document);
        let chart = charts.get(index).ok_or_else(|| {
            HwpError::RenderError(format!(
                "차트 순번 {} 범위 초과 (차트 {}개)",
                index + 1,
                charts.len()
            ))
        })?;
        Ok(self.chart_data_at(chart))
    }

    /// 문서의 모든 차트를 문서 순서로 열거한 JSON 배열을 돌려준다.
    ///
    /// 항목은 `ChartRef` 직렬화 그대로다 — 가공 층을 두지 않아야 CLI(`--chart N`)·
    /// 코어(by_index)·studio 가 같은 순번을 본다. studio 는 이 목록을 선택 컨트롤과
    /// 대조해 순번 주소를 얻는다(#4694).
    pub fn list_charts_native(&self) -> Result<String, HwpError> {
        serde_json::to_string(&collect_charts(&self.document))
            .map_err(|e| HwpError::RenderError(format!("차트 열거 직렬화 실패: {e}")))
    }

    /// 차트의 숫자 데이터를 바꾼다 — **①②에 함께 쓴다**.
    ///
    /// ①만 고치면 HWP 변환에서 조용히 사라진다(#4055 한컴 실측, #4099 의 fold 로 계약
    /// 확정). 두 표현을 특정하지 못하면 **아무것도 쓰지 않는다** — 반쪽만 새 값인 파일이
    /// 나가는 것이 최악이라, 어디에 썼는지를 `wrote[]` 로 항상 드러낸다.
    pub fn set_chart_data_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        edits_json: &str,
    ) -> Result<String, HwpError> {
        let chart = self.resolve_chart_ref(section_idx, parent_para_idx, control_idx)?;
        Ok(self.apply_chart_edits(&chart, edits_json))
    }

    /// 문서 순번(0-based)으로 차트를 바꾼다 — CLI `--chart N` 의 뒷면이자 **정본 주소**다.
    pub fn set_chart_data_by_index_native(
        &mut self,
        index: usize,
        edits_json: &str,
    ) -> Result<String, HwpError> {
        let charts = collect_charts(&self.document);
        let chart = charts
            .get(index)
            .ok_or_else(|| {
                HwpError::RenderError(format!(
                    "차트 순번 {} 범위 초과 (차트 {}개)",
                    index + 1,
                    charts.len()
                ))
            })?
            .clone();
        Ok(self.apply_chart_edits(&chart, edits_json))
    }

    fn apply_chart_edits(&mut self, chart: &ChartRef, edits_json: &str) -> String {
        let edits: ChartEdits = match serde_json::from_str(edits_json) {
            Ok(v) => v,
            Err(e) => return refused("editsParse", format!("편집 JSON 을 읽을 수 없습니다: {e}")),
        };
        let representations = match scan_chart_representations(&self.document, chart) {
            Ok(v) => v,
            Err(response) => return response,
        };
        let (data, _) = representations.primary().expect("표현 하나 이상 확인함");
        let (nested_xml, nested_data) = match representations.nested_for_write() {
            Ok(v) => v,
            Err(response) => return response,
        };

        let invalid = validate(&data, &edits);
        if !invalid.is_empty() {
            return serde_json::json!({
                "ok": false, "chart": chart.index + 1, "invalid": invalid, "wrote": [],
            })
            .to_string();
        }

        let scatter = data.series.first().map(|s| s.axis) == Some(SeriesAxis::Scatter);
        let (plan, changed) = plan_edits(&data, &edits, scatter);

        // 바뀐 칸이 없으면 **한 바이트도 건드리지 않는다.** 슬롯을 되쓰기만 해도
        // 중첩 CFB 재포장이 섹터 배치를 바꿔 무편집 왕복의 바이트 동일이 깨진다.
        if plan.is_empty() {
            return serde_json::json!({
                "ok": true, "chart": chart.index + 1,
                "changedCount": 0, "changed": [], "wrote": [], "dryRun": edits.dry_run,
            })
            .to_string();
        }

        let zip_patched = match &representations.zip {
            Some((xml, data)) => match apply_chart_edits(xml, data, &plan) {
                Ok(v) => Some(v),
                Err(e) => return refused("chartPatch", e.to_string()),
            },
            None => None,
        };
        let nested_patched = match apply_chart_edits(nested_xml, nested_data, &plan) {
            Ok(v) => v,
            Err(e) => return refused("chartPatch", e.to_string()),
        };

        // [#5652] self-check — 산출을 다시 읽어 목표 행렬과 같을 때만 쓴다. 스캐너가 못 읽거나
        // (비순차 idx·계열 0건) 목표와 다르면 한 바이트도 쓰지 않는다. ①② 둘 다 있으면 두
        // 산출이 같은 논리 차트여야 한다.
        let nested_rescan = match scan_chart_values(&nested_patched) {
            Ok(v) => v,
            Err(e) => {
                return refused(
                    "selfCheckFailed",
                    format!("편집 산출(②)을 다시 읽지 못합니다 — 쓰지 않습니다: {e}"),
                )
            }
        };
        if let Err(why) = rescan_matches(&nested_rescan, &edits, scatter) {
            return refused(
                "selfCheckFailed",
                format!("편집 산출(②)이 목표 행렬과 다릅니다 — 쓰지 않습니다: {why}"),
            );
        }
        if let Some(zip_patched) = &zip_patched {
            let zip_rescan = match scan_chart_values(zip_patched) {
                Ok(v) => v,
                Err(e) => {
                    return refused(
                        "selfCheckFailed",
                        format!("편집 산출(①)을 다시 읽지 못합니다 — 쓰지 않습니다: {e}"),
                    )
                }
            };
            if let Err(why) = rescan_matches(&zip_rescan, &edits, scatter) {
                return refused(
                    "selfCheckFailed",
                    format!("편집 산출(①)이 목표 행렬과 다릅니다 — 쓰지 않습니다: {why}"),
                );
            }
            if !same_chart_data(&zip_rescan, &nested_rescan) {
                return refused(
                    "selfCheckFailed",
                    "편집 산출 ①과 ②가 같은 논리 차트가 아닙니다 — 쓰지 않습니다.".to_string(),
                );
            }
        }

        if edits.dry_run {
            return serde_json::json!({
                "ok": true, "chart": chart.index + 1,
                "changedCount": changed.len(), "changed": changed,
                "wrote": [], "dryRun": true,
            })
            .to_string();
        }

        // ② 재포장을 **먼저** 시도한다. 실패하면 ①도 쓰지 않아 문서가 원형으로 남는다.
        let nested_idx = chart.nested_copy.expect("위에서 확인함");
        let nested_original = self.document.bin_data_content[nested_idx].data.load();
        let nested_new = match replace_ole_stream(&nested_original, OOXML_STREAM, &nested_patched) {
            Ok(v) => v,
            Err(e) => return refused("nestedRepack", e.to_string()),
        };

        let mut wrote = Vec::new();
        if let (Some(zip_idx), Some(zip_patched)) = (chart.zip_part, zip_patched) {
            self.document.bin_data_content[zip_idx].data = zip_patched.into();
            wrote.push("zipPart");
        }
        self.document.bin_data_content[nested_idx].data = nested_new.into();
        wrote.push("nestedCopy");

        // [#4100] `bin_data_epoch` 는 `sourceImageKey`(ImageNode)의 "id→바이트 세션 안정"
        // 전제를 지키는 키다. 이 편집이 **기존 id 의 바이트를 제자리에서 바꾸는 첫
        // 연산**이라 그 전제를 깬다 — 올려서 소비자가 그림 바이트를 다시 받게 한다.
        self.bump_bin_data_epoch();
        // [#4603 리뷰] 차트는 ImageNode 가 아니라 RawSvg 노드라 epoch 키와 무관하다 —
        // 렌더된 SVG 조각이 page_tree_cache / layer_tree_json_cache 에 소유값으로 남아,
        // 캐시가 살아 있는 한 재렌더가 옛 차트를 돌려준다. 기하 불변 편집의 관용구대로
        // (queries/field_query.rs 의 set_active_field 선례) 페이지 캐시를 통째로 비운다.
        // dry-run·거부·무변경은 위에서 조기 반환했으므로 실제로 쓴 경우에만 닿는다.
        self.invalidate_page_tree_cache();

        serde_json::json!({
            "ok": true, "chart": chart.index + 1,
            "changedCount": changed.len(), "changed": changed,
            "wrote": wrote, "dryRun": false,
        })
        .to_string()
    }

    fn chart_data_at(&self, chart: &ChartRef) -> String {
        match scan_chart_representations(&self.document, chart) {
            Ok(representations) => {
                let (data, source) = representations.primary().expect("표현 하나 이상 확인함");
                chart_data_json(chart, data, source).to_string()
            }
            Err(response) => response,
        }
    }
}
