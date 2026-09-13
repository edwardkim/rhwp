//! next_style_id is an edge, not an attribute used before mapping its graph.
use super::{invalid, Meter, Resource, Resources};
use crate::{
    error::HwpError,
    model::{document::DocInfo, style::Style},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn prepare(
    result: &mut Resources,
    source: &DocInfo,
    target: &DocInfo,
    reachable: &BTreeSet<Resource>,
    meter: &mut Meter,
) -> Result<(), HwpError> {
    let mut source_styles = BTreeMap::new();
    for &resource in reachable {
        let Resource::Style(id) = resource else {
            continue;
        };
        meter.charge(&source.styles[id as usize])?;
        let mut style = source.styles[id as usize].clone();
        style.raw_data = None;
        style.para_shape_id = result.id16(Resource::Para(style.para_shape_id))?;
        style.char_shape_id = result.id16(Resource::Char(u32::from(style.char_shape_id)))?;
        source_styles.insert(id, style);
    }
    for &root in source_styles.keys() {
        if result.map.contains_key(&Resource::Style(root)) {
            continue;
        }
        let mut found = None;
        for candidate in 0..target.styles.len() + result.delta.styles.len() {
            let candidate =
                u8::try_from(candidate).map_err(|_| invalid("style space exceeds u8"))?;
            if let Some(bindings) =
                matching_chain(root, candidate, &source_styles, target, result, meter)?
            {
                found = Some(bindings);
                break;
            }
        }
        if let Some(bindings) = found {
            for (old, new) in bindings {
                if result
                    .map
                    .insert(Resource::Style(old), u32::from(new))
                    .is_none()
                {
                    result.counts.reused += 1;
                }
            }
            continue;
        }
        // Reserve all unresolved nodes of this chain, including a cycle, before connecting any edge.
        let mut chain = Vec::new();
        let mut old = root;
        while !result.map.contains_key(&Resource::Style(old)) {
            meter.tick()?;
            let new = u8::try_from(target.styles.len() + result.delta.styles.len() + chain.len())
                .map_err(|_| invalid("style ID space exhausted"))?;
            result.map.insert(Resource::Style(old), u32::from(new));
            chain.push(old);
            old = source_styles[&old].next_style_id;
        }
        for old in chain {
            let mut style = source_styles[&old].clone();
            style.next_style_id = u8::try_from(result.id(Resource::Style(style.next_style_id))?)
                .map_err(|_| invalid("next style exceeds u8"))?;
            result.delta.styles.push(style);
            result.counts.added += 1;
        }
    }
    Ok(())
}

fn matching_chain(
    mut old: u8,
    mut new: u8,
    source: &BTreeMap<u8, Style>,
    target: &DocInfo,
    result: &Resources,
    meter: &mut Meter,
) -> Result<Option<BTreeMap<u8, u8>>, HwpError> {
    let mut bindings = BTreeMap::new();
    loop {
        meter.tick()?;
        if let Some(mapped) = result.map.get(&Resource::Style(old)) {
            return Ok((*mapped == u32::from(new)).then_some(bindings));
        }
        if let Some(mapped) = bindings.get(&old) {
            return Ok((*mapped == new).then_some(bindings));
        }
        let a = &source[&old];
        let b = target.styles.get(new as usize).or_else(|| {
            (new as usize)
                .checked_sub(target.styles.len())
                .and_then(|i| result.delta.styles.get(i))
        });
        let Some(b) = b else {
            return Ok(None);
        };
        let mut ka = meter.key(a)?;
        let mut kb = meter.key(b)?;
        ka.as_object_mut()
            .expect("Style object")
            .remove("next_style_id");
        kb.as_object_mut()
            .expect("Style object")
            .remove("next_style_id");
        if ka != kb {
            return Ok(None);
        }
        bindings.insert(old, new);
        old = a.next_style_id;
        new = b.next_style_id;
    }
}
