//! OOXML 차트 (DrawingML) 파싱 및 SVG 렌더링 (Task #195 단계 8)
//!
//! HWP 파일 내 OLE 개체의 `OOXMLChartContents` 스트림은 Microsoft OOXML DrawingML
//! 차트 XML로 저장된다. 이 모듈은 해당 XML을 파싱하여 데이터 모델로 변환한 뒤,
//! 네이티브 SVG 차트로 렌더링한다.
//!
//! ## 지원 범위 (1차)
//! - `c:barChart` (세로/가로 막대)
//! - `c:lineChart` (꺾은선)
//! - `c:pieChart` (원형)
//!
//! ## 범위 외
//! - 3D 차트, 영역/산점도, 복합 차트, 보조축, 추세선, 애니메이션, 세밀 스타일

pub mod parser;
pub mod renderer;

/// OOXML 차트 데이터 모델
#[derive(Debug, Clone, Default)]
pub struct OoxmlChart {
    pub chart_type: OoxmlChartType,
    pub title: Option<String>,
    pub series: Vec<OoxmlSeries>,
    pub categories: Vec<String>,
}

/// 차트 종류
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum OoxmlChartType {
    /// 세로 막대 (barDir=col)
    Column,
    /// 가로 막대 (barDir=bar)
    Bar,
    /// 꺾은선
    Line,
    /// 원형
    Pie,
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
            Self::Unknown => "미지원",
        }
    }
}

/// 데이터 시리즈 (막대 한 묶음 또는 선 하나)
#[derive(Debug, Clone, Default)]
pub struct OoxmlSeries {
    pub name: String,
    pub values: Vec<f64>,
    /// RGB 색상 (`0xRRGGBB`), 파서가 확정 못하면 None (렌더러가 기본 팔레트 적용)
    pub color: Option<u32>,
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
}
