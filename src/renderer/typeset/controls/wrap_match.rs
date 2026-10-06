//! 후속 어울림 문단의 저장 줄 매칭과 그림 anchor 조회.
//! 본문 문단 인덱스만 사용한다. 미주의 전역 인덱스 경로와 합치지 않는다.
//! 원본 조건/상수를 보존하며 페이지 상태·IR을 변경하지 않는다.

use crate::model::control::Control;
use crate::model::page::PageDef;
use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::WrapAnchorRef;

#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct WrapBand {
    pub cs: i32,
    pub sw: i32,
    pub anchor_para: usize,
    pub any_seg: bool,
}

pub(in crate::renderer::typeset) struct WrapMatch {
    pub matched: bool,
    pub is_empty_para: bool,
}

pub(in crate::renderer::typeset) fn classify(
    para: &Paragraph,
    paragraphs: &[Paragraph],
    page_def: &PageDef,
    band: WrapBand,
) -> WrapMatch {
    let para_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
    let para_sw = para
        .line_segs
        .first()
        .map(|s| s.segment_width as i32)
        .unwrap_or(0);
    let is_empty_para = para
        .text
        .chars()
        .all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n')
        && para.controls.is_empty();
    let any_seg_matches = para
        .line_segs
        .iter()
        .any(|s| s.column_start == band.cs && s.segment_width as i32 == band.sw);
    let body_w =
        (page_def.width as i32) - (page_def.margin_left as i32) - (page_def.margin_right as i32);
    let sw0_match = band.sw == 0 && is_empty_para && para_sw > 0 && para_sw < body_w / 2;
    // [Task #724] HWP5 변환본 case: anchor host 의 wrap=Square image 위치/폭/margin
    // 으로 expected_cs 정확 계산 후 para_cs 일치 확인. anchor cs=0 (caption-style)
    // 한정 가드. expected_cs = (image_x_offset + width + 2*margin) - body_left.
    let anchor_image_match = if band.cs == 0 {
        let body_left = page_def.margin_left as i32;
        let expected_cs_hu = paragraphs
            .get(band.anchor_para)
            .and_then(|p| {
                p.controls.iter().find_map(|c| {
                    let cm = match c {
                        Control::Picture(pic) => Some(&pic.common),
                        Control::Shape(s) => {
                            if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                                Some(&pic.common)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    cm.filter(|cm| {
                        !cm.treat_as_char
                            && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square)
                    })
                    .map(|cm| {
                        cm.horizontal_offset as i32 + cm.width as i32 + 2 * cm.margin.right as i32
                            - body_left
                    })
                })
            })
            .unwrap_or(0);
        expected_cs_hu > 0
            && (para_cs - expected_cs_hu).abs() < 200
            && para_sw > 0
            && para_cs + para_sw <= body_w + 200
    } else {
        false
    };
    // [Task #901] cs 일치 + 합리적 sw 매칭 (anchor 의 wrap zone region 다양성).
    // pic2.hwp paragraph 1 (cs=24470 sw=18050) vs anchor (wrap_around_cs=24470 sw=2570)
    // — cs 같지만 sw 다름 (다른 wrap region). 기존 매칭 실패 → wrap_anchors 미등록
    // → paragraph 좌측 그려짐. anchor_any_seg 가 활성이면 cs 정확 일치 만으로
    // wrap zone 내부 paragraph 로 인정.
    let cs_only_match = band.any_seg && para_cs == band.cs && para_sw > 0;
    let matched = (para_cs == band.cs && para_sw == band.sw)
        || (any_seg_matches && (is_empty_para || band.any_seg))
        || sw0_match
        || anchor_image_match
        || cs_only_match;
    WrapMatch {
        matched,
        is_empty_para,
    }
}

pub(in crate::renderer::typeset) fn anchor_is_picture(
    paragraphs: &[Paragraph],
    band: WrapBand,
) -> bool {
    paragraphs
        .get(band.anchor_para)
        .map(|p| {
            p.controls.iter().any(|c| match c {
                Control::Picture(pic) => !pic.common.treat_as_char,
                Control::Shape(s) => {
                    if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                        !pic.common.treat_as_char
                    } else {
                        false
                    }
                }
                _ => false,
            })
        })
        .unwrap_or(false)
}

pub(in crate::renderer::typeset) fn picture_anchor(
    paragraphs: &[Paragraph],
    band: WrapBand,
) -> WrapAnchorRef {
    let anchor_margin_right = paragraphs
        .get(band.anchor_para)
        .and_then(|p| {
            p.controls.iter().find_map(|c| {
                let cm = match c {
                    Control::Picture(pic) => Some(&pic.common),
                    Control::Shape(s) => {
                        if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                            Some(&pic.common)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                cm.filter(|cm| {
                    !cm.treat_as_char
                        && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square)
                })
                .map(|cm| cm.margin.right as i32)
            })
        })
        .unwrap_or(0);
    WrapAnchorRef {
        anchor_para_index: band.anchor_para,
        anchor_cs: band.cs,
        anchor_sw: band.sw,
        anchor_image_margin_right: anchor_margin_right,
        band_y_range: None,
    }
}
