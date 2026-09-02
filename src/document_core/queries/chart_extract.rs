//! [#4100] 문서에서 차트를 찾아 두 표현(①②) 슬롯으로 잇는다.
//!
//! ## 값 하나가 두 곳에 있다
//!
//! | | 표현 | 어디에 | 한컴이 읽나 |
//! |---|---|---|---|
//! | ① | `Chart/chartN.xml` | HWPX zip 파트 = `bin_data_content` 의 `ooxml_chart` 슬롯 | 예 |
//! | ② | `OOXMLChartContents` | 중첩 CFB = `BinData/*.ole` 슬롯 | 예 (HWP5 엔 이것뿐) |
//!
//! **HWP5 입력에는 ①이 아예 없다.** `Chart/*.xml` 파트가 없고 차트는 평범한 `OleShape`
//! 라 `chart_id_ref`·`chart_switch_fallback` 이 둘 다 `None` 이다. "①② 동시 기록"은
//! HWP5 에서 ②만으로 자연 축약된다.
//!
//! ## ①↔② 조인 키
//!
//! HWPX 파서는 `<hp:switch>` 안에 `<hp:chart>` 와 `<hp:default><hp:ole>` 이 **둘 다**
//! 있을 때만 뒤쪽을 `chart_switch_fallback` 에 매단다(`parser/hwpx/section.rs`).
//! 그것이 유일한 연결 고리라, 없으면 ②를 특정할 수 없고 **아무것도 쓰지 않는다** —
//! 반쪽만 새 값인 파일이 나가는 것이 최악이다.
//!
//! ## 편집은 컨트롤을 건드리지 않는다
//!
//! 바꾸는 것은 `bin_data_content` 슬롯의 바이트뿐이다. 그래서 컨트롤을 mutable 로 잡을
//! 필요가 없고, 글상자·머리말 같은 컨테이너 안의 차트도 열거만 되면 똑같이 편집된다.

use serde::Serialize;

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{OleShape, ShapeObject};
use crate::parser::ole_container::parse_ole_container;
use crate::renderer::layout::find_bin_data_index;

/// 컨테이너 재귀 상한 — `table_extract` 와 같은 값.
const MAX_NEST_DEPTH: usize = 8;

/// HWPX 차트 파트가 `bin_data_content` 에서 쓰는 sparse id 대역의 기준값.
/// `Chart/chartN.xml` 이 `60000 + N` 이다.
const CHART_ID_BASE: u32 = 60000;

/// 본문 문단에서 차트까지 내려가는 컨테이너 경로 한 단계.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChartContainerRef {
    /// 컨테이너 종류: textbox, header, footer, footnote, endnote, tableCell.
    pub kind: &'static str,
    /// 부모 문단 안의 컨트롤 인덱스.
    pub control: usize,
    /// 컨테이너 안의 문단 인덱스.
    pub paragraph: usize,
    /// tableCell 경로일 때만 셀 인덱스.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<usize>,
}

/// 문서 안 차트 하나의 위치와 슬롯.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChartRef {
    /// 문서 순서 0-based 순번. CLI `--chart N` 은 여기에 1 을 더한 값이다.
    pub index: usize,
    pub section: usize,
    /// 본문 문단 인덱스 (컨테이너 안이면 그 컨테이너를 품은 본문 문단).
    pub paragraph: usize,
    /// 그 문단 안 컨트롤 인덱스.
    pub control: usize,
    /// 컨테이너 경로. 비어 있으면 본문 직속이다.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub container: Vec<ChartContainerRef>,
    /// ① `Chart/chartN.xml` 슬롯의 `bin_data_content` 인덱스. HWP5 는 `None`.
    #[serde(rename = "zipPart", skip_serializing_if = "Option::is_none")]
    pub zip_part: Option<usize>,
    /// ② 중첩 CFB 슬롯의 `bin_data_content` 인덱스. 조인 키가 없으면 `None`.
    #[serde(rename = "nestedCopy", skip_serializing_if = "Option::is_none")]
    pub nested_copy: Option<usize>,
}

impl ChartRef {
    /// 본문 직속인가 — 코어의 `(section, paragraph, control)` 3인자로 지목 가능한가.
    pub fn is_top_level(&self) -> bool {
        self.container.is_empty()
    }
}

/// 차트 XML 을 어느 표현에서 읽었는가.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartSource {
    /// ① zip 파트
    ZipPart,
    /// ② 중첩 CFB
    NestedCopy,
}

/// 이 OLE 가 차트인가 — 그리고 어느 슬롯을 쓰는가.
///
/// ①은 슬롯의 `extension == "ooxml_chart"` 로, ②는 중첩 CFB 안
/// `OOXMLChartContents` 유무로 본다. 이름이 아니라 내용으로 판별하는 이유는 HWP5 의
/// `OleShape` 가 수식·글맵시 등 무엇이든 될 수 있어서다 — 렌더러도 같은 방법을 쓴다.
fn resolve_slots(doc: &Document, ole: &OleShape) -> (Option<usize>, Option<usize>) {
    let zip_part = if ole.bin_data_id > CHART_ID_BASE {
        find_bin_data_index(&doc.bin_data_content, ole.bin_data_id as u16)
            .filter(|&i| doc.bin_data_content[i].extension == "ooxml_chart")
    } else {
        None
    };

    // ② 는 fallback 이 있으면 그쪽, 없으면 자기 자신(HWP5 · bare OLE).
    let nested_id = match ole.chart_switch_fallback.as_ref() {
        Some(fallback) => fallback.bin_data_id,
        None if zip_part.is_none() => ole.bin_data_id,
        None => return (zip_part, None),
    };
    let nested_copy = find_bin_data_index(&doc.bin_data_content, nested_id as u16).filter(|&i| {
        let bytes = doc.bin_data_content[i].data.load();
        bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0])
            && parse_ole_container(&bytes).is_some_and(|c| c.has_ooxml_chart())
    });

    (zip_part, nested_copy)
}

fn collect_from_paragraph(
    doc: &Document,
    para: &Paragraph,
    section: usize,
    root_paragraph: usize,
    container: &[ChartContainerRef],
    depth: usize,
    out: &mut Vec<ChartRef>,
) {
    if depth >= MAX_NEST_DEPTH {
        return;
    }
    for (control, ctrl) in para.controls.iter().enumerate() {
        match ctrl {
            Control::Shape(shape) => {
                if let ShapeObject::Ole(ole) = shape.as_ref() {
                    let (zip_part, nested_copy) = resolve_slots(doc, ole);
                    if zip_part.is_some() || nested_copy.is_some() {
                        out.push(ChartRef {
                            index: out.len(),
                            section,
                            paragraph: root_paragraph,
                            control,
                            container: container.to_vec(),
                            zip_part,
                            nested_copy,
                        });
                    }
                }
                // 글상자 안 문단 재귀 — 공문서는 차트를 글상자에 넣는 배치가 흔하다.
                if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                    for (paragraph, p) in tb.paragraphs.iter().enumerate() {
                        let mut path = container.to_vec();
                        path.push(ChartContainerRef {
                            kind: "textbox",
                            control,
                            paragraph,
                            cell: None,
                        });
                        collect_from_paragraph(
                            doc,
                            p,
                            section,
                            root_paragraph,
                            &path,
                            depth + 1,
                            out,
                        );
                    }
                }
            }
            Control::Table(table) => {
                for (cell, c) in table.cells.iter().enumerate() {
                    for (paragraph, p) in c.paragraphs.iter().enumerate() {
                        let mut path = container.to_vec();
                        path.push(ChartContainerRef {
                            kind: "tableCell",
                            control,
                            paragraph,
                            cell: Some(cell),
                        });
                        collect_from_paragraph(
                            doc,
                            p,
                            section,
                            root_paragraph,
                            &path,
                            depth + 1,
                            out,
                        );
                    }
                }
            }
            Control::Header(h) => collect_from_container(
                doc,
                &h.paragraphs,
                section,
                root_paragraph,
                container,
                control,
                "header",
                depth,
                out,
            ),
            Control::Footer(f) => collect_from_container(
                doc,
                &f.paragraphs,
                section,
                root_paragraph,
                container,
                control,
                "footer",
                depth,
                out,
            ),
            Control::Footnote(fnote) => collect_from_container(
                doc,
                &fnote.paragraphs,
                section,
                root_paragraph,
                container,
                control,
                "footnote",
                depth,
                out,
            ),
            Control::Endnote(en) => collect_from_container(
                doc,
                &en.paragraphs,
                section,
                root_paragraph,
                container,
                control,
                "endnote",
                depth,
                out,
            ),
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_from_container(
    doc: &Document,
    paragraphs: &[Paragraph],
    section: usize,
    root_paragraph: usize,
    container: &[ChartContainerRef],
    control: usize,
    kind: &'static str,
    depth: usize,
    out: &mut Vec<ChartRef>,
) {
    for (paragraph, p) in paragraphs.iter().enumerate() {
        let mut path = container.to_vec();
        path.push(ChartContainerRef {
            kind,
            control,
            paragraph,
            cell: None,
        });
        collect_from_paragraph(doc, p, section, root_paragraph, &path, depth + 1, out);
    }
}

/// 문서의 모든 차트를 **문서 순서**로 모은다.
///
/// 차트가 아닌 OLE(수식·글맵시 등)는 걸러진다 — 두 슬롯 중 하나도 차트로 해소되지 않으면
/// 담지 않는다.
pub fn collect_charts(doc: &Document) -> Vec<ChartRef> {
    let mut out = Vec::new();
    for (section, sec) in doc.sections.iter().enumerate() {
        for (paragraph, para) in sec.paragraphs.iter().enumerate() {
            collect_from_paragraph(doc, para, section, paragraph, &[], 0, &mut out);
        }
    }
    // 컨테이너 재귀로 순번이 흔들리지 않게 다시 매긴다 (`table_extract` 와 같은 처리).
    for (i, chart) in out.iter_mut().enumerate() {
        chart.index = i;
    }
    out
}

/// ① `Chart/chartN.xml` 에 든 OOXML XML 을 읽는다.
///
/// 빈 zip 파트는 유효한 차트 표현이 아니다. 호출자는 ②와 함께 쓰기 전에 이 값을 별도로
/// 확인해, ②의 바이트를 ①에 복제하는 식의 손실 보정을 하지 않아야 한다.
pub fn zip_chart_xml(doc: &Document, chart: &ChartRef) -> Option<Vec<u8>> {
    let i = chart.zip_part?;
    let bytes = doc.bin_data_content.get(i)?.data.load();
    (!bytes.is_empty()).then_some(bytes)
}

/// ② 중첩 CFB의 `OOXMLChartContents` 에 든 OOXML XML 을 읽는다.
pub fn nested_chart_xml(doc: &Document, chart: &ChartRef) -> Option<Vec<u8>> {
    let i = chart.nested_copy?;
    let nested = doc.bin_data_content.get(i)?.data.load();
    parse_ole_container(&nested)?.ooxml_chart
}

/// 차트의 우선 OOXML XML 을 읽는다. ①이 있으면 ①, 없으면 ②에서 꺼낸다.
///
/// 이 함수는 읽기 편의용 선택만 한다. ①과 ②의 논리적 동일성 검증과 독립 패치는
/// `object_ops::chart`의 쓰기 경로가 수행한다.
pub fn chart_xml(doc: &Document, chart: &ChartRef) -> Option<(Vec<u8>, ChartSource)> {
    zip_chart_xml(doc, chart)
        .map(|xml| (xml, ChartSource::ZipPart))
        .or_else(|| nested_chart_xml(doc, chart).map(|xml| (xml, ChartSource::NestedCopy)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bin_data::BinDataContent;

    /// `bin_data_id` 는 보통 **1-based 인덱스**이고 60000+N 만 진짜 id 다.
    /// 두 규칙이 갈리면 읽을 때와 쓸 때가 다른 슬롯을 가리킨다 (계획서 R2).
    #[test]
    fn slot_lookup_follows_the_index_then_id_rule() {
        let content = vec![
            BinDataContent {
                id: 7,
                data: b"first".to_vec().into(),
                extension: "png".to_string(),
            },
            BinDataContent {
                id: 60001,
                data: b"chart".to_vec().into(),
                extension: "ooxml_chart".to_string(),
            },
        ];
        // 1 → 인덱스 0 (id 가 7 이어도)
        assert_eq!(find_bin_data_index(&content, 1), Some(0));
        // 2 → 인덱스 1
        assert_eq!(find_bin_data_index(&content, 2), Some(1));
        // 60001 → 인덱스 범위 밖이므로 id 검색
        assert_eq!(find_bin_data_index(&content, 60001), Some(1));
        assert_eq!(find_bin_data_index(&content, 0), None);
        assert_eq!(find_bin_data_index(&content, 9), None);
    }
}
