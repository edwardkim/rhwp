/// 추후 이미지/패스/셰이프 캐시 공유를 위한 arena placeholder.
///
/// 레이어드 렌더러 전환 1차에서는 leaf payload를 직접 보관하되,
/// IR 상에 arena 자리를 먼저 확보해 이후 Skia/CanvasKit 자원 공유로 확장한다.
#[derive(Debug, Clone, Default)]
pub struct ResourceArena;
