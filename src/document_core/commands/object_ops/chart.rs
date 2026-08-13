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
use crate::ooxml_chart::data::{scan_chart_values, ChartData, ChartScanError, SeriesAxis};
use crate::ooxml_chart::patch::{apply_value_edits, EditTarget, ValueEdit};
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
fn same_chart_data(left: &ChartData, right: &ChartData) -> bool {
    left.series.len() == right.series.len()
        && left.series.iter().zip(&right.series).all(|(left, right)| {
            left.name == right.name
                && left.axis == right.axis
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
#[derive(Debug, Deserialize)]
struct ChartEdits {
    /// 카테고리 라벨(분산형이면 X). 주면 대조하고, **분산형에서만** 기록한다.
    #[serde(default)]
    labels: Option<Vec<String>>,
    series: Vec<SeriesEdits>,
    /// 검증과 diff 만 하고 쓰지 않는다.
    #[serde(default, rename = "dryRun")]
    dry_run: bool,
}

#[derive(Debug, Deserialize)]
struct SeriesEdits {
    /// 주면 계열명을 대조한다. B1 은 계열명을 **바꾸지 않는다**(구조 변경 = B2).
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
fn validate(data: &ChartData, edits: &ChartEdits) -> Vec<serde_json::Value> {
    let mut out = Vec::new();

    if edits.series.len() != data.series.len() {
        out.push(serde_json::json!({
            "reason": "seriesCountMismatch",
            "expected": data.series.len(),
            "actual": edits.series.len(),
            "message": format!(
                "계열 수 {} 가 차트의 계열 수 {} 와 다릅니다 — 계열 신설·삭제는 하지 않습니다.",
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
                    "계열 {} 의 값 개수 {} 가 차트의 {} 와 다릅니다 — 점 신설·삭제는 하지 않습니다.",
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
                    "message": format!("계열 {} 의 이름이 다릅니다 — B1 은 계열명을 바꾸지 않습니다.", i),
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
        out.push(serde_json::json!({
            "reason": if scatter { "sharedXRequired" } else { "sharedCategoryRequired" },
            "message": if scatter {
                "계열마다 X 값이 달라 CSV 의 X 한 열로 표현할 수 없습니다."
            } else {
                "계열마다 카테고리 라벨이 달라 CSV 의 한 라벨 열로 표현할 수 없습니다."
            },
        }));
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
                    "라벨 개수 {} 가 계열 {} 의 {} 와 다릅니다.", labels.len(), i, series.labels.len()
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
                        "카테고리 라벨 {} 이 다릅니다 — B1 은 라벨을 바꾸지 않습니다.", p
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

/// 바뀐 칸만 골라 편집 목록과 `changed[]` 봉투를 만든다.
fn plan_edits(
    data: &ChartData,
    edits: &ChartEdits,
    scatter: bool,
) -> (Vec<ValueEdit>, Vec<serde_json::Value>) {
    let mut plan = Vec::new();
    let mut changed = Vec::new();

    for (i, (want, have)) in edits.series.iter().zip(&data.series).enumerate() {
        for (p, (text, point)) in want.values.iter().zip(&have.values).enumerate() {
            if text != &point.text {
                plan.push(ValueEdit {
                    series: i,
                    point: p,
                    target: EditTarget::Value,
                    text: text.clone(),
                });
                changed.push(serde_json::json!({
                    "series": i, "point": p, "from": point.text, "to": text,
                }));
            }
        }
    }

    if scatter {
        if let Some(labels) = &edits.labels {
            for (i, series) in data.series.iter().enumerate() {
                for (p, (text, point)) in labels.iter().zip(&series.labels).enumerate() {
                    if text != &point.text {
                        plan.push(ValueEdit {
                            series: i,
                            point: p,
                            target: EditTarget::Label,
                            text: text.clone(),
                        });
                        changed.push(serde_json::json!({
                            "series": i, "x": p, "from": point.text, "to": text,
                        }));
                    }
                }
            }
        }
    }

    (plan, changed)
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
            Some((xml, data)) => match apply_value_edits(xml, data, &plan) {
                Ok(v) => Some(v),
                Err(e) => return refused("chartPatch", e.to_string()),
            },
            None => None,
        };
        let nested_patched = match apply_value_edits(nested_xml, nested_data, &plan) {
            Ok(v) => v,
            Err(e) => return refused("chartPatch", e.to_string()),
        };

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
