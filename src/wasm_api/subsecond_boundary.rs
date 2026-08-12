//! Subsecond 핫패치가 실제로 도달하는 렌더 경계 (#4577).
//!
//! `subsecond::HotFn::current(f)` 는 **`f` 하나만** 점프 테이블로 우회시킨다. 점프 테이블은
//! `map.get(&real)` 단일 키 조회이고, wasm `apply_patch` 는 메모리와 간접 함수 테이블을
//! **늘리기만** 할 뿐 이미 있는 슬롯을 다시 묶지 않는다. 즉 **전이적 재링크가 없다** — JS 가
//! base 모듈의 export 를 직접 부르면 그 아래 호출은 끝까지 옛 코드다.
//!
//! 그래서 "무엇이 핫패치되는가"는 export 마다 흩어진 `#[cfg]` 블록이 아니라 아래
//! `hot_render_boundaries!` **목록 하나**가 정한다. 한 줄을 더하면 세 가지가 함께 생긴다.
//!
//! 1. 그 경계의 dispatcher — `#[cfg(feature = "subsecond-dev")]` 분기는 이 파일의 매크로
//!    안에 **한 벌만** 있다. 복사할 `#[cfg]` 블록이 없으니 빠뜨릴 것도 없다.
//! 2. `patch_revision()` 의 한 칸 — 새 경계만 건드린 패치가 재도색을 못 부르는 구멍이
//!    생기지 않는다. `getSubsecondPatchRevision` 은 이 함수 하나만 부른다.
//! 3. `hot_render_exports()` 의 export 이름 — 아래 테스트가 "이 export 는 경계 뒤에 있어야
//!    한다"를 기계로 검사한다.
//!
//! 반대 방향도 닫혀 있다. dispatcher 마다 `#[deny(dead_code)]` 가 붙으므로 목록에 올려 놓고
//! export 를 배선하지 않으면 빌드가 깨진다 — 저장소 전역은 `dead_code = "allow"` 라 이
//! 국소 deny 가 없으면 조용히 통과한다.
//!
//! ## 인자 9개 상한
//!
//! `subsecond` 는 `HotFunction` 을 인자 9개까지만 구현한다(`impl_hot_function!` 의
//! `Fn9Marker` 가 마지막이다). `&HwpDocument` 가 첫 자리를 쓰므로 경계 함수가 실제로 받을 수
//! 있는 인자는 8개다. 부분 재도색은 `x/y/width/height` 를 `BoundingBox` 하나로 접어 이 상한
//! 안에 들어간다 — 그대로 펴 두면 10개라 `HotFn::current` 가 아예 컴파일되지 않는다.

use wasm_bindgen::JsValue;

use super::HwpDocument;
#[cfg(target_arch = "wasm32")]
use crate::renderer::render_tree::BoundingBox;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

/// 렌더 경계 목록에서 dispatcher · 패치 리비전 · export 이름표를 함께 만든다.
///
/// 항목 문법은 `exports [...] fn <dispatcher>(<인자>) -> <반환> = <패치 대상>;` 이다.
/// `#[cfg(...)]` 만 항목 앞에 붙일 수 있다 — 설명은 `//` 주석으로 적는다(문서 주석은
/// 아래 집계 함수의 문장 자리에 붙을 수 없다).
macro_rules! hot_render_boundaries {
    (
        $(
            $(#[cfg($cfg:meta)])?
            exports [$($export:literal),+ $(,)?]
            fn $name:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty = $target:path;
        )+
    ) => {
        $(
            // 목록에 올려 놓고 export 를 배선하지 않으면 여기서 빌드가 깨진다. 저장소
            // 전역이 `dead_code = "allow"`(Cargo.toml) 라 이 한 줄이 없으면 조용히 통과한다.
            $(#[cfg($cfg)])?
            #[deny(dead_code)]
            #[inline]
            pub(crate) fn $name($($arg: $ty),*) -> $ret {
                #[cfg(feature = "subsecond-dev")]
                {
                    let mut hot = subsecond::HotFn::current($target);
                    return hot.call(($($arg,)*));
                }
                #[cfg(not(feature = "subsecond-dev"))]
                $target($($arg),*)
            }
        )+

        /// 이 타깃에서 실제로 컴파일된 경계 수. `patch_revision()` 의 칸 수와 같다.
        #[cfg(all(test, feature = "subsecond-dev"))]
        pub(crate) fn compiled_boundary_count() -> usize {
            let mut count = 0usize;
            $(
                $(#[cfg($cfg)])?
                {
                    count += 1;
                }
            )+
            count
        }

        /// 경계 뒤에 있는 wasm export 이름 전부.
        ///
        /// 타깃별 `#[cfg]` 를 타지 않는다 — 네이티브 테스트에서도 wasm32 전용 경계가
        /// 선언되어 있는지 볼 수 있어야 한다.
        #[cfg(test)]
        pub(crate) fn hot_render_exports() -> Vec<&'static str> {
            let mut exports: Vec<&'static str> = Vec::new();
            $(
                exports.extend_from_slice(&[$($export),+]);
            )+
            exports
        }

        /// 경계마다 현재 함수 주소를 이어 붙인 패치 리비전.
        ///
        /// studio 는 이 문자열이 바뀌면 캐시를 버리고 다시 그린다. 문자열을 해석하지 않고
        /// 이전 값과 비교만 하므로 칸 수가 늘어도 계약이 깨지지 않는다.
        #[cfg(feature = "subsecond-dev")]
        pub(crate) fn patch_revision() -> String {
            use std::fmt::Write as _;

            let mut revision = String::new();
            $(
                $(#[cfg($cfg)])?
                {
                    if !revision.is_empty() {
                        revision.push(':');
                    }
                    let _ = write!(
                        revision,
                        "{:016x}",
                        crate::subsecond_dev::hot_fn_ptr($target)
                    );
                }
            )+
            revision
        }
    };
}

hot_render_boundaries! {
    // 전체 재도색. 본문·overlay·static plane 을 각각 그린다 — `renderer::web_canvas` 와
    // `renderer::layer_renderer` 의 페인트 전부가 이 아래에 있다.
    #[cfg(target_arch = "wasm32")]
    exports ["renderPageToCanvasFiltered", "renderPageToCanvasFilteredWithProfile"]
    fn render_page_to_canvas_filtered_with_profile(
        document: &HwpDocument,
        page_num: u32,
        canvas: &HtmlCanvasElement,
        scale: f64,
        layer_kind: &str,
        profile: &str,
    ) -> Result<(), JsValue> = super::render_page_to_canvas_filtered_with_profile_impl;

    // 타이핑 중 부분 재도색. 같은 페인트 코드를 dirty rect 로만 다시 재생한다. 경계가 없으면
    // 전체 재도색만 새 코드로 그리고 그 뒤 키 입력마다 자기 줄을 옛 코드로 되돌린다 (#4577).
    #[cfg(target_arch = "wasm32")]
    exports ["renderPagePatchToCanvasFilteredWithProfile"]
    fn render_page_patch_to_canvas_filtered_with_profile(
        document: &HwpDocument,
        page_num: u32,
        canvas: &HtmlCanvasElement,
        scale: f64,
        layer_kind: &str,
        profile: &str,
        patch: BoundingBox,
    ) -> Result<(), JsValue> = super::render_page_patch_to_canvas_filtered_with_profile_impl;

    // 레이어 트리 직렬화. `getPageLayerTree` 는 같은 대상의 screen profile 기본값이라 같은
    // 경계를 쓴다 — PageRenderer 는 overlay 좁은 질의가 안 될 때 이쪽으로 되돌아온다.
    exports ["getPageLayerTree", "getPageLayerTreeWithProfile"]
    fn get_page_layer_tree_with_profile(
        document: &HwpDocument,
        page_num: u32,
        profile: &str,
        omit_image_bytes: bool,
    ) -> Result<String, JsValue> = super::get_page_layer_tree_with_profile_impl;

    // 레이어 평면 분류. `getLayerPlaneSummary` 를 먹여 static flow 분리 여부, overlay 캔버스
    // 생성 여부, **부분 재도색 자격 여부**를 정한다. 여기가 옛 코드면 페인트는 새 코드로
    // 그리고 합성 판정은 옛 코드로 내려 둘이 어긋난다.
    exports ["getPageOverlayImages"]
    fn get_page_overlay_images(
        document: &HwpDocument,
        page_num: u32,
    ) -> Result<String, JsValue> = super::get_page_overlay_images_impl;

    // 본문 그림의 DOM 배치. 같은 평면 분류와 조상 clip 접기를 써서 `<img>` 의 bbox 를 낸다.
    // 이 값은 경계 뒤에서 그린 캔버스 위에 합성되므로 둘의 분류가 갈리면 그림이 어긋난다.
    exports ["getPageFlowImageOps"]
    fn get_page_flow_image_ops(
        document: &HwpDocument,
        page_num: u32,
    ) -> Result<String, JsValue> = super::get_page_flow_image_ops_impl;
}

/// 일부러 경계 밖에 둔 export 와 그 이유. 위 목록과 한 쌍이라 붙여 둔다.
///
/// "아직 안 했다"가 아니라 **경계를 두어도 얻는 것이 없거나 오히려 어긋난다**는 판단이다.
/// 판단이 바뀌면 여기서 빼고 위 `hot_render_boundaries!` 로 옮기면 된다.
#[cfg(test)]
const DELIBERATELY_COLD_EXPORTS: &[(&str, &str)] = &[
    (
        "getPageSourceImageKeys",
        "그림 신원 키만 만든다 — 페인트도 평면 분류도 아래에 없다. 캐시 신원을 \
         핫패치하면 이미 저장된 서명과 비교가 불가능해져 재디코드만 늘어난다.",
    ),
    (
        "getSourceImageBytes",
        "결과를 studio 가 키별 object URL 로 메모이즈한다(FlowImageUrlCache). 키가 \
         바뀌기 전에는 다시 들어오지 않으므로 경계를 두어도 관측되지 않는다.",
    ),
    (
        "getPageInfo",
        "아래에 페인트도 평면 분류도 없다 — 이미 계산된 페이지네이션(`find_page`)을 읽고 \
         PageDef 여백을 px 로 환산할 뿐이다. 게다가 그중 낡을 수 있는 값(page 크기·단 \
         영역)은 `invalidateSubsecondRenderCaches` 가 비우지 않는 pagination 에서 오므로 \
         경계를 두면 '새 여백 산술 + 옛 페이지네이션' 이라는 반만 새 기록이 된다. \
         한 페이지를 그릴 때마다, 또 `refreshPages()` 마다 문서 전 페이지분이 도는 \
         이 목록에서 가장 잦은 질의이기도 하다.",
    ),
    (
        "getCanvasKitReplayPlanWithProfile",
        "PageRenderer 가 부르지 않는다 — 소비자는 e2e 기준선 수집기뿐이다. CanvasKit \
         페인트는 TypeScript 이고 그 입력 트리는 이미 경계 뒤에서 온다.",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// `PageRenderer` 가 한 페이지를 그리는 동안 부르는 wasm export 중 **렌더 결과나 합성
    /// 판정을 만들어 내는** 것들. 이 목록이 늘어나면 경계도 함께 늘어나야 한다.
    const RENDER_PATH_EXPORTS_THAT_MUST_BE_HOT: &[(&str, &str)] = &[
        (
            "renderPageToCanvasFilteredWithProfile",
            "page-renderer.ts renderFlowCanvas·applyOverlays — 전체 재도색 페인트",
        ),
        (
            "renderPagePatchToCanvasFilteredWithProfile",
            "page-renderer.ts renderFocusedPagePatch — 타이핑 중 부분 재도색 페인트",
        ),
        (
            "getPageLayerTree",
            "page-renderer.ts getLayerPlaneSummaryFromTree·getFlowImagePaintOps 폴백",
        ),
        (
            "getPageLayerTreeWithProfile",
            "page-renderer.ts renderPageCanvasKit — CanvasKit 백엔드가 재생할 레이어 트리",
        ),
        (
            "getPageOverlayImages",
            "page-renderer.ts getLayerPlaneSummary — 합성·부분 재도색 자격 판정",
        ),
        (
            "getPageFlowImageOps",
            "page-renderer.ts getFlowImagePaintOps — 본문 그림 DOM 배치",
        ),
    ];

    #[test]
    fn every_render_path_export_sits_behind_a_hot_patch_boundary() {
        let hot = hot_render_exports();
        let missing: Vec<&str> = RENDER_PATH_EXPORTS_THAT_MUST_BE_HOT
            .iter()
            .filter(|(export, _)| !hot.contains(export))
            .map(|(export, _)| *export)
            .collect();
        assert!(
            missing.is_empty(),
            "이 export 들은 PageRenderer 의 렌더 경로에 있는데 핫패치 경계 뒤에 없다 — \
             패치를 걸어도 base 모듈의 옛 코드가 그린다: {missing:?}"
        );
    }

    #[test]
    fn each_export_is_declared_by_exactly_one_boundary() {
        let mut hot = hot_render_exports();
        let declared = hot.len();
        hot.sort_unstable();
        hot.dedup();
        assert_eq!(
            declared,
            hot.len(),
            "같은 export 가 두 경계에 선언되어 있다 — 어느 쪽으로 배선했는지 알 수 없다"
        );
    }

    #[test]
    fn deliberately_cold_exports_are_not_also_declared_hot() {
        let hot = hot_render_exports();
        for (export, reason) in DELIBERATELY_COLD_EXPORTS {
            assert!(
                !hot.contains(export),
                "{export} 는 경계 밖으로 두기로 해 놓고 경계 목록에도 들어 있다"
            );
            assert!(
                !reason.trim().is_empty(),
                "{export} 를 경계 밖에 두는 이유가 비어 있다"
            );
        }
    }

    /// 리비전은 경계마다 한 칸이어야 한다. 칸이 빠지면 그 경계만 바뀐 패치가 studio 의
    /// 재도색을 부르지 못한다.
    #[cfg(feature = "subsecond-dev")]
    #[test]
    fn patch_revision_covers_every_compiled_boundary() {
        let revision = patch_revision();
        let mut segments: Vec<&str> = revision.split(':').collect();
        assert_eq!(
            segments.len(),
            compiled_boundary_count(),
            "패치 리비전이 경계 수와 다르다: {revision}"
        );
        assert!(
            segments.iter().all(|segment| segment.len() == 16),
            "리비전 칸은 16자리 함수 주소여야 한다: {revision}"
        );

        let probed = segments.len();
        segments.sort_unstable();
        segments.dedup();
        assert_eq!(
            probed,
            segments.len(),
            "두 경계가 같은 주소를 낸다 — 한쪽 패치가 리비전에 나타나지 않는다: {revision}"
        );
    }
}
