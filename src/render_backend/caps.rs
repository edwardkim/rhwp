//! 백엔드 능력 선언 — 소비자가 백엔드 종류로 `match` 하는 대신 **질의**하게 만든다.
//!
//! 지금 코드베이스에서 "이 백엔드가 벡터 텍스트를 낼 수 있나"를 묻는 방법은
//! 백엔드 타입을 아는 것뿐이다(예: `VariantSelectionBackend` 열거로 분기).
//! 새 백엔드를 붙일 때마다 그 분기를 전부 찾아 고쳐야 한다는 뜻이다.
//! `BackendCapabilities` 는 그 지식을 백엔드 자신에게 되돌려준다.

/// 백엔드가 무엇을 할 수 있는지 스스로 밝힌다 — 소비자가 분기 대신 질의한다.
///
/// 모든 필드는 **그 백엔드의 최종 산출물이 그 성질을 보존하는가**를 뜻한다.
/// 중간 단계에서 무엇을 하는지가 아니다. 예를 들어 PDF 는 내부적으로 SVG 를
/// 거치더라도 최종 PDF 가 벡터 텍스트를 담으면 `vector_text: true` 다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCapabilities {
    /// 백엔드 이름. 진단 메시지와 회귀 기준선 파일명에 쓰는 안정 식별자다.
    pub name: &'static str,
    /// 텍스트를 글리프/문자로 보존하는가(= 산출물에서 텍스트를 다시 선택·검색할 수 있는가).
    pub vector_text: bool,
    /// 폰트 바이트를 산출물 안에 실을 수 있는가.
    pub embedded_fonts: bool,
    /// 그라디언트 채우기를 표현할 수 있는가.
    pub gradients: bool,
    /// 클립 영역을 표현할 수 있는가.
    pub clipping: bool,
    /// 래스터 이미지를 실을 수 있는가.
    pub images: bool,
    /// 산출물이 픽셀뿐인가(벡터 정보를 잃는가).
    ///
    /// `raster_only == true` 이면 `vector_text` 는 반드시 `false` 다.
    /// [`BackendCapabilities::is_consistent`] 가 이 불변식을 판정한다.
    pub raster_only: bool,
    /// 한 번의 `finish` 로 여러 페이지를 담을 수 있는가.
    ///
    /// `false` 면 소비자는 페이지마다 백엔드를 새로 만들어야 한다.
    pub multi_page: bool,
    /// 같은 op 시퀀스를 넣으면 같은 산출물이 나오는가(바이트 동일).
    ///
    /// 백엔드 간 정합 시험은 이 값이 `true` 인 백엔드에서만 기준선을 뜰 수 있다.
    pub deterministic: bool,
}

impl BackendCapabilities {
    /// 아무것도 못 하는 기준선. 계측용 백엔드가 여기서 출발한다.
    pub const fn none(name: &'static str) -> Self {
        Self {
            name,
            vector_text: false,
            embedded_fonts: false,
            gradients: false,
            clipping: false,
            images: false,
            raster_only: false,
            multi_page: false,
            deterministic: false,
        }
    }

    /// 벡터 출력 백엔드(SVG·PDF)가 흔히 갖는 조합.
    pub const fn vector(name: &'static str) -> Self {
        Self {
            name,
            vector_text: true,
            embedded_fonts: false,
            gradients: true,
            clipping: true,
            images: true,
            raster_only: false,
            multi_page: true,
            deterministic: true,
        }
    }

    /// 래스터 출력 백엔드(Skia·Canvas)가 흔히 갖는 조합.
    ///
    /// `raster_only` 가 켜지므로 `vector_text` 는 자동으로 꺼진다.
    pub const fn raster(name: &'static str) -> Self {
        Self {
            name,
            vector_text: false,
            embedded_fonts: false,
            gradients: true,
            clipping: true,
            images: true,
            raster_only: true,
            multi_page: false,
            deterministic: false,
        }
    }

    /// 한 가지 능력을 질의한다.
    ///
    /// 소비자 코드가 필드를 직접 읽는 대신 이 메서드를 쓰면, 나중에 능력이
    /// 늘어나도 호출 형태가 바뀌지 않는다.
    pub fn supports(&self, feature: BackendFeature) -> bool {
        match feature {
            BackendFeature::VectorText => self.vector_text,
            BackendFeature::EmbeddedFonts => self.embedded_fonts,
            BackendFeature::Gradients => self.gradients,
            BackendFeature::Clipping => self.clipping,
            BackendFeature::Images => self.images,
            BackendFeature::MultiPage => self.multi_page,
            BackendFeature::Deterministic => self.deterministic,
        }
    }

    /// 선언이 자기모순이 아닌가 — 래스터 전용인데 벡터 텍스트를 주장하지는 않는가.
    pub fn is_consistent(&self) -> bool {
        !(self.raster_only && self.vector_text)
    }

    /// `self` 가 `required` 의 모든 능력을 포함하는가.
    ///
    /// 백엔드를 고를 때 "이 문서를 손실 없이 내보내려면 무엇이 필요한가"를
    /// 능력 집합으로 적어두고 후보 백엔드에 물어보는 데 쓴다.
    pub fn covers(&self, required: &[BackendFeature]) -> bool {
        required.iter().all(|feature| self.supports(*feature))
    }
}

/// 질의 가능한 능력 한 가지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendFeature {
    /// 텍스트를 텍스트로 보존.
    VectorText,
    /// 폰트 바이트 내장.
    EmbeddedFonts,
    /// 그라디언트.
    Gradients,
    /// 클립 영역.
    Clipping,
    /// 래스터 이미지.
    Images,
    /// 여러 페이지를 한 산출물에.
    MultiPage,
    /// 결정론 출력.
    Deterministic,
}

impl BackendFeature {
    /// 진단·기준선 파일에 쓰는 안정 문자열.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VectorText => "vectorText",
            Self::EmbeddedFonts => "embeddedFonts",
            Self::Gradients => "gradients",
            Self::Clipping => "clipping",
            Self::Images => "images",
            Self::MultiPage => "multiPage",
            Self::Deterministic => "deterministic",
        }
    }
}
