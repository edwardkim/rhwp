use super::{invalid, Meter, Resource, Resources};
use crate::{
    error::HwpError,
    model::{
        bin_data::{BinData, BinDataContent, BinDataType},
        document::Document,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(super) fn prepare(
    result: &mut Resources,
    source: &Document,
    target: &Document,
    reachable: &BTreeSet<Resource>,
    meter: &mut Meter,
    byte_limit: usize,
) -> Result<(), HwpError> {
    let mut pool = Pool {
        source,
        target,
        meter,
        remaining: byte_limit,
        source_map: BTreeMap::new(),
        target_bytes: BTreeMap::new(),
    };
    for &resource in reachable {
        let source_index = match resource {
            Resource::Image(id) => source
                .bin_data_content
                .get(id as usize - 1)
                .map(|_| id as usize - 1)
                .or_else(|| source.bin_data_content.iter().position(|c| c.id == id)),
            Resource::Storage(id) => source.bin_data_content.iter().position(|c| c.id == id),
            _ => continue,
        }
        .ok_or_else(|| invalid(format!("missing {resource:?}")))?;
        let (ordinal, storage) = pool.import(source_index, result)?;
        result.map.insert(
            resource,
            u32::from(if matches!(resource, Resource::Storage(_)) {
                storage
            } else {
                ordinal
            }),
        );
    }
    result.counts.binary_bytes_read = byte_limit - pool.remaining;
    Ok(())
}

struct Pool<'a> {
    source: &'a Document,
    target: &'a Document,
    meter: &'a mut Meter,
    remaining: usize,
    source_map: BTreeMap<usize, (u16, u16)>,
    target_bytes: BTreeMap<usize, Arc<[u8]>>,
}

impl Pool<'_> {
    fn record(
        &mut self,
        document: &Document,
        content: &BinDataContent,
    ) -> Result<BinData, HwpError> {
        // DocInfo contains storage IDs; image references are a different namespace.
        let mut found = None;
        for record in &document.doc_info.bin_data_list {
            self.meter.tick()?;
            if record.storage_id == content.id {
                if found.is_some() {
                    return Err(invalid("ambiguous BinData storage metadata"));
                }
                found = Some(record);
            }
        }
        let record =
            found.ok_or_else(|| invalid(format!("missing metadata for storage {}", content.id)))?;
        if record.data_type == BinDataType::Link
            || record.abs_path.is_some()
            || record.rel_path.is_some()
        {
            return Err(invalid("external BinData links are not imported"));
        }
        self.meter.charge(record)?;
        let mut value = record.clone();
        value.raw_data = None;
        value.storage_id = 0; // only for semantic comparison, assigned before commit
        Ok(value)
    }

    fn load(&mut self, content: &BinDataContent) -> Result<Arc<[u8]>, HwpError> {
        let bytes = content.data.load_limited_shared(self.remaining)
            .ok_or_else(|| invalid(format!("storage {}: binary byte budget exceeded, unavailable bytes, or resolver lacks bounded reads", content.id)))?;
        self.remaining -= bytes.len();
        if bytes.is_empty() {
            return Err(invalid(format!(
                "storage {} has empty binary payload",
                content.id
            )));
        }
        Ok(bytes)
    }

    fn import(&mut self, index: usize, result: &mut Resources) -> Result<(u16, u16), HwpError> {
        if let Some(ids) = self.source_map.get(&index) {
            return Ok(*ids);
        }
        let content = &self.source.bin_data_content[index];
        self.meter.charge(&content.extension)?;
        let mut record = self.record(self.source, content)?;
        let key = self.meter.key(&record)?;
        let bytes = self.load(content)?;
        let mut matched = None;
        for (i, target_content) in self.target.bin_data_content.iter().enumerate() {
            self.meter.tick()?;
            // Renderer resolves image ordinals first, HWPX writer uses storage IDs.
            // Only reuse an entry whose reference has the same meaning on both paths.
            if usize::from(target_content.id) != i + 1 {
                continue;
            }
            if content.extension != target_content.extension {
                continue;
            }
            let candidate = self.record(self.target, target_content)?;
            if self.meter.key(&candidate)? != key {
                continue;
            }
            let existing = if let Some(existing) = self.target_bytes.get(&i) {
                existing.clone()
            } else {
                let existing = self.load(target_content)?;
                self.target_bytes.insert(i, existing.clone());
                existing
            };
            if existing == bytes {
                matched = Some((i, target_content.id));
                break;
            }
        }
        if matched.is_none() {
            for (i, candidate) in result.binaries.iter().enumerate() {
                self.meter.tick()?;
                let mut metadata = result.delta.bin_data_list[i].clone();
                metadata.storage_id = 0;
                if candidate.extension == content.extension
                    && self.meter.key(&metadata)? == key
                    && candidate.data.load_limited_shared(bytes.len()).as_deref()
                        == Some(bytes.as_ref())
                {
                    matched = Some((self.target.bin_data_content.len() + i, candidate.id));
                    break;
                }
            }
        }
        let ids = if let Some((i, storage)) = matched {
            result.counts.binaries_reused += 1;
            (
                u16::try_from(i + 1).map_err(|_| invalid("image ordinal exceeds u16"))?,
                storage,
            )
        } else {
            // Appending both lists preserves every old ordinal; do not repair or reorder target entries.
            if self.target.bin_data_content.len() != self.target.doc_info.bin_data_list.len() {
                return Err(invalid("target BinData lists are not aligned for append"));
            }
            let ordinal =
                u16::try_from(self.target.bin_data_content.len() + result.binaries.len() + 1)
                    .map_err(|_| invalid("image ordinal exceeds u16"))?;
            for id in self
                .target
                .bin_data_content
                .iter()
                .map(|c| c.id)
                .chain(
                    self.target
                        .doc_info
                        .bin_data_list
                        .iter()
                        .map(|b| b.storage_id),
                )
                .chain(result.binaries.iter().map(|c| c.id))
            {
                self.meter.tick()?;
                if id == ordinal {
                    return Err(invalid(
                        "new image ordinal collides with an existing storage ID",
                    ));
                }
            }
            let storage = ordinal;
            record.storage_id = storage;
            result.delta.bin_data_list.push(record);
            result.binaries.push(BinDataContent {
                id: storage,
                extension: content.extension.clone(),
                data: crate::model::bin_data::BinDataBytes::from_shared(bytes),
            });
            result.counts.binaries_added += 1;
            (ordinal, storage)
        };
        self.source_map.insert(index, ids);
        Ok(ids)
    }
}
