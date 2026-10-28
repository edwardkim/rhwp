//! Resolve only identities relevant to the source, while scanning the whole owned
//! document for outside endpoints/ambiguous owners. Never repair the original.
use super::{
    reject, Located, ParagraphBlockPathStep as Step, ParagraphBlockValidationError as Error,
    SourceNode,
};
use crate::document_core::commands::paragraph_block::owned::{self, Node};
use crate::{
    error::HwpError,
    model::{
        control::{Control, Field},
        document::Document,
        identity::subject_alias,
        paragraph::Paragraph,
        shape::ShapeObject,
    },
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Target {
    first: usize,
    owner: Option<usize>,
    ambiguous: bool,
    ends: usize,
    outside: bool,
}
impl Target {
    fn observe(&mut self, owner: usize) {
        self.ambiguous |= self.owner.is_some_and(|old| old != owner);
        self.owner.get_or_insert(owner);
    }
    fn is_unique(&self) -> bool {
        self.owner.is_some() && !self.ambiguous
    }
}
type Targets = BTreeMap<u32, Target>;

fn owner<T>(value: &T) -> usize {
    value as *const T as usize
}
fn field(c: &Control) -> Option<&Field> {
    if let Control::Field(f) = c {
        Some(f)
    } else {
        None
    }
}

fn subjects(node: SourceNode<'_>) -> Option<(usize, [u32; 3])> {
    match node {
        SourceNode::Shape(s) => Some((
            owner(s),
            [
                s.common().instance_id,
                subject_alias(s.common().instance_id),
                s.drawing().map_or(0, |d| d.inst_id),
            ],
        )),
        SourceNode::Control(Control::Picture(p)) => Some((
            owner(p.as_ref()),
            [p.common.instance_id, subject_alias(p.common.instance_id), 0],
        )),
        _ => None,
    }
}
fn connector(node: SourceNode<'_>) -> Option<(usize, [u32; 2])> {
    if let SourceNode::Shape(ShapeObject::Line(l)) = node {
        l.connector
            .as_ref()
            .map(|c| (owner(&l.common), [c.start_subject_id, c.end_subject_id]))
    } else {
        None
    }
}
fn source_node<'a>(node: &Node<'a>) -> Option<SourceNode<'a>> {
    match node {
        Node::Para(p) => Some(SourceNode::Paragraph(p)),
        Node::Control(c) => Some(SourceNode::Control(c)),
        Node::Shape(s) => Some(SourceNode::Shape(s)),
        _ => None,
    }
}

pub(super) fn validate(
    doc: &Document,
    nodes: &[Located<'_>],
    max_records: usize,
) -> Result<(), Error> {
    let mut fields = Targets::new();
    let mut targets = Targets::new();
    let mut paras = BTreeMap::new();
    let mut connectors = BTreeSet::new();
    for (index, located) in nodes.iter().enumerate() {
        match located.node {
            SourceNode::Paragraph(p) => {
                paras.insert(owner(p), index);
            }
            SourceNode::Control(Control::Field(f)) => {
                if fields
                    .insert(
                        f.field_id,
                        Target {
                            first: index,
                            ..Default::default()
                        },
                    )
                    .is_some()
                {
                    return Err(reject(
                        &located.path,
                        "ambiguousField",
                        format!("duplicate begin ID {}", f.field_id),
                    ));
                }
            }
            _ => {}
        }
        if let Some((_, ids)) = subjects(located.node) {
            for id in ids.into_iter().filter(|id| *id != 0) {
                targets.entry(id).or_insert_with(|| Target {
                    first: index,
                    ..Default::default()
                });
            }
        }
        if let Some((key, _)) = connector(located.node) {
            connectors.insert(key);
        }
    }
    // Validate every endpoint inside the source, including ends without begins.
    for located in nodes {
        if let SourceNode::Paragraph(p) = located.node {
            validate_paragraph(p, located, nodes, &fields, max_records)?;
        }
        if let Some((_, ids)) = connector(located.node) {
            for id in ids.into_iter().filter(|id| *id != 0) {
                if !targets.contains_key(&id) {
                    return Err(reject(
                        &located.path,
                        "externalConnector",
                        format!("subject {id} is outside the source or missing"),
                    ));
                }
            }
        }
    }
    // Reference arrays are charged independently of owned nodes: a single
    // paragraph can contain many end records. No unbounded global reference map.
    let mut records = 0usize;
    owned::inspect_document(doc, max_records, |node| {
        let Some(source) = source_node(node) else {
            return Ok(());
        };
        match source {
            SourceNode::Control(Control::Field(f)) => {
                if let Some(t) = fields.get_mut(&f.field_id) {
                    t.observe(owner(f));
                }
            }
            SourceNode::Paragraph(p) => {
                let length = p
                    .field_ranges
                    .len()
                    .checked_add(p.orphan_field_ends.len())
                    .ok_or_else(|| super::super::invalid("reference count overflow"))?;
                records = records
                    .checked_add(length)
                    .ok_or_else(|| super::super::invalid("reference count overflow"))?;
                if records > max_records {
                    return Err(super::super::invalid("reference scan budget exceeded"));
                }
                let inside = paras.contains_key(&owner(p));
                for r in &p.field_ranges {
                    if let Some(f) = p.controls.get(r.control_idx).and_then(field) {
                        if let Some(t) = fields.get_mut(&f.field_id) {
                            t.ends += 1;
                            t.outside |= !inside;
                        }
                    }
                }
                for end in &p.orphan_field_ends {
                    if let Some(t) = fields.get_mut(&end.begin_id_ref) {
                        t.ends += 1;
                        t.outside |= !inside;
                    }
                }
            }
            _ => {}
        }
        if let Some((key, ids)) = subjects(source) {
            for id in ids.into_iter().filter(|id| *id != 0) {
                if let Some(t) = targets.get_mut(&id) {
                    t.observe(key);
                }
            }
        }
        if let Some((key, ids)) = connector(source) {
            for id in ids.into_iter().filter(|id| *id != 0) {
                if let Some(t) = targets.get_mut(&id) {
                    t.outside |= !connectors.contains(&key);
                }
            }
        }
        Ok::<(), HwpError>(())
    })?;
    for t in fields.values() {
        let path = &nodes[t.first].path;
        if !t.is_unique() {
            return Err(reject(
                path,
                "ambiguousField",
                "begin ID has multiple owners in the original document",
            ));
        }
        if t.outside {
            return Err(reject(
                path,
                "externalField",
                "field begins inside the source but ends outside",
            ));
        }
        if t.ends != 1 {
            return Err(reject(
                path,
                "fieldClosure",
                format!("expected one end marker, found {}", t.ends),
            ));
        }
    }
    for located in nodes {
        if let Some((_, ids)) = connector(located.node) {
            for id in ids.into_iter().filter(|id| *id != 0) {
                if !targets[&id].is_unique() {
                    return Err(reject(
                        &located.path,
                        "ambiguousConnector",
                        format!("subject {id} has multiple owners"),
                    ));
                }
            }
        }
    }
    if let Some(t) = targets
        .values()
        .filter(|t| t.outside)
        .min_by_key(|t| t.first)
    {
        return Err(reject(
            &nodes[t.first].path,
            "externalConnector",
            "outside connector refers to a source object",
        ));
    }
    Ok(())
}

fn validate_paragraph(
    p: &Paragraph,
    located: &Located<'_>,
    nodes: &[Located<'_>],
    fields: &Targets,
    limit: usize,
) -> Result<(), Error> {
    if p.field_ranges
        .len()
        .saturating_add(p.orphan_field_ends.len())
        > limit
    {
        return Err(reject(
            &located.path,
            "budgetOrAddress",
            "reference scan budget exceeded",
        ));
    }
    let chars = p.text.chars().count();
    for r in &p.field_ranges {
        if p.controls.get(r.control_idx).and_then(field).is_none()
            || r.start_char_idx > r.end_char_idx
            || r.end_char_idx > chars
            || r.control_idx
                .checked_add(r.inner_slot_count)
                .is_none_or(|end| end >= p.controls.len())
        {
            return Err(reject(
                &located.path,
                "invalidFieldRange",
                "invalid field control/text/slot range",
            ));
        }
    }
    for end in &p.orphan_field_ends {
        let Some(target) = fields.get(&end.begin_id_ref) else {
            return Err(reject(
                &located.path,
                "externalField",
                format!(
                    "begin {} is outside the source or missing",
                    end.begin_id_ref
                ),
            ));
        };
        if end.char_idx > chars {
            return Err(reject(
                &located.path,
                "invalidFieldRange",
                "field end lies outside paragraph text",
            ));
        }
        let begin = &nodes[target.first];
        // A field cannot cross into a different cell/textbox/caption owner list.
        let begin_para = &begin.path[..begin.path.len() - 1]; // remove Control
        let (Some(Step::Paragraph(begin_index)), Some(Step::Paragraph(end_index))) =
            (begin_para.last(), located.path.last())
        else {
            return Err(reject(
                &located.path,
                "fieldOrder",
                "invalid paragraph ownership path",
            ));
        };
        if begin_para[..begin_para.len() - 1] != located.path[..located.path.len() - 1]
            || begin_index >= end_index
        {
            return Err(reject(
                &located.path,
                "fieldOrder",
                "field end precedes begin or crosses an owned paragraph-list boundary",
            ));
        }
        if let SourceNode::Control(Control::Field(f)) = begin.node {
            if end.begin_ctrl_id != 0 && f.ctrl_id != 0 && end.begin_ctrl_id != f.ctrl_id {
                return Err(reject(
                    &located.path,
                    "fieldTypeMismatch",
                    "end marker control type differs from begin",
                ));
            }
        }
    }
    Ok(())
}
