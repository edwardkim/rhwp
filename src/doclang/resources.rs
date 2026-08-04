//! Resource URI and asset handling shared by exporters.
//!
//! The parser keeps picture bytes in the Semantic IR. Exporters decide whether
//! those bytes are embedded as data URIs, returned as writable assets, or
//! referenced through a caller-provided URI prefix.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;

use crate::doclang::ir::block::{Block, ListItem};
use crate::doclang::ir::table::Table;
use crate::doclang::ir::SirDocument;

/// How binary resources such as pictures are referenced by exporters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ResourcePolicy {
    /// Embed bytes directly in the output as a `data:*;base64,...` URI.
    #[default]
    Inline,
    /// Return the bytes as assets and reference them by `uri_prefix/name.ext`.
    AssetDir { uri_prefix: String },
    /// Reference assets by `uri_prefix/name.ext` without returning bytes.
    UriPrefix { uri_prefix: String },
}

impl ResourcePolicy {
    /// Inline binary resources as base64 data URIs.
    pub fn inline() -> Self {
        Self::Inline
    }

    /// Emit asset references under a path/URI prefix and return bytes to write.
    pub fn asset_dir(uri_prefix: impl Into<String>) -> Self {
        Self::AssetDir {
            uri_prefix: uri_prefix.into(),
        }
    }

    /// Emit URI references under a prefix without returning asset bytes.
    pub fn uri_prefix(uri_prefix: impl Into<String>) -> Self {
        Self::UriPrefix {
            uri_prefix: uri_prefix.into(),
        }
    }
}

/// A binary asset returned by an exporter when [`ResourcePolicy::AssetDir`] is
/// selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAsset {
    /// Relative asset name, e.g. `section-0-block-3.png`.
    pub path: String,
    /// MIME type inferred from the original extension.
    pub mime: String,
    /// Raw bytes to write to `path`.
    pub data: Vec<u8>,
}

/// The URI emitted into an output plus any asset bytes the caller must persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    /// URI to place in XML, Markdown, or payload.
    pub uri: String,
    /// MIME type inferred from the source extension.
    pub mime: String,
    /// Asset bytes when the selected policy returns writable assets.
    pub asset: Option<ResourceAsset>,
}

/// Resolve one picture resource according to `policy`.
pub fn resolve_resource(
    policy: &ResourcePolicy,
    loc: &str,
    extension: &str,
    data: &[u8],
) -> ResourceRef {
    let mime = mime_for_extension(extension).to_string();
    match policy {
        ResourcePolicy::Inline => ResourceRef {
            uri: data_uri(&mime, data),
            mime,
            asset: None,
        },
        ResourcePolicy::AssetDir { uri_prefix } => {
            let path = asset_name(loc, extension);
            ResourceRef {
                uri: join_uri(uri_prefix, &path),
                mime: mime.clone(),
                asset: Some(ResourceAsset {
                    path,
                    mime,
                    data: data.to_vec(),
                }),
            }
        }
        ResourcePolicy::UriPrefix { uri_prefix } => {
            let path = asset_name(loc, extension);
            ResourceRef {
                uri: join_uri(uri_prefix, &path),
                mime,
                asset: None,
            }
        }
    }
}

/// Collect all writable assets referenced by `doc` under `policy`.
///
/// Inline and URI-prefix policies return an empty vector. Asset names follow
/// the same location scheme used by the XML and Markdown exporters.
pub fn collect_assets(doc: &SirDocument, policy: &ResourcePolicy) -> Vec<ResourceAsset> {
    let mut assets = Vec::new();
    for (si, section) in doc.sections.iter().enumerate() {
        for (bi, block) in section.blocks.iter().enumerate() {
            let loc = format!("section[{si}]/block[{bi}]");
            collect_block_assets(block, &loc, policy, &mut assets);
        }
    }
    assets
}

/// Map a lower-case image file extension to a MIME type for data URIs.
pub fn mime_for_extension(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        _ => "application/octet-stream",
    }
}

/// Build a data URI for an in-memory resource.
pub fn data_uri(mime: &str, data: &[u8]) -> String {
    format!("data:{mime};base64,{}", BASE64.encode(data))
}

/// Build a deterministic filename from an IR location and extension.
pub fn asset_name(loc: &str, extension: &str) -> String {
    let mut stem = String::with_capacity(loc.len());
    for ch in loc.chars() {
        if ch.is_ascii_alphanumeric() {
            stem.push(ch.to_ascii_lowercase());
        } else if !stem.ends_with('-') {
            stem.push('-');
        }
    }
    let stem = stem.trim_matches('-');
    let stem = if stem.is_empty() { "asset" } else { stem };
    let ext = safe_extension(extension);
    format!("{stem}.{ext}")
}

fn safe_extension(extension: &str) -> String {
    let ext: String = extension
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect();
    if ext.is_empty() {
        "bin".to_string()
    } else {
        ext
    }
}

fn join_uri(prefix: &str, path: &str) -> String {
    let prefix = prefix.trim_end_matches('/');
    if prefix.is_empty() {
        path.to_string()
    } else {
        format!("{prefix}/{path}")
    }
}

fn collect_block_assets(
    block: &Block,
    loc: &str,
    policy: &ResourcePolicy,
    assets: &mut Vec<ResourceAsset>,
) {
    match block {
        Block::Picture {
            data, extension, ..
        } => {
            if let Some(asset) = resolve_resource(policy, loc, extension, data).asset {
                assets.push(asset);
            }
        }
        Block::List { items, .. } => {
            for (ii, item) in items.iter().enumerate() {
                let item_loc = format!("{loc}/ldiv[{ii}]");
                collect_list_item_assets(item, &item_loc, policy, assets);
            }
        }
        Block::Table(table) => collect_table_assets(table, loc, policy, assets),
        Block::Footnote { content, .. }
        | Block::PageHeader { content, .. }
        | Block::PageFooter { content, .. } => {
            collect_nested_assets(content, loc, policy, assets);
        }
        Block::Paragraph { .. }
        | Block::Heading { .. }
        | Block::Formula(_)
        | Block::PageBreak
        | Block::ThreadStart { .. }
        | Block::ThreadContinuation { .. }
        | Block::Custom { .. } => {}
    }
}

fn collect_list_item_assets(
    item: &ListItem,
    loc: &str,
    policy: &ResourcePolicy,
    assets: &mut Vec<ResourceAsset>,
) {
    collect_nested_assets(&item.content, loc, policy, assets);
}

fn collect_table_assets(
    table: &Table,
    loc: &str,
    policy: &ResourcePolicy,
    assets: &mut Vec<ResourceAsset>,
) {
    for (ci, cell) in table.cells.iter().enumerate() {
        let cell_loc = format!("{loc}/cell[{ci}]");
        collect_nested_assets(&cell.content, &cell_loc, policy, assets);
    }
}

fn collect_nested_assets(
    blocks: &[Block],
    loc: &str,
    policy: &ResourcePolicy,
    assets: &mut Vec<ResourceAsset>,
) {
    for (bi, block) in blocks.iter().enumerate() {
        let child_loc = format!("{loc}/block[{bi}]");
        collect_block_assets(block, &child_loc, policy, assets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_policy_builds_data_uri() {
        let r = resolve_resource(
            &ResourcePolicy::Inline,
            "section[0]/block[1]",
            "png",
            b"abc",
        );
        assert_eq!(r.mime, "image/png");
        assert!(r.uri.starts_with("data:image/png;base64,"));
        assert!(r.asset.is_none());
    }

    #[test]
    fn asset_dir_policy_returns_writable_asset() {
        let r = resolve_resource(
            &ResourcePolicy::asset_dir("assets"),
            "section[0]/block[1]",
            "JPG",
            b"abc",
        );
        assert_eq!(r.uri, "assets/section-0-block-1.jpg");
        let asset = r.asset.unwrap();
        assert_eq!(asset.path, "section-0-block-1.jpg");
        assert_eq!(asset.mime, "image/jpeg");
        assert_eq!(asset.data, b"abc");
    }

    #[test]
    fn uri_prefix_policy_does_not_return_bytes() {
        let r = resolve_resource(
            &ResourcePolicy::uri_prefix("https://cdn.example/images/"),
            "section[0]/block[1]",
            "gif",
            b"abc",
        );
        assert_eq!(r.uri, "https://cdn.example/images/section-0-block-1.gif");
        assert!(r.asset.is_none());
    }

    #[test]
    fn collect_assets_uses_exporter_locations() {
        let doc = SirDocument {
            doclang_version: "0.6",
            sections: vec![crate::doclang::ir::Section {
                blocks: vec![Block::Picture {
                    data: b"abc".to_vec(),
                    extension: "png".into(),
                    geometry: None,
                    lost: None,
                    prov: None,
                }],
            }],
        };
        let assets = collect_assets(&doc, &ResourcePolicy::asset_dir("assets"));
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].path, "section-0-block-0.png");
        assert_eq!(assets[0].data, b"abc");
    }

    #[test]
    fn asset_name_is_safe_and_deterministic() {
        assert_eq!(
            asset_name("section[0]/block[12]/cell[3]", "P.N.G"),
            "section-0-block-12-cell-3.png"
        );
        assert_eq!(asset_name("!!!", ""), "asset.bin");
    }
}
