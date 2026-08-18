//! PaintOp 종류 카탈로그 — 백엔드 계약이 쓰는 안정 어휘.
//!
//! `paint_op_kind` 문자열이 LayerTree JSON `"type"` 과 같고, 각 종류가
//! 어느 replay plane 에 속하며 어떤 capability 가 있어야 산출물에 남는지를
//! 한 표로 고정한다. 새 어댑터는 이 표를 읽고 자기 광고를 맞춘다.

use crate::paint::{paint_op_replay_plane, PaintOp, PaintReplayPlane};

use super::caps::BackendFeature;
use super::util::paint_op_kind;

/// 카탈로그에 오른 PaintOp 한 종류의 계약 행.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintOpKindSpec {
    /// LayerTree JSON `"type"` 과 같은 안정 이름.
    pub kind: &'static str,
    /// 기본 replay plane. 레이어 `text_wrap` 이 있으면 덮어쓴다.
    pub default_plane: PaintReplayPlane,
    /// 이 op 를 산출물에 남기려면 켜져 있어야 하는 capability.
    /// `None` 이면 계측 백엔드만으로도 생명주기 시험이 된다.
    pub required_feature: Option<BackendFeature>,
    /// 얇은 어댑터가 leaf 로 평탄화해도 의미가 남는가.
    pub survives_flatten: bool,
    /// 결정론 추적 로그에 bbox 한 줄로 남는가.
    pub appears_in_trace: bool,
    /// 한국어 한 줄 설명.
    pub summary_ko: &'static str,
}

impl PaintOpKindSpec {
    /// `kind` 문자열이 이 행과 같은가.
    pub fn matches_kind(self, kind: &str) -> bool {
        self.kind == kind
    }

    /// 기본 plane 의 안정 문자열.
    pub fn plane_name(self) -> &'static str {
        self.default_plane.as_str()
    }

    /// 필요한 capability 의 안정 문자열. 없으면 `"none"`.
    pub fn feature_name(self) -> &'static str {
        self.required_feature
            .map(BackendFeature::as_str)
            .unwrap_or("none")
    }
}

/// 카탈로그 전체. 순서는 `paint_op_kind` match 와 같다.
pub const PAINT_OP_KIND_SPECS: &[PaintOpKindSpec] = &[
    PaintOpKindSpec {
        kind: "pageBackground",
        default_plane: PaintReplayPlane::Background,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "페이지 배경. 재생은 항상 첫 plane 이다.",
    },
    PaintOpKindSpec {
        kind: "textRun",
        default_plane: PaintReplayPlane::Flow,
        required_feature: Some(BackendFeature::VectorText),
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "선택·검색 가능한 텍스트 런.",
    },
    PaintOpKindSpec {
        kind: "glyphRun",
        default_plane: PaintReplayPlane::Flow,
        required_feature: Some(BackendFeature::VectorText),
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "셰이핑된 글리프 런. 폰트 메트릭이 이미 풀린 형태.",
    },
    PaintOpKindSpec {
        kind: "glyphOutline",
        default_plane: PaintReplayPlane::Flow,
        required_feature: Some(BackendFeature::VectorText),
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "글리프 외곽선. 텍스트 등가군이지 일반 Path 가 아니다.",
    },
    PaintOpKindSpec {
        kind: "charOverlap",
        default_plane: PaintReplayPlane::Flow,
        required_feature: Some(BackendFeature::VectorText),
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "글자겹침 명시 visual op.",
    },
    PaintOpKindSpec {
        kind: "textControlMark",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "문단 끝·줄바꿈·필드 마커. 화면 전용일 수 있다.",
    },
    PaintOpKindSpec {
        kind: "tabLeader",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "탭 리더 점선·실선 geometry.",
    },
    PaintOpKindSpec {
        kind: "textDecoration",
        default_plane: PaintReplayPlane::Flow,
        required_feature: Some(BackendFeature::VectorText),
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "밑줄·취소선·강조점.",
    },
    PaintOpKindSpec {
        kind: "footnoteMarker",
        default_plane: PaintReplayPlane::Flow,
        required_feature: Some(BackendFeature::VectorText),
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "각주·미주 위첨자 마커.",
    },
    PaintOpKindSpec {
        kind: "line",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "직선. 계측 백엔드도 센다.",
    },
    PaintOpKindSpec {
        kind: "rectangle",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "사각형. 그라디언트가 있으면 Gradients 가 필요하다.",
    },
    PaintOpKindSpec {
        kind: "ellipse",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "타원. 사각형과 같은 스타일 계약을 따른다.",
    },
    PaintOpKindSpec {
        kind: "path",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "임의 패스. 화살표·연결선을 담을 수 있다.",
    },
    PaintOpKindSpec {
        kind: "image",
        default_plane: PaintReplayPlane::Flow,
        required_feature: Some(BackendFeature::Images),
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "래스터 이미지. BehindText/InFrontOfText wrap 이 plane 을 바꾼다.",
    },
    PaintOpKindSpec {
        kind: "equation",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "수식 SVG 조각. 텍스트 추출은 script 필드를 쓴다.",
    },
    PaintOpKindSpec {
        kind: "formObject",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "양식 컨트롤(단추·체크·입력).",
    },
    PaintOpKindSpec {
        kind: "placeholder",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "차트/OLE/그림-없음 자리표시. 인쇄 등가에서는 숨길 수 있다.",
    },
    PaintOpKindSpec {
        kind: "rawSvg",
        default_plane: PaintReplayPlane::Flow,
        required_feature: None,
        survives_flatten: true,
        appears_in_trace: true,
        summary_ko: "미리 렌더된 SVG 조각(OLE 등).",
    },
];

/// 카탈로그 행 수. `paint_op_kind` match 가지 수와 같아야 한다.
pub const PAINT_OP_KIND_COUNT: usize = 18;

/// `kind` 로 카탈로그 행을 찾는다.
pub fn spec_for_kind(kind: &str) -> Option<&'static PaintOpKindSpec> {
    PAINT_OP_KIND_SPECS
        .iter()
        .find(|spec| spec.matches_kind(kind))
}

/// 카탈로그에 오른 모든 kind 이름. 순서는 안정이다.
pub fn all_kind_names() -> impl Iterator<Item = &'static str> {
    PAINT_OP_KIND_SPECS.iter().map(|spec| spec.kind)
}

/// `PaintOp` 한 개의 카탈로그 행과 실제 plane 을 같이 본다.
pub fn classify_op(op: &PaintOp) -> ClassifiedOp {
    let kind = paint_op_kind(op);
    let spec = spec_for_kind(kind).expect("paint_op_kind 는 카탈로그에 있어야 한다");
    ClassifiedOp {
        kind,
        spec,
        plane: paint_op_replay_plane(op),
        bounds: OpBounds::from_op(op),
    }
}

/// 분류 결과. 추적 로그 한 줄을 만들 때 쓴다.
#[derive(Debug, Clone, Copy)]
pub struct ClassifiedOp {
    /// `paint_op_kind` 값.
    pub kind: &'static str,
    /// 카탈로그 행.
    pub spec: &'static PaintOpKindSpec,
    /// 실제 replay plane (레이어 wrap 반영).
    pub plane: PaintReplayPlane,
    /// bbox 스냅샷.
    pub bounds: OpBounds,
}

/// 추적·픽스처가 공유하는 bbox 스냅샷. 좌표는 px, 소수 2자리로 찍는다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpBounds {
    /// 왼쪽 (px).
    pub x: f64,
    /// 위 (px).
    pub y: f64,
    /// 폭 (px).
    pub width: f64,
    /// 높이 (px).
    pub height: f64,
}

impl OpBounds {
    /// `PaintOp::bounds` 에서 스냅샷을 뜬다.
    pub fn from_op(op: &PaintOp) -> Self {
        let b = op.bounds();
        Self {
            x: b.x,
            y: b.y,
            width: b.width,
            height: b.height,
        }
    }

    /// 추적 로그와 같은 `{:.2},{:.2},{:.2},{:.2}` 형식.
    pub fn trace_csv(self) -> String {
        format!(
            "{:.2},{:.2},{:.2},{:.2}",
            self.x, self.y, self.width, self.height
        )
    }

    /// 유한하고 폭·높이가 음수가 아닌가. 0 크기는 허용한다(선분).
    pub fn is_finite(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width >= 0.0
            && self.height >= 0.0
    }
}

/// 카탈로그가 `paint_op_kind` 와 1:1 인지 검사한다.
pub fn catalog_covers_kind(kind: &str) -> bool {
    spec_for_kind(kind).is_some()
}

/// 카탈로그 불변식 — 행 수, 중복 없음, plane 이름 안정.
pub fn catalog_invariants_hold() -> Result<(), String> {
    if PAINT_OP_KIND_SPECS.len() != PAINT_OP_KIND_COUNT {
        return Err(format!(
            "카탈로그 행 수 {} != PAINT_OP_KIND_COUNT {PAINT_OP_KIND_COUNT}",
            PAINT_OP_KIND_SPECS.len()
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for spec in PAINT_OP_KIND_SPECS {
        if !seen.insert(spec.kind) {
            return Err(format!("중복 kind: {}", spec.kind));
        }
        if spec.kind.is_empty() {
            return Err("빈 kind".into());
        }
        if spec.summary_ko.is_empty() {
            return Err(format!("{} 설명이 비었다", spec.kind));
        }
        if !spec.appears_in_trace {
            return Err(format!(
                "{} 는 추적 로그에 빠져서는 안 된다 — 계측 계약이 깨진다",
                spec.kind
            ));
        }
        let plane = spec.plane_name();
        if !matches!(
            plane,
            "background" | "behindText" | "flow" | "inFrontOfText"
        ) {
            return Err(format!("{} plane 이름 불명: {plane}", spec.kind));
        }
    }
    Ok(())
}
