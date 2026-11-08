//! 문단 구성에 필요한 엔진 관측값의 읽기 전용 경계.
//!
//! 생성 시 값을 미리 읽거나 RefCell을 빌리지 않는다. 각 조회가 기존 구성 경로의
//! get/borrow 위치에서 실행되도록 하며, 구성 모듈에는 set/borrow_mut를 노출하지 않는다.

use std::cell::{Cell, Ref, RefCell};

use crate::model::provenance::LayoutCompatibilityProfile;
use crate::renderer::float_placement::FloatCarveEvidence;

pub(in crate::renderer::typeset) struct ParagraphFormatContext<'a> {
    dpi: f64,
    profile: &'a Cell<LayoutCompatibilityProfile>,
    uniform_filler_ladder: &'a Cell<bool>,
    float_carve_evidence: &'a RefCell<Vec<FloatCarveEvidence>>,
    next_stored_vpos: Option<i32>,
}

impl<'a> ParagraphFormatContext<'a> {
    pub(in crate::renderer::typeset) fn new(
        dpi: f64,
        profile: &'a Cell<LayoutCompatibilityProfile>,
        uniform_filler_ladder: &'a Cell<bool>,
        float_carve_evidence: &'a RefCell<Vec<FloatCarveEvidence>>,
        next_stored_vpos: Option<i32>,
    ) -> Self {
        Self {
            dpi,
            profile,
            uniform_filler_ladder,
            float_carve_evidence,
            next_stored_vpos,
        }
    }

    /// 본문 흐름에서 구성 중인 문단 바로 뒤 문단의 첫 저장 줄 vpos(없으면 None).
    pub(super) fn next_stored_vpos(&self) -> Option<i32> {
        self.next_stored_vpos
    }

    pub(super) fn dpi(&self) -> f64 {
        self.dpi
    }

    pub(super) fn profile(&self) -> LayoutCompatibilityProfile {
        self.profile.get()
    }

    pub(super) fn uniform_filler_ladder(&self) -> bool {
        self.uniform_filler_ladder.get()
    }

    pub(super) fn float_carve_evidence(&self) -> Ref<'_, Vec<FloatCarveEvidence>> {
        self.float_carve_evidence.borrow()
    }
}
