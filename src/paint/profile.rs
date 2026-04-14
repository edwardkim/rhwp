/// Layer builder/profile 힌트
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderProfile {
    FastPreview,
    #[default]
    Screen,
    Print,
    HighQuality,
}
