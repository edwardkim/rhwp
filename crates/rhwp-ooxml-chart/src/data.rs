//! [#4100] OOXML 차트 XML 의 숫자 값을 **바이트 구간으로** 특정한다.
//!
//! ## 왜 별도 스캐너인가
//!
//! 같은 모듈의 [`parser`](super::parser) 는 렌더용 **손실 파서**다 —
//! `c:pt idx`·`c:f`·`c:externalData`·`extLst` 를 읽지 않는다. 파싱→재방출로
//! 왕복시키면 모델에 없는 것이 전부 사라지는데, `samples/chart` 코퍼스 28종은
//! 전건이 `c:extLst` 와 `ho:hncChartStyle` 을 갖고 있다. 그래서 편집은 재직렬화가
//! 아니라 **`c:v` 텍스트 구간만 바꾸는 최소 diff 바이트 수술**로 한다(#4055 실측).
//!
//! ## 구간을 얻는 방법
//!
//! 문자열 검색이 아니라 `quick_xml` + [`Reader::buffer_position`] 오프셋 추적이다.
//! `src/parser/hwpx/header.rs` 의 `paraHead` 원본 보존과 같은 관용구다 — 여는 태그
//! 직후 위치를 시작으로, **이번 이터레이션 read 직전 위치**를 끝으로 잡아 닫는 태그
//! 길이를 계산하지 않고 byte-exact 구간을 얻는다. 이 방식이라야
//!
//! - 접두어(`c:`)를 고정하지 않고 (`local_name` 비교),
//! - 코퍼스에 1건 있는 `c:numLit`/`c:strLit` 리터럴도 캐시형과 같은 경로로
//!
//! 잡힌다.
//!
//! `Reader::from_str` 이어야 `buffer_position()` 이 곧 입력 바이트 오프셋이다.

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::ops::Range;

/// 값 점 하나 — `<c:v>` 텍스트의 원본 바이트 구간과 그 텍스트.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartPoint {
    /// `c:pt idx` 속성값. 코퍼스는 전건 `0..n-1` 순차지만 해석하지 않고 그대로 싣는다.
    pub idx: u32,
    /// 원본 바이트 안 텍스트 구간 `[start, end)`.
    ///
    /// 빈 요소 `<c:v/>` 는 텍스트 구간이 없어 `None` 이다 — **읽을 수는 있고 고칠 수만
    /// 없다.** 결측치는 실데이터에서 흔하므로 문서 전체 읽기를 막지 않고, 그 점을 지목한
    /// 편집만 거부한다(`patch::PatchError::ValueNotPatchable`).
    pub span: Option<Range<usize>>,
    /// 구간의 바이트 그대로. **이스케이프를 해제하지 않는다** — 이 문자열을 되쓰면
    /// 바이트가 원본과 같아야 한다는 것이 최소 diff 의 기준이다.
    pub text: String,
    /// [#5652] `<c:pt …>…</c:pt>` **요소** 구간 `[start, end)`.
    ///
    /// `span` 이 텍스트를 고치는 주소라면 이것은 점을 **지우는** 주소다. 빈 값
    /// `<c:v/>` 도 요소 구간은 있다 — 지울 수는 있고 고칠 수만 없다. `c:pt` 없이
    /// 오는 `c:tx` 리터럴 `<c:tx><c:v>이름</c:v></c:tx>` 의 `c:v` 는 점이 아니라 `None`.
    pub element_span: Option<Range<usize>>,
}

/// 계열의 가로축 표현.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesAxis {
    /// `c:cat` + `c:val` — 라벨은 문자열이고 B1 에서 편집 대상이 아니다.
    Category,
    /// `c:xVal` + `c:yVal` (분산형) — X 도 수치이고 편집 대상이다.
    Scatter,
}

/// [#5652] 계열을 둘러싼 plot 요소(`c:barChart`·`c:pieChart`…)의 종류.
///
/// B2 의 종류별 가드(원형은 계열 1 고정, 주식형은 계열 수 고정)가 쓴다. 렌더용 손실
/// 파서(`OoxmlChart::parse`)를 두 번째로 불러 얻지 않고 **같은 스캔에서** 기록한다 —
/// 같은 바이트를 두 파서로 읽으면 콤보 차트 같은 경계에서 두 판정이 갈릴 수 있다.
/// 콤보(plot 요소가 둘 이상)에서는 계열마다 자기 plot 이 실린다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotKind {
    Bar,
    Line,
    Area,
    Pie,
    OfPie,
    Doughnut,
    Radar,
    Scatter,
    Bubble,
    Stock,
    /// `…Chart` 로 끝나지만 위 목록에 없는 plot 요소(`surfaceChart` 등).
    Other,
}

/// [#5652] `c:ptCount` 선언 — 속성값 텍스트 구간과 선언값.
///
/// 구조 편집은 점 개수를 바꾸므로 이 값을 **항상 재계산**해 되쓴다(#5447 실측 — 28/28
/// 한컴 불변식). 구간은 속성값 텍스트(`val="2"` 의 `2`)만이라 치환이 최소 diff 다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtCount {
    pub span: Range<usize>,
    pub value: u32,
}

/// [#5652] 라벨/값 블록(`c:cat`·`c:val`·`c:xVal`·`c:yVal`) 하나의 구조 좌표.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockShape {
    /// `<c:val>…</c:val>` 블록 전체 구간.
    pub element_span: Range<usize>,
    /// 캐시/리터럴 안의 `c:ptCount`. 없거나(리터럴에서 생략 가능) 속성값 구간을 확정하지
    /// 못하면 `None` — 그 블록의 개수 변경은 거부된다(fail-closed).
    pub pt_count: Option<PtCount>,
    /// 점 삽입 앵커 — 마지막 `c:pt` 요소 끝. 점이 없으면 `c:ptCount` 요소 끝, 그것도
    /// 없으면 캐시 닫는 태그 직전. 캐시(`numCache`/`strCache`/`numLit`/`strLit`)가 아예
    /// 없으면(다층 `multiLvlStrCache` 포함) `None`.
    pub insert_at: Option<usize>,
}

/// 계열 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartSeries {
    /// `c:tx` 의 계열명. 참조도 리터럴도 없으면 `None`.
    pub name: Option<String>,
    pub axis: SeriesAxis,
    /// [`SeriesAxis::Category`] 면 `c:cat` 라벨, [`SeriesAxis::Scatter`] 면 `c:xVal`.
    pub labels: Vec<ChartPoint>,
    /// [`SeriesAxis::Category`] 면 `c:val`, [`SeriesAxis::Scatter`] 면 `c:yVal`.
    pub values: Vec<ChartPoint>,
    /// 라벨이 `c:multiLvlStrRef`(다층 카테고리)에서 왔는가.
    ///
    /// 코퍼스 0건이다. 스캐너는 표지만 세우고 판단하지 않는다 — `multiLvlStrCache`
    /// 는 캐시 목록에 없어 층 라벨이 실리지 않는다(`labels` 빈 목록). 라벨을 행
    /// 머리로 쓰는 CSV 층이 이 표지를 보고 거부한다. 값 편집 자체는 막지 않는다.
    pub labels_multi_level: bool,
    /// [#5652] 둘러싼 plot 요소의 종류.
    pub plot: PlotKind,
    /// [#5652] `c:ser` qname 의 접두어(코퍼스 `"c:"`, 접두어 없으면 `""`). 스캐너는
    /// 접두어를 고정하지 않으므로 새 요소(`<c:pt>`)를 합성할 때 이것을 쓴다.
    pub prefix: String,
    /// [#5652] `<c:ser>…</c:ser>` 요소 구간 — 계열 복제 템플릿이자 삭제 주소.
    pub element_span: Range<usize>,
    /// [#5652] 계열명 `c:tx` 캐시 텍스트 구간. 이름이 없으면 `None`.
    pub name_span: Option<Range<usize>>,
    /// [#5652] `c:idx val` 속성값 구간 — 복제 계열의 채번 자리.
    pub idx_span: Option<Range<usize>>,
    /// [#5652] `c:order val` 속성값 구간.
    pub order_span: Option<Range<usize>>,
    /// [#5652] 라벨 블록(`c:cat`/`c:xVal`) 좌표. 블록이 없으면 `None`.
    pub labels_shape: Option<BlockShape>,
    /// [#5652] 값 블록(`c:val`/`c:yVal`) 좌표.
    pub values_shape: Option<BlockShape>,
    /// [#6053] 계열 최상위 `<c:spPr>…</c:spPr>` 구간. 계열을 새로 끼울 때 복제본에서 들어내
    /// **기본 스타일**로 되돌리는 데 쓴다 — 한컴 편집기가 더한 계열에는 `c:spPr` 이 없다.
    pub sp_pr_span: Option<Range<usize>>,
    /// [#6053] 계열 최상위 `<c:marker>` 안의 `<c:symbol …/>` 구간. 들어내면 Auto 마커가 된다.
    pub symbol_span: Option<Range<usize>>,
}

/// 차트 하나의 편집 가능한 데이터 지도.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChartData {
    pub series: Vec<ChartSeries>,
    /// [#6037] plot 안에 `<c:upDownBars>` 캔들 장치가 있는가.
    ///
    /// 주식형 OHLC 만 갖는다. 이 장치는 **첫 계열과 끝 계열**을 몸통으로 삼으므로,
    /// 계열을 더하거나 지워 **끝 계열이 바뀌면 몸통이 엉뚱한 짝으로 다시 잡혀** 캔들이
    /// 통째로 검은 박스가 된다(#6037 한컴 실측 — 꼬리 삽입·꼬리 삭제 2건). 반대로 중간
    /// 삽입·중간 삭제는 끝이 유지되어 정상이고, 장치가 없는 HLC 는 꼬리를 건드려도 정상이다.
    ///
    /// 렌더 쪽 모델의 동명 필드(`crate::OoxmlChartSeries::has_up_down_bars`, C2a #2277)와
    /// 같은 토큰을 보지만, 이쪽은 편집 검증용이라 **차트 단위**로 둔다.
    pub has_up_down_bars: bool,
}

/// 스캔 실패 사유.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartScanError {
    /// 입력이 UTF-8 이 아니다. OOXML 차트 파트는 UTF-8 XML 이어야 한다.
    NotUtf8,
    /// XML 파싱 오류.
    Xml(String),
    /// `c:ser` 가 하나도 없다 — 편집할 데이터가 없다.
    NoSeries,
    /// `c:pt idx` 가 벡터 위치와 다르다 — 희소·역순·중복.
    ///
    /// B1 은 점을 **벡터 출현 순서**로 지목한다(`idx` 소비처 0). 비순차 문서에서는
    /// CSV 행 번호와 편집 주소가 조용히 다른 점을 가리키므로, 아무것도 읽지도 쓰지도
    /// 않는다. `idx`/`c:ptCount` 논리 행 모델(빈 점 지원)은 후속 PR 몫이다.
    /// `pt_index` 의 0 폴백(파싱 실패 → 0)은 위치 0 에서만 이 검사를 통과한다 —
    /// `Option` 화도 같은 후속 몫으로 남긴다.
    NonSequentialPointIndex {
        /// 계열 번호(0-based) — `validate()` 의 기존 관례.
        series: usize,
        /// 어느 블록인가 — `"cat"`/`"val"`/`"xVal"`/`"yVal"`.
        block: &'static str,
        /// 벡터 위치.
        position: usize,
        /// 그 위치의 실제 `idx` 값.
        idx: u32,
    },
}

impl std::fmt::Display for ChartScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotUtf8 => write!(f, "차트 XML 이 UTF-8 이 아니다"),
            Self::Xml(msg) => write!(f, "차트 XML 파싱 실패: {msg}"),
            Self::NoSeries => write!(f, "차트에 c:ser 가 없다"),
            Self::NonSequentialPointIndex {
                series,
                block,
                position,
                idx,
            } => write!(
                f,
                "계열 {series} 의 c:{block} 에서 c:pt idx 가 순차가 아니다 — \
                 {position} 번째 점의 idx 가 {idx} 다 (희소·역순·중복 idx 는 아직 \
                 지원하지 않는다)"
            ),
        }
    }
}

/// 스캔에서 **통째로 건너뛰는** 서브트리.
///
/// 확장·데이터라벨·추세선 안에는 값처럼 생긴 것이 들어 있다. 특히 `c:extLst` 는 표준
/// 확장 지점이라 `c15:filteredCategoryTitle` 처럼 **`c:cat`/`c:pt`/`c:v` 를 통째로 품는**
/// 확장이 들어올 수 있다. 이 스캐너는 접두어를 고정하지 않으므로(그래야 `chart:` 든
/// `c:` 든 돈다) 네임스페이스로 걸러내지 못한다 — 서브트리째 건너뛰는 쪽이 확실하다.
///
/// 같은 이유로 `c:dLbls`/`c:dLbl` 도 뺀다. 데이터 라벨의 `c:tx` 가 계열명으로 새어
/// 들어올 수 있는데, B1 은 라벨을 읽지도 쓰지도 않는다.
///
/// [#5652] `c:dPt`(점별 서식)도 뺀다 — 안에 `c:idx` 가 있어 계열의 `c:idx` 구간을 덮을 수
/// 있다(코퍼스 0건, 방어).
const SKIPPED_SUBTREES: &[&[u8]] = &[
    b"extLst",
    b"dLbls",
    b"dLbl",
    b"trendline",
    b"errBars",
    b"dPt",
];

fn is_skipped(local: &[u8]) -> bool {
    SKIPPED_SUBTREES.contains(&local)
}

/// `c:ser` 안에서 지금 어느 자식 블록에 있는가.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    /// `c:tx` — 계열명
    Tx,
    /// `c:cat` — 카테고리 라벨
    Cat,
    /// `c:val` — 값
    Val,
    /// `c:xVal` — 분산형 X
    XVal,
    /// `c:yVal` — 분산형 Y
    YVal,
}

/// [#5652] 열려 있는 라벨/값 블록의 좌표를 모으는 중간 상태.
struct BlockBuild {
    /// 블록 여는 태그 `<` 오프셋.
    start: usize,
    pt_count: Option<PtCount>,
    insert_at: Option<usize>,
}

/// 스캔 진행 상태.
struct ScanState {
    series: Vec<ChartSeries>,
    cur: Option<ChartSeries>,
    section: Section,
    /// `numCache`/`numLit`/`strCache`/`strLit` 안인가.
    in_cache: bool,
    /// 현재 `c:pt` 의 `idx`.
    cur_idx: Option<u32>,
    /// `<c:v>` 여는 태그 직후 오프셋.
    v_start: Option<usize>,
    /// 현재 계열의 라벨이 다층 참조에서 왔는가.
    multi_level: bool,
    /// [#5652] 지금 둘러싼 plot 요소의 종류. plot 요소 밖이면 `Other`.
    plot: PlotKind,
    /// [#5652] 현재 `<c:ser>` 여는 태그 `<` 오프셋.
    ser_start: usize,
    /// [#5652] 현재 `<c:pt>` 여는 태그 `<` 오프셋과, 그 시점의 블록 점 개수.
    /// 개수를 함께 두는 이유 — `c:v` 없는 `c:pt` 는 점을 싣지 않으므로 `End(pt)` 에서
    /// "이 pt 가 점을 실었는가"를 개수 증가로 판정해 엉뚱한 앞 점에 구간을 달지 않는다.
    pt_start: Option<(usize, usize)>,
    /// [#5652] 열려 있는 라벨/값 블록.
    block: Option<BlockBuild>,
    /// [#6037] `<c:upDownBars>` 를 한 번이라도 봤는가 — 차트 단위 플래그.
    has_up_down_bars: bool,
    /// [#6053] 계열 최상위 `c:spPr` 의 시작 오프셋과 깊이 — 마커 등 하위 spPr 과 섞이지 않게 센다.
    sp_pr_depth: usize,
    sp_pr_start: Option<usize>,
    /// [#6053] 계열 최상위 `c:marker` 안인가 — 그 안의 `c:symbol` 만 잡는다.
    marker_depth: usize,
    /// [#6053] 확장꼴 `<c:symbol …></c:symbol>`(코퍼스 0건, 방어)의 여는 태그 오프셋.
    symbol_start: Option<usize>,
}

impl ScanState {
    fn new() -> Self {
        Self {
            series: Vec::new(),
            cur: None,
            section: Section::None,
            in_cache: false,
            cur_idx: None,
            v_start: None,
            multi_level: false,
            plot: PlotKind::Other,
            ser_start: 0,
            pt_start: None,
            block: None,
            has_up_down_bars: false,
            sp_pr_depth: 0,
            sp_pr_start: None,
            marker_depth: 0,
            symbol_start: None,
        }
    }

    /// 현재 섹션이 라벨/값 블록(`c:tx` 가 아닌 쪽)인가.
    fn in_block(&self) -> bool {
        matches!(
            self.section,
            Section::Cat | Section::Val | Section::XVal | Section::YVal
        )
    }

    /// 현재 블록의 점 목록 길이 — `pt_start` 의 개수 판정용.
    fn block_len(&self) -> usize {
        let Some(cur) = self.cur.as_ref() else {
            return 0;
        };
        match self.section {
            Section::Cat | Section::XVal => cur.labels.len(),
            Section::Val | Section::YVal => cur.values.len(),
            _ => 0,
        }
    }

    /// 현재 블록의 마지막 점.
    fn last_point_mut(&mut self) -> Option<&mut ChartPoint> {
        let section = self.section;
        let cur = self.cur.as_mut()?;
        match section {
            Section::Cat | Section::XVal => cur.labels.last_mut(),
            Section::Val | Section::YVal => cur.values.last_mut(),
            _ => None,
        }
    }

    /// 지금 `<c:v>` 를 값으로 거둘 문맥인가.
    ///
    /// `c:tx` 는 캐시·`c:pt` 없이 `<c:tx><c:v>이름</c:v></c:tx>` 로도 쓰이므로
    /// 블록 안이면 곧바로 거둔다. 나머지 블록은 캐시/리터럴 안의 `c:pt` 로 제한해
    /// `c:f`(참조 수식)·`c:formatCode` 같은 형제 텍스트를 값으로 오인하지 않는다.
    fn capturing(&self) -> bool {
        if self.cur.is_none() {
            return false;
        }
        match self.section {
            Section::None => false,
            Section::Tx => true,
            _ => self.in_cache && self.cur_idx.is_some(),
        }
    }
}

/// `c:pt` 의 `idx` 속성.
fn pt_index(e: &BytesStart) -> u32 {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref().as_bytes() == b"idx" {
            return attr.value.as_ref().parse().unwrap_or(0);
        }
    }
    0
}

/// [#5652] plot 요소 local name → 종류. plot 요소가 아니면 `None`.
///
/// `…Chart` 로 끝나는 요소만 plot 이다 — `chart`·`chartSpace` 는 아니다(대소문자 구분).
fn plot_kind(local: &[u8]) -> Option<PlotKind> {
    Some(match local {
        b"barChart" | b"bar3DChart" => PlotKind::Bar,
        b"lineChart" | b"line3DChart" => PlotKind::Line,
        b"areaChart" | b"area3DChart" => PlotKind::Area,
        b"pieChart" | b"pie3DChart" => PlotKind::Pie,
        b"ofPieChart" => PlotKind::OfPie,
        b"doughnutChart" => PlotKind::Doughnut,
        b"radarChart" => PlotKind::Radar,
        b"scatterChart" => PlotKind::Scatter,
        b"bubbleChart" => PlotKind::Bubble,
        b"stockChart" => PlotKind::Stock,
        other if other.ends_with(b"Chart") => PlotKind::Other,
        _ => return None,
    })
}

/// [#5652] qname 에서 local name 을 뺀 접두어 — `c:ser` → `"c:"`, `ser` → `""`.
fn prefix_of(e: &BytesStart) -> String {
    let qname = e.name();
    let qname = qname.as_ref();
    let local_len = e.local_name().as_ref().len();
    qname[..qname.len().saturating_sub(local_len)].to_owned()
}

/// [#5652] 속성 `name` 의 값 — quick_xml 이 파싱한 그대로(이스케이프 해제 없음).
fn attr_value(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|attr| attr.key.local_name().as_ref().as_bytes() == name)
        .map(|attr| attr.value.as_ref().to_owned())
}

/// [#5652] 태그 하나의 raw 구간(`tag`, `<` 부터 `>` 까지) 안에서 속성 `name` 의 값 텍스트
/// 구간을 찾는다. 반환 구간은 `tag` 기준 상대 오프셋이다.
///
/// quick_xml 은 속성의 바이트 오프셋을 주지 않으므로 **그 태그 하나의 바이트 안에서만**
/// 수동으로 찾는다(이벤트 경계 밖을 보지 않는다). 호출자는 찾은 구간을 다시 잘라
/// [`attr_value`] 와 같은지 확인한다 — 다르면 구간을 버린다(fail-closed).
fn attr_value_span_in_tag(tag: &str, name: &str) -> Option<Range<usize>> {
    let bytes = tag.as_bytes();
    let mut from = 0usize;
    while let Some(found) = tag[from..].find(name) {
        let start = from + found;
        let after_name = start + name.len();
        // 속성 이름은 공백 뒤에 오고(`<c:ptCount val=`), 뒤에는 `=` 가 온다.
        let preceded_by_space = start > 0 && bytes[start - 1].is_ascii_whitespace();
        let mut i = after_name;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if preceded_by_space && i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            let quote = *bytes.get(i)?;
            if quote != b'"' && quote != b'\'' {
                return None;
            }
            let value_start = i + 1;
            let value_len = tag[value_start..].find(quote as char)?;
            return Some(value_start..value_start + value_len);
        }
        from = after_name;
    }
    None
}

/// [#5652] `Start`/`Empty` 이벤트 하나의 속성값 구간을 **입력 기준 절대 오프셋**으로 얻는다.
///
/// `tag_range` 는 그 이벤트의 raw 태그 구간(이터레이션 read 직전 위치 .. 직후 위치).
/// 재슬라이스가 quick_xml 의 속성값과 다르면 `None`.
fn attr_value_span(
    text: &str,
    tag_range: Range<usize>,
    e: &BytesStart,
    name: &str,
) -> Option<(Range<usize>, String)> {
    let value = attr_value(e, name.as_bytes())?;
    let tag = text.get(tag_range.clone())?;
    let rel = attr_value_span_in_tag(tag, name)?;
    let abs = tag_range.start + rel.start..tag_range.start + rel.end;
    (text.get(abs.clone())? == value).then_some((abs, value))
}

fn open_section(state: &mut ScanState, local: &[u8], pos_before: usize) {
    let section = match local {
        b"tx" => Section::Tx,
        b"cat" => Section::Cat,
        b"val" => Section::Val,
        b"xVal" => Section::XVal,
        b"yVal" => Section::YVal,
        _ => return,
    };
    if state.cur.is_some() {
        state.section = section;
        // 블록이 열릴 때 축을 확정한다. xVal/yVal 이 하나라도 나오면 분산형이다.
        if let Some(cur) = state.cur.as_mut() {
            if matches!(section, Section::XVal | Section::YVal) {
                cur.axis = SeriesAxis::Scatter;
            }
        }
        // [#5652] 라벨/값 블록이면 구조 좌표 수집을 시작한다.
        if state.in_block() {
            state.block = Some(BlockBuild {
                start: pos_before,
                pt_count: None,
                insert_at: None,
            });
        }
    }
}

/// [#5652] 블록이 닫힐 때 모은 좌표를 계열에 싣는다.
fn close_block(state: &mut ScanState, end: usize) {
    let section = state.section;
    let Some(build) = state.block.take() else {
        return;
    };
    let shape = BlockShape {
        element_span: build.start..end,
        pt_count: build.pt_count,
        insert_at: build.insert_at,
    };
    if let Some(cur) = state.cur.as_mut() {
        match section {
            Section::Cat | Section::XVal => cur.labels_shape = Some(shape),
            Section::Val | Section::YVal => cur.values_shape = Some(shape),
            _ => {}
        }
    }
}

fn push_point(state: &mut ScanState, point: ChartPoint) {
    let section = state.section;
    let Some(cur) = state.cur.as_mut() else {
        return;
    };
    match section {
        Section::Tx => {
            if cur.name.is_none() {
                // [#5652] 구간도 함께 둔다 — 계열명 제자리 치환의 주소다.
                cur.name_span = point.span;
                cur.name = Some(point.text);
            }
        }
        Section::Cat | Section::XVal => cur.labels.push(point),
        Section::Val | Section::YVal => cur.values.push(point),
        Section::None => {}
    }
}

/// [#5652] 라벨/값 블록 캐시 안의 `c:ptCount` 를 기록한다 — 첫 것만, 캐시 밖(`c:tx`·다층)은 무시.
fn note_pt_count(state: &mut ScanState, text: &str, tag_range: Range<usize>, e: &BytesStart) {
    if !(state.in_cache && state.in_block() && state.cur.is_some()) {
        return;
    }
    let Some(block) = state.block.as_mut() else {
        return;
    };
    if block.pt_count.is_some() {
        return;
    }
    // 구간을 못 잡거나 수치가 아니면 `None` 으로 둔다 — 그 블록의 개수 변경만 거부된다.
    block.pt_count = attr_value_span(text, tag_range, e, "val").and_then(|(span, value)| {
        value
            .parse::<u32>()
            .ok()
            .map(|value| PtCount { span, value })
    });
}

/// [#5652] 계열 직속 `c:idx`/`c:order` 의 속성값 구간 — 첫 것만, 블록·캐시 안은 무시.
fn note_series_index(state: &mut ScanState, text: &str, tag_range: Range<usize>, e: &BytesStart) {
    if state.section != Section::None || state.in_cache {
        return;
    }
    let Some(cur) = state.cur.as_mut() else {
        return;
    };
    let slot = match e.local_name().as_ref().as_bytes() {
        b"idx" => &mut cur.idx_span,
        b"order" => &mut cur.order_span,
        _ => return,
    };
    if slot.is_none() {
        *slot = attr_value_span(text, tag_range, e, "val").map(|(span, _)| span);
    }
}

/// 차트 XML 을 훑어 계열별 값의 바이트 구간을 얻는다.
///
/// 반환된 [`ChartPoint::span`] 은 **입력 `xml` 슬라이스 기준**이며
/// `&xml[span] == text.as_bytes()` 가 항상 성립한다.
///
/// [#5652] 같은 스캔에서 구조 좌표(점·계열 요소 구간, `ptCount`, 삽입 앵커, 계열명·idx·order
/// 구간, plot 종류)도 함께 기록한다 — 두 번 읽지 않는다(두 좌표계가 어긋날 수 있다).
pub fn scan_chart_values(xml: &[u8]) -> Result<ChartData, ChartScanError> {
    let text = std::str::from_utf8(xml).map_err(|_| ChartScanError::NotUtf8)?;

    let mut reader = Reader::from_str(text);
    let mut state = ScanState::new();
    let mut buf = Vec::new();
    // 건너뛰는 서브트리 안의 요소 깊이. 0 이면 정상 스캔 중이다.
    let mut skip_depth = 0usize;

    loop {
        let pos_before = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if skip_depth > 0 => {
                let _ = e;
                skip_depth += 1;
            }
            Ok(Event::End(ref e)) if skip_depth > 0 => {
                let _ = e;
                skip_depth -= 1;
            }
            Ok(Event::Empty(_)) if skip_depth > 0 => {}
            Ok(Event::Start(ref e)) if is_skipped(e.local_name().as_ref().as_bytes()) => {
                skip_depth = 1
            }
            Ok(Event::Start(ref e)) => {
                // 이번 이벤트의 raw 태그 구간 — `<` 부터 `>` 까지. 속성값 구간은 이 안에서만 찾는다.
                let tag_range = pos_before..reader.buffer_position() as usize;
                match e.local_name().as_ref().as_bytes() {
                    b"ser" => {
                        state.cur = Some(ChartSeries {
                            name: None,
                            axis: SeriesAxis::Category,
                            labels: Vec::new(),
                            values: Vec::new(),
                            labels_multi_level: false,
                            plot: state.plot,
                            prefix: prefix_of(e),
                            element_span: pos_before..pos_before,
                            name_span: None,
                            idx_span: None,
                            order_span: None,
                            labels_shape: None,
                            values_shape: None,
                            sp_pr_span: None,
                            symbol_span: None,
                        });
                        state.multi_level = false;
                        state.ser_start = pos_before;
                        state.block = None;
                        state.sp_pr_depth = 0;
                        state.sp_pr_start = None;
                        state.marker_depth = 0;
                        state.symbol_start = None;
                    }
                    // [#6053] 계열 최상위 spPr·marker 를 깊이로 가려 잡는다. dPt·dLbls 등
                    // spPr 을 품는 다른 자식은 SKIPPED_SUBTREES 라 여기 못 온다.
                    b"spPr" if state.cur.is_some() => {
                        state.sp_pr_depth += 1;
                        if state.sp_pr_depth == 1 && state.marker_depth == 0 {
                            state.sp_pr_start = Some(pos_before);
                        }
                    }
                    b"marker" if state.cur.is_some() => state.marker_depth += 1,
                    // 확장꼴 `<c:symbol …></c:symbol>`(코퍼스 0건, 방어) — 요소 전체 구간을 잡는다.
                    b"symbol" if state.marker_depth == 1 => {
                        state.symbol_start = Some(pos_before);
                    }
                    b"multiLvlStrRef" => state.multi_level = true,
                    b"numCache" | b"numLit" | b"strCache" | b"strLit" => state.in_cache = true,
                    b"pt" => {
                        state.cur_idx = Some(pt_index(e));
                        state.pt_start = Some((pos_before, state.block_len()));
                    }
                    b"v" => {
                        if state.capturing() {
                            // 여는 태그 `<c:v>` 직후 = 텍스트 시작.
                            state.v_start = Some(reader.buffer_position() as usize);
                        }
                    }
                    // `<c:ptCount val="n"></c:ptCount>` 꼴(코퍼스 0건) — 빈 요소와 같이 다룬다.
                    b"ptCount" => note_pt_count(&mut state, text, tag_range, e),
                    // [#6037] 캔들 장치 — plot 의 자식이라 계열 밖에서 나온다.
                    b"upDownBars" => state.has_up_down_bars = true,
                    b"idx" | b"order" => note_series_index(&mut state, text, tag_range, e),
                    local => {
                        if let Some(kind) = plot_kind(local) {
                            state.plot = kind;
                        } else {
                            open_section(&mut state, local, pos_before);
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag_range = pos_before..reader.buffer_position() as usize;
                match e.local_name().as_ref().as_bytes() {
                    // 빈 요소 `<c:v/>` — 결측치다. 텍스트 구간이 없으니 구간 없이 싣는다.
                    // 읽기는 되고 그 점의 편집만 거부된다.
                    b"v" if state.capturing() => {
                        let idx = state.cur_idx.unwrap_or(0);
                        push_point(
                            &mut state,
                            ChartPoint {
                                idx,
                                span: None,
                                text: String::new(),
                                element_span: None,
                            },
                        );
                    }
                    b"ptCount" => {
                        note_pt_count(&mut state, text, tag_range.clone(), e);
                        // 점이 하나도 없으면 ptCount 요소 끝이 삽입 앵커다.
                        if state.in_cache {
                            if let Some(block) = state.block.as_mut() {
                                block.insert_at = Some(tag_range.end);
                            }
                        }
                    }
                    b"idx" | b"order" => note_series_index(&mut state, text, tag_range, e),
                    // [#6037] `<c:upDownBars/>` 자기닫힘 꼴(코퍼스 0건) — 여는 태그와 같이 본다.
                    b"upDownBars" => state.has_up_down_bars = true,
                    // [#6053] `<c:symbol val="…"/>` — 계열 최상위 marker 안의 것만.
                    b"symbol" if state.marker_depth == 1 => {
                        if let Some(ser) = state.cur.as_mut() {
                            if ser.symbol_span.is_none() {
                                ser.symbol_span = Some(tag_range.clone());
                            }
                        }
                    }
                    // 자기닫힘 `<c:marker/>` 는 열고 바로 닫는다.
                    b"marker" if state.cur.is_some() => {}
                    // 자기닫힘 `<c:spPr/>`(코퍼스 0건, 방어) — 계열 최상위일 때만 구간으로.
                    b"spPr" if state.cur.is_some() => {
                        if state.sp_pr_depth == 0 && state.marker_depth == 0 {
                            if let Some(ser) = state.cur.as_mut() {
                                if ser.sp_pr_span.is_none() {
                                    ser.sp_pr_span = Some(tag_range.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let end = reader.buffer_position() as usize;
                match e.local_name().as_ref().as_bytes() {
                    b"v" => {
                        if let Some(start) = state.v_start.take() {
                            // 닫는 태그 직전 = 텍스트 끝. 닫는 태그 길이를 계산하지 않는다.
                            let span = start..pos_before;
                            let idx = state.cur_idx.unwrap_or(0);
                            let body = text
                                .get(span.clone())
                                .map(str::to_string)
                                .unwrap_or_default();
                            push_point(
                                &mut state,
                                ChartPoint {
                                    idx,
                                    span: Some(span),
                                    text: body,
                                    element_span: None,
                                },
                            );
                        }
                    }
                    b"pt" => {
                        state.cur_idx = None;
                        if let Some((start, len_before)) = state.pt_start.take() {
                            // 이 pt 가 점을 실었을 때만 마지막 점에 요소 구간을 단다.
                            if state.in_block() && state.block_len() > len_before {
                                if let Some(point) = state.last_point_mut() {
                                    point.element_span = Some(start..end);
                                }
                            }
                            // 캐시 안의 pt 요소 끝이 곧 다음 점의 삽입 앵커다.
                            if state.in_cache {
                                if let Some(block) = state.block.as_mut() {
                                    block.insert_at = Some(end);
                                }
                            }
                        }
                    }
                    b"ptCount" => {
                        if state.in_cache {
                            if let Some(block) = state.block.as_mut() {
                                block.insert_at = Some(end);
                            }
                        }
                    }
                    b"numCache" | b"numLit" | b"strCache" | b"strLit" => {
                        if state.in_cache {
                            if let Some(block) = state.block.as_mut() {
                                // 점도 ptCount 도 없는 빈 캐시 — 닫는 태그 직전.
                                block.insert_at.get_or_insert(pos_before);
                            }
                        }
                        state.in_cache = false;
                    }
                    b"tx" | b"cat" | b"val" | b"xVal" | b"yVal" => {
                        close_block(&mut state, end);
                        state.section = Section::None;
                    }
                    b"ser" => {
                        if let Some(mut cur) = state.cur.take() {
                            cur.labels_multi_level = state.multi_level;
                            cur.element_span = state.ser_start..end;
                            state.series.push(cur);
                        }
                        state.section = Section::None;
                        state.in_cache = false;
                        state.cur_idx = None;
                        state.block = None;
                        state.sp_pr_depth = 0;
                        state.sp_pr_start = None;
                        state.marker_depth = 0;
                        state.symbol_start = None;
                    }
                    // [#6053] 계열 최상위 spPr 구간 확정. marker 하위 spPr 은 시작을 기록하지
                    // 않았으므로(`sp_pr_start` None) 여기서도 아무것도 확정하지 않는다.
                    b"spPr" if state.cur.is_some() => {
                        if state.sp_pr_depth > 0 {
                            state.sp_pr_depth -= 1;
                            if state.sp_pr_depth == 0 {
                                if let Some(start) = state.sp_pr_start.take() {
                                    if let Some(ser) = state.cur.as_mut() {
                                        if ser.sp_pr_span.is_none() {
                                            ser.sp_pr_span = Some(start..end);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"marker" if state.cur.is_some() => {
                        state.marker_depth = state.marker_depth.saturating_sub(1);
                    }
                    b"symbol" if state.marker_depth == 1 => {
                        if let Some(start) = state.symbol_start.take() {
                            if let Some(ser) = state.cur.as_mut() {
                                if ser.symbol_span.is_none() {
                                    ser.symbol_span = Some(start..end);
                                }
                            }
                        }
                    }
                    local => {
                        if plot_kind(local).is_some() {
                            state.plot = PlotKind::Other;
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ChartScanError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    if state.series.is_empty() {
        return Err(ChartScanError::NoSeries);
    }

    // [#4603 리뷰] fail-closed — `idx[i] == i` 가 아니면(희소·역순·중복) 여기서 멈춘다.
    // 읽기(chart_data_at)·쓰기(apply_chart_edits)·CSV 가 전부 이 함수를 지나므로 세
    // 경로가 한 번에 닫힌다. 코퍼스는 전건 순차(계획서 §2 실측 "비순차 0")라 합성
    // 테스트만이 이 경로를 밟는다.
    for (si, series) in state.series.iter().enumerate() {
        let scatter = series.axis == SeriesAxis::Scatter;
        let blocks: [(&'static str, &[ChartPoint], bool); 2] = [
            // 다층 카테고리의 labels 는 면제한다. 지금은 `multiLvlStrCache` 가 캐시
            // 목록에 없어 층 라벨이 실리지도 않지만(빈 목록 — 자명 통과), 실리게
            // 되면 idx 가 층마다 0..n-1 로 **반복되는 것이 정상 형상**이라 이 검사를
            // 통과할 수 없다. values 는 다층 여부와 무관하게 검사한다.
            (
                if scatter { "xVal" } else { "cat" },
                &series.labels,
                series.labels_multi_level,
            ),
            (if scatter { "yVal" } else { "val" }, &series.values, false),
        ];
        for (block, points, multi_level_exempt) in blocks {
            if multi_level_exempt {
                continue;
            }
            if let Some((position, p)) = points
                .iter()
                .enumerate()
                .find(|(i, p)| p.idx as usize != *i)
            {
                return Err(ChartScanError::NonSequentialPointIndex {
                    series: si,
                    block,
                    position,
                    idx: p.idx,
                });
            }
        }
    }

    Ok(ChartData {
        series: state.series,
        has_up_down_bars: state.has_up_down_bars,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATEGORY_CHART: &str = concat!(
        r#"<c:chartSpace><c:chart><c:plotArea><c:barChart>"#,
        r#"<c:ser><c:idx val="0"/><c:order val="0"/>"#,
        r#"<c:tx><c:strRef><c:f>Sheet1!$B$1</c:f><c:strCache><c:ptCount val="1"/>"#,
        r#"<c:pt idx="0"><c:v>계열 1</c:v></c:pt></c:strCache></c:strRef></c:tx>"#,
        r#"<c:cat><c:strRef><c:f>Sheet1!$A$2:$A$3</c:f><c:strCache><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v>항목 1</c:v></c:pt><c:pt idx="1"><c:v>항목 2</c:v></c:pt>"#,
        r#"</c:strCache></c:strRef></c:cat>"#,
        r#"<c:val><c:numRef><c:f>Sheet1!$B$2:$B$3</c:f><c:numCache>"#,
        r#"<c:formatCode>General</c:formatCode><c:ptCount val="2"/>"#,
        r#"<c:pt idx="0"><c:v>4.3</c:v></c:pt><c:pt idx="1"><c:v>2.5</c:v></c:pt>"#,
        r#"</c:numCache></c:numRef></c:val></c:ser>"#,
        r#"</c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
    );

    #[test]
    fn scans_name_labels_and_values() {
        let data = scan_chart_values(CATEGORY_CHART.as_bytes()).expect("스캔");
        assert_eq!(data.series.len(), 1);
        let s = &data.series[0];
        assert_eq!(s.name.as_deref(), Some("계열 1"));
        assert_eq!(s.axis, SeriesAxis::Category);
        assert_eq!(
            s.labels.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            ["항목 1", "항목 2"]
        );
        assert_eq!(
            s.values.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            ["4.3", "2.5"]
        );
        assert_eq!(s.values[1].idx, 1);
    }

    #[test]
    fn spans_slice_back_to_their_own_text() {
        let xml = CATEGORY_CHART.as_bytes();
        let data = scan_chart_values(xml).expect("스캔");
        for p in data.series[0].values.iter().chain(&data.series[0].labels) {
            let span = p.span.clone().expect("코퍼스 모양엔 빈 값이 없다");
            assert_eq!(&xml[span], p.text.as_bytes());
        }
    }

    #[test]
    fn reference_formulas_are_not_mistaken_for_values() {
        let data = scan_chart_values(CATEGORY_CHART.as_bytes()).expect("스캔");
        let texts: Vec<&str> = data.series[0]
            .values
            .iter()
            .chain(&data.series[0].labels)
            .map(|p| p.text.as_str())
            .collect();
        assert!(
            !texts.iter().any(|t| t.contains("Sheet1!")),
            "c:f 가 값으로 잡혔다: {texts:?}"
        );
        assert!(!texts.contains(&"General"), "formatCode 가 잡혔다");
    }

    #[test]
    fn scatter_series_use_x_and_y() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:scatterChart><c:ser>"#,
            r#"<c:xVal><c:numRef><c:numCache><c:ptCount val="2"/>"#,
            r#"<c:pt idx="0"><c:v>0.7</c:v></c:pt><c:pt idx="1"><c:v>1.8</c:v></c:pt>"#,
            r#"</c:numCache></c:numRef></c:xVal>"#,
            r#"<c:yVal><c:numRef><c:numCache><c:ptCount val="2"/>"#,
            r#"<c:pt idx="0"><c:v>2.7</c:v></c:pt><c:pt idx="1"><c:v>3.2</c:v></c:pt>"#,
            r#"</c:numCache></c:numRef></c:yVal>"#,
            r#"</c:ser></c:scatterChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("스캔");
        let s = &data.series[0];
        assert_eq!(s.axis, SeriesAxis::Scatter);
        assert_eq!(
            s.labels.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            ["0.7", "1.8"]
        );
        assert_eq!(
            s.values.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            ["2.7", "3.2"]
        );
    }

    #[test]
    fn literal_values_take_the_same_path_as_cached_ones() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
            r#"<c:cat><c:strLit><c:ptCount val="1"/>"#,
            r#"<c:pt idx="0"><c:v>항목 1</c:v></c:pt></c:strLit></c:cat>"#,
            r#"<c:val><c:numLit><c:ptCount val="1"/>"#,
            r#"<c:pt idx="0"><c:v>7</c:v></c:pt></c:numLit></c:val>"#,
            r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("스캔");
        assert_eq!(data.series[0].values[0].text, "7");
        assert_eq!(data.series[0].labels[0].text, "항목 1");
    }

    #[test]
    fn prefix_is_not_hardcoded() {
        let xml = CATEGORY_CHART.replace("c:", "chart:");
        let data = scan_chart_values(xml.as_bytes()).expect("접두어가 달라도 스캔된다");
        assert_eq!(data.series[0].values.len(), 2);
    }

    /// 결측치 `<c:v/>` 는 **읽히고**, 구간이 없어 편집만 막힌다.
    ///
    /// 처음에는 스캔 자체를 거부했는데 범위가 틀렸다 — 값 하나가 비었다고 문서 전체
    /// 읽기를 막으면 결측이 흔한 실데이터에서 `chart-to-csv` 가 통째로 실패한다.
    #[test]
    fn empty_value_element_is_readable_with_no_span() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
            r#"<c:val><c:numLit><c:ptCount val="2"/>"#,
            r#"<c:pt idx="0"><c:v/></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt>"#,
            r#"</c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("읽을 수 있어야 한다");
        let values = &data.series[0].values;
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].text, "");
        assert_eq!(values[0].span, None, "빈 요소는 구간이 없다");
        assert_eq!(values[1].text, "2");
        assert!(values[1].span.is_some());
    }

    /// **확장 서브트리는 값으로 읽지 않는다.**
    ///
    /// `c:extLst` 는 표준 확장 지점이라 `c15:filteredCategoryTitle` 처럼 `c:cat`·`c:pt`·
    /// `c:v` 를 통째로 품는 확장이 들어올 수 있다. 접두어를 고정하지 않는 스캐너는
    /// 네임스페이스로 걸러낼 수 없어 서브트리째 건너뛴다.
    ///
    /// 코퍼스는 `c:ser` 안 `extLst` 가 0건이라 이 경로를 한 번도 밟지 않는다 — 그래서
    /// 합성으로만 덮인다. 모델 파서(`ooxml_chart::parser`)도 `local_name` 만 보므로
    /// **오라클로 세워도 같이 틀린다**는 것이 이 테스트를 따로 두는 이유다.
    #[test]
    fn extension_subtrees_are_not_scanned_as_values() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
            r#"<c:val><c:numRef><c:numCache><c:ptCount val="1"/>"#,
            r#"<c:pt idx="0"><c:v>4.3</c:v></c:pt></c:numCache></c:numRef></c:val>"#,
            r#"<c:extLst><c:ext uri="{02D57815}"><c15:filteredCategoryTitle>"#,
            r#"<c:cat><c:strLit><c:pt idx="0"><c:v>숨은 항목</c:v></c:pt></c:strLit></c:cat>"#,
            r#"<c:val><c:numLit><c:pt idx="0"><c:v>99999</c:v></c:pt></c:numLit></c:val>"#,
            r#"</c15:filteredCategoryTitle></c:ext></c:extLst>"#,
            r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("스캔");
        assert_eq!(data.series.len(), 1);
        let texts: Vec<&str> = data.series[0]
            .values
            .iter()
            .chain(&data.series[0].labels)
            .map(|p| p.text.as_str())
            .collect();
        assert_eq!(texts, ["4.3"], "확장 안의 값이 새어 들어왔다: {texts:?}");
    }

    /// 데이터 라벨의 `c:tx` 가 계열명으로 새지 않는다.
    ///
    /// `c:tx` 는 참조 없이 쓰이는 산출기가 있어 문맥 가드가 느슨한데, `c:dLbl` 안에도
    /// `c:tx` 가 온다. 계열명이 없는 차트에서만 드러나는 오염이라 합성으로 고정한다.
    #[test]
    fn data_label_text_does_not_become_the_series_name() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
            r#"<c:dLbls><c:dLbl><c:idx val="0"/><c:tx><c:v>라벨 텍스트</c:v></c:tx></c:dLbl></c:dLbls>"#,
            r#"<c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>"#,
            r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("스캔");
        assert_eq!(data.series[0].name, None, "라벨이 계열명이 됐다");
        assert_eq!(data.series[0].values.len(), 1);
        assert_eq!(data.series[0].values[0].text, "1");
    }

    #[test]
    fn chart_without_series_is_refused() {
        let xml = r#"<c:chartSpace><c:chart><c:plotArea></c:plotArea></c:chart></c:chartSpace>"#;
        assert_eq!(
            scan_chart_values(xml.as_bytes()),
            Err(ChartScanError::NoSeries)
        );
    }

    #[test]
    fn non_utf8_input_is_refused() {
        assert_eq!(
            scan_chart_values(&[0xff, 0xfe, 0x00]),
            Err(ChartScanError::NotUtf8)
        );
    }

    // -----------------------------------------------------------------------
    // [#4603 리뷰] 비순차 `c:pt idx` fail-closed — 코퍼스는 전건 순차(실측 0건)라
    // 이 경계는 합성으로만 덮인다.
    // -----------------------------------------------------------------------

    /// 라벨·값의 idx 를 바꿔 끼운 카테고리 차트 픽스처.
    fn chart_with_indices(label_idx: &[u32], value_idx: &[(u32, &str)]) -> String {
        let labels: String = label_idx
            .iter()
            .enumerate()
            .map(|(i, idx)| format!(r#"<c:pt idx="{idx}"><c:v>항목 {i}</c:v></c:pt>"#))
            .collect();
        let values: String = value_idx
            .iter()
            .map(|(idx, v)| format!(r#"<c:pt idx="{idx}"><c:v>{v}</c:v></c:pt>"#))
            .collect();
        format!(
            concat!(
                r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
                r#"<c:cat><c:strRef><c:strCache><c:ptCount val="{n}"/>{labels}"#,
                r#"</c:strCache></c:strRef></c:cat>"#,
                r#"<c:val><c:numRef><c:numCache><c:ptCount val="{n}"/>{values}"#,
                r#"</c:numCache></c:numRef></c:val>"#,
                r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
            ),
            n = label_idx.len(),
            labels = labels,
            values = values,
        )
    }

    /// 희소 — 메인테이너 합성 그대로: 라벨 idx 0,1,2 + 값 idx 0(10)·2(30).
    ///
    /// 현재 위치 기반 CSV 는 이것을 A=10, B=30, C=빈칸으로 **오정렬**한다 —
    /// 기대(A=10, B=빈칸, C=30)는 `idx`/`ptCount` 논리 행 모델이 필요해 후속 PR
    /// 몫이고, B1 은 읽기도 쓰기도 거부한다.
    #[test]
    fn sparse_point_index_is_refused() {
        let xml = chart_with_indices(&[0, 1, 2], &[(0, "10"), (2, "30")]);
        assert_eq!(
            scan_chart_values(xml.as_bytes()),
            Err(ChartScanError::NonSequentialPointIndex {
                series: 0,
                block: "val",
                position: 1,
                idx: 2,
            })
        );
    }

    #[test]
    fn reversed_point_index_is_refused() {
        let xml = chart_with_indices(&[0, 1], &[(1, "10"), (0, "20")]);
        assert_eq!(
            scan_chart_values(xml.as_bytes()),
            Err(ChartScanError::NonSequentialPointIndex {
                series: 0,
                block: "val",
                position: 0,
                idx: 1,
            })
        );
    }

    #[test]
    fn duplicate_point_index_is_refused() {
        let xml = chart_with_indices(&[0, 1], &[(0, "10"), (0, "20")]);
        assert_eq!(
            scan_chart_values(xml.as_bytes()),
            Err(ChartScanError::NonSequentialPointIndex {
                series: 0,
                block: "val",
                position: 1,
                idx: 0,
            })
        );
    }

    /// 값은 순차인데 **라벨만** 비순차 — block 필드가 어느 쪽인지 드러낸다.
    #[test]
    fn non_sequential_label_index_is_refused() {
        let xml = chart_with_indices(&[0, 2], &[(0, "10"), (1, "20")]);
        assert_eq!(
            scan_chart_values(xml.as_bytes()),
            Err(ChartScanError::NonSequentialPointIndex {
                series: 0,
                block: "cat",
                position: 1,
                idx: 2,
            })
        );
    }

    /// 분산형은 같은 벡터가 `c:xVal` 에서 오므로 block 이름도 그렇게 나간다.
    #[test]
    fn scatter_x_block_is_named_xval_in_the_refusal() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:scatterChart><c:ser>"#,
            r#"<c:xVal><c:numRef><c:numCache><c:ptCount val="2"/>"#,
            r#"<c:pt idx="0"><c:v>0.7</c:v></c:pt><c:pt idx="2"><c:v>1.8</c:v></c:pt>"#,
            r#"</c:numCache></c:numRef></c:xVal>"#,
            r#"<c:yVal><c:numRef><c:numCache><c:ptCount val="2"/>"#,
            r#"<c:pt idx="0"><c:v>2.7</c:v></c:pt><c:pt idx="1"><c:v>3.2</c:v></c:pt>"#,
            r#"</c:numCache></c:numRef></c:yVal>"#,
            r#"</c:ser></c:scatterChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        assert_eq!(
            scan_chart_values(xml.as_bytes()),
            Err(ChartScanError::NonSequentialPointIndex {
                series: 0,
                block: "xVal",
                position: 1,
                idx: 2,
            })
        );
    }

    /// 다층 카테고리(`c:multiLvlStrRef`) 문서는 여전히 읽힌다 — 면제의 회귀 가드.
    ///
    /// 층 라벨은 idx 가 층마다 0..n-1 로 반복되는 것이 정상 형상이라 순차 검사
    /// 대상이 아니다. 현재 스캐너는 `multiLvlStrCache` 를 캐시 목록에 넣지 않아
    /// 층 라벨을 싣지도 않는다(빈 목록) — 표지(`labels_multi_level`)만 세운다.
    /// 값 쪽은 다층 여부와 무관하게 검사된다.
    #[test]
    fn multi_level_labels_keep_the_document_readable() {
        let xml = concat!(
            r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
            r#"<c:cat><c:multiLvlStrRef><c:multiLvlStrCache><c:ptCount val="2"/>"#,
            r#"<c:lvl><c:pt idx="0"><c:v>상반기</c:v></c:pt><c:pt idx="1"><c:v>하반기</c:v></c:pt></c:lvl>"#,
            r#"<c:lvl><c:pt idx="0"><c:v>1월</c:v></c:pt><c:pt idx="1"><c:v>7월</c:v></c:pt></c:lvl>"#,
            r#"</c:multiLvlStrCache></c:multiLvlStrRef></c:cat>"#,
            r#"<c:val><c:numRef><c:numCache><c:ptCount val="2"/>"#,
            r#"<c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt>"#,
            r#"</c:numCache></c:numRef></c:val>"#,
            r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        );
        let data = scan_chart_values(xml.as_bytes()).expect("다층 라벨 문서는 읽혀야 한다");
        assert!(data.series[0].labels_multi_level);
        assert!(data.series[0].labels.is_empty(), "층 라벨은 실리지 않는다");
        assert_eq!(data.series[0].values.len(), 2, "값은 그대로 읽힌다");
    }
}
