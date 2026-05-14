use std::collections::HashMap;

use crate::paint::font::FontResourceTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageResourceId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SvgResourceId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontBlobResourceId(pub usize);

pub const RESOURCE_KEY_ALGORITHM: &str = "blake3";

/// 레이어 replay가 공유하는 바이너리/문자열 자원 저장소.
///
/// 현재 leaf payload는 대부분 직접 보관하지만, P12부터 font blob/face
/// identity는 glyph replay contract에서 참조할 수 있게 한다.
#[derive(Debug, Clone, Default)]
pub struct ResourceArena {
    font_blob_bytes: Vec<Vec<u8>>,
    font_blob_hashes: Vec<u64>,
    font_blob_fingerprints: Vec<[u8; 16]>,
    font_blob_lookup: HashMap<u64, Vec<FontBlobResourceId>>,
    font_resources: FontResourceTable,
}

impl ResourceArena {
    pub fn intern_font_blob_bytes(&mut self, bytes: &[u8]) -> FontBlobResourceId {
        let hash = resource_hash(bytes);
        if let Some(candidates) = self.font_blob_lookup.get(&hash) {
            for id in candidates {
                if self.font_blob_bytes[id.0].as_slice() == bytes {
                    return *id;
                }
            }
        }

        let id = FontBlobResourceId(self.font_blob_bytes.len());
        self.font_blob_bytes.push(bytes.to_vec());
        self.font_blob_hashes.push(hash);
        self.font_blob_fingerprints
            .push(resource_fingerprint(bytes));
        self.font_blob_lookup.entry(hash).or_default().push(id);
        id
    }

    pub fn font_blob_bytes(&self, id: FontBlobResourceId) -> Option<&[u8]> {
        self.font_blob_bytes.get(id.0).map(Vec::as_slice)
    }

    pub fn font_blob_count(&self) -> usize {
        self.font_blob_bytes.len()
    }

    pub fn font_blob_hash(&self, id: FontBlobResourceId) -> Option<u64> {
        self.font_blob_hashes.get(id.0).copied()
    }

    pub fn font_blob_fingerprint(&self, id: FontBlobResourceId) -> Option<[u8; 16]> {
        self.font_blob_fingerprints.get(id.0).copied()
    }

    pub fn font_blob_resources(&self) -> impl Iterator<Item = (FontBlobResourceId, &[u8])> + '_ {
        self.font_blob_bytes
            .iter()
            .enumerate()
            .map(|(index, bytes)| (FontBlobResourceId(index), bytes.as_slice()))
    }

    pub fn font_resources(&self) -> &FontResourceTable {
        &self.font_resources
    }

    pub fn font_resources_mut(&mut self) -> &mut FontResourceTable {
        &mut self.font_resources
    }
}

fn resource_hash(bytes: impl AsRef<[u8]>) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes.as_ref() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn resource_fingerprint(bytes: impl AsRef<[u8]>) -> [u8; 16] {
    let digest = blake3::hash(bytes.as_ref());
    let mut fingerprint = [0; 16];
    fingerprint.copy_from_slice(&digest.as_bytes()[..16]);
    fingerprint
}

pub fn resource_digest_hex(bytes: impl AsRef<[u8]>) -> String {
    blake3::hash(bytes.as_ref()).to_hex().to_string()
}

pub fn image_resource_key(byte_len: usize, digest: &str) -> String {
    resource_key("img", byte_len, digest)
}

pub fn svg_resource_key(byte_len: usize, digest: &str) -> String {
    resource_key("svg", byte_len, digest)
}

pub fn font_blob_resource_key(byte_len: usize, digest: &str) -> String {
    resource_key("font", byte_len, digest)
}

fn resource_key(kind: &str, byte_len: usize, digest: &str) -> String {
    format!("{kind}:{RESOURCE_KEY_ALGORITHM}:{byte_len}:{digest}")
}

#[cfg(test)]
mod tests {
    use super::{
        font_blob_resource_key, image_resource_key, resource_digest_hex, resource_fingerprint,
        svg_resource_key, FontBlobResourceId, ResourceArena,
    };

    #[test]
    fn interns_duplicate_font_blobs_once() {
        let mut arena = ResourceArena::default();
        let font_a = arena.intern_font_blob_bytes(&[5, 6, 7, 8]);
        let font_b = arena.intern_font_blob_bytes(&[5, 6, 7, 8]);

        assert_eq!(font_a, FontBlobResourceId(0));
        assert_eq!(font_b, FontBlobResourceId(0));
        assert_eq!(arena.font_blob_count(), 1);
        assert_eq!(arena.font_blob_bytes(font_a), Some(&[5, 6, 7, 8][..]));
        assert_eq!(arena.font_blob_hash(font_a), arena.font_blob_hash(font_b));
        assert!(arena.font_blob_hash(font_a).is_some());
        assert_eq!(
            arena.font_blob_fingerprint(font_a),
            Some(resource_fingerprint([5, 6, 7, 8]))
        );
        assert_eq!(
            arena.font_blob_resources().collect::<Vec<_>>(),
            vec![(FontBlobResourceId(0), &[5, 6, 7, 8][..])]
        );
    }

    #[test]
    fn resource_digest_is_stable_and_content_dependent() {
        let digest = resource_digest_hex([1, 2, 3, 4]);
        assert_eq!(digest.len(), 64);
        assert_eq!(digest, resource_digest_hex([1, 2, 3, 4]));
        assert_ne!(digest, resource_digest_hex([1, 2, 3, 5]));
    }

    #[test]
    fn resource_keys_include_kind_algorithm_length_and_digest() {
        assert_eq!(image_resource_key(4, "abcd"), "img:blake3:4:abcd");
        assert_eq!(
            svg_resource_key(6, "0123456789abcdef"),
            "svg:blake3:6:0123456789abcdef"
        );
        assert_eq!(font_blob_resource_key(8, "feed"), "font:blake3:8:feed");
    }
}
