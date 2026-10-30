/// Layer builder/profile 힌트
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderProfile {
    FastPreview,
    #[default]
    Screen,
    Print,
    HighQuality,
}

impl RenderProfile {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "fastPreview" | "fast-preview" => Some(Self::FastPreview),
            "screen" => Some(Self::Screen),
            "print" => Some(Self::Print),
            "highQuality" | "high-quality" => Some(Self::HighQuality),
            _ => None,
        }
    }

    pub fn shows_editor_visuals(self) -> bool {
        matches!(self, Self::FastPreview | Self::Screen)
    }
}

#[cfg(test)]
mod tests {
    use super::RenderProfile;

    #[test]
    fn parses_api_and_cli_profile_names() {
        assert_eq!(
            RenderProfile::parse("fastPreview"),
            Some(RenderProfile::FastPreview)
        );
        assert_eq!(
            RenderProfile::parse("fast-preview"),
            Some(RenderProfile::FastPreview)
        );
        assert_eq!(RenderProfile::parse("screen"), Some(RenderProfile::Screen));
        assert_eq!(RenderProfile::parse("print"), Some(RenderProfile::Print));
        assert_eq!(
            RenderProfile::parse("highQuality"),
            Some(RenderProfile::HighQuality)
        );
        assert_eq!(
            RenderProfile::parse("high-quality"),
            Some(RenderProfile::HighQuality)
        );
        assert_eq!(RenderProfile::parse(""), None);
        assert_eq!(RenderProfile::parse("unknown"), None);
    }

    #[test]
    fn editor_visuals_are_limited_to_interactive_profiles() {
        assert!(RenderProfile::FastPreview.shows_editor_visuals());
        assert!(RenderProfile::Screen.shows_editor_visuals());
        assert!(!RenderProfile::Print.shows_editor_visuals());
        assert!(!RenderProfile::HighQuality.shows_editor_visuals());
    }
}
