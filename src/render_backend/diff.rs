//! 전어댑터 상호 비교 — 있는 어댑터끼리만 맞대고 없으면 skip.
//!
//! M06-4 하네스가 파일 존재로 skip 하는 것과 같은 정신이다.
//! 이 모듈은 **컴파일된 어댑터** 의 구조·capability·추적 로그를 맞댄다.
//! 다른 형식끼리 바이트 해시를 맞대지 않는다.

use std::collections::BTreeMap;

use crate::paint::PageLayerTree;

use super::backends::{DrawStats, NullBackend, TraceBackend};
use super::caps::{BackendCapabilities, BackendFeature};
use super::png_adapter::PngBackend;
use super::skia_adapter::SkiaBackend;
use super::svg_adapter::SvgBackend;
use super::traits::{RenderBackend, RenderBackendError};
use super::util::{paint_op_kind, replay_page};

/// 비교에 참가하는 백엔드 가족.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackendFamily {
    /// 계측 Null.
    Null,
    /// 계측 Trace.
    Trace,
    /// 벡터 SVG.
    Svg,
    /// 래스터 PNG.
    Png,
    /// 래스터 Skia 문서.
    Skia,
}

impl BackendFamily {
    /// 안정 이름.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Trace => "trace",
            Self::Svg => "svg",
            Self::Png => "png",
            Self::Skia => "skia",
        }
    }

    /// 산출물 형식 가족. 같은 가족끼리만 바이트를 맞댄다.
    pub fn output_family(self) -> OutputFamily {
        match self {
            Self::Null => OutputFamily::Stats,
            Self::Trace => OutputFamily::Trace,
            Self::Svg => OutputFamily::Svg,
            Self::Png => OutputFamily::PngBytes,
            Self::Skia => OutputFamily::RasterDoc,
        }
    }

    /// 이 빌드에 항상 있는가. png/skia 타입은 항상 컴파일된다.
    pub fn always_present(self) -> bool {
        matches!(
            self,
            Self::Null | Self::Trace | Self::Svg | Self::Png | Self::Skia
        )
    }
}

/// 산출물 형식 가족.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFamily {
    /// `DrawStats`.
    Stats,
    /// 추적 문자열.
    Trace,
    /// SVG 문서.
    Svg,
    /// PNG 바이트.
    PngBytes,
    /// `RasterRenderOutput`.
    RasterDoc,
}

impl OutputFamily {
    /// 안정 이름.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stats => "stats",
            Self::Trace => "trace",
            Self::Svg => "svg",
            Self::PngBytes => "png",
            Self::RasterDoc => "raster",
        }
    }
}

/// 비교 한 쌍의 판정.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairVerdict {
    /// 같은 형식 가족이고 구조가 같다.
    Match,
    /// 같은 형식 가족인데 어긋난다.
    Mismatch(String),
    /// 다른 형식 가족 — 바이트를 맞대지 않는다.
    SkippedDifferentFamily,
}

/// 한 백엔드가 한 장면에 대해 남긴 요약.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendShot {
    /// 가족.
    pub family: BackendFamily,
    /// 광고.
    pub caps: BackendCapabilities,
    /// 추적 로그 (TraceBackend 로 같은 트리를 재생한 것).
    pub trace: String,
    /// op 총수.
    pub op_count: usize,
    /// kind 별 개수.
    pub per_kind: BTreeMap<String, usize>,
}

/// 장면을 계측 백엔드로 재생해 요약을 뜬다.
pub fn shot_from_tree(family: BackendFamily, tree: &PageLayerTree) -> Result<BackendShot, String> {
    let mut null = NullBackend::new();
    replay_page(&mut null, tree).map_err(|err| err.to_string())?;
    let stats: DrawStats = null.finish().map_err(|err| err.to_string())?;

    let mut trace_backend = TraceBackend::new();
    replay_page(&mut trace_backend, tree).map_err(|err| err.to_string())?;
    let trace = trace_backend.finish().map_err(|err| err.to_string())?;

    let caps = match family {
        BackendFamily::Null => NullBackend::new().capabilities(),
        BackendFamily::Trace => TraceBackend::new().capabilities(),
        BackendFamily::Svg => SvgBackend::new().capabilities(),
        BackendFamily::Png => PngBackend::new().capabilities(),
        BackendFamily::Skia => SkiaBackend::new().capabilities(),
    };

    let mut per_kind = BTreeMap::new();
    for (kind, count) in stats.per_kind {
        per_kind.insert(kind.to_string(), count);
    }

    Ok(BackendShot {
        family,
        caps,
        trace,
        op_count: stats.ops,
        per_kind,
    })
}

/// 두 샷을 비교한다. 다른 형식 가족은 skip.
pub fn compare_shots(left: &BackendShot, right: &BackendShot) -> PairVerdict {
    if left.family.output_family() != right.family.output_family() {
        return PairVerdict::SkippedDifferentFamily;
    }
    if left.op_count != right.op_count {
        return PairVerdict::Mismatch(format!("op_count {} != {}", left.op_count, right.op_count));
    }
    if left.per_kind != right.per_kind {
        return PairVerdict::Mismatch(format!(
            "per_kind {:?} != {:?}",
            left.per_kind, right.per_kind
        ));
    }
    if left.trace != right.trace {
        return PairVerdict::Mismatch("trace 불일치".into());
    }
    PairVerdict::Match
}

/// 모든 가족이 같은 트리에서 같은 추적 로그를 받는가.
///
/// 추적 로그는 TraceBackend 가 만들므로 가족마다 같지 않으면
/// `shot_from_tree` 구현이 깨진 것이다.
pub fn all_families_share_trace(tree: &PageLayerTree) -> Result<String, String> {
    let families = [
        BackendFamily::Null,
        BackendFamily::Trace,
        BackendFamily::Svg,
        BackendFamily::Png,
        BackendFamily::Skia,
    ];
    let mut traces = Vec::new();
    for family in families {
        let shot = shot_from_tree(family, tree)?;
        traces.push((family, shot.trace));
    }
    let first = &traces[0].1;
    for (family, trace) in &traces {
        if trace != first {
            return Err(format!("{} 추적이 null 과 다르다", family.as_str()));
        }
    }
    Ok(first.clone())
}

/// `deterministic` 광고가 켜진 백엔드만 기준선을 뜰 수 있다.
pub fn can_gold_output(caps: BackendCapabilities) -> bool {
    caps.supports(BackendFeature::Deterministic)
}

/// SVG 를 같은 트리로 두 번 그려 바이트가 같은지 본다.
pub fn svg_is_deterministic(tree: &PageLayerTree) -> Result<(), String> {
    let a = render_svg(tree)?;
    let b = render_svg(tree)?;
    if a != b {
        return Err("SvgBackend 가 결정론 광고를 어겼다".into());
    }
    if !a.starts_with("<svg") {
        return Err("SVG 산출물이 <svg 로 시작하지 않는다".into());
    }
    Ok(())
}

fn render_svg(tree: &PageLayerTree) -> Result<String, String> {
    let mut backend = SvgBackend::new();
    replay_page(&mut backend, tree).map_err(|err: RenderBackendError| err.to_string())?;
    backend.finish().map_err(|err| err.to_string())
}

/// 트리에 등장하는 kind 집합.
pub fn kind_set(tree: &PageLayerTree) -> BTreeMap<&'static str, usize> {
    fn walk<'a>(node: &'a crate::paint::LayerNode, out: &mut BTreeMap<&'static str, usize>) {
        match &node.kind {
            crate::paint::LayerNodeKind::Group { children, .. } => {
                for child in children {
                    walk(child, out);
                }
            }
            crate::paint::LayerNodeKind::ClipRect { child, .. } => walk(child, out),
            crate::paint::LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    *out.entry(paint_op_kind(op)).or_insert(0) += 1;
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(&tree.root, &mut out);
    out
}
