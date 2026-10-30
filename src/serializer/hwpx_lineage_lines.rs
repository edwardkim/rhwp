//! [#8310] 원본 HWPX 의 저장 줄 계약을 지닌 HWP 변환본의 줄 캐시를 HWPX 와 같게 쓴다.
//!
//! HWPX 로드는 저장 줄 없는 문단을 합성(`LineSeg::TAG_IMPLEMENTATION_PROPERTY`)하고
//! 구역 vpos 를 다시 계산한 뒤, 원본 파일 좌표를 `source_line_seg_vertical_pos` 로 남긴다
//! (#5847). 재계산 좌표와 합성 줄은 렌더 전용이다. 이 변환본
//! (`Document::hwpx_origin_keeps_source_lines`)은 다시 열 때 HWPX 와 같은 XML import
//! 규칙으로 줄을 재구성하므로, 파일에는 HWPX 저장기와 같은 줄 캐시 — 원본 좌표와
//! "저장 줄 없음" — 를 써야 같은 조판 입력이 된다. 투영은 HWPX 자기검증과 같은
//! `project_exported_line_caches` 를 쓴다. IR 은 바꾸지 않고 구역 저장 사본에만 적용한다.
//!
//! HWP5 계보 저장본과 종전 값의 변환본은 다시 열 때 이 재구성을 하지 않으므로 대상이 아니다.
use std::borrow::Cow;

use crate::model::document::{Document, Section};

pub(super) fn prepare_section<'a>(
    section: Cow<'a, Section>,
    document: &Document,
) -> Cow<'a, Section> {
    if !document.hwpx_origin_keeps_source_lines() || section.raw_provenance_permits_reuse() {
        return section;
    }
    let mut staged = section.into_owned();
    super::hwpx::roundtrip::project_exported_line_caches(&mut staged.paragraphs);
    for master in &mut staged.section_def.master_pages {
        super::hwpx::roundtrip::project_exported_line_caches(&mut master.paragraphs);
    }
    staged.raw_stream = None;
    Cow::Owned(staged)
}
