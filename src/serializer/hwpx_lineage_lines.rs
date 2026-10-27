//! [#8310] HWPX 계보 HWP 의 본문 저장 줄을 원본 HWPX 와 같은 계약으로 쓴다.
//!
//! HWPX 로드는 저장 줄 없는 본문 문단을 합성(`LineSeg::TAG_IMPLEMENTATION_PROPERTY`)하고
//! 구역 vpos 를 다시 계산한 뒤, 원본 파일 좌표를 `source_line_seg_vertical_pos` 로 남긴다
//! (#5847). 재계산 좌표와 합성 줄은 렌더 전용이다. HWPX 계보 HWP(`/RhwpHwpxOrigin`)는
//! 다시 열 때 HWPX 와 같은 XML import 규칙으로 줄을 재구성하므로, 파일에는 원본 HWPX 가
//! 지닌 값 — 원본 좌표와 "저장 줄 없음" — 을 그대로 써야 같은 조판 입력이 된다.
//! HWPX 저장기가 같은 두 값을 되돌려 쓰는 것과 같은 계약이다.
//!
//! HWP5 계보 저장본은 다시 열 때 이 재구성을 하지 않으므로 대상이 아니다. 로드 시
//! 재계산과 스냅샷이 본문 문단에만 있으므로 본문 문단만 다룬다.
use std::borrow::Cow;

use crate::model::document::{Document, Section, HWPX_ORIGIN_STREAM_PATH};
use crate::model::paragraph::{LineSeg, Paragraph};

pub(super) fn prepare_section<'a>(
    section: Cow<'a, Section>,
    document: &Document,
) -> Cow<'a, Section> {
    let hwpx_lineage = document
        .extra_streams
        .iter()
        .any(|(path, _)| path == HWPX_ORIGIN_STREAM_PATH);
    if !hwpx_lineage
        || section.raw_provenance_permits_reuse()
        || !section.paragraphs.iter().any(needs_restore)
    {
        return section;
    }
    let mut staged = section.into_owned();
    for para in &mut staged.paragraphs {
        restore_source_lines(para);
    }
    staged.raw_stream = None;
    Cow::Owned(staged)
}

fn all_synthetic(para: &Paragraph) -> bool {
    !para.line_segs.is_empty()
        && para
            .line_segs
            .iter()
            .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
}

fn source_vpos(para: &Paragraph) -> Option<&[i32]> {
    para.source_line_seg_vertical_pos
        .as_deref()
        .filter(|v| v.len() == para.line_segs.len())
}

fn needs_restore(para: &Paragraph) -> bool {
    all_synthetic(para)
        || source_vpos(para).is_some_and(|v| {
            v.iter()
                .zip(&para.line_segs)
                .any(|(&vpos, seg)| vpos != seg.vertical_pos)
        })
}

fn restore_source_lines(para: &mut Paragraph) {
    if all_synthetic(para) {
        para.line_segs.clear();
        para.layout_only_fill_lines = 0;
        return;
    }
    if let Some(v) = source_vpos(para).map(<[i32]>::to_vec) {
        for (seg, vpos) in para.line_segs.iter_mut().zip(v) {
            seg.vertical_pos = vpos;
        }
    }
}
