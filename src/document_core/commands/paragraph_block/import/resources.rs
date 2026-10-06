//! Reachable-only, detached resource delta. Existing target entries are never edited.
mod binary;
mod styles;

use super::super::{budget, invalid, validation::Resource};
use super::ImportParagraphBlockLimits;
use crate::{
    error::HwpError,
    model::{
        document::{DocInfo, Document},
        style::*,
    },
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResourceCounts {
    pub added: usize,
    pub reused: usize,
    pub binaries_added: usize,
    pub binaries_reused: usize,
    pub metadata_bytes: usize,
    pub binary_bytes_read: usize,
}

pub(super) struct Resources {
    pub delta: DocInfo,
    pub binaries: Vec<crate::model::bin_data::BinDataContent>,
    pub map: BTreeMap<Resource, u32>,
    pub counts: ImportResourceCounts,
}

pub(super) struct Meter {
    left: usize,
    steps: usize,
}
impl Meter {
    pub fn tick(&mut self) -> Result<(), HwpError> {
        self.steps = self
            .steps
            .checked_sub(1)
            .ok_or_else(|| invalid("resource lookup budget exceeded"))?;
        Ok(())
    }
    pub fn charge<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), HwpError> {
        self.tick()?;
        self.left -= budget::measure_value(value, self.left)?;
        Ok(())
    }
    pub fn key<T: Serialize>(&mut self, value: &T) -> Result<serde_json::Value, HwpError> {
        self.charge(value)?;
        let mut key = serde_json::to_value(value).map_err(|e| invalid(e.to_string()))?;
        if let Some(object) = key.as_object_mut() {
            object.remove("raw_data");
            canonical_font_reference(object);
            if let Some(substitute) = object
                .get_mut("subst_font")
                .and_then(serde_json::Value::as_object_mut)
            {
                canonical_font_reference(substitute);
            }
        }
        Ok(key)
    }
}

fn canonical_font_reference(object: &mut serde_json::Map<String, serde_json::Value>) {
    if object.get("is_embedded") == Some(&serde_json::Value::Bool(true))
        && object
            .get("resolved_bin_data_id")
            .is_some_and(serde_json::Value::is_number)
    {
        // Manifest spelling is source-local. The resolved target storage ID is the meaning.
        object.remove("bin_item_id_ref");
    }
}

/// Compare after references have been mapped. Checked ceilings precede append.
fn intern<T: Serialize>(
    existing: &[T],
    added: &mut Vec<T>,
    value: T,
    max_index: usize,
    meter: &mut Meter,
    counts: &mut ImportResourceCounts,
) -> Result<u32, HwpError> {
    let key = meter.key(&value)?;
    for (index, item) in existing.iter().chain(added.iter()).enumerate() {
        if meter.key(item)? == key {
            if index > max_index {
                return Err(invalid("reused resource ID exceeds encoding range"));
            }
            counts.reused += 1;
            return Ok(index as u32);
        }
    }
    let index = existing
        .len()
        .checked_add(added.len())
        .ok_or_else(|| invalid("resource count overflow"))?;
    if index > max_index {
        return Err(invalid("new resource ID exceeds encoding range"));
    }
    added.push(value);
    counts.added += 1;
    Ok(index as u32)
}

impl Resources {
    pub fn id(&self, resource: Resource) -> Result<u32, HwpError> {
        self.map
            .get(&resource)
            .copied()
            .ok_or_else(|| invalid(format!("unmapped source {resource:?}")))
    }
    pub fn id16(&self, resource: Resource) -> Result<u16, HwpError> {
        u16::try_from(self.id(resource)?).map_err(|_| invalid(format!("{resource:?} exceeds u16")))
    }
    pub fn border(&self, old: u16) -> Result<u16, HwpError> {
        if old == 0 {
            Ok(0)
        } else {
            self.id16(Resource::Border(old))
        }
    }
    pub fn fill(&self, fill: &mut Fill) -> Result<(), HwpError> {
        if let Some(image) = &mut fill.image {
            image.bin_data_id = self.id16(Resource::Image(image.bin_data_id))?;
        }
        Ok(())
    }

    pub fn prepare(
        source: &Document,
        target: &Document,
        reachable: &BTreeSet<Resource>,
        outline: u16,
        limits: ImportParagraphBlockLimits,
    ) -> Result<Self, HwpError> {
        let mut result = Self {
            delta: DocInfo::default(),
            binaries: vec![],
            map: BTreeMap::new(),
            counts: ImportResourceCounts::default(),
        };
        let mut meter = Meter {
            left: limits.max_resource_metadata_bytes,
            steps: limits.block.max_document_nodes,
        };
        binary::prepare(
            &mut result,
            source,
            target,
            reachable,
            &mut meter,
            limits.max_binary_bytes,
        )?;
        // The non-style graph is acyclic: binary -> font/border/tab -> char ->
        // numbering/bullet -> paragraph. Styles are resolved separately as a graph.
        for phase in 0..5 {
            for &resource in reachable {
                meter.tick()?;
                let src = &source.doc_info;
                let dst = &target.doc_info;
                let index = match (phase, resource) {
                    (0, Resource::Font(language, id)) => {
                        meter.charge(&src.font_faces[language][id as usize])?;
                        let mut v = src.font_faces[language][id as usize].clone();
                        v.raw_data = None;
                        if v.is_embedded {
                            let mapped =
                                result
                                    .id16(Resource::Storage(v.resolved_bin_data_id.ok_or_else(
                                        || invalid("embedded font storage missing"),
                                    )?))?;
                            v.resolved_bin_data_id = Some(mapped);
                            v.bin_item_id_ref = format!("image{mapped}");
                        }
                        if let Some(sub) = &mut v.subst_font {
                            if sub.is_embedded {
                                let mapped = result.id16(Resource::Storage(
                                    sub.resolved_bin_data_id.ok_or_else(|| {
                                        invalid("embedded substitute font storage missing")
                                    })?,
                                ))?;
                                sub.resolved_bin_data_id = Some(mapped);
                                sub.bin_item_id_ref = format!("image{mapped}");
                            }
                        }
                        while result.delta.font_faces.len() <= language {
                            result.delta.font_faces.push(vec![]);
                        }
                        intern(
                            dst.font_faces.get(language).map_or(&[], Vec::as_slice),
                            &mut result.delta.font_faces[language],
                            v,
                            u16::MAX as usize,
                            &mut meter,
                            &mut result.counts,
                        )?
                    }
                    (1, Resource::Border(id)) => {
                        meter.charge(&src.border_fills[id as usize - 1])?;
                        let mut v = src.border_fills[id as usize - 1].clone();
                        v.raw_data = None;
                        result.fill(&mut v.fill)?;
                        intern(
                            &dst.border_fills,
                            &mut result.delta.border_fills,
                            v,
                            u16::MAX as usize - 1,
                            &mut meter,
                            &mut result.counts,
                        )? + 1
                    }
                    (1, Resource::Tab(id)) => {
                        meter.charge(&src.tab_defs[id as usize])?;
                        let mut v = src.tab_defs[id as usize].clone();
                        v.raw_data = None;
                        intern(
                            &dst.tab_defs,
                            &mut result.delta.tab_defs,
                            v,
                            u16::MAX as usize,
                            &mut meter,
                            &mut result.counts,
                        )?
                    }
                    (2, Resource::Char(id)) => {
                        meter.charge(&src.char_shapes[id as usize])?;
                        let mut v = src.char_shapes[id as usize].clone();
                        v.raw_data = None;
                        for (language, font) in v.font_ids.iter_mut().enumerate() {
                            *font = result.id16(Resource::Font(language, *font))?;
                        }
                        v.border_fill_id = result.border(v.border_fill_id)?;
                        intern(
                            &dst.char_shapes,
                            &mut result.delta.char_shapes,
                            v,
                            u16::MAX as usize,
                            &mut meter,
                            &mut result.counts,
                        )?
                    }
                    (3, Resource::Number(id)) => {
                        let original = &src.numberings[id as usize - 1];
                        if original.raw_para_heads.is_some() {
                            return Err(invalid(format!(
                                "Number({id}): opaque HWPX paraHead import is not supported"
                            )));
                        }
                        meter.charge(original)?;
                        let mut v = original.clone();
                        v.raw_data = None;
                        for h in &mut v.heads {
                            if h.char_shape_id != u32::MAX {
                                h.char_shape_id = result.id(Resource::Char(h.char_shape_id))?;
                            }
                        }
                        intern(
                            &dst.numberings,
                            &mut result.delta.numberings,
                            v,
                            u16::MAX as usize - 1,
                            &mut meter,
                            &mut result.counts,
                        )? + 1
                    }
                    (3, Resource::Bullet(id)) => {
                        let original = &src.bullets[id as usize - 1];
                        if original.raw_para_head.is_some() {
                            return Err(invalid(format!(
                                "Bullet({id}): opaque HWPX paraHead import is not supported"
                            )));
                        }
                        meter.charge(original)?;
                        let mut v = original.clone();
                        v.raw_data = None;
                        if v.char_shape_id != u32::MAX {
                            v.char_shape_id = result.id(Resource::Char(v.char_shape_id))?;
                        }
                        intern(
                            &dst.bullets,
                            &mut result.delta.bullets,
                            v,
                            u16::MAX as usize - 1,
                            &mut meter,
                            &mut result.counts,
                        )? + 1
                    }
                    (4, Resource::Para(id)) => {
                        meter.charge(&src.para_shapes[id as usize])?;
                        let mut v = src.para_shapes[id as usize].clone();
                        v.raw_data = None;
                        v.tab_def_id = result.id16(Resource::Tab(v.tab_def_id))?;
                        v.border_fill_id = result.border(v.border_fill_id)?;
                        let old = if v.head_type == HeadType::Outline && v.numbering_id == 0 {
                            outline
                        } else {
                            v.numbering_id
                        };
                        if old != 0 {
                            v.numbering_id = match v.head_type {
                                HeadType::Number | HeadType::Outline => {
                                    result.id16(Resource::Number(old))?
                                }
                                HeadType::Bullet => result.id16(Resource::Bullet(old))?,
                                HeadType::None => v.numbering_id,
                            };
                        }
                        intern(
                            &dst.para_shapes,
                            &mut result.delta.para_shapes,
                            v,
                            u16::MAX as usize,
                            &mut meter,
                            &mut result.counts,
                        )?
                    }
                    _ => continue,
                };
                result.map.insert(resource, index);
            }
        }
        styles::prepare(
            &mut result,
            &source.doc_info,
            &target.doc_info,
            reachable,
            &mut meter,
        )?;
        result.counts.metadata_bytes = limits.max_resource_metadata_bytes - meter.left;
        Ok(result)
    }

    /// Reserve on target before any semantic mutation; capacity changes are not document edits.
    pub fn reserve(&self, target: &mut Document) -> Result<(), HwpError> {
        let d = &mut target.doc_info;
        let s = &self.delta;
        macro_rules! reserve { ($($field:ident),*) => { $(d.$field.try_reserve(s.$field.len()).map_err(|_| invalid("resource destination allocation failed"))?;)* }; }
        reserve!(
            bin_data_list,
            border_fills,
            char_shapes,
            tab_defs,
            numberings,
            bullets,
            para_shapes,
            styles
        );
        d.font_faces
            .try_reserve(s.font_faces.len().saturating_sub(d.font_faces.len()))
            .map_err(|_| invalid("font slot allocation failed"))?;
        for (dst, src) in d.font_faces.iter_mut().zip(&s.font_faces) {
            dst.try_reserve(src.len())
                .map_err(|_| invalid("font allocation failed"))?;
        }
        target
            .bin_data_content
            .try_reserve(self.binaries.len())
            .map_err(|_| invalid("binary destination allocation failed"))?;
        Ok(())
    }

    pub fn commit(self, target: &mut Document) {
        let d = &mut target.doc_info;
        let s = self.delta;
        macro_rules! append { ($($field:ident),*) => { $(d.$field.extend(s.$field);)* }; }
        append!(
            bin_data_list,
            border_fills,
            char_shapes,
            tab_defs,
            numberings,
            bullets,
            para_shapes,
            styles
        );
        for (language, fonts) in s.font_faces.into_iter().enumerate() {
            if let Some(dst) = d.font_faces.get_mut(language) {
                dst.extend(fonts);
            } else {
                d.font_faces.push(fonts);
            }
        }
        d.bullet_count = d.bullets.len() as u32;
        d.raw_stream_dirty = true;
        target.bin_data_content.extend(self.binaries);
    }
}
