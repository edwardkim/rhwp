//! [#4100] 차트 값의 최소 diff 치환.
//!
//! [`super::data::scan_chart_values`] 가 준 바이트 구간만 갈아끼우고 나머지 바이트는
//! 건드리지 않는다. 재직렬화를 하지 않으므로 `c:f`·`c:externalData`·`extLst`·
//! `ho:hncChartStyle` 처럼 모델이 모르는 것이 그대로 살아남는다.
//!
//! 이 층은 **기계적**이다 — 값이 수치인지, CSV 행·열 수가 맞는지 같은 의미 검증은
//! 코어 한 곳에 둔다(검증기를 코어와 CLI 로 가르지 않는다). 여기서 거부하는 것은
//! 주소 오류·중복 지목·XML 을 깨뜨리는 텍스트뿐이다.

use super::data::{ChartData, SeriesAxis};
use std::ops::Range;

/// 무엇을 바꾸는가.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTarget {
    /// `c:val`(분산형은 `c:yVal`) 의 값.
    Value,
    /// `c:xVal` 의 X 값. **분산형에서만** 유효하다 — 카테고리 라벨 변경은 구조
    /// 변경이라 B1 범위 밖(B2)이다.
    Label,
}

/// 편집 한 건 — 계열 순번, 점 순번, 대상, 새 텍스트.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueEdit {
    pub series: usize,
    pub point: usize,
    pub target: EditTarget,
    pub text: String,
}

/// 치환 거부 사유. **하나라도 걸리면 한 바이트도 쓰지 않는다.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchError {
    SeriesOutOfRange {
        series: usize,
        len: usize,
    },
    PointOutOfRange {
        series: usize,
        point: usize,
        len: usize,
    },
    /// 카테고리 라벨을 바꾸려 했다 — B2 범위다.
    LabelNotEditable {
        series: usize,
    },
    /// 같은 점을 두 번 지목했다.
    DuplicateTarget {
        series: usize,
        point: usize,
    },
    /// 빈 요소 `<c:v/>` — 결측치라 텍스트 구간이 없다.
    ///
    /// 읽기는 되고 이 점의 쓰기만 막힌다. 값을 넣으려면 요소 자체를 다시 써야 하는데
    /// 그건 최소 diff 가 아니라 구조 변경이라 B2 다.
    ValueNotPatchable {
        series: usize,
        point: usize,
    },
    /// XML 을 깨뜨리는 문자(`<`, `>`, `&`, 제어문자)가 들어 있다.
    ///
    /// 이스케이프해서 넣지 않고 거부한다. 최소 diff 의 전제는 "쓴 텍스트가 곧
    /// 파일의 바이트"이고, 몰래 이스케이프하면 왕복이 그 전제를 잃는다.
    UnsafeText {
        series: usize,
        point: usize,
    },
    /// 구간이 입력 바이트 밖이다 — 스캔에 쓴 XML 과 다른 바이트를 넘겼다는 뜻이다.
    SpanOutOfRange {
        series: usize,
        point: usize,
    },
    /// 두 구간이 겹친다.
    OverlappingSpans {
        series: usize,
        point: usize,
    },
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SeriesOutOfRange { series, len } => {
                write!(f, "계열 {series} 없음 (계열 {len}개)")
            }
            Self::PointOutOfRange { series, point, len } => {
                write!(f, "계열 {series} 의 점 {point} 없음 (점 {len}개)")
            }
            Self::LabelNotEditable { series } => {
                write!(f, "계열 {series} 의 카테고리 라벨은 B1 에서 바꾸지 않는다")
            }
            Self::DuplicateTarget { series, point } => {
                write!(f, "계열 {series} 점 {point} 을 두 번 지목했다")
            }
            Self::ValueNotPatchable { series, point } => write!(
                f,
                "계열 {series} 점 {point} 은 빈 값(<c:v/>)이라 제자리 치환 대상이 아니다"
            ),
            Self::UnsafeText { series, point } => {
                write!(f, "계열 {series} 점 {point} 의 값에 XML 특수문자가 있다")
            }
            Self::SpanOutOfRange { series, point } => {
                write!(f, "계열 {series} 점 {point} 의 구간이 입력 밖이다")
            }
            Self::OverlappingSpans { series, point } => {
                write!(f, "계열 {series} 점 {point} 의 구간이 다른 편집과 겹친다")
            }
        }
    }
}

/// `<c:v>` 안에 그대로 놓아도 XML 이 깨지지 않는 텍스트인가.
fn is_safe_text(text: &str) -> bool {
    !text
        .chars()
        .any(|c| matches!(c, '<' | '>' | '&') || c.is_control())
}

/// 결정된 치환 하나.
struct Planned<'a> {
    span: Range<usize>,
    text: &'a str,
    series: usize,
    point: usize,
}

/// 값 구간을 새 텍스트로 갈아끼운 XML 을 만든다.
///
/// `data` 는 **같은 `xml`** 에서 나온 스캔이어야 한다 — 구간이 그 바이트 기준이다.
/// 모든 편집이 자기 원래 텍스트를 되쓰면 결과는 입력과 바이트 동일하다.
pub fn apply_value_edits(
    xml: &[u8],
    data: &ChartData,
    edits: &[ValueEdit],
) -> Result<Vec<u8>, PatchError> {
    let mut planned: Vec<Planned<'_>> = Vec::with_capacity(edits.len());
    let mut seen: Vec<(usize, usize, EditTarget)> = Vec::with_capacity(edits.len());

    for edit in edits {
        let series = data
            .series
            .get(edit.series)
            .ok_or(PatchError::SeriesOutOfRange {
                series: edit.series,
                len: data.series.len(),
            })?;

        let points = match edit.target {
            EditTarget::Value => &series.values,
            EditTarget::Label => {
                if series.axis != SeriesAxis::Scatter {
                    return Err(PatchError::LabelNotEditable {
                        series: edit.series,
                    });
                }
                &series.labels
            }
        };

        let point = points.get(edit.point).ok_or(PatchError::PointOutOfRange {
            series: edit.series,
            point: edit.point,
            len: points.len(),
        })?;

        if !is_safe_text(&edit.text) {
            return Err(PatchError::UnsafeText {
                series: edit.series,
                point: edit.point,
            });
        }

        let key = (edit.series, edit.point, edit.target);
        if seen.contains(&key) {
            return Err(PatchError::DuplicateTarget {
                series: edit.series,
                point: edit.point,
            });
        }
        seen.push(key);

        let span = point.span.clone().ok_or(PatchError::ValueNotPatchable {
            series: edit.series,
            point: edit.point,
        })?;

        if span.start > span.end || span.end > xml.len() {
            return Err(PatchError::SpanOutOfRange {
                series: edit.series,
                point: edit.point,
            });
        }

        planned.push(Planned {
            span,
            text: &edit.text,
            series: edit.series,
            point: edit.point,
        });
    }

    planned.sort_by_key(|p| p.span.start);

    let mut out = Vec::with_capacity(xml.len());
    let mut cursor = 0usize;
    for p in &planned {
        if p.span.start < cursor {
            return Err(PatchError::OverlappingSpans {
                series: p.series,
                point: p.point,
            });
        }
        out.extend_from_slice(&xml[cursor..p.span.start]);
        out.extend_from_slice(p.text.as_bytes());
        cursor = p.span.end;
    }
    out.extend_from_slice(&xml[cursor..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::data::scan_chart_values;
    use super::*;

    const CHART: &str = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
        r#"<c:val><c:numRef><c:f>Sheet1!$B$2:$B$3</c:f><c:numCache><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v>4.3</c:v></c:pt><c:pt idx="1"><c:v>2.5</c:v></c:pt>"#,
        r#"</c:numCache></c:numRef></c:val>"#,
        r#"</c:ser></c:barChart></c:plotArea>"#,
        r#"<c:extLst><ho:hncChartStyle val="1"/></c:extLst></c:chart></c:chartSpace>"#,
    );

    fn edit(series: usize, point: usize, text: &str) -> ValueEdit {
        ValueEdit {
            series,
            point,
            target: EditTarget::Value,
            text: text.to_string(),
        }
    }

    #[test]
    fn identity_edit_is_byte_identical() {
        let xml = CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        let edits: Vec<ValueEdit> = data.series[0]
            .values
            .iter()
            .enumerate()
            .map(|(i, p)| edit(0, i, &p.text))
            .collect();
        assert_eq!(apply_value_edits(xml, &data, &edits).expect("패치"), xml);
    }

    #[test]
    fn unknown_elements_survive_the_edit() {
        let xml = CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        let out = apply_value_edits(xml, &data, &[edit(0, 0, "91.7")]).expect("패치");
        let text = String::from_utf8(out).expect("UTF-8");
        assert!(text.contains("<c:v>91.7</c:v>"));
        assert!(text.contains("<c:v>2.5</c:v>"), "다른 값이 바뀌었다");
        assert!(text.contains("ho:hncChartStyle"), "모델 밖 요소가 사라졌다");
        assert!(text.contains("Sheet1!$B$2:$B$3"), "c:f 가 사라졌다");
    }

    #[test]
    fn multiple_edits_apply_in_any_order() {
        let xml = CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        let forward = apply_value_edits(xml, &data, &[edit(0, 0, "1"), edit(0, 1, "2")]);
        let reverse = apply_value_edits(xml, &data, &[edit(0, 1, "2"), edit(0, 0, "1")]);
        assert_eq!(forward.expect("정방향"), reverse.expect("역방향"));
    }

    #[test]
    fn no_edits_returns_the_input() {
        let xml = CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        assert_eq!(apply_value_edits(xml, &data, &[]).expect("패치"), xml);
    }

    #[test]
    fn shorter_and_longer_replacements_shift_only_the_tail() {
        let xml = CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        let out = apply_value_edits(xml, &data, &[edit(0, 0, "1234567")]).expect("패치");
        assert_eq!(out.len(), xml.len() + 4);
        let out = apply_value_edits(xml, &data, &[edit(0, 0, "1")]).expect("패치");
        assert_eq!(out.len(), xml.len() - 2);
    }

    #[test]
    fn unsafe_text_is_refused() {
        let xml = CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        for bad in ["1<2", "a&b", "x>y", "a\nb"] {
            assert!(
                apply_value_edits(xml, &data, &[edit(0, 0, bad)]).is_err(),
                "`{bad}` 는 거부되어야 한다"
            );
        }
    }

    #[test]
    fn bad_addresses_are_refused() {
        let xml = CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        assert_eq!(
            apply_value_edits(xml, &data, &[edit(9, 0, "1")]),
            Err(PatchError::SeriesOutOfRange { series: 9, len: 1 })
        );
        assert_eq!(
            apply_value_edits(xml, &data, &[edit(0, 9, "1")]),
            Err(PatchError::PointOutOfRange {
                series: 0,
                point: 9,
                len: 2
            })
        );
    }

    #[test]
    fn duplicate_target_is_refused() {
        let xml = CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        assert_eq!(
            apply_value_edits(xml, &data, &[edit(0, 0, "1"), edit(0, 0, "2")]),
            Err(PatchError::DuplicateTarget {
                series: 0,
                point: 0
            })
        );
    }

    /// 결측치는 읽히지만 그 점의 편집만 거부된다 — 문서 전체가 막히지 않는다.
    #[test]
    fn blank_value_cannot_be_patched_but_its_neighbours_can() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
            r#"<c:val><c:numLit><c:ptCount val="2"/>"#,
            r#"<c:pt idx="0"><c:v/></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt>"#,
            r#"</c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("스캔");
        assert_eq!(
            apply_value_edits(xml.as_bytes(), &data, &[edit(0, 0, "5")]),
            Err(PatchError::ValueNotPatchable {
                series: 0,
                point: 0
            })
        );
        let out =
            apply_value_edits(xml.as_bytes(), &data, &[edit(0, 1, "5")]).expect("이웃은 된다");
        assert!(String::from_utf8_lossy(&out).contains("<c:v>5</c:v>"));
    }

    #[test]
    fn category_label_edit_is_refused() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
            r#"<c:cat><c:strLit><c:pt idx="0"><c:v>항목</c:v></c:pt></c:strLit></c:cat>"#,
            r#"<c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
            r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("스캔");
        assert_eq!(
            apply_value_edits(
                xml.as_bytes(),
                &data,
                &[ValueEdit {
                    series: 0,
                    point: 0,
                    target: EditTarget::Label,
                    text: "새 항목".to_string(),
                }]
            ),
            Err(PatchError::LabelNotEditable { series: 0 })
        );
    }
}
