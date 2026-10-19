//! 자동 번호를 실제 note 소속과 원본 표제의 저장 번호에 따라 할당한다.

use crate::model::{control::AutoNumberType, document::Document};

fn counter_index_for_notes(t: AutoNumberType) -> Option<usize> {
    match t {
        AutoNumberType::Page => Some(0),
        AutoNumberType::Footnote => Some(1),
        AutoNumberType::Endnote => Some(2),
        AutoNumberType::Picture => Some(3),
        AutoNumberType::Table => Some(4),
        AutoNumberType::Equation => Some(5),
        // 총 쪽수는 페이지네이션이 끝난 뒤 결정되는 표시값이다. 일반 자동번호
        // 카운터를 증가시키거나 NewNumber로 재설정할 대상이 아니다.
        AutoNumberType::TotalPage => None,
    }
}

fn non_note_counter_index(kind: AutoNumberType) -> Option<usize> {
    match kind {
        AutoNumberType::Footnote | AutoNumberType::Endnote => None,
        _ => counter_index_for_notes(kind),
    }
}

fn footnote_counter_index(kind: AutoNumberType) -> Option<usize> {
    if kind == AutoNumberType::Endnote {
        None
    } else {
        counter_index_for_notes(kind)
    }
}

fn endnote_counter_index(kind: AutoNumberType) -> Option<usize> {
    if kind == AutoNumberType::Footnote {
        None
    } else {
        counter_index_for_notes(kind)
    }
}

/// 문서 내 모든 AutoNumber 컨트롤에 번호를 할당한다.
/// NewNumber 컨트롤을 만나면 해당 종류의 카운터를 리셋한다.
pub(crate) fn assign_auto_numbers(doc: &mut Document) {
    use crate::model::control::AutoNumberType;

    // 번호 종류별 카운터 — DocProperties 시작번호로 초기화
    let mut counters = [
        doc.doc_properties.page_start_num.saturating_sub(1),
        doc.doc_properties.footnote_start_num.saturating_sub(1),
        doc.doc_properties.endnote_start_num.saturating_sub(1),
        doc.doc_properties.picture_start_num.saturating_sub(1),
        doc.doc_properties.table_start_num.saturating_sub(1),
        doc.doc_properties.equation_start_num.saturating_sub(1),
    ];

    // 모든 섹션, 문단, 컨트롤 순회
    for section in &mut doc.sections {
        // 구역별 시작번호 반영: 0이 아니면 해당 카운터를 리셋
        let sd = &section.section_def;
        if sd.picture_num > 0 {
            counters[3] = sd.picture_num.saturating_sub(1);
        }
        if sd.table_num > 0 {
            counters[4] = sd.table_num.saturating_sub(1);
        }
        if sd.equation_num > 0 {
            counters[5] = sd.equation_num.saturating_sub(1);
        }
        if sd.page_num > 0 {
            counters[0] = sd.page_num.saturating_sub(1);
        }

        // 본문 문단
        for para in &mut section.paragraphs {
            assign_auto_numbers_in_controls(
                &mut para.controls,
                &mut counters,
                non_note_counter_index,
            );
        }
    }
}

fn assign_auto_numbers_in_controls(
    controls: &mut [crate::model::control::Control],
    counters: &mut [u16; 6],
    counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
) {
    use crate::model::control::Control;

    fn assign_caption_auto_numbers(
        caption: &mut Option<crate::model::shape::Caption>,
        counters: &mut [u16; 6],
        counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
    ) {
        if let Some(caption) = caption {
            for para in &mut caption.paragraphs {
                assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
            }
        }
    }

    fn assign_text_box_auto_numbers(
        text_box: &mut Option<crate::model::shape::TextBox>,
        counters: &mut [u16; 6],
        counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
    ) {
        if let Some(text_box) = text_box {
            for para in &mut text_box.paragraphs {
                assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
            }
        }
    }

    for ctrl in controls.iter_mut() {
        match ctrl {
            Control::AutoNumber(an) => {
                // 실제 note 본문에 속하지 않은 표제는 원본에 저장된 번호를 표시한다.
                // 각주/미주 본문에서는 owner의 카운터로 순차 번호를 할당한다.
                if matches!(
                    an.number_type,
                    AutoNumberType::Footnote | AutoNumberType::Endnote
                ) && counter_index(an.number_type).is_none()
                    && an.number > 0
                {
                    an.assigned_number = an.number;
                    continue;
                }
                let index = counter_index(an.number_type).or_else(|| {
                    matches!(
                        an.number_type,
                        AutoNumberType::Footnote | AutoNumberType::Endnote
                    )
                    .then(|| counter_index_for_notes(an.number_type))
                    .flatten()
                });
                if let Some(idx) = index {
                    counters[idx] += 1;
                    an.assigned_number = counters[idx];
                    an.number = counters[idx];
                }
            }
            Control::Table(table) => {
                // 표 내부 셀의 문단도 처리
                for cell in &mut table.cells {
                    for para in &mut cell.paragraphs {
                        assign_auto_numbers_in_controls(
                            &mut para.controls,
                            counters,
                            counter_index,
                        );
                    }
                }
                // 표 캡션 처리
                assign_caption_auto_numbers(&mut table.caption, counters, counter_index);
            }
            Control::Picture(pic) => {
                // 그림 캡션 처리
                assign_caption_auto_numbers(&mut pic.caption, counters, counter_index);
            }
            Control::Shape(shape) => {
                use crate::model::shape::ShapeObject;

                match shape.as_mut() {
                    ShapeObject::Line(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Rectangle(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Ellipse(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Arc(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Polygon(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Curve(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Group(s) => {
                        assign_caption_auto_numbers(&mut s.caption, counters, counter_index);
                    }
                    ShapeObject::Picture(s) => {
                        assign_caption_auto_numbers(&mut s.caption, counters, counter_index);
                    }
                    ShapeObject::Chart(s) => {
                        if s.caption.is_some() {
                            assign_caption_auto_numbers(&mut s.caption, counters, counter_index);
                        } else {
                            assign_caption_auto_numbers(
                                &mut s.drawing.caption,
                                counters,
                                counter_index,
                            );
                        }
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Ole(s) => {
                        if s.caption.is_some() {
                            assign_caption_auto_numbers(&mut s.caption, counters, counter_index);
                        } else {
                            assign_caption_auto_numbers(
                                &mut s.drawing.caption,
                                counters,
                                counter_index,
                            );
                        }
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                }
            }
            Control::Header(h) => {
                for para in &mut h.paragraphs {
                    assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
                }
            }
            Control::Footer(f) => {
                for para in &mut f.paragraphs {
                    assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
                }
            }
            Control::Footnote(fn_) => {
                for para in &mut fn_.paragraphs {
                    assign_auto_numbers_in_controls(
                        &mut para.controls,
                        counters,
                        footnote_counter_index,
                    );
                }
            }
            Control::Endnote(en) => {
                for para in &mut en.paragraphs {
                    assign_auto_numbers_in_controls(
                        &mut para.controls,
                        counters,
                        endnote_counter_index,
                    );
                }
            }
            Control::NewNumber(nn) => {
                if let Some(idx) = counter_index_for_notes(nn.number_type) {
                    counters[idx] = nn.number.saturating_sub(1);
                }
            }
            _ => {}
        }
    }
}
