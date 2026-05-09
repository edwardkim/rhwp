pub const RESOURCE_KEY_ALGORITHM: &str = "blake3";

/// 추후 이미지/패스/셰이프 캐시 공유를 위한 arena placeholder.
///
/// 레이어드 렌더러 전환 1차에서는 leaf payload를 직접 보관하되,
/// IR 상에 arena 자리를 먼저 확보해 이후 Skia/CanvasKit 자원 공유로 확장한다.
#[derive(Debug, Clone, Default)]
pub struct ResourceArena;

pub fn resource_digest_hex(bytes: impl AsRef<[u8]>) -> String {
    blake3::hash(bytes.as_ref()).to_hex().to_string()
}

pub fn image_resource_key(byte_len: usize, digest: &str) -> String {
    resource_key("img", byte_len, digest)
}

pub fn svg_resource_key(byte_len: usize, digest: &str) -> String {
    resource_key("svg", byte_len, digest)
}

fn resource_key(kind: &str, byte_len: usize, digest: &str) -> String {
    format!("{kind}:{RESOURCE_KEY_ALGORITHM}:{byte_len}:{digest}")
}

#[cfg(test)]
mod tests {
    use super::{image_resource_key, resource_digest_hex, svg_resource_key};

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
    }
}
