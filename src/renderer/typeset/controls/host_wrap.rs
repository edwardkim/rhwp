//! 표 없는 host 문단의 저장 어울림 밴드·자기 anchor 준비 Query.
//! 기존 호환 조건만 조회한다. 페이지 상태, IR 또는 엔진 flag를 변경하지 않는다.

use crate::model::control::Control;
use crate::model::page::PageDef;
use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::WrapAnchorRef;

#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct StoredHostBand {
    pub anchor_cs: i32,
    pub anchor_sw: i32,
}

pub(super) fn stored_candidate(para: &Paragraph) -> Option<StoredHostBand> {
    let has_non_tac_pic_square = para.controls.iter().any(|c| {
        let cm = match c {
            Control::Picture(p) => Some(&p.common),
            Control::Shape(s) => {
                if let crate::model::shape::ShapeObject::Picture(p) = s.as_ref() {
                    Some(&p.common)
                } else {
                    None
                }
            }
            _ => None,
        };
        cm.map(|cm| {
            !cm.treat_as_char && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square)
        })
        .unwrap_or(false)
    });
    if has_non_tac_pic_square {
        let anchor_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
        let anchor_sw = para
            .line_segs
            .first()
            .map(|s| s.segment_width as i32)
            .unwrap_or(0);
        Some(StoredHostBand {
            anchor_cs,
            anchor_sw,
        })
    } else {
        None
    }
}

pub(super) fn can_arm(band: StoredHostBand, col_w_hu: i32) -> bool {
    let StoredHostBand {
        anchor_cs,
        anchor_sw,
    } = band;
    // [#1956] 전체 폭 밴드 가드 — 옆 공간이 없으면 arming 하지 않는다.
    let band_full_width = anchor_sw > 0 && (anchor_sw - col_w_hu).abs() < 3000;
    (anchor_cs > 0 || anchor_sw > 0) && !band_full_width
}

/// 밴드를 적용한 뒤 자기 문단의 anchor 등록 여부를 판단한다.
pub(super) fn host_anchor(
    para: &Paragraph,
    page_def: &PageDef,
    para_idx: usize,
    band: StoredHostBand,
) -> Option<WrapAnchorRef> {
    let StoredHostBand {
        anchor_cs,
        anchor_sw,
    } = band;
    // [Task #722] anchor host paragraph 자체도 wrap_anchors 등록.
    // LINE_SEG cs/sw 가 wrap zone 으로 인코딩되어 있으면 host paragraph 의
    // 줄도 image 우측 wrap zone 에 layout 되어야 한다 (한컴 PDF 권위 정합).
    // 미등록 시 paragraph_layout 의 wrap_anchor 분기 미진입 → col_area
    // 전체 폭 layout → image 영역 침범 → image z-order 후 그려져 가려짐.
    //
    // Case 가드 (Stage 3~5 진단):
    //   - LINE_SEG ≥ 2 → wrap zone (multi-line)
    //   - LINE_SEG 1 + caption_room ≤ line_height → wrap zone (image 가
    //     body_top 자체에 위치 → image 위 caption 영역 없음, 강제 wrap)
    //   - LINE_SEG 1 + caption_room > line_height → caption-style (자기
    //     미등록 → col_area 전체 폭 layout, image 위 자유 영역 표시)
    let body_top_hu = page_def.margin_top as i32;
    let line_height_hu = para
        .line_segs
        .first()
        .map(|s| s.line_height as i32)
        .unwrap_or(900);
    let (image_voff_hu, image_margin_right_hu) = para
        .controls
        .iter()
        .find_map(|c| {
            let cm = match c {
                Control::Picture(p) => Some(&p.common),
                Control::Shape(s) => {
                    if let crate::model::shape::ShapeObject::Picture(p) = s.as_ref() {
                        Some(&p.common)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            cm.filter(|cm| {
                !cm.treat_as_char && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square)
            })
            .map(|cm| (cm.vertical_offset as i32, cm.margin.right as i32))
        })
        .unwrap_or((0, 0));
    let caption_room_hu = image_voff_hu - body_top_hu;
    let is_caption_style = para.line_segs.len() == 1 && caption_room_hu > line_height_hu;
    // [PR #732 후속 — exam_science 회귀 가드] image_mr=0 (margin 부재) 이면
    // 본 환경 OLD 동작 보존 — Task #722 host_self register skip.
    // 본질: image_mr > 0 인 경우 (한컴 viewer 가 inter-image-text gap 으로
    // margin 적용) 만 host_self register 가 의미. exam_science p.21/37/60 의
    // Square wrap picture 는 image_mr=0 (호스트 margin 부재) 이므로 OLD 의
    // col_area-full-width layout 정합 (line_seg cs=0/sw=실제 wrap zone 인코딩
    // 으로 한컴 정합 이미 유지). hwp3-sample5.hwp 의 page 8/27/48 (Task #722
    // 본질 영역) 은 image_mr > 0 으로 가드 통과 → 정합 유지.
    if !is_caption_style && image_margin_right_hu > 0 {
        Some(WrapAnchorRef {
            anchor_para_index: para_idx,
            anchor_cs,
            anchor_sw,
            anchor_image_margin_right: image_margin_right_hu,
            band_y_range: None,
        })
    } else {
        None
    }
}
