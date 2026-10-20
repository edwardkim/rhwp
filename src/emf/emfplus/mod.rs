//! 표준 EMF+의 지원 가능한 벡터 스트림만 완전하게 재생한다.
//! 일부 레코드를 버려 만든 출력을 성공으로 반환하지 않는다.

mod bitmap;
mod objects;
mod player;
mod stream;

pub(crate) fn convert_supported(bytes: &[u8], viewport: (f64, f64)) -> Option<Vec<u8>> {
    let mut budget = player::Budget::default();
    player::render(bytes, (0.0, 0.0, viewport.0, viewport.1), 0, &mut budget)
        .map(String::into_bytes)
}
