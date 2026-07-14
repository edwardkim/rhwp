use crate::parser::HmlImportMetadata;

use super::xml::XmlWriter;

pub(crate) fn write_at_anchor(
    writer: &mut XmlWriter,
    metadata: &HmlImportMetadata,
    parent: &str,
    anchor: usize,
) {
    let mut fragments = metadata
        .preserved_fragments
        .iter()
        .filter(|fragment| fragment.parent == parent && fragment.modeled_siblings_before == anchor)
        .collect::<Vec<_>>();
    fragments.sort_by_key(|fragment| fragment.order);
    for fragment in fragments {
        writer.raw(&fragment.raw_xml);
    }
}
