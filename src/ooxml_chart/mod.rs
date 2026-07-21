//! OOXML 차트 (DrawingML) 파싱 및 SVG 렌더링
//!
//! HWP 파일 내 OLE 개체의 `OOXMLChartContents` 스트림 또는 HWPX `Chart/chartN.xml`은
//! Microsoft OOXML DrawingML 차트 XML로 저장된다. 이 모듈은 해당 XML을 파싱하여
//! 데이터 모델로 변환한 뒤, 네이티브 SVG 차트로 렌더링한다.
//!
//! ## 지원 범위
//! - `c:barChart` (세로/가로 막대)
//! - `c:lineChart` (꺾은선) — 누적/백프로 누적(`c:grouping`) + 표식(plot 레벨
//!   `c:marker`) 포함 (C1d #2129)
//! - `c:pieChart` (원형)
//! - `c:bar3DChart` — **시어 투영 3D 렌더** (C2b #2278 v2): `c:view3D` 카메라
//!   파싱 + 압출 3면(top/side/front) + 방(뒷벽·바닥·격자). 두께는 gapWidth 규칙.
//! - `c:pie3DChart` — **타원+측벽 3D 렌더** (C2b #2278 Stage 2): 타원비
//!   `sin(rotX)·cos(perspective/2°)`(정답지 실측 캘리브레이션) + 하반부 측벽.
//! - `c:ofPieChart` — **보조플롯 렌더** (C2b #2278 Stage 3): 주 원(결합 슬라이스
//!   포함) + 보조 원(pie)/누적 막대(bar) + `c:serLines` 연결선.
//! - `c:scatterChart` (분산형) — `c:xVal`/`c:yVal` (x,y) 쌍, 2개 수치축,
//!   `c:scatterStyle`로 표식/직선/곡선 구분 (C1b #1660).
//! - `c:stockChart` (주식형) — `c:hiLowLines` 고저선 + `c:upDownBars` 캔들,
//!   계열 역할은 XML 순서 규약(3계열=고/저/종, 4계열=시/고/저/종) (C2a #2277).
//! - **콤보 차트** (barChart + lineChart 혼합) — 시리즈별 타입 보존
//! - **이중 Y축** (primary + secondary) — 시리즈별 축 그룹 매핑
//!
//! ## 범위 외
//! - 영역형, 추세선, 애니메이션, 세밀 스타일

pub mod parser;
pub mod renderer;

/// OOXML 차트 데이터 모델
#[derive(Debug, Clone, Default)]
pub struct OoxmlChart {
    /// 주 차트 타입 (콤보인 경우 첫 번째 plotType이 들어감; 렌더러는 시리즈별 타입 우선)
    pub chart_type: OoxmlChartType,
    /// 명시 제목 텍스트 (`c:title > … > a:t`). 자동 제목 판단은 아래 플래그로 별도
    /// 수행 — 이 필드는 명시 텍스트 전용을 유지해 파서의 빈 차트 조기 반환 가드
    /// (`series.is_empty() && title.is_none()`)에 영향을 주지 않는다. (C1c #1882 갭①)
    pub title: Option<String>,
    /// `c:title` 요소 존재 여부. 한컴은 제목 텍스트가 없어도 이 요소가 있고
    /// `autoTitleDeleted=0`이면 자동 제목 "차트 제목"을 렌더한다. (C1c #1882 갭①)
    pub has_title_elem: bool,
    /// `c:autoTitleDeleted val="1"` — 자동 제목 억제 플래그. (C1c #1882 갭①)
    pub auto_title_deleted: bool,
    pub series: Vec<OoxmlSeries>,
    pub categories: Vec<String>,
    /// 시리즈 중 하나라도 보조축을 쓰면 true
    pub has_secondary_axis: bool,
    /// 막대(bar/bar3D) plot의 `c:grouping` (clustered/stacked/percentStacked).
    /// 막대 렌더러만 사용. line/pie 무관. (C1a #1453 막대 누적 보정)
    pub grouping: BarGrouping,
    /// 라인(lineChart) plot의 `c:grouping` (standard/stacked/percentStacked).
    /// 순수 라인 렌더러(render_line) 전용 — 콤보의 line 시리즈에는 미적용(코퍼스 무해당).
    /// 막대 grouping과 별도 필드인 이유: 콤보(bar+line 공존)에서 단일 필드 공유 시
    /// XML 문서 순서에 따라 상호 오염. (C1d #2129)
    pub line_grouping: BarGrouping,
    /// 라인 plot 레벨 `<c:marker val="1"/>` — 표식(마커) 표시 여부. 계열 내부
    /// `<c:marker>`(val 없음, symbol/size 래퍼)와 구분됨. (C1d #2129)
    pub line_markers: bool,
    /// 분산형 `c:scatterStyle` (표식/직선/곡선). scatter 렌더러만 사용. (C1b #1660)
    pub scatter_style: ScatterStyle,
    /// 범례 위치 (`c:legendPos`). 한컴 코퍼스는 전 샘플 `val="r"`. (C1c #1882 갭③)
    pub legend_pos: LegendPos,
    /// 3D plot(`bar3DChart`/`pie3DChart`/`line3DChart`) 여부. 축 정책(묶은 0~5
    /// 무헤드룸/누적세로 +1 step — C1c #1882) + 막대 투영 렌더 분기(C2b #2278 v2)에
    /// 사용. 카메라 파라미터는 `view3d` 필드.
    pub is_3d: bool,
    /// stock plot의 `<c:hiLowLines/>` 존재 — 고저선. HLC/OHLC 공통. (C2a #2277)
    pub has_hi_low_lines: bool,
    /// stock plot의 `<c:upDownBars>` 존재 — 시가↔종가 캔들. OHLC만. (C2a #2277)
    pub has_up_down_bars: bool,
    /// `<c:upDownBars><c:gapWidth val>` — 캔들 폭 = cat_span/(1+gap/100).
    /// 미지정 시 렌더러가 150(정답지 실측) 폴백. (C2a #2277)
    pub up_down_gap_width: Option<f64>,
    /// `c:view3D` 3D 카메라. 3D plot(`bar3DChart`/`pie3DChart`/`line3DChart`)에서
    /// Some — view3D 요소가 plotArea보다 앞에 오므로 view3D arm에서 초기화하고
    /// plot arm은 get_or_insert 폴백만 수행. (C2b #2278 v2)
    pub view3d: Option<View3D>,
    /// bar/bar3D plot의 `c:gapWidth` — 3D 두께 규칙 slot/(n_eff+gap/100)용.
    /// stock의 `up_down_gap_width`(upDownBars 내부 동명 요소)와 별도 필드로
    /// 상호 오염 방지. 미지정 시 렌더러가 150 폴백. (C2b #2278 v2)
    pub bar_gap_width: Option<f64>,
    /// bar3D plot의 `c:gapDepth` — 방(씬) 깊이 = 막대 깊이×(1+gap/100).
    /// 미지정 시 렌더러가 150 폴백. (C2b #2278 v2)
    pub gap_depth: Option<f64>,
    /// ofPie(원형대원형/원형대가로막대형) 보조플롯 정보. chart_type은 Pie를 유지하고
    /// (#1453 라우팅 앵커) 이 필드 유무로 render_of_pie를 분기. (C2b #2278)
    pub of_pie: Option<OfPieInfo>,
}

/// `c:ofPieChart` 보조플롯 파라미터 (C2b #2278)
#[derive(Debug, Clone, PartialEq)]
pub struct OfPieInfo {
    /// `c:ofPieType val` — pie=원형대원형(보조 원), bar=원형대가로막대형(누적 막대)
    pub of_pie_type: OfPieType,
    /// `c:splitPos val` — 보조 플롯으로 보낼 마지막 카테고리 수. 코퍼스 부재 →
    /// None → 기본 2. (스키마상 double — f64 파싱, 사용 시 반올림·클램프)
    pub split_pos: Option<f64>,
    /// `c:secondPieSize val` (% — 스키마 기본 75) — 보조 플롯 크기 / 주 원 대비
    pub second_pie_size: f64,
    /// `c:serLines` 존재 — 결합 슬라이스→보조 플롯 연결선 2줄
    pub has_ser_lines: bool,
}

impl Default for OfPieInfo {
    fn default() -> Self {
        Self {
            of_pie_type: OfPieType::Pie,
            split_pos: None,
            second_pie_size: 75.0,
            has_ser_lines: false,
        }
    }
}

/// ofPie 보조 플롯 종류 (C2b #2278)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OfPieType {
    #[default]
    Pie,
    Bar,
}

/// `c:view3D` 3D 카메라 파라미터 (C2b #2278 v2).
///
/// 투영 알고리즘 자체는 ECMA-376 비정의(구현 재량) — 본 렌더러는 rAngAx=1을
/// 시어(2.5차원: 앞면 직립, 깊이만 사선 — 한컴 차트 스펙 rev1.2 표 100
/// ProjectionType=1과 동계), rAngAx=0을 회전+원근으로 해석한다.
/// 기본값 15/20/30/true는 XSD 기본이 아니라 Office(MS-OE376) 관례 — 코퍼스
/// 막대 4종 실측값과 일치.
#[derive(Debug, Clone, PartialEq)]
pub struct View3D {
    /// `c:rotX` — 상승(elevation) 각도, 도 단위 (-90..90). 기본 15
    pub rot_x: f64,
    /// `c:rotY` — 회전(rotation) 각도, 도 단위 (0..360). 기본 20
    pub rot_y: f64,
    /// `c:perspective` (0..240). 기본 30 — rAngAx=1(시어)에서는 미적용
    pub perspective: f64,
    /// `c:rAngAx` — 직각 축(right-angle axes). 기본 true
    pub r_ang_ax: bool,
    /// `c:hPercent` — 높이 비율. 기본 100 (현 단계 기록만, 미적용)
    pub h_percent: f64,
    /// `c:depthPercent` — 막대 깊이/폭 비율(%). 기본 100.
    /// 주의: ECMA 정의("차트 폭 대비 %")와 다른 **의도적 편차** — 막대 스케일
    /// 비례 유지 목적의 캘리브레이션 결정. (C2b #2278 v2)
    pub depth_percent: f64,
}

impl Default for View3D {
    fn default() -> Self {
        Self {
            rot_x: 15.0,
            rot_y: 20.0,
            perspective: 30.0,
            r_ang_ax: true,
            h_percent: 100.0,
            depth_percent: 100.0,
        }
    }
}

/// 계열 내부 `<c:marker>` 상태 — stock 종가 마커 판별용. plot 레벨
/// `<c:marker val>`(`line_markers`)과 별개. (C2a #2277)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SeriesMarker {
    /// `<c:marker>` 요소 없음
    #[default]
    NotSpecified,
    /// `<c:symbol val="none"/>` — 표식 억제 (stock 시/고/저 실측)
    None,
    /// `<c:marker>` 래퍼 존재·symbol 부재 — 자동 표식 (stock 종가 실측)
    Auto,
    /// 명시 심볼 (diamond/square/triangle/x 등 — 코퍼스 밖, 사이클 폴백과 병행)
    Named(String),
}

/// 범례 위치 (`c:legendPos`). C1c #1882 갭③.
///
/// 기본값 Bottom — `c:legend`/`legendPos` 미존재 시 현행 하단 배치를 유지한다
/// (모델을 직접 구성하는 기존 테스트·XML 보호). Right만 우측 세로 스택으로
/// 렌더하며 Left/Top은 하단 폴백(코퍼스 전 샘플이 r — 확장은 후속).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendPos {
    #[default]
    Bottom,
    Right,
    Left,
    Top,
}

/// 막대/라인 공용 그룹화 방식 (`c:grouping`). 막대는 `clustered`/`standard`,
/// 라인은 `standard`를 Clustered로 흡수. 라인 누적은 `line_grouping`에 저장. (C1d #2129)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarGrouping {
    /// 묶은(side-by-side). `clustered`/`standard` 흡수.
    #[default]
    Clustered,
    /// 누적 (시리즈를 카테고리별로 쌓음).
    Stacked,
    /// 백분율 누적 (카테고리 합을 100%로 정규화).
    PercentStacked,
}

/// 분산형 표현 방식 (`c:scatterStyle`). C1b #1660.
///
/// 한컴 분산형 5종은 이 값만으로 렌더가 결정된다(곡선 2종은 동일 `smoothMarker`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScatterStyle {
    /// 표식만 (점만, 선 없음).
    #[default]
    Marker,
    /// 직선 (선만, 표식 없음).
    Line,
    /// 직선 + 표식.
    LineMarker,
    /// 부드러운 곡선 + 표식.
    SmoothMarker,
}

impl ScatterStyle {
    /// `(선 표시, 곡선 여부, 표식 표시)`.
    pub fn flags(&self) -> (bool, bool, bool) {
        match self {
            Self::Marker => (false, false, true),
            Self::Line => (true, false, false),
            Self::LineMarker => (true, false, true),
            Self::SmoothMarker => (true, true, true),
        }
    }
}

/// 차트 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OoxmlChartType {
    /// 세로 막대 (barDir=col)
    Column,
    /// 가로 막대 (barDir=bar)
    Bar,
    /// 꺾은선
    Line,
    /// 원형
    Pie,
    /// 분산형 (x,y 산점도) (C1b #1660)
    Scatter,
    /// 주식형 (hiLowLines 고저선 / upDownBars 캔들) (C2a #2277)
    Stock,
    #[default]
    Unknown,
}

impl OoxmlChartType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Column => "세로 막대",
            Self::Bar => "가로 막대",
            Self::Line => "꺾은선",
            Self::Pie => "원형",
            Self::Scatter => "분산형",
            Self::Stock => "주식형",
            Self::Unknown => "미지원",
        }
    }
}

/// 데이터 시리즈 (막대 한 묶음 또는 선 하나)
#[derive(Debug, Clone, Default)]
pub struct OoxmlSeries {
    pub name: String,
    /// Y 값 (막대/선/원형). 분산형에서는 `c:yVal`.
    pub values: Vec<f64>,
    /// 분산형 X 값 (`c:xVal`). 분산형 전용이며 그 외 차트에서는 빈 Vec. (C1b #1660)
    pub x_values: Vec<f64>,
    /// RGB 색상 (`0xRRGGBB`), 파서가 확정 못하면 None (렌더러가 기본 팔레트 적용)
    pub color: Option<u32>,
    /// 시리즈 본인의 차트 타입 (콤보 차트에서 바/라인 구분용)
    pub series_type: OoxmlChartType,
    /// 이 시리즈가 속한 플롯의 c:axId 값 목록 (parser 내부에서 axis 분류에 사용)
    pub axis_ids: Vec<String>,
    /// 0 = 기본축(왼쪽/아래), 1 = 보조축(오른쪽/위)
    pub axis_group: u8,
    /// 숫자 포맷 코드 (예: "#,##0")
    pub format_code: Option<String>,
    /// 계열 내부 `<c:marker>` 상태 — stock 종가 마커 판별용. (C2a #2277)
    pub marker_symbol: SeriesMarker,
    /// 계열 레벨 `<c:explosion val>` (%) — 쪼개진원형: 전 슬라이스를 중심각
    /// 방향으로 반지름×값/100 만큼 이동(코퍼스 25). dPt 단위 explosion은
    /// 코퍼스 부재로 범위 외. (C2b #2278 Stage 3 v3)
    pub explosion: Option<f64>,
}

impl OoxmlChart {
    /// 파싱 입력: OOXMLChartContents 원본 바이트 (UTF-8 XML)
    pub fn parse(xml: &[u8]) -> Option<Self> {
        parser::parse_chart_xml(xml)
    }

    /// 주어진 영역에 SVG 조각으로 렌더링한다.
    /// 반환값은 `<g>...</g>` 또는 여러 요소로 구성된 SVG 문자열 조각.
    pub fn render_svg(&self, x: f64, y: f64, w: f64, h: f64) -> String {
        renderer::render_chart_svg(self, x, y, w, h)
    }

    /// 시리즈가 여러 타입을 섞어 쓰는지 (콤보 차트) 여부
    pub fn is_combo(&self) -> bool {
        let mut types: std::collections::HashSet<OoxmlChartType> = std::collections::HashSet::new();
        for s in &self.series {
            if s.series_type != OoxmlChartType::Unknown {
                types.insert(s.series_type);
            }
        }
        types.len() > 1
    }
}
