//! rhwp 가 저장한 HWP5 본문의 `PARA_LINE_SEG` tag bit 31 의미 표지(#8939).
//!
//! 사양에서 bit 31 은 「구현 편의 property」다. 한/글은 실제 저장 줄에도 이 비트를 싣고,
//! rhwp 는 같은 비트(`LineSeg::TAG_IMPLEMENTATION_PROPERTY`)를 자기가 계산한 합성 줄의
//! 표지로 IR 과 저장 파일에 쓴다. 비트 값만으로는 두 저장 주체를 구별할 수 없으므로
//! rhwp 는 자기 의미로 쓴 구역마다 그 BodyText 레코드 바이트(압축·암호 전)의
//! SHA-256 을 이 보조 스트림에 남긴다.
//!
//! 파서는 구역 바이트의 해시가 이 표지에 있을 때만 bit 31 을 rhwp 합성 표지로 읽는다.
//! 표지가 없거나(한/글 저장본), 다른 프로그램이 본문을 다시 써서 해시가 맞지 않으면
//! bit 31 은 저장 줄의 한/글 속성이며 IR 의 합성 표지가 아니다.

use sha2::{Digest, Sha256};

pub(crate) const HWP_STREAM: &str = "/RhwpLineSegTags";
pub(crate) const MAX_BYTES: usize = 64 * 1024;
const MAGIC: &[u8; 8] = b"RHWPLT01";

/// 구역 레코드 바이트의 다이제스트.
pub(crate) fn section_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// rhwp 의미로 쓴 구역 다이제스트 목록을 표지 스트림으로 만든다.
pub(crate) fn encode(digests: &[[u8; 32]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(MAGIC.len() + digests.len() * 32);
    out.extend_from_slice(MAGIC);
    for digest in digests {
        out.extend_from_slice(digest);
    }
    out
}

/// 표지 스트림을 읽는다. 형식이 맞지 않으면 표지가 없는 것으로 본다.
pub(crate) fn decode(bytes: &[u8]) -> Vec<[u8; 32]> {
    let Some(body) = bytes.strip_prefix(MAGIC.as_slice()) else {
        return Vec::new();
    };
    if body.len() % 32 != 0 {
        return Vec::new();
    }
    body.chunks_exact(32)
        .map(|chunk| chunk.try_into().expect("32바이트 조각"))
        .collect()
}

/// 보조 스트림의 이름만으로 현재 구역의 캐시를 rhwp 관리 대상으로 읽지 않는다.
/// 파서와 같은 실제 BodyText 바이트의 인증을 할당 없이 확인한다.
pub(crate) fn authenticates_section(bytes: &[u8], section: &[u8]) -> bool {
    let Some(body) = bytes.strip_prefix(MAGIC) else {
        return false;
    };
    if bytes.len() > MAX_BYTES || body.len() % 32 != 0 {
        return false;
    }
    let digest = section_digest(section);
    body.chunks_exact(32).any(|entry| entry == digest)
}
