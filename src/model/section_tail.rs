//! HWP5 구역 끝 목록의 저장 소유. 적용/렌더링 플래그와 저장 위치는 별개다.

use super::control::{Control, Field, FieldType};
use super::document::{Document, RawRecord};
use super::paragraph::Paragraph;
use super::shape::{Caption, ShapeObject};

/// 미해석 정보까지 포함하는 원본 목록. 본문 편집으로 이 범위를 버리지 않는다.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SectionTail {
    pub records: Vec<RawRecord>,
    /// 로드 후 모델 정규화가 끝난 시점의 바탕쪽·메모 참조 투영.
    #[serde(skip)]
    pub(crate) projection_seal: Option<[u8; 32]>,
}

enum Node<'a> {
    Paragraphs(&'a [Paragraph]),
    Shape(&'a ShapeObject),
}

fn caption<'a>(stack: &mut Vec<Node<'a>>, caption: Option<&'a Caption>) {
    if let Some(caption) = caption {
        stack.push(Node::Paragraphs(&caption.paragraphs));
    }
}

/// 본문 글자/캐럿 위치는 제외하고, 원본 목록의 의미·참조를 바꾸는 모델만 봉인한다.
/// 메모는 마지막 구역에 저장돼 다른 구역의 anchor를 참조할 수 있으므로 문서 단위다.
pub(crate) fn projection_digest(doc: &Document) -> [u8; 32] {
    let masters: Vec<_> = doc
        .sections
        .iter()
        .map(|s| &s.section_def.master_pages)
        .collect();
    let mut memos: Vec<&Field> = Vec::new();
    let mut stack: Vec<_> = doc
        .sections
        .iter()
        .rev()
        .map(|s| Node::Paragraphs(&s.paragraphs))
        .collect();
    while let Some(node) = stack.pop() {
        match node {
            Node::Paragraphs(paragraphs) => {
                for paragraph in paragraphs {
                    for control in &paragraph.controls {
                        match control {
                            Control::Table(table) => {
                                for cell in &table.cells {
                                    stack.push(Node::Paragraphs(&cell.paragraphs));
                                }
                                caption(&mut stack, table.caption.as_ref());
                            }
                            Control::Shape(shape) => stack.push(Node::Shape(shape)),
                            Control::Picture(picture) => {
                                caption(&mut stack, picture.caption.as_ref())
                            }
                            Control::Header(h) => stack.push(Node::Paragraphs(&h.paragraphs)),
                            Control::Footer(h) => stack.push(Node::Paragraphs(&h.paragraphs)),
                            Control::Footnote(n) => stack.push(Node::Paragraphs(&n.paragraphs)),
                            Control::Endnote(n) => stack.push(Node::Paragraphs(&n.paragraphs)),
                            Control::HiddenComment(c) => {
                                stack.push(Node::Paragraphs(&c.paragraphs))
                            }
                            Control::Field(field) => {
                                if field.field_type == FieldType::Memo
                                    || field.command.starts_with("MEMO/")
                                {
                                    memos.push(field);
                                }
                                stack.push(Node::Paragraphs(&field.memo_paragraphs));
                            }
                            Control::SectionDef(sd) => {
                                for master in &sd.master_pages {
                                    stack.push(Node::Paragraphs(&master.paragraphs));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Node::Shape(shape) => {
                if let Some(drawing) = shape.drawing() {
                    if let Some(text_box) = &drawing.text_box {
                        stack.push(Node::Paragraphs(&text_box.paragraphs));
                    }
                    caption(&mut stack, drawing.caption.as_ref());
                }
                match shape {
                    ShapeObject::Group(group) => {
                        caption(&mut stack, group.caption.as_ref());
                        stack.extend(group.children.iter().map(Node::Shape));
                    }
                    ShapeObject::Picture(picture) => caption(&mut stack, picture.caption.as_ref()),
                    _ => {}
                }
            }
        }
    }
    let mut hasher = blake3::Hasher::new();
    serde_json::to_writer(&mut hasher, &(masters, memos))
        .expect("구역 끝 모델 투영은 메모리에서 직렬화한다");
    *hasher.finalize().as_bytes()
}

pub(crate) fn validate_preservation(doc: &Document) -> Result<(), String> {
    if !doc.sections.iter().any(|s| s.trailing_lists.is_some()) {
        return Ok(());
    }
    let digest = projection_digest(doc);
    for (index, section) in doc.sections.iter().enumerate() {
        if let Some(tail) = &section.trailing_lists {
            if tail.projection_seal != Some(digest) {
                return Err(format!("구역 {index}의 원본 바탕쪽·메모 목록과 현재 모델의 참조가 달라졌습니다. 이 목록 자체의 재작성은 아직 지원하지 않습니다."));
            }
        }
    }
    Ok(())
}
