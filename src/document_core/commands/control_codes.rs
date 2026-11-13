//! 한컴 편집 > 조판 부호 지우기와 찾아가기 > 조판 부호 (한컴 12.30 목록 이름 그대로).
//!
//! 문서를 본문·표 셀·글상자·캡션·머리말·꼬리말·각주·미주까지 문서 차례로 훑어 갈래별 자리를
//! 준다. 지우기는 한컴 12.30 저장본으로 확인한 대로 한다.
//!
//! - 필드(누름틀·날짜·파일 경로·문서 요약·메모·메일 머지·교정 부호·상호 참조·하이퍼링크 …)는
//!   글을 남기고 필드만 푼다. 하이퍼링크는 링크를 걸 때 덮은 색·밑줄도 되돌린다.
//! - 표·그림·각주·머리말 같은 개체는 내용째 지운다. 홀로 있던 문단은 빈 문단으로 남는다.
//! - 탭·강제 줄 나눔·고정폭·묶음 빈 칸은 그 글자를 지운다.
//! - 개요 문단은 바탕글로, 문단 번호는 번호 없는 문단 모양으로 바꾼다. 강제 쪽 나눔은 나눔만 끈다.
//! - 구역 시작은 그 구역을 앞 구역에 합치고, 바탕쪽은 구역의 바탕쪽을 모두 지운다.
//!
//! 숨은 설명·메모 본문 안은 훑지 않는다(편집 주소가 없다).

use crate::document_core::hyperlink::HyperlinkTarget;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::bin_data::BinDataType;
use crate::model::control::{AutoNumberType, Control, Field, FieldType};
use crate::model::document::Document;
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::model::shape::ShapeObject;
use crate::model::style::HeadType;

/// 경로 한 단계 `(컨트롤, 셀, 그 안 문단)`. 셀 경로 API와 같은 꼴이다.
type Step = (usize, usize, usize);

/// 표·도형 캡션의 셀 번호(셀 경로 API와 같다).
const CAPTION_CELL: usize = 65534;

/// 조판 부호 자리 하나를 지우는 방법.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Erase {
    /// 컨트롤을 내용째 지운다.
    Control(usize),
    /// 필드를 풀고 글을 남긴다.
    Field(usize),
    /// 하이퍼링크 필드를 풀고 링크 서식을 되돌린다.
    Hyperlink(usize, u32),
    /// 글자 하나를 지운다.
    Char(usize),
    /// 제목 차례 표시를 지운다.
    TitleMark(usize),
    /// 개요 문단을 바탕글로 바꾼다.
    Outline,
    /// 문단 번호를 뗀다.
    Numbering,
    /// 강제 쪽 나눔을 끈다.
    PageBreak,
    /// 구역을 앞 구역에 합친다.
    Section,
    /// 구역의 바탕쪽을 모두 지운다.
    MasterPages,
    /// 찾아가기에만 쓰는 자리(문단 시작·끝, 하이픈, 단 나누기).
    None,
}

/// 경로 한 단계의 공개 꼴.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCodeStep {
    pub control_index: usize,
    pub cell_index: usize,
    pub cell_para_index: usize,
    /// `cell`·`caption`·`textBox`·`footnote`·`endnote`·`header`·`footer`
    pub area: &'static str,
    /// 머리말·꼬리말의 적용 쪽(0 양 쪽·1 짝수 쪽·2 홀수 쪽).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_to: Option<u8>,
}

/// 조판 부호 자리 하나.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCode {
    /// 한컴 목록 이름.
    pub kind: &'static str,
    pub section: usize,
    /// 본문 문단. 안쪽 자리는 그 자리를 담은 본문 문단이다.
    pub para: usize,
    /// 본문 문단에서 이 자리를 담은 문단까지의 경로. 본문이면 비었다.
    pub path: Vec<ControlCodeStep>,
    /// 그 문단의 글자 자리.
    pub offset: usize,
    /// 컨트롤 번호(컨트롤·필드 자리).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control: Option<usize>,
    /// 글자 번호(탭·강제 줄 나눔·빈 칸·하이픈 자리).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char: Option<usize>,
    /// 지우면 그 문단에서 줄어드는 논리 칸 수(글자, 글자처럼 취급한 개체·주석·덧말·겹침).
    pub width: usize,
    #[serde(skip)]
    erase: Erase,
}

impl ControlCode {
    fn steps(&self) -> Vec<Step> {
        self.path
            .iter()
            .map(|s| (s.control_index, s.cell_index, s.cell_para_index))
            .collect()
    }

    /// `other`가 이 자리의 컨트롤 안에 드는가.
    fn contains(&self, other: &ControlCode) -> bool {
        let Erase::Control(ci) = self.erase else {
            return false;
        };
        let (mine, theirs) = (self.steps(), other.steps());
        self.section == other.section
            && self.para == other.para
            && theirs.len() > mine.len()
            && theirs[..mine.len()] == mine[..]
            && theirs[mine.len()].0 == ci
    }
}

/// 컨트롤이 속하는 한컴 갈래. 앞의 것이 대표 이름이다.
fn control_kinds(doc: &Document, control: &Control, at: &Place) -> Vec<&'static str> {
    let picture = |bin_data_id: u16| {
        let linked = bin_data_id > 0
            && doc
                .doc_info
                .bin_data_list
                .get(usize::from(bin_data_id) - 1)
                .is_some_and(|bin| bin.data_type == BinDataType::Link);
        vec![
            "그림",
            if linked {
                "연결된 그림"
            } else {
                "삽입 그림"
            },
        ]
    };
    let kind = match control {
        // 구역의 첫 구역 정의·단 정의는 구역 시작과 함께 간다.
        Control::SectionDef(_) if at.section > 0 && at.para == 0 && at.path.is_empty() => {
            "구역 시작"
        }
        Control::ColumnDef(_) if !(at.para == 0 && at.path.is_empty()) => "새 단 정의",
        Control::Table(_) => "표",
        Control::Picture(p) => return picture(p.image_attr.bin_data_id),
        Control::Shape(shape) => match &**shape {
            ShapeObject::Picture(p) => return picture(p.image_attr.bin_data_id),
            ShapeObject::Chart(_) => "차트",
            ShapeObject::Ole(_) => "OLE 개체",
            ShapeObject::Line(line) if line.connector.is_some() => "개체 연결선",
            ShapeObject::Group(_) => "그리기",
            other if other.textbox_shape_count() > 0 => "글상자",
            _ => "그리기",
        },
        Control::Header(_) => "머리말",
        Control::Footer(_) => "꼬리말",
        Control::Footnote(_) => "각주",
        Control::Endnote(_) => "미주",
        Control::AutoNumber(n) => match n.number_type {
            AutoNumberType::Page => "쪽 번호",
            AutoNumberType::Footnote => "각주 번호",
            AutoNumberType::Endnote => "미주 번호",
            AutoNumberType::Picture => "그림 번호",
            AutoNumberType::Table => "표 번호",
            AutoNumberType::Equation => "수식 번호",
            AutoNumberType::TotalPage => "전체 쪽수",
        },
        Control::NewNumber(n) => match n.number_type {
            AutoNumberType::Page => "새 쪽 번호",
            AutoNumberType::Footnote => "새 각주 번호",
            AutoNumberType::Endnote => "새 미주 번호",
            AutoNumberType::Picture => "새 그림 번호",
            AutoNumberType::Table => "새 표 번호",
            AutoNumberType::Equation => "새 수식 번호",
            AutoNumberType::TotalPage => return Vec::new(),
        },
        Control::PageNumberPos(_) => "쪽 번호 위치",
        Control::Bookmark(_) => "책갈피",
        Control::IndexMark(_) => "찾아보기",
        Control::Hyperlink(_) => "하이퍼링크",
        Control::Ruby(_) => "덧말",
        Control::CharOverlap(_) => "글자 겹치기",
        Control::PageHide(_) => "감추기",
        Control::HiddenComment(_) => "숨은 설명",
        Control::Equation(_) => "수식",
        Control::Form(_) => "양식 개체",
        Control::Field(field) => return field_kinds(field),
        _ => return Vec::new(),
    };
    vec![kind]
}

/// 필드가 속하는 한컴 갈래. 교정 부호는 묶음 이름 `교정 부호`와 낱 이름을 함께 준다.
fn field_kinds(field: &Field) -> Vec<&'static str> {
    let memo_command = field.command.starts_with("MEMO/");
    if field.raw_type.as_deref() == Some("PROOFREADING_MARKS_CHANGE") {
        return vec!["교정 부호", "교정 부호:메모 고침표"];
    }
    let kind = match field.field_type {
        _ if memo_command => "메모",
        FieldType::ClickHere => "누름틀",
        FieldType::Date => "현재 날짜",
        FieldType::DocDate => "만든 날짜",
        FieldType::Path => "파일 경로",
        FieldType::Bookmark => "책갈피",
        FieldType::MailMerge => "메일 머지",
        FieldType::CrossRef => "상호 참조",
        FieldType::Formula => "계산식",
        FieldType::Summary => "문서 요약",
        FieldType::UserInfo => "사용자 정보",
        FieldType::Hyperlink => "하이퍼링크",
        FieldType::Memo => "메모",
        FieldType::PrivateInfoSecurity => "개인 정보",
        FieldType::TableOfContents => "차례",
        FieldType::Unknown => {
            return revision_kind(&field.command)
                .map(|name| vec!["교정 부호", name])
                .unwrap_or_default()
        }
    };
    vec![kind]
}

/// 교정 부호 필드의 낱 이름. 한컴 12.30 저장본의 명령(`$Revision<이름>…;`)으로 가른다.
fn revision_kind(command: &str) -> Option<&'static str> {
    let rest = command.strip_prefix("$Revision")?;
    let name_end = rest.find([';', '?']).unwrap_or(rest.len());
    Some(match &rest[..name_end] {
        // 띄움표 묶음은 끝의 번호로 가른다: `$RevisionSign;1;`, `$RevisionSign?내용;15;`.
        "Sign" => match rest.trim_end_matches(';').rsplit(';').next()? {
            "1" => "교정 부호:띄움표",
            "2" => "교정 부호:줄 바꿈표",
            "3" => "교정 부호:줄 비움표",
            "14" => "교정 부호:부호 넣음표",
            "15" => "교정 부호:넣음표",
            _ => return None,
        },
        "SimpleChange" => "교정 부호:고침표",
        "Clipping" => "교정 부호:뺌표",
        "Delete" => "교정 부호:지움표",
        "Attach" => "교정 부호:붙임표",
        "LineAttach" => "교정 부호:줄 붙임표",
        "Sawtooth" => "교정 부호:톱니표",
        "Thinking" => "교정 부호:생각표",
        "Praise" => "교정 부호:칭찬표",
        "Line" => "교정 부호:줄표",
        "Transfer" => "교정 부호:자리 바꿈표",
        _ => return None,
    })
}

/// 지금 훑는 문단의 자리.
struct Place {
    section: usize,
    para: usize,
    path: Vec<ControlCodeStep>,
}

struct Walk<'a> {
    doc: &'a Document,
    kinds: &'a [String],
    out: Vec<ControlCode>,
}

impl Walk<'_> {
    fn first_wanted(&self, kinds: &[&'static str]) -> Option<&'static str> {
        kinds
            .iter()
            .copied()
            .find(|kind| self.kinds.iter().any(|k| k == kind))
    }

    fn push(&mut self, kinds: &[&'static str], at: &Place, offset: usize, erase: Erase) {
        self.push_wide(kinds, at, offset, erase, 0);
    }

    fn push_wide(
        &mut self,
        kinds: &[&'static str],
        at: &Place,
        offset: usize,
        erase: Erase,
        width: usize,
    ) {
        let Some(kind) = self.first_wanted(kinds) else {
            return;
        };
        let (control, char) = match erase {
            Erase::Control(ci) | Erase::Field(ci) | Erase::Hyperlink(ci, _) => (Some(ci), None),
            Erase::Char(i) => (None, Some(i)),
            _ => (None, None),
        };
        self.out.push(ControlCode {
            kind,
            section: at.section,
            para: at.para,
            path: at.path.clone(),
            offset,
            control,
            char,
            width,
            erase,
        });
    }

    fn paragraph(&mut self, p: &Paragraph, at: &mut Place) {
        let shape = self
            .doc
            .doc_info
            .para_shapes
            .get(usize::from(p.para_shape_id));
        match shape.map(|s| s.head_type) {
            Some(HeadType::Outline) => self.push(&["개요 문단"], at, 0, Erase::Outline),
            Some(HeadType::Number) => self.push(&["문단 번호 모양"], at, 0, Erase::Numbering),
            _ => {}
        }
        match p.column_type {
            ColumnBreakType::Page if !p.page_break_synthesized => {
                self.push(&["강제 쪽 나눔"], at, 0, Erase::PageBreak)
            }
            ColumnBreakType::MultiColumn => self.push(&["단 나누기"], at, 0, Erase::None),
            _ => {}
        }
        self.push(&["문단 시작"], at, 0, Erase::None);

        // 글자와 컨트롤과 제목 차례 표시를 글 차례로 섞는다. 같은 자리면 표시·컨트롤·글자 차례다.
        let positions = p.control_text_positions();
        let mut items: Vec<(usize, u8, usize)> = p
            .title_marks
            .iter()
            .enumerate()
            .map(|(i, m)| (m.char_idx, 0, i))
            .collect();
        items.extend(
            (0..p.controls.len()).map(|ci| (positions.get(ci).copied().unwrap_or(0), 1, ci)),
        );
        items.extend(
            p.text
                .chars()
                .enumerate()
                .filter(|(_, ch)| matches!(*ch, '\t' | '\n' | '\u{00A0}' | '\u{2007}' | '\u{00AD}'))
                .map(|(i, _)| (i, 2, i)),
        );
        items.sort();
        for (pos, class, index) in items {
            match class {
                0 => {
                    let kind = if p.title_marks[index].ignore {
                        "제목 차례"
                    } else {
                        "차례 숨김"
                    };
                    self.push(&[kind], at, pos, Erase::TitleMark(index));
                }
                1 => self.control(p, index, pos, at),
                _ => {
                    let (kind, erase) = match p.text.chars().nth(index) {
                        Some('\t') => ("탭", Erase::Char(index)),
                        Some('\n') => ("강제 줄 나눔", Erase::Char(index)),
                        Some('\u{00A0}') => ("묶음 빈 칸", Erase::Char(index)),
                        Some('\u{2007}') => ("고정폭 빈 칸", Erase::Char(index)),
                        _ => ("[하이픈]", Erase::None),
                    };
                    let width = usize::from(erase != Erase::None);
                    self.push_wide(&[kind], at, pos, erase, width);
                }
            }
        }
        self.push(&["문단 끝"], at, p.text.chars().count(), Erase::None);
    }

    fn control(&mut self, p: &Paragraph, ci: usize, pos: usize, at: &mut Place) {
        let control = &p.controls[ci];
        let kinds = control_kinds(self.doc, control, at);
        if !kinds.is_empty() {
            let erase = match control {
                Control::Field(f) if f.field_type == FieldType::Hyperlink => {
                    Erase::Hyperlink(ci, f.field_id)
                }
                // 짝 범위가 없는 필드는 컨트롤만 지운다.
                Control::Field(_) if p.field_ranges.iter().any(|r| r.control_idx == ci) => {
                    Erase::Field(ci)
                }
                Control::SectionDef(_) => Erase::Section,
                _ => Erase::Control(ci),
            };
            let width = match erase {
                Erase::Control(_) => usize::from(control.is_logical_inline()),
                _ => 0,
            };
            self.push_wide(&kinds, at, pos, erase, width);
        }
        for (cell, area, apply_to, paragraphs) in child_lists(control) {
            for (i, child) in paragraphs.iter().enumerate() {
                at.path.push(ControlCodeStep {
                    control_index: ci,
                    cell_index: cell,
                    cell_para_index: i,
                    area,
                    apply_to,
                });
                self.paragraph(child, at);
                at.path.pop();
            }
        }
    }
}

/// 컨트롤이 품은 문단 목록과 그 셀 번호. 셀 경로 API가 짚는 것만 준다.
fn child_lists(control: &Control) -> Vec<(usize, &'static str, Option<u8>, &[Paragraph])> {
    let mut lists: Vec<(usize, &'static str, Option<u8>, &[Paragraph])> = Vec::new();
    match control {
        Control::Table(table) => {
            for (i, cell) in table.cells.iter().enumerate() {
                lists.push((i, "cell", None, &cell.paragraphs));
            }
            if let Some(caption) = &table.caption {
                lists.push((CAPTION_CELL, "caption", None, &caption.paragraphs));
            }
        }
        Control::Shape(shape) => {
            for i in 0..shape.textbox_shape_count() {
                if let Some(text_box) = crate::document_core::helpers::get_textbox_at(shape, i) {
                    lists.push((i, "textBox", None, &text_box.paragraphs));
                }
            }
            if let Some(caption) = DocumentCore::shape_caption_ref(shape) {
                lists.push((CAPTION_CELL, "caption", None, &caption.paragraphs));
            }
        }
        Control::Picture(picture) => {
            if let Some(caption) = &picture.caption {
                lists.push((0, "caption", None, &caption.paragraphs));
            }
        }
        Control::Footnote(note) => lists.push((0, "footnote", None, &note.paragraphs)),
        Control::Endnote(note) => lists.push((0, "endnote", None, &note.paragraphs)),
        Control::Header(h) => lists.push((
            0,
            "header",
            Some(crate::document_core::header_footer_apply_to_u8(h.apply_to)),
            &h.paragraphs,
        )),
        Control::Footer(f) => lists.push((
            0,
            "footer",
            Some(crate::document_core::header_footer_apply_to_u8(f.apply_to)),
            &f.paragraphs,
        )),
        _ => {}
    }
    lists
}

impl DocumentCore {
    /// `kinds`(한컴 목록 이름) 갈래의 조판 부호 자리를 문서 차례로 준다.
    pub fn control_codes_native(&self, kinds: &[String]) -> Vec<ControlCode> {
        let mut walk = Walk {
            doc: &self.document,
            kinds,
            out: Vec::new(),
        };
        for (section, s) in self.document.sections.iter().enumerate() {
            let mut at = Place {
                section,
                para: 0,
                path: Vec::new(),
            };
            if !s.section_def.master_pages.is_empty() {
                walk.push(&["바탕쪽"], &at, 0, Erase::MasterPages);
            }
            for (para, p) in s.paragraphs.iter().enumerate() {
                at.para = para;
                walk.paragraph(p, &mut at);
            }
        }
        walk.out
    }

    /// `kinds` 갈래의 조판 부호를 지운다. `indices`가 있으면 [`Self::control_codes_native`]가 준
    /// 차례 번호 가운데 그것만 지운다. 지운 수를 돌려준다.
    pub fn erase_control_codes_native(
        &mut self,
        kinds: &[String],
        indices: Option<&[usize]>,
    ) -> Result<usize, HwpError> {
        let all = self.control_codes_native(kinds);
        let chosen: Vec<&ControlCode> = all
            .iter()
            .enumerate()
            .filter(|(i, code)| {
                code.erase != Erase::None && indices.is_none_or(|list| list.contains(i))
            })
            .map(|(_, code)| code)
            .collect();
        // 지울 컨트롤 안의 자리는 그 컨트롤과 함께 사라진다.
        let targets: Vec<&ControlCode> = chosen
            .iter()
            .copied()
            .filter(|code| !chosen.iter().any(|outer| outer.contains(code)))
            .collect();
        let mut body: Vec<(usize, usize)> = Vec::new();
        let mut nested: Vec<(usize, usize, Vec<Step>)> = Vec::new();
        let mut notes: Vec<usize> = Vec::new();
        let mut sections: Vec<usize> = Vec::new();
        let mut merges: Vec<usize> = Vec::new();
        let mut masters: Vec<usize> = Vec::new();
        let mut styles_changed = false;
        // 뒤에서부터 지워야 앞 자리의 컨트롤·글자 번호가 그대로다.
        for code in targets.iter().rev() {
            let steps = code.steps();
            match code.erase {
                Erase::Section => {
                    merges.push(code.section);
                    continue;
                }
                Erase::MasterPages => {
                    masters.push(code.section);
                    continue;
                }
                Erase::Hyperlink(_, field_id) => {
                    let target = HyperlinkTarget {
                        section: code.section,
                        para: code.para,
                        cell_path: steps.clone(),
                    };
                    self.remove_hyperlink_with_format_native(&target, field_id, true)?;
                    styles_changed = true;
                }
                Erase::Outline | Erase::Numbering => {
                    let outline = code.erase == Erase::Outline;
                    let base = self
                        .code_paragraph_mut(code.section, code.para, &steps)?
                        .para_shape_id;
                    // 한컴 12.30은 개요 문단을 바탕글 스타일과 그 문단 모양으로 바꾸고,
                    // 문단 번호는 번호만 뗀다.
                    let shape_id = if outline {
                        self.document
                            .doc_info
                            .styles
                            .first()
                            .map_or(0, |style| style.para_shape_id)
                    } else {
                        self.document.find_or_create_para_shape(
                            base,
                            &crate::model::style::ParaShapeMods {
                                head_type: Some(HeadType::None),
                                ..Default::default()
                            },
                        )
                    };
                    let para = self.code_paragraph_mut(code.section, code.para, &steps)?;
                    if outline {
                        para.style_id = 0;
                    }
                    para.para_shape_id = shape_id;
                    para.invalidate_layout_inputs();
                    styles_changed = true;
                }
                erase => {
                    let para = self.code_paragraph_mut(code.section, code.para, &steps)?;
                    match erase {
                        Erase::Control(ci) => {
                            if let Control::Footnote(_) | Control::Endnote(_) = para.controls[ci] {
                                notes.push(code.section);
                            }
                            let removed = para.remove_control(ci);
                            para.invalidate_layout_inputs();
                            match &removed {
                                Control::Table(table) => {
                                    let key = super::super::TableTextReflowKey::from_table(table);
                                    self.render_normalization.text_reflowed_tables.remove(&key);
                                }
                                // 여러 문단에 걸친 필드는 뒤 문단의 끝 표지도 걷는다(글은 남는다).
                                Control::Field(field) if field.field_id != 0 => {
                                    if let Some((section, para, steps)) = self
                                        .remove_orphan_field_end(
                                            code.section,
                                            code.para,
                                            &steps,
                                            field.field_id,
                                        )
                                    {
                                        sections.push(section);
                                        if steps.is_empty() {
                                            body.push((section, para));
                                        } else {
                                            nested.push((section, para, steps));
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        Erase::Field(ci) => {
                            if !para.unwrap_field_control(ci) {
                                return Err(HwpError::InvalidField(
                                    "필드 자리를 확인할 수 없습니다".into(),
                                ));
                            }
                        }
                        Erase::Char(i) => {
                            para.delete_text_at(i, 1);
                        }
                        Erase::TitleMark(i) => para.remove_title_mark(i),
                        Erase::PageBreak => {
                            para.column_type = ColumnBreakType::None;
                            para.raw_break_type = 0;
                            para.invalidate_layout_inputs();
                        }
                        _ => {}
                    }
                }
            }
            // 패스스루 저장을 끊어 지운 결과가 저장에 남게 한다.
            self.document.sections[code.section].raw_stream = None;
            sections.push(code.section);
            if steps.is_empty() {
                body.push((code.section, code.para));
            } else {
                nested.push((code.section, code.para, steps));
            }
        }
        if styles_changed {
            self.rebuild_resolved_styles();
        }
        self.reflow_after_code_erase(body, nested, &sections);
        notes.sort_unstable();
        notes.dedup();
        for section in notes {
            self.renumber_footnotes_in_section(section);
            self.refresh_endnote_numbers(section);
            self.refresh_footnote_numbers(section);
        }
        masters.sort_unstable();
        masters.dedup();
        for section in masters {
            let count = self.document.sections[section]
                .section_def
                .master_pages
                .len();
            for slot in (0..count).rev() {
                let apply_to = crate::document_core::MASTER_PAGE_APPLY_BASE + slot as u8;
                self.delete_master_page_native(section, apply_to)?;
            }
        }
        // 구역 합치기는 뒤 구역의 문단 번호를 바꾸므로 맨 뒤에, 뒤 구역부터 한다.
        merges.sort_unstable();
        merges.dedup();
        for section in merges.into_iter().rev() {
            self.merge_section_native(section)?;
        }
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        Ok(targets.len())
    }

    /// 여러 문단에 걸친 필드의 고아 끝 표지를 같은 문단 목록의 뒤 문단에서 찾아 걷는다.
    /// 걷은 문단의 자리(본문이면 빈 경로)를 돌려준다.
    fn remove_orphan_field_end(
        &mut self,
        section: usize,
        para: usize,
        steps: &[Step],
        field_id: u32,
    ) -> Option<(usize, usize, Vec<Step>)> {
        let owns = |p: &Paragraph| {
            p.orphan_field_ends
                .iter()
                .position(|end| end.begin_id_ref == field_id)
        };
        let Some(&(ctrl, cell, inner)) = steps.last() else {
            // 본문 필드는 구역을 넘어 닫힐 수 있다.
            for s in section..self.document.sections.len() {
                let first = if s == section { para + 1 } else { 0 };
                let paragraphs = &mut self.document.sections[s].paragraphs;
                for (p, paragraph) in paragraphs.iter_mut().enumerate().skip(first) {
                    if let Some(index) = owns(paragraph) {
                        paragraph.remove_orphan_field_end(index);
                        return Some((s, p, Vec::new()));
                    }
                }
            }
            return None;
        };
        let list = self
            .get_cell_paragraphs_mut_by_path(section, para, steps)
            .ok()?;
        for (p, paragraph) in list.iter_mut().enumerate().skip(inner + 1) {
            if let Some(index) = owns(paragraph) {
                paragraph.remove_orphan_field_end(index);
                let mut path = steps.to_vec();
                *path.last_mut()? = (ctrl, cell, p);
                return Some((section, para, path));
            }
        }
        None
    }

    fn code_paragraph_mut(
        &mut self,
        section: usize,
        para: usize,
        steps: &[Step],
    ) -> Result<&mut Paragraph, HwpError> {
        match steps.last() {
            None => self
                .document
                .sections
                .get_mut(section)
                .and_then(|s| s.paragraphs.get_mut(para))
                .ok_or_else(|| HwpError::RenderError("조판 부호 문단을 찾을 수 없습니다".into())),
            Some(&(_, _, inner)) => self
                .get_cell_paragraphs_mut_by_path(section, para, steps)?
                .get_mut(inner)
                .ok_or_else(|| HwpError::RenderError("조판 부호 문단을 찾을 수 없습니다".into())),
        }
    }

    /// 지운 문단의 줄을 다시 짜고 구역을 다시 조판한다.
    fn reflow_after_code_erase(
        &mut self,
        mut body: Vec<(usize, usize)>,
        mut nested: Vec<(usize, usize, Vec<Step>)>,
        sections: &[usize],
    ) {
        body.sort_unstable();
        body.dedup();
        let mut starts: std::collections::BTreeMap<usize, (usize, Option<i32>)> =
            Default::default();
        for &(section, para) in &body {
            let end = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section].paragraphs[para],
            );
            starts.entry(section).or_insert((para, end));
            self.reflow_paragraph(section, para);
        }
        for (section, (start, end)) in starts {
            let hwp3 = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section].paragraphs,
                start,
                None,
                end,
                &self.styles,
                self.dpi,
                hwp3,
            );
        }
        nested.sort_unstable();
        nested.dedup();
        let mut lists: std::collections::BTreeMap<(usize, usize, Vec<Step>), usize> =
            Default::default();
        for (section, para, path) in &nested {
            let Some(&(_, _, inner)) = path.last() else {
                continue;
            };
            self.reflow_cell_paragraph_by_path(*section, *para, path, inner);
            let mut list = path.clone();
            if let Some(last) = list.last_mut() {
                last.2 = 0;
            }
            lists
                .entry((*section, *para, list))
                .and_modify(|start| *start = (*start).min(inner))
                .or_insert(inner);
        }
        for ((section, para, path), start) in lists {
            self.recalculate_cell_paragraph_vpos_by_path(section, para, &path, start, None);
        }
        let mut sections = sections.to_vec();
        sections.sort_unstable();
        sections.dedup();
        for section in sections {
            self.document.sections[section].raw_stream = None;
            self.recompose_section(section);
        }
    }
}
