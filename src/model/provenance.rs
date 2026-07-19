//! [#2403 Stage 1] 문서 출처(provenance)와 레이아웃 호환 정책 표면.
//!
//! 소스 포맷·변환 계보 판단은 파싱 시점에 한 번 확정되는 값이다. 이 모듈은
//! 그 판단의 소유권을 typed 값으로 모은다 — 렌더러/레이아웃은 흩어진 boolean
//! 필드 대신 [`LayoutCompatibilityProfile`] 질의를 사용한다 (Stage 1 은 기존
//! 분기의 1:1 기계 대응만, 시멘틱 변경 없음).

/// 파싱된 문서의 원본 컨테이너 포맷.
///
/// `parser::FileFormat` 의 감지 전용 항목(DRM/Empty/Unknown)은 파싱된
/// `Document` 에 도달하지 않으므로 여기 없다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceFormat {
    /// HWP 5.x 바이너리 (CFB)
    #[default]
    Hwp5,
    /// HWPX (OWPML ZIP)
    Hwpx,
    /// HWP 3.x 바이너리
    Hwp3,
    /// Standalone HWPML
    Hml,
}

/// 문서 출처 서명 — 파서가 확정하며 이후 read-only.
///
/// 기존 `Document.is_hwp3_variant`/`is_hwpx_variant` 는 Stage 1 동안 shim 으로
/// 존치하고 같은 쓰기 지점에서 이 값과 동기된다 (쓰기 지점은 파서 한정).
/// 생성기/재저장 서명 필드는 #2373 판별자 트랙이 채운다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourceProvenance {
    /// 원본 컨테이너 포맷.
    pub format: SourceFormat,
    /// 한컴 HWP3→HWP5 변환본 (휴리스틱 식별, Task #1001) — `is_hwp3_variant` 동치.
    pub hwp3_lineage: bool,
    /// rhwp HWPX→HWP 변환본 (`/RhwpHwpxOrigin` 마커, Issue #1770) —
    /// `is_hwpx_variant` 동치.
    pub hwpx_lineage: bool,
}

/// 레이아웃 호환 정책 질의 표면.
///
/// Stage 1 은 기존 boolean 분기의 1:1 대응이다 — 질의 이름은 "무엇을 켜는가"
/// 를 말하고, 값 계산은 [`crate::model::document::Document::layout_profile`] 이
/// 기존 파생식을 그대로 따른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutCompatibilityProfile {
    hwp3_layout: bool,
    hwp3_native_layout: bool,
    hwpx_stored_layout: bool,
    hwp5_origin_hwpx: bool,
}

impl LayoutCompatibilityProfile {
    pub(crate) fn new(
        hwp3_layout: bool,
        hwp3_native_layout: bool,
        hwpx_stored_layout: bool,
        hwp5_origin_hwpx: bool,
    ) -> Self {
        Self {
            hwp3_layout,
            hwp3_native_layout,
            hwpx_stored_layout,
            hwp5_origin_hwpx,
        }
    }

    /// HWP3 계보 레이아웃 보정(ParaShape 단위 정규화 등) 적용 여부 —
    /// 기존 `is_hwp3_variant` 분기 동치 (HWP3→HWP5 변환본 휴리스틱).
    pub fn hwp3_layout(&self) -> bool {
        self.hwp3_layout
    }

    /// 원본이 HWP3 파일인지 — 변환본 휴리스틱과 분리된 HWP3 저장 LINE_SEG
    /// 계약 분기. 기존 `is_hwp3_source` 동치.
    pub fn hwp3_native_layout(&self) -> bool {
        self.hwp3_native_layout
    }

    /// 저장 lineseg 를 HWPX 시멘틱으로 해석할지 여부(RowBreak 분할 tolerance
    /// 등) — 기존 `is_hwpx_source` 분기 동치: HWPX 컨테이너이면서 rhwp
    /// HWP5→HWPX 산출물이 아니거나, rhwp HWPX→HWP 변환본인 경우.
    pub fn hwpx_stored_layout(&self) -> bool {
        self.hwpx_stored_layout
    }

    /// rhwp 가 HWP5 원본에서 내보낸 HWPX 인지 — HWPX 컨테이너라도 HWP5 원본의
    /// 저장 행 높이·pagination marker 를 보존한다. 기존 `is_hwp5_origin_hwpx` 동치.
    pub fn hwp5_origin_hwpx(&self) -> bool {
        self.hwp5_origin_hwpx
    }
}
