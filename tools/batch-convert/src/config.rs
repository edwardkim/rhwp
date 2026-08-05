use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// PDF 내보내기 옵션 — 모든 필드가 `rhwp export-pdf` 의 실제 플래그로 그대로
/// 전달된다. rhwp CLI 에 대응 플래그가 없는 옵션은 이 계약에 존재하지 않는다.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PdfOptions {
    /// `--backend <svg|direct>` (생략 시 rhwp 기본값 svg)
    #[serde(default)]
    pub backend: Option<String>,

    /// `--profile <screen|print|high-quality|fast-preview>`
    #[serde(default)]
    pub profile: Option<String>,

    /// `--raster-dpi <DPI>` — direct backend 전용 (svg backend 는 거부)
    #[serde(default)]
    pub raster_dpi: Option<f64>,

    /// `--text-as-paths` — svg backend 전용 (direct backend 는 거부)
    #[serde(default)]
    pub text_as_paths: bool,
}

/// PNG 내보내기 옵션 — `rhwp export-png` 플래그와 1:1 대응.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PngOptions {
    /// `--profile <screen|print|high-quality|fast-preview>` (rhwp 기본: high-quality)
    #[serde(default)]
    pub profile: Option<String>,

    /// `--dpi <값>` — PNG pHYs 메타데이터. `scale` 미지정 시 scale=dpi/96 자동 계산
    #[serde(default)]
    pub dpi: Option<f64>,

    /// `--scale <배율>` — 렌더링 배율 (rhwp 기본: 1.0)
    #[serde(default)]
    pub scale: Option<f64>,

    /// `--max-dimension <픽셀>` — 긴 변 최대 픽셀 (VLM 입력 한도용)
    #[serde(default)]
    pub max_dimension: Option<u32>,
}

/// SVG 내보내기 옵션 — `rhwp export-svg` 플래그와 1:1 대응.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SvgOptions {
    /// `--profile <screen|print|high-quality|fast-preview>`
    /// (rhwp 는 `--profile` 과 `--embed-fonts` 동시 지정을 거부한다)
    #[serde(default)]
    pub profile: Option<String>,

    /// `--embed-fonts` — 사용 글자만 서브셋 임베딩
    #[serde(default)]
    pub embed_fonts: bool,
}

/// 전체 변환 설정. 알 수 없는 필드는 파싱 단계에서 거부한다 — 선언만 되고
/// 동작하지 않는 필드가 조용히 살아남지 못하게 하는 계약 장치다.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversionConfig {
    /// 활성화할 출력 포맷 (하나 이상 true 여야 한다)
    pub formats: FormatsConfig,

    /// PDF 옵션
    #[serde(default)]
    pub pdf: PdfOptions,

    /// PNG 옵션
    #[serde(default)]
    pub png: PngOptions,

    /// SVG 옵션
    #[serde(default)]
    pub svg: SvgOptions,

    /// 동작 옵션
    #[serde(default)]
    pub behavior: BehaviorOptions,
}

/// 포맷 활성화 설정 — 네 필드 모두 명시해야 한다.
/// (텍스트 내보내기는 `rhwp export-text` 에 배치 변환에서 쓸 수 있는 추가
/// 플래그가 없어 별도 옵션 섹션이 없다)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatsConfig {
    pub pdf: bool,
    pub png: bool,
    pub svg: bool,
    pub text: bool,
}

/// 동작 설정 — 전 필드가 변환 경로에 실제로 연결되어 있다.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorOptions {
    /// false 면 이미 존재하는 산출물을 포맷 단위로 건너뛴다 (재작성하지 않음)
    #[serde(default = "default_true")]
    pub overwrite: bool,

    /// true 면 출력 루트에 포맷별 하위 폴더(pdf/·png/·svg/·text/)를 만든다
    #[serde(default = "default_true")]
    pub create_format_dirs: bool,

    /// true 면 변환에 실패한 원본을 `<출력>/failed/` 로 복사해 모은다
    #[serde(default = "default_false")]
    pub collect_failed: bool,

    /// true 면 첫 파일 실패가 확정되는 즉시 아직 시작하지 않은 파일을 건너뛴다
    #[serde(default = "default_false")]
    pub fail_fast: bool,

    /// 포맷별 rhwp 호출이 실패했을 때 추가로 재시도할 횟수 (총 시도 = 1 + N)
    #[serde(default = "default_retries")]
    pub max_retries: u32,

    /// true 면 활성 포맷의 산출물이 전부 존재하는 파일을 통째로 건너뛴다
    #[serde(default = "default_false")]
    pub skip_existing: bool,
}

impl ConversionConfig {
    /// JSON 설정 파일 로드 + 계약 검증
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path.display()))?;
        let config: ConversionConfig =
            serde_json::from_str(&content).context("Failed to parse configuration JSON")?;
        config.validate()?;
        Ok(config)
    }

    /// rhwp CLI 가 파일 단위로 거부할 조합을 배치 시작 전에 걸러낸다 —
    /// 잘못된 설정으로 수백 건을 전부 실패시키는 대신 즉시 명확히 실패한다.
    pub fn validate(&self) -> Result<()> {
        let formats = &self.formats;
        if !(formats.pdf || formats.png || formats.svg || formats.text) {
            bail!("설정 오류: 활성화된 출력 포맷이 없습니다 (formats.pdf/png/svg/text 중 하나 이상은 true 여야 합니다)");
        }

        if let Some(backend) = self.pdf.backend.as_deref() {
            if backend != "svg" && backend != "direct" {
                bail!(
                    "설정 오류: pdf.backend 는 \"svg\" 또는 \"direct\" 여야 합니다 (현재: {:?})",
                    backend
                );
            }
        }
        let direct_backend = self.pdf.backend.as_deref() == Some("direct");
        if self.pdf.raster_dpi.is_some() && !direct_backend {
            bail!("설정 오류: pdf.raster_dpi 는 pdf.backend=\"direct\" 에서만 쓸 수 있습니다 (rhwp export-pdf --raster-dpi 규칙)");
        }
        if self.pdf.text_as_paths && direct_backend {
            bail!("설정 오류: pdf.text_as_paths 는 svg backend 전용입니다 (direct backend 는 --text-as-paths 를 거부)");
        }
        if let Some(dpi) = self.pdf.raster_dpi {
            if !dpi.is_finite() || dpi <= 0.0 {
                bail!(
                    "설정 오류: pdf.raster_dpi 는 양수여야 합니다 (현재: {})",
                    dpi
                );
            }
        }

        if let Some(dpi) = self.png.dpi {
            if !dpi.is_finite() || dpi <= 0.0 {
                bail!("설정 오류: png.dpi 는 양수여야 합니다 (현재: {})", dpi);
            }
        }
        if let Some(scale) = self.png.scale {
            if !scale.is_finite() || scale <= 0.0 {
                bail!("설정 오류: png.scale 은 양수여야 합니다 (현재: {})", scale);
            }
        }
        if self.png.max_dimension == Some(0) {
            bail!("설정 오류: png.max_dimension 은 1 이상이어야 합니다");
        }

        if self.svg.embed_fonts && self.svg.profile.is_some() {
            bail!("설정 오류: svg.embed_fonts 와 svg.profile 은 함께 쓸 수 없습니다 (rhwp export-svg 가 --embed-fonts 와 --profile 동시 지정을 거부)");
        }

        Ok(())
    }
}

impl Default for ConversionConfig {
    fn default() -> Self {
        ConversionConfig {
            formats: FormatsConfig {
                pdf: true,
                png: false,
                svg: false,
                text: true,
            },
            pdf: PdfOptions::default(),
            png: PngOptions::default(),
            svg: SvgOptions::default(),
            behavior: BehaviorOptions::default(),
        }
    }
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_retries() -> u32 {
    3
}

impl Default for BehaviorOptions {
    fn default() -> Self {
        BehaviorOptions {
            overwrite: true,
            create_format_dirs: true,
            collect_failed: false,
            fail_fast: false,
            max_retries: 3,
            skip_existing: false,
        }
    }
}
