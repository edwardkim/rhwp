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

/// 저장된 개체 번호 범주와 실제 note 소속에 따라 표시 번호를 할당한다.
pub(crate) fn assign_auto_numbers(doc: &mut Document) {
    assign_auto_numbers_with_source(doc, true, true);
}

/// HWPX의 각주·쪽 번호 캐시는 보존하고 개체에 종속된 표시 번호만 해석한다.
pub(crate) fn assign_object_numbers(doc: &mut Document) {
    assign_auto_numbers_with_source(doc, true, false);
}

/// 개체 번호 범주를 갖지 않는 입력의 필드 순서 계약을 보존한다.
/// 해당 형식의 암묵적 개체 번호는 그 형식의 파서에서 정규화한다.
pub(crate) fn assign_auto_numbers_from_fields(doc: &mut Document) {
    assign_auto_numbers_with_source(doc, false, true);
}

struct AutoNumberCounters {
    values: [u16; 6],
    current_objects: [Option<u16>; 3],
    use_object_categories: bool,
    assign_other_fields: bool,
}

impl AutoNumberCounters {
    fn enter_object(&mut self, control: &crate::model::control::Control) -> Option<u16> {
        use crate::model::{control::Control, shape::ObjectNumberingType};
        if !self.use_object_categories {
            return None;
        }
        let category = match control {
            Control::Table(table) => table.common.numbering_type,
            Control::Picture(picture) => picture.common.numbering_type,
            Control::Shape(shape) => shape.common().numbering_type,
            Control::Equation(equation) => equation.common.numbering_type,
            _ => return None,
        };
        let index = match category {
            ObjectNumberingType::None => return Some(1),
            ObjectNumberingType::Picture => 3,
            ObjectNumberingType::Table => 4,
            ObjectNumberingType::Equation => 5,
        };
        self.values[index] = self.values[index].saturating_add(1);
        self.current_objects[index - 3] = Some(self.values[index]);
        Some(self.values[index])
    }
}

fn assign_auto_numbers_with_source(
    doc: &mut Document,
    use_object_categories: bool,
    assign_other_fields: bool,
) {
    // 번호 종류별 카운터 — DocProperties 시작번호로 초기화
    let mut counters = AutoNumberCounters {
        values: [
            doc.doc_properties.page_start_num.saturating_sub(1),
            doc.doc_properties.footnote_start_num.saturating_sub(1),
            doc.doc_properties.endnote_start_num.saturating_sub(1),
            doc.doc_properties.picture_start_num.saturating_sub(1),
            doc.doc_properties.table_start_num.saturating_sub(1),
            doc.doc_properties.equation_start_num.saturating_sub(1),
        ],
        current_objects: [None; 3],
        use_object_categories,
        assign_other_fields,
    };

    // 모든 섹션, 문단, 컨트롤 순회
    for section in &mut doc.sections {
        // 구역별 시작번호 반영: 0이 아니면 해당 카운터를 리셋
        let sd = &section.section_def;
        if sd.picture_num > 0 {
            counters.values[3] = sd.picture_num.saturating_sub(1);
        }
        if sd.table_num > 0 {
            counters.values[4] = sd.table_num.saturating_sub(1);
        }
        if sd.equation_num > 0 {
            counters.values[5] = sd.equation_num.saturating_sub(1);
        }
        if sd.page_num > 0 {
            counters.values[0] = sd.page_num.saturating_sub(1);
        }

        // 본문 문단
        for para in &mut section.paragraphs {
            assign_auto_numbers_in_controls(
                &mut para.controls,
                &mut counters,
                non_note_counter_index,
                None,
            );
        }
    }
}

fn assign_auto_numbers_in_controls(
    controls: &mut [crate::model::control::Control],
    counters: &mut AutoNumberCounters,
    counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
    caption_number: Option<u16>,
) {
    use crate::model::control::Control;

    fn assign_caption_auto_numbers(
        caption: &mut Option<crate::model::shape::Caption>,
        counters: &mut AutoNumberCounters,
        counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
        object_number: u16,
    ) {
        if let Some(caption) = caption {
            for para in &mut caption.paragraphs {
                assign_auto_numbers_in_controls(
                    &mut para.controls,
                    counters,
                    counter_index,
                    Some(object_number),
                );
            }
        }
    }

    fn assign_text_box_auto_numbers(
        text_box: &mut Option<crate::model::shape::TextBox>,
        counters: &mut AutoNumberCounters,
        counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
    ) {
        if let Some(text_box) = text_box {
            for para in &mut text_box.paragraphs {
                assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index, None);
            }
        }
    }

    for ctrl in controls.iter_mut() {
        let object_number = counters.enter_object(ctrl).unwrap_or(1);
        match ctrl {
            Control::AutoNumber(an) => {
                // 독립 Print: 캡션은 개체 번호를 반복 표시하고, 개체 밖의 필드는
                // 현재 번호를 읽는다. 필드 개수는 개체 순번을 증가시키지 않는다.
                // NONE 개체의 캡션은 시작 번호와 무관하게 기본 표시1을 소유한다.
                if counters.use_object_categories
                    && matches!(
                        an.number_type,
                        AutoNumberType::Picture | AutoNumberType::Table | AutoNumberType::Equation
                    )
                {
                    let index = counter_index_for_notes(an.number_type).unwrap();
                    an.assigned_number = caption_number
                        .or(counters.current_objects[index - 3])
                        .unwrap_or(1);
                    // 저장된 number는 캐시이므로 표시 번호로 원본 값을 덮어쓰지 않는다.
                    continue;
                }
                if !counters.assign_other_fields {
                    continue;
                }
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
                    counters.values[idx] += 1;
                    an.assigned_number = counters.values[idx];
                    an.number = counters.values[idx];
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
                            None,
                        );
                    }
                }
                // 표 캡션 처리
                assign_caption_auto_numbers(
                    &mut table.caption,
                    counters,
                    counter_index,
                    object_number,
                );
            }
            Control::Picture(pic) => {
                // 그림 캡션 처리
                assign_caption_auto_numbers(
                    &mut pic.caption,
                    counters,
                    counter_index,
                    object_number,
                );
            }
            Control::Shape(shape) => {
                use crate::model::shape::ShapeObject;

                match shape.as_mut() {
                    ShapeObject::Line(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                            object_number,
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
                            object_number,
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
                            object_number,
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
                            object_number,
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
                            object_number,
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
                            object_number,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Group(s) => {
                        assign_caption_auto_numbers(
                            &mut s.caption,
                            counters,
                            counter_index,
                            object_number,
                        );
                    }
                    ShapeObject::Picture(s) => {
                        assign_caption_auto_numbers(
                            &mut s.caption,
                            counters,
                            counter_index,
                            object_number,
                        );
                    }
                    ShapeObject::Chart(s) => {
                        if s.caption.is_some() {
                            assign_caption_auto_numbers(
                                &mut s.caption,
                                counters,
                                counter_index,
                                object_number,
                            );
                        } else {
                            assign_caption_auto_numbers(
                                &mut s.drawing.caption,
                                counters,
                                counter_index,
                                object_number,
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
                            assign_caption_auto_numbers(
                                &mut s.caption,
                                counters,
                                counter_index,
                                object_number,
                            );
                        } else {
                            assign_caption_auto_numbers(
                                &mut s.drawing.caption,
                                counters,
                                counter_index,
                                object_number,
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
                    assign_auto_numbers_in_controls(
                        &mut para.controls,
                        counters,
                        counter_index,
                        None,
                    );
                }
            }
            Control::Footer(f) => {
                for para in &mut f.paragraphs {
                    assign_auto_numbers_in_controls(
                        &mut para.controls,
                        counters,
                        counter_index,
                        None,
                    );
                }
            }
            Control::Footnote(fn_) => {
                for para in &mut fn_.paragraphs {
                    assign_auto_numbers_in_controls(
                        &mut para.controls,
                        counters,
                        footnote_counter_index,
                        None,
                    );
                }
            }
            Control::Endnote(en) => {
                for para in &mut en.paragraphs {
                    assign_auto_numbers_in_controls(
                        &mut para.controls,
                        counters,
                        endnote_counter_index,
                        None,
                    );
                }
            }
            Control::NewNumber(nn) => {
                if let Some(idx) = counter_index_for_notes(nn.number_type) {
                    counters.values[idx] = nn.number.saturating_sub(1);
                }
            }
            _ => {}
        }
    }
}
