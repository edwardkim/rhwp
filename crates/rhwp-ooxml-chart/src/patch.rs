//! [#4100] 차트 값의 최소 diff 치환. [#5652] 구조 편집(점·계열 꼬리 삽입·삭제, 계열명·라벨)도
//! 같은 **구간→바이트열 splice** 로 한다.
//!
//! [`super::data::scan_chart_values`] 가 준 바이트 구간만 갈아끼우고 나머지 바이트는
//! 건드리지 않는다. 재직렬화를 하지 않으므로 `c:f`·`c:externalData`·`extLst`·
//! `ho:hncChartStyle` 처럼 모델이 모르는 것이 그대로 살아남는다.
//!
//! 이 층은 **기계적**이다 — 값이 수치인지, CSV 행·열 수가 맞는지, 원형에 계열을 더해도
//! 되는지 같은 의미 검증은 코어 한 곳에 둔다(검증기를 코어와 CLI 로 가르지 않는다).
//! 여기서 거부하는 것은 주소 오류·중복 지목·겹침·XML 을 깨뜨리는 텍스트·구조 좌표 부재뿐이다.
//!
//! ## 구조 편집의 형태 (#5652)
//!
//! 행렬은 목표 상태이고 치수 diff 는 **위치 기반 꼬리 증감**이다 — 점은 블록 꼬리에 붙이거나
//! 꼬리부터 지우고, 계열은 마지막 계열을 복제해 뒤에 붙이거나 꼬리부터 지운다. 그래서
//! `c:pt idx`·`c:idx`/`c:order` 재번호가 생기지 않고(항상 `0..n-1`), `c:ptCount` 재계산만 남는다.
//! 삽입은 길이 0 구간에 바이트를 놓는 것, 삭제는 구간에 빈 바이트를 놓는 것이라 값 치환과
//! 같은 단일 패스 복사로 돈다.

use super::data::{scan_chart_values, BlockShape, ChartData, ChartPoint, ChartSeries};
use std::ops::Range;

/// 무엇을 바꾸는가.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTarget {
    /// `c:val`(분산형은 `c:yVal`) 의 값.
    Value,
    /// `c:cat` 카테고리 라벨 또는 `c:xVal` 분산형 X. [#5652] 카테고리 라벨도 캐시 텍스트
    /// 치환 대상이다 — 허용 여부(의도 플래그)는 코어가 정한다.
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

/// [#5652] 편집 한 건 — 값 치환과 구조 편집을 한 목록에 섞어 넘긴다.
///
/// 주소는 전부 **논리 주소**(계열·점 순번)다. 바이트 구간은 적용 시점에 표현(①/②)마다
/// 따로 해석하므로, 같은 목록을 두 표현에 그대로 적용할 수 있다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartEdit {
    /// 기존 점의 텍스트 치환(B1).
    Value(ValueEdit),
    /// 계열명 `c:tx` 캐시 텍스트 치환. `c:tx` 가 없는 계열은 거부된다.
    SeriesName { series: usize, text: String },
    /// 블록 꼬리에 점 추가 — `<pt idx="n"><v>…</v></pt>` 를 삽입 앵커에 놓고 `ptCount` 를 늘린다.
    AppendPoints {
        series: usize,
        target: EditTarget,
        texts: Vec<String>,
    },
    /// 블록 꼬리 점 삭제 — 앞 `keep` 개만 남기고 `ptCount` 를 줄인다. `keep == 0` 은 거부.
    TruncatePoints {
        series: usize,
        target: EditTarget,
        keep: usize,
    },
    /// 마지막 계열을 복제해 뒤에 붙인다 — `c:idx`/`c:order` 는 새 위치, 이름·라벨·값을 바꿔 넣는다.
    ///
    /// `values` 는 **최종** 행 수(같은 목록의 점 증감을 반영한 뒤)와 같아야 한다. `labels` 가
    /// `None` 이면 템플릿 계열이 같은 목록에서 받는 라벨 편집을 그대로 물려받는다. `name` 이
    /// `None` 이면 템플릿 이름을 물려받는다.
    AppendSeries {
        name: Option<String>,
        labels: Option<Vec<String>>,
        values: Vec<String>,
    },
    /// 꼬리 계열 삭제 — 앞 `keep` 개만 남긴다. `keep == 0` 은 거부.
    TruncateSeries { keep: usize },
    /// [#6053] 계열을 `at` 자리(**최종 문서 기준**)에 삽입한다.
    ///
    /// 마지막 계열을 복제해 앵커(그 자리를 차지할 원본 계열의 요소 시작)에 놓되, 복제본의
    /// 계열 최상위 `c:spPr` 와 `c:marker` 안 `c:symbol` 을 **들어내** 기본 스타일(Auto 마커·
    /// 기본 선)로 되돌린다 — 한컴 편집기가 더한 계열과 같은 모양이다. 밀려나는 뒤 계열들의
    /// `c:idx`/`c:order` 는 한 칸씩 재번호된다. `labels`/`name` 규칙은
    /// [`ChartEdit::AppendSeries`] 와 같다.
    ///
    /// 같은 목록에서 [`ChartEdit::AppendSeries`]/[`ChartEdit::TruncateSeries`] 와 섞이면
    /// 거부된다 — 위치 기반 꼬리 증감과 정체 기반 재번호는 좌표계가 다르다.
    InsertSeries {
        at: usize,
        name: Option<String>,
        labels: Option<Vec<String>>,
        values: Vec<String>,
    },
    /// [#6053] `at` 계열(**원본 기준**)을 삭제한다 — 요소 구간을 비우고 뒤 계열들을 재번호한다.
    ///
    /// 혼합 금지는 [`ChartEdit::InsertSeries`] 와 같다.
    RemoveSeries { at: usize },
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
    /// 같은 점을 두 번 지목했다.
    DuplicateTarget {
        series: usize,
        point: usize,
    },
    /// 빈 요소 `<c:v/>` — 결측치라 텍스트 구간이 없다.
    ///
    /// 읽기는 되고 이 점의 쓰기만 막힌다. 값을 넣으려면 요소 자체를 다시 써야 하는데
    /// 그건 최소 diff 가 아니라 구조 변경이다 — 꼬리 삭제([`ChartEdit::TruncatePoints`])는 된다.
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
    /// [#5652] 계열명 치환인데 `c:tx` 캐시 텍스트 구간이 없다.
    SeriesNameNotPatchable {
        series: usize,
    },
    /// [#5652] 계열명에 XML 특수문자가 있다.
    UnsafeName {
        series: usize,
    },
    /// [#5652] 블록의 개수를 바꾸려는데 구조 좌표(삽입 앵커·`ptCount`·점 요소 구간)가 없다 —
    /// 캐시가 없거나 다층 카테고리(`multiLvlStrCache`)다.
    BlockNotResizable {
        series: usize,
        target: EditTarget,
    },
    /// [#5652] 블록의 마지막 점까지 지우려 했다 — 빈 블록은 스캐너가 다시 읽지 못한다.
    EmptyBlockRefused {
        series: usize,
        target: EditTarget,
    },
    /// [#5652] 마지막 계열까지 지우려 했다 — `c:ser` 0건은 `NoSeries` 로 읽히지 않는다.
    EmptySeriesRefused,
    /// [#5652] 복제 템플릿(마지막 계열)에 `c:idx`/`c:order` 구간이 없거나 복제본을 다시 읽지 못했다.
    SeriesNotClonable {
        series: usize,
    },
    /// [#5652] 신설 계열의 라벨/값 개수가 최종 행 수와 다르다.
    LengthMismatch {
        series: usize,
        expected: usize,
        actual: usize,
    },
    /// [#5652] 구조 편집 구간(점 요소·계열 요소·`ptCount`·계열명)이 다른 편집과 겹친다 —
    /// 지울 점을 동시에 치환하거나, 같은 블록의 개수를 두 번 바꾸거나, 계열 삭제와 신설을 섞었다.
    OverlappingStructureEdits {
        series: usize,
    },
    /// [#5652] 구조 편집 구간이 입력 바이트 밖이다.
    StructureSpanOutOfRange {
        series: usize,
    },
    /// [#6053] 삽입·삭제로 자리가 밀리는 계열에 `c:idx`/`c:order` 구간이 없어 재번호할 수 없다.
    SeriesNotRenumberable {
        series: usize,
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
            Self::SeriesNameNotPatchable { series } => {
                write!(
                    f,
                    "계열 {series} 은 c:tx 계열명 캐시가 없어 이름을 바꿀 수 없다"
                )
            }
            Self::UnsafeName { series } => {
                write!(f, "계열 {series} 의 이름에 XML 특수문자가 있다")
            }
            Self::BlockNotResizable { series, target } => write!(
                f,
                "계열 {series} 의 {} 블록은 캐시 구조 좌표가 없어 개수를 바꿀 수 없다",
                target_name(*target)
            ),
            Self::EmptyBlockRefused { series, target } => write!(
                f,
                "계열 {series} 의 {} 블록에서 마지막 점까지 지울 수 없다",
                target_name(*target)
            ),
            Self::EmptySeriesRefused => write!(f, "마지막 계열은 지울 수 없다"),
            Self::SeriesNotClonable { series } => {
                write!(f, "계열 {series} 을 복제할 수 없다 — c:idx/c:order 구간이 없거나 복제본을 다시 읽지 못했다")
            }
            Self::LengthMismatch {
                series,
                expected,
                actual,
            } => write!(
                f,
                "계열 {series} 의 점 개수 {actual} 가 최종 행 수 {expected} 와 다르다"
            ),
            Self::OverlappingStructureEdits { series } => {
                write!(f, "계열 {series} 의 구조 편집 구간이 다른 편집과 겹친다")
            }
            Self::StructureSpanOutOfRange { series } => {
                write!(f, "계열 {series} 의 구조 편집 구간이 입력 밖이다")
            }
            Self::SeriesNotRenumberable { series } => {
                write!(
                    f,
                    "계열 {series} 의 c:idx/c:order 구간이 없어 재번호할 수 없다"
                )
            }
        }
    }
}

fn target_name(target: EditTarget) -> &'static str {
    match target {
        EditTarget::Value => "값",
        EditTarget::Label => "라벨",
    }
}

impl PatchError {
    /// 복제본(계열 0 하나짜리 조각) 안에서 난 오류를 바깥 계열 번호로 옮긴다.
    fn with_series(self, series: usize) -> Self {
        match self {
            Self::SeriesOutOfRange { len, .. } => Self::SeriesOutOfRange { series, len },
            Self::PointOutOfRange { point, len, .. } => {
                Self::PointOutOfRange { series, point, len }
            }
            Self::DuplicateTarget { point, .. } => Self::DuplicateTarget { series, point },
            Self::ValueNotPatchable { point, .. } => Self::ValueNotPatchable { series, point },
            Self::UnsafeText { point, .. } => Self::UnsafeText { series, point },
            Self::SpanOutOfRange { point, .. } => Self::SpanOutOfRange { series, point },
            Self::OverlappingSpans { point, .. } => Self::OverlappingSpans { series, point },
            Self::SeriesNameNotPatchable { .. } => Self::SeriesNameNotPatchable { series },
            Self::UnsafeName { .. } => Self::UnsafeName { series },
            Self::BlockNotResizable { target, .. } => Self::BlockNotResizable { series, target },
            Self::EmptyBlockRefused { target, .. } => Self::EmptyBlockRefused { series, target },
            Self::EmptySeriesRefused => Self::EmptySeriesRefused,
            Self::SeriesNotClonable { .. } => Self::SeriesNotClonable { series },
            Self::LengthMismatch {
                expected, actual, ..
            } => Self::LengthMismatch {
                series,
                expected,
                actual,
            },
            Self::OverlappingStructureEdits { .. } => Self::OverlappingStructureEdits { series },
            Self::StructureSpanOutOfRange { .. } => Self::StructureSpanOutOfRange { series },
            Self::SeriesNotRenumberable { .. } => Self::SeriesNotRenumberable { series },
        }
    }
}

/// `<c:v>` 안에 그대로 놓아도 XML 이 깨지지 않는 텍스트인가.
///
/// [#5652] 코어가 계열명·라벨·새 점 텍스트를 선검사할 수 있게 공개한다.
pub fn is_safe_text(text: &str) -> bool {
    !text
        .chars()
        .any(|c| matches!(c, '<' | '>' | '&') || c.is_control())
}

/// 결정된 splice 하나 — 구간을 이 바이트열로 바꾼다. 길이 0 구간이면 삽입, 빈 바이트열이면 삭제.
struct Splice {
    span: Range<usize>,
    bytes: Vec<u8>,
    /// 계획 순서. 같은 오프셋의 삽입 여럿은 이 순서대로 놓인다.
    seq: usize,
    /// 오류 보고용 주소.
    addr: Addr,
}

/// splice 의 논리 주소.
#[derive(Clone, Copy)]
enum Addr {
    Point { series: usize, point: usize },
    Structure { series: usize },
}

impl Addr {
    fn overlap(self) -> PatchError {
        match self {
            Self::Point { series, point } => PatchError::OverlappingSpans { series, point },
            Self::Structure { series } => PatchError::OverlappingStructureEdits { series },
        }
    }
    fn out_of_range(self) -> PatchError {
        match self {
            Self::Point { series, point } => PatchError::SpanOutOfRange { series, point },
            Self::Structure { series } => PatchError::StructureSpanOutOfRange { series },
        }
    }
}

/// 정렬 + 커서 단일 패스 복사. 구간이 겹치면 거부한다.
///
/// 정렬 키는 `(start, end, seq)` — 같은 위치의 길이 0 삽입은 계획 순서를 지키고, 삽입은
/// 같은 위치에서 시작하는 치환·삭제보다 앞에 놓인다.
fn splice(xml: &[u8], mut planned: Vec<Splice>) -> Result<Vec<u8>, PatchError> {
    for p in &planned {
        if p.span.start > p.span.end || p.span.end > xml.len() {
            return Err(p.addr.out_of_range());
        }
    }
    planned.sort_by_key(|p| (p.span.start, p.span.end, p.seq));

    let mut out = Vec::with_capacity(xml.len());
    let mut cursor = 0usize;
    for p in &planned {
        if p.span.start < cursor {
            return Err(p.addr.overlap());
        }
        out.extend_from_slice(&xml[cursor..p.span.start]);
        out.extend_from_slice(&p.bytes);
        cursor = p.span.end;
    }
    out.extend_from_slice(&xml[cursor..]);
    Ok(out)
}

/// 편집 목록을 splice 목록으로 푼다.
struct Planner<'a> {
    xml: &'a [u8],
    data: &'a ChartData,
    edits: &'a [ChartEdit],
    splices: Vec<Splice>,
    /// 점 치환의 중복 지목 검사.
    seen: Vec<(usize, usize, EditTarget)>,
    /// 이번 목록에서 신설한 계열 수 — 다음 신설 계열의 `c:idx`/`c:order`.
    appended_series: usize,
    truncated_series: bool,
    /// [#6053] 정체 경로 삽입 — (최종 문서 기준 자리, 복제본 바이트). 앵커·재번호는
    /// [`Planner::finish`] 가 일괄 계산한다.
    inserted: Vec<(usize, Vec<u8>)>,
    /// [#6053] 정체 경로 삭제 — 원본 기준 자리.
    removed: Vec<usize>,
}

impl<'a> Planner<'a> {
    fn new(xml: &'a [u8], data: &'a ChartData, edits: &'a [ChartEdit]) -> Self {
        Self {
            xml,
            data,
            edits,
            splices: Vec::with_capacity(edits.len()),
            seen: Vec::new(),
            appended_series: 0,
            truncated_series: false,
            inserted: Vec::new(),
            removed: Vec::new(),
        }
    }

    fn push(&mut self, span: Range<usize>, bytes: Vec<u8>, addr: Addr) {
        let seq = self.splices.len();
        self.splices.push(Splice {
            span,
            bytes,
            seq,
            addr,
        });
    }

    fn series(&self, series: usize) -> Result<&'a ChartSeries, PatchError> {
        self.data
            .series
            .get(series)
            .ok_or(PatchError::SeriesOutOfRange {
                series,
                len: self.data.series.len(),
            })
    }

    fn points(series: &'a ChartSeries, target: EditTarget) -> &'a [ChartPoint] {
        match target {
            EditTarget::Value => &series.values,
            EditTarget::Label => &series.labels,
        }
    }

    fn shape(series: &'a ChartSeries, target: EditTarget) -> Option<&'a BlockShape> {
        match target {
            EditTarget::Value => series.values_shape.as_ref(),
            EditTarget::Label => series.labels_shape.as_ref(),
        }
    }

    /// 같은 목록의 점 증감을 반영한 블록의 최종 점 개수.
    fn final_len(&self, series: usize, target: EditTarget) -> Result<usize, PatchError> {
        let mut len = Self::points(self.series(series)?, target).len();
        for edit in self.edits {
            match edit {
                ChartEdit::AppendPoints {
                    series: s,
                    target: t,
                    texts,
                } if *s == series && *t == target => len += texts.len(),
                ChartEdit::TruncatePoints {
                    series: s,
                    target: t,
                    keep,
                } if *s == series && *t == target => len = *keep,
                _ => {}
            }
        }
        Ok(len)
    }

    fn add(&mut self, edit: &ChartEdit) -> Result<(), PatchError> {
        match edit {
            ChartEdit::Value(edit) => self.value(edit),
            ChartEdit::SeriesName { series, text } => self.series_name(*series, text),
            ChartEdit::AppendPoints {
                series,
                target,
                texts,
            } => self.append_points(*series, *target, texts),
            ChartEdit::TruncatePoints {
                series,
                target,
                keep,
            } => self.truncate_points(*series, *target, *keep),
            ChartEdit::AppendSeries {
                name,
                labels,
                values,
            } => self.append_series(name.as_deref(), labels.as_deref(), values),
            ChartEdit::TruncateSeries { keep } => self.truncate_series(*keep),
            ChartEdit::InsertSeries {
                at,
                name,
                labels,
                values,
            } => self.insert_series(*at, name.as_deref(), labels.as_deref(), values),
            ChartEdit::RemoveSeries { at } => self.remove_series(*at),
        }
    }

    fn value(&mut self, edit: &ValueEdit) -> Result<(), PatchError> {
        let series = self.series(edit.series)?;
        let points = Self::points(series, edit.target);
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
        if self.seen.contains(&key) {
            return Err(PatchError::DuplicateTarget {
                series: edit.series,
                point: edit.point,
            });
        }
        self.seen.push(key);
        let span = point.span.clone().ok_or(PatchError::ValueNotPatchable {
            series: edit.series,
            point: edit.point,
        })?;
        self.push(
            span,
            edit.text.as_bytes().to_vec(),
            Addr::Point {
                series: edit.series,
                point: edit.point,
            },
        );
        Ok(())
    }

    fn series_name(&mut self, series: usize, text: &str) -> Result<(), PatchError> {
        let s = self.series(series)?;
        let span = s
            .name_span
            .clone()
            .ok_or(PatchError::SeriesNameNotPatchable { series })?;
        if !is_safe_text(text) {
            return Err(PatchError::UnsafeName { series });
        }
        self.push(span, text.as_bytes().to_vec(), Addr::Structure { series });
        Ok(())
    }

    /// 블록의 `ptCount` 속성값을 `count` 로 치환하는 splice.
    fn recount(
        &mut self,
        series: usize,
        shape: &BlockShape,
        count: usize,
        target: EditTarget,
    ) -> Result<(), PatchError> {
        let pt_count = shape
            .pt_count
            .as_ref()
            .ok_or(PatchError::BlockNotResizable { series, target })?;
        self.push(
            pt_count.span.clone(),
            count.to_string().into_bytes(),
            Addr::Structure { series },
        );
        Ok(())
    }

    fn append_points(
        &mut self,
        series: usize,
        target: EditTarget,
        texts: &[String],
    ) -> Result<(), PatchError> {
        let s = self.series(series)?;
        let shape =
            Self::shape(s, target).ok_or(PatchError::BlockNotResizable { series, target })?;
        let at = shape
            .insert_at
            .ok_or(PatchError::BlockNotResizable { series, target })?;
        if shape.pt_count.is_none() {
            return Err(PatchError::BlockNotResizable { series, target });
        }
        let n_old = Self::points(s, target).len();
        let mut bytes = Vec::new();
        for (i, text) in texts.iter().enumerate() {
            if !is_safe_text(text) {
                return Err(PatchError::UnsafeText {
                    series,
                    point: n_old + i,
                });
            }
            bytes.extend_from_slice(point_element(&s.prefix, n_old + i, text).as_bytes());
        }
        if texts.is_empty() {
            return Ok(());
        }
        self.push(at..at, bytes, Addr::Structure { series });
        self.recount(series, shape, n_old + texts.len(), target)
    }

    fn truncate_points(
        &mut self,
        series: usize,
        target: EditTarget,
        keep: usize,
    ) -> Result<(), PatchError> {
        let s = self.series(series)?;
        let points = Self::points(s, target);
        if keep == 0 {
            return Err(PatchError::EmptyBlockRefused { series, target });
        }
        if keep >= points.len() {
            return Err(PatchError::PointOutOfRange {
                series,
                point: keep,
                len: points.len(),
            });
        }
        let shape =
            Self::shape(s, target).ok_or(PatchError::BlockNotResizable { series, target })?;
        let (Some(first), Some(last)) = (
            points[keep].element_span.clone(),
            points[points.len() - 1].element_span.clone(),
        ) else {
            return Err(PatchError::BlockNotResizable { series, target });
        };
        self.push(
            first.start..last.end,
            Vec::new(),
            Addr::Structure { series },
        );
        self.recount(series, shape, keep, target)
    }

    fn truncate_series(&mut self, keep: usize) -> Result<(), PatchError> {
        if keep == 0 {
            return Err(PatchError::EmptySeriesRefused);
        }
        let len = self.data.series.len();
        if keep >= len {
            return Err(PatchError::SeriesOutOfRange { series: keep, len });
        }
        if self.appended_series > 0
            || self.truncated_series
            || !self.inserted.is_empty()
            || !self.removed.is_empty()
        {
            return Err(PatchError::OverlappingStructureEdits { series: keep });
        }
        self.truncated_series = true;
        let first = self.data.series[keep].element_span.start;
        let last = self.data.series[len - 1].element_span.end;
        self.push(first..last, Vec::new(), Addr::Structure { series: keep });
        Ok(())
    }

    /// 마지막 계열을 템플릿으로 복제본 바이트를 만들어 그 뒤에 삽입한다.
    fn append_series(
        &mut self,
        name: Option<&str>,
        labels: Option<&[String]>,
        values: &[String],
    ) -> Result<(), PatchError> {
        let len = self.data.series.len();
        let new_index = len + self.appended_series;
        if self.truncated_series || !self.inserted.is_empty() || !self.removed.is_empty() {
            return Err(PatchError::OverlappingStructureEdits { series: new_index });
        }
        let clone = self.clone_series(new_index, name, labels, values, false)?;
        let at = self.data.series[len - 1].element_span.end;
        self.push(at..at, clone, Addr::Structure { series: new_index });
        self.appended_series += 1;
        Ok(())
    }

    /// [#6053] 복제본을 `at` 자리(최종 문서 기준)에 끼운다 — 앵커·재번호는 [`Planner::finish`].
    fn insert_series(
        &mut self,
        at: usize,
        name: Option<&str>,
        labels: Option<&[String]>,
        values: &[String],
    ) -> Result<(), PatchError> {
        if self.appended_series > 0 || self.truncated_series || !self.removed.is_empty() {
            return Err(PatchError::OverlappingStructureEdits { series: at });
        }
        let clone = self.clone_series(at, name, labels, values, true)?;
        self.inserted.push((at, clone));
        Ok(())
    }

    /// [#6053] `at` 계열(원본 기준)을 지운다 — 재번호는 [`Planner::finish`].
    fn remove_series(&mut self, at: usize) -> Result<(), PatchError> {
        if self.appended_series > 0 || self.truncated_series || !self.inserted.is_empty() {
            return Err(PatchError::OverlappingStructureEdits { series: at });
        }
        let len = self.data.series.len();
        if at >= len {
            return Err(PatchError::SeriesOutOfRange { series: at, len });
        }
        if self.removed.contains(&at) {
            return Err(PatchError::OverlappingStructureEdits { series: at });
        }
        self.removed.push(at);
        Ok(())
    }

    /// 마지막 계열을 템플릿으로 복제본 바이트를 만든다 — [`ChartEdit::AppendSeries`] 와
    /// [`ChartEdit::InsertSeries`] 가 같은 기계를 쓴다.
    ///
    /// 복제본은 템플릿 요소 바이트 조각을 **다시 스캔**해(구간이 조각 기준으로 나온다) 같은
    /// splice 로 고친다 — 재직렬화 없이 `c:f`·확장이 그대로 살아남는다. `number` 는 복제본의
    /// `c:idx`/`c:order` 값(= 최종 문서 기준 자리)이자 오류 보고 주소다. `strip_style` 이면
    /// 계열 최상위 `c:spPr` 와 `c:marker` 안 `c:symbol` 을 들어내 기본 스타일(Auto 마커·기본
    /// 선)로 되돌린다 — 한컴 편집기가 더한 계열에는 둘 다 없다(#6053 실측).
    fn clone_series(
        &self,
        number: usize,
        name: Option<&str>,
        labels: Option<&[String]>,
        values: &[String],
        strip_style: bool,
    ) -> Result<Vec<u8>, PatchError> {
        let len = self.data.series.len();
        let template_index = len - 1;
        let template = self.series(template_index)?;
        let new_index = number;
        let (Some(idx_span), Some(order_span)) = (&template.idx_span, &template.order_span) else {
            return Err(PatchError::SeriesNotClonable { series: new_index });
        };

        // 템플릿의 최종 모양 — 같은 목록의 점 증감을 반영한 행 수.
        let rows = self.final_len(template_index, EditTarget::Value)?;
        if values.len() != rows {
            return Err(PatchError::LengthMismatch {
                series: new_index,
                expected: rows,
                actual: values.len(),
            });
        }
        let has_labels = template.labels_shape.is_some() || !template.labels.is_empty();
        if let Some(labels) = labels {
            if labels.len() != rows {
                return Err(PatchError::LengthMismatch {
                    series: new_index,
                    expected: rows,
                    actual: labels.len(),
                });
            }
        }

        // 조각을 다시 스캔한다 — 구간은 조각 기준.
        let fragment = &self.xml[template.element_span.clone()];
        let fdata = scan_chart_values(fragment)
            .map_err(|_| PatchError::SeriesNotClonable { series: new_index })?;
        let fseries = fdata
            .series
            .first()
            .ok_or(PatchError::SeriesNotClonable { series: new_index })?;

        let mut fedits: Vec<ChartEdit> = Vec::new();
        // 값: 겹치는 점은 치환, 남는 쪽은 꼬리 증감.
        fill_block(&mut fedits, fseries, EditTarget::Value, values);
        // 라벨: 주면 값처럼, 안 주면 템플릿이 받는 라벨 편집을 물려받는다.
        if has_labels {
            match labels {
                Some(labels) => fill_block(&mut fedits, fseries, EditTarget::Label, labels),
                None => {
                    for edit in self.edits {
                        match edit {
                            ChartEdit::Value(v)
                                if v.series == template_index && v.target == EditTarget::Label =>
                            {
                                fedits.push(ChartEdit::Value(ValueEdit {
                                    series: 0,
                                    ..v.clone()
                                }));
                            }
                            ChartEdit::AppendPoints {
                                series,
                                target: EditTarget::Label,
                                texts,
                            } if *series == template_index => {
                                fedits.push(ChartEdit::AppendPoints {
                                    series: 0,
                                    target: EditTarget::Label,
                                    texts: texts.clone(),
                                });
                            }
                            ChartEdit::TruncatePoints {
                                series,
                                target: EditTarget::Label,
                                keep,
                            } if *series == template_index => {
                                fedits.push(ChartEdit::TruncatePoints {
                                    series: 0,
                                    target: EditTarget::Label,
                                    keep: *keep,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        if let Some(name) = name {
            fedits.push(ChartEdit::SeriesName {
                series: 0,
                text: name.to_string(),
            });
        }

        let mut inner = Planner::new(fragment, &fdata, &fedits);
        for edit in &fedits {
            inner.add(edit).map_err(|e| e.with_series(new_index))?;
        }
        // 빈 점 `<c:v/>` 는 텍스트 구간이 없어 치환이 막히지만, 신설 계열에서는 요소째 새로 쓴다.
        // (fill_block 이 그런 점을 건너뛰고 여기서 요소 치환으로 채운다.)
        for (i, text) in values.iter().enumerate() {
            if let Some(p) = fseries.values.get(i) {
                if p.span.is_none() {
                    let Some(element) = p.element_span.clone() else {
                        return Err(PatchError::BlockNotResizable {
                            series: new_index,
                            target: EditTarget::Value,
                        });
                    };
                    if !is_safe_text(text) {
                        return Err(PatchError::UnsafeText {
                            series: new_index,
                            point: i,
                        });
                    }
                    inner.push(
                        element,
                        point_element(&fseries.prefix, i, text).into_bytes(),
                        Addr::Point {
                            series: new_index,
                            point: i,
                        },
                    );
                }
            }
        }
        if let Some(labels) = labels {
            for (i, text) in labels.iter().enumerate() {
                if let Some(p) = fseries.labels.get(i) {
                    if p.span.is_none() {
                        let Some(element) = p.element_span.clone() else {
                            return Err(PatchError::BlockNotResizable {
                                series: new_index,
                                target: EditTarget::Label,
                            });
                        };
                        if !is_safe_text(text) {
                            return Err(PatchError::UnsafeText {
                                series: new_index,
                                point: i,
                            });
                        }
                        inner.push(
                            element,
                            point_element(&fseries.prefix, i, text).into_bytes(),
                            Addr::Point {
                                series: new_index,
                                point: i,
                            },
                        );
                    }
                }
            }
        }
        // 채번 — 조각 기준 구간은 fseries 의 것을 쓴다(템플릿 구간을 조각 오프셋으로 옮긴 것과 같다).
        let (Some(fidx), Some(forder)) = (&fseries.idx_span, &fseries.order_span) else {
            return Err(PatchError::SeriesNotClonable { series: new_index });
        };
        debug_assert_eq!(
            fidx.end - fidx.start,
            idx_span.end - idx_span.start,
            "템플릿과 조각의 idx 구간 길이가 다르다"
        );
        debug_assert_eq!(forder.end - forder.start, order_span.end - order_span.start);
        inner.push(
            fidx.clone(),
            new_index.to_string().into_bytes(),
            Addr::Structure { series: new_index },
        );
        inner.push(
            forder.clone(),
            new_index.to_string().into_bytes(),
            Addr::Structure { series: new_index },
        );
        // [#6053] 정체 경로 — 복제본의 스타일 구간을 들어내 기본으로 되돌린다. 꼬리 경로
        // (`AppendSeries`)는 템플릿 스타일을 그대로 물려받는다(바이트 불변 계약).
        if strip_style {
            if let Some(span) = fseries.sp_pr_span.clone() {
                inner.push(span, Vec::new(), Addr::Structure { series: new_index });
            }
            if let Some(span) = fseries.symbol_span.clone() {
                inner.push(span, Vec::new(), Addr::Structure { series: new_index });
            }
        }
        splice(fragment, inner.splices).map_err(|e| e.with_series(new_index))
    }

    /// [#6053] 정체 경로의 앵커·재번호 일괄 계산 — 편집을 전부 받은 뒤 한 번 돈다.
    fn finish(&mut self) -> Result<(), PatchError> {
        if !self.inserted.is_empty() && !self.removed.is_empty() {
            // add() 가 먼저 거르지만, 방어로 한 번 더.
            return Err(PatchError::OverlappingStructureEdits {
                series: self.inserted[0].0,
            });
        }
        if !self.inserted.is_empty() {
            self.finish_inserts()?;
        }
        if !self.removed.is_empty() {
            self.finish_removes()?;
        }
        Ok(())
    }

    /// 자리가 밀린 원본 계열의 `c:idx`/`c:order` 를 새 자리로 치환한다.
    ///
    /// `dPt` 안의 `c:idx` 는 스캐너가 서브트리째 건너뛰므로 여기 못 온다
    /// (`dpt_idx_does_not_override_series_idx` 가 고정).
    fn renumber(&mut self, series: usize, number: usize) -> Result<(), PatchError> {
        let s = self.series(series)?;
        let (Some(idx), Some(order)) = (s.idx_span.clone(), s.order_span.clone()) else {
            return Err(PatchError::SeriesNotRenumberable { series });
        };
        self.push(
            idx,
            number.to_string().into_bytes(),
            Addr::Structure { series },
        );
        self.push(
            order,
            number.to_string().into_bytes(),
            Addr::Structure { series },
        );
        Ok(())
    }

    /// 삽입 자리를 뺀 빈 칸에 원본 계열이 순서대로 들어간다고 보고, 각 삽입은 "바로 뒤에 올
    /// 원본"의 요소 시작을 앵커로 쓴다(뒤에 원본이 없으면 마지막 원본의 요소 끝).
    fn finish_inserts(&mut self) -> Result<(), PatchError> {
        let mut inserted = std::mem::take(&mut self.inserted);
        inserted.sort_by_key(|(at, _)| *at);
        let n = self.data.series.len();
        let n_final = n + inserted.len();
        for pair in inserted.windows(2) {
            if pair[0].0 == pair[1].0 {
                return Err(PatchError::OverlappingStructureEdits { series: pair[0].0 });
            }
        }
        if let Some((at, _)) = inserted.last() {
            if *at >= n_final {
                return Err(PatchError::SeriesOutOfRange {
                    series: *at,
                    len: n_final,
                });
            }
        }
        let insert_pos: Vec<usize> = inserted.iter().map(|(at, _)| *at).collect();
        // 원본 i 의 최종 자리 — 삽입 자리를 뺀 빈 칸을 순서대로 차지한다.
        let orig_final: Vec<usize> = (0..n_final)
            .filter(|slot| insert_pos.binary_search(slot).is_err())
            .collect();
        for (at, bytes) in inserted {
            // 최종 자리가 `at` 보다 큰 첫 원본 앞에 놓는다.
            let anchor = match orig_final.partition_point(|&f| f < at) {
                a if a < n => self.data.series[a].element_span.start,
                _ => self.data.series[n - 1].element_span.end,
            };
            self.push(anchor..anchor, bytes, Addr::Structure { series: at });
        }
        for (i, &slot) in orig_final.iter().enumerate() {
            if slot != i {
                self.renumber(i, slot)?;
            }
        }
        Ok(())
    }

    fn finish_removes(&mut self) -> Result<(), PatchError> {
        let mut removed = std::mem::take(&mut self.removed);
        removed.sort_unstable();
        let n = self.data.series.len();
        if removed.len() >= n {
            return Err(PatchError::EmptySeriesRefused);
        }
        for &at in &removed {
            let span = self.data.series[at].element_span.clone();
            self.push(span, Vec::new(), Addr::Structure { series: at });
        }
        for i in 0..n {
            if removed.binary_search(&i).is_ok() {
                continue;
            }
            let shift = removed.partition_point(|&r| r < i);
            if shift > 0 {
                self.renumber(i, i - shift)?;
            }
        }
        Ok(())
    }
}

/// 블록 하나를 `texts` 로 채우는 편집 — 겹치는 점은 치환(빈 점은 건너뛴다 — 호출자가 요소째
/// 쓴다), 모자라면 꼬리 추가, 남으면 꼬리 삭제.
fn fill_block(
    edits: &mut Vec<ChartEdit>,
    series: &ChartSeries,
    target: EditTarget,
    texts: &[String],
) {
    let points = match target {
        EditTarget::Value => &series.values,
        EditTarget::Label => &series.labels,
    };
    let overlap = points.len().min(texts.len());
    for (i, text) in texts.iter().enumerate().take(overlap) {
        if points[i].span.is_some() {
            edits.push(ChartEdit::Value(ValueEdit {
                series: 0,
                point: i,
                target,
                text: text.clone(),
            }));
        }
    }
    if texts.len() > points.len() {
        edits.push(ChartEdit::AppendPoints {
            series: 0,
            target,
            texts: texts[points.len()..].to_vec(),
        });
    } else if texts.len() < points.len() {
        edits.push(ChartEdit::TruncatePoints {
            series: 0,
            target,
            keep: texts.len(),
        });
    }
}

/// 새 점 요소 — 코퍼스(한컴) 형상 그대로 `<c:pt idx="n"><c:v>text</c:v></c:pt>`.
fn point_element(prefix: &str, idx: usize, text: &str) -> String {
    format!("<{prefix}pt idx=\"{idx}\"><{prefix}v>{text}</{prefix}v></{prefix}pt>")
}

/// [#5652] 편집 목록(값 치환 + 구조 편집)을 적용한 XML 을 만든다.
///
/// `data` 는 **같은 `xml`** 에서 나온 스캔이어야 한다 — 구간이 그 바이트 기준이다.
/// 하나라도 거부되면 `Err` 이고 아무것도 쓰지 않는다.
pub fn apply_chart_edits(
    xml: &[u8],
    data: &ChartData,
    edits: &[ChartEdit],
) -> Result<Vec<u8>, PatchError> {
    let mut planner = Planner::new(xml, data, edits);
    for edit in edits {
        planner.add(edit)?;
    }
    planner.finish()?;
    splice(xml, planner.splices)
}

/// 값 구간을 새 텍스트로 갈아끼운 XML 을 만든다.
///
/// `data` 는 **같은 `xml`** 에서 나온 스캔이어야 한다 — 구간이 그 바이트 기준이다.
/// 모든 편집이 자기 원래 텍스트를 되쓰면 결과는 입력과 바이트 동일하다.
///
/// [#5652] [`apply_chart_edits`] 의 값 전용 얇은 래퍼다 — B1 호출부는 그대로 돈다.
pub fn apply_value_edits(
    xml: &[u8],
    data: &ChartData,
    edits: &[ValueEdit],
) -> Result<Vec<u8>, PatchError> {
    let edits: Vec<ChartEdit> = edits.iter().cloned().map(ChartEdit::Value).collect();
    apply_chart_edits(xml, data, &edits)
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

    /// [#5652] 카테고리 라벨도 캐시 텍스트 치환 대상이다 — B1 의 `LabelNotEditable` 거부는
    /// 사라졌고, 허용 여부(의도 플래그)는 코어가 정한다. `c:f` 는 그대로다.
    #[test]
    fn category_label_edit_replaces_the_cache_text() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
            r#"<c:cat><c:strRef><c:f>Sheet1!$A$2</c:f><c:strCache><c:ptCount val="1"/>"#,
            r#"<c:pt idx="0"><c:v>항목</c:v></c:pt></c:strCache></c:strRef></c:cat>"#,
            r#"<c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
            r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("스캔");
        let out = apply_value_edits(
            xml.as_bytes(),
            &data,
            &[ValueEdit {
                series: 0,
                point: 0,
                target: EditTarget::Label,
                text: "새 항목".to_string(),
            }],
        )
        .expect("라벨 치환");
        let text = String::from_utf8(out).expect("UTF-8");
        assert!(text.contains("<c:v>새 항목</c:v>"), "{text}");
        assert!(text.contains("Sheet1!$A$2"), "c:f 가 사라졌다");
        assert!(text.contains("<c:v>1</c:v>"), "값이 바뀌었다");
    }
}
