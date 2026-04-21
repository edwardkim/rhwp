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
    pub fn parse_name(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "fast" | "fast-preview" | "fastpreview" | "preview" => Some(Self::FastPreview),
            "screen" => Some(Self::Screen),
            "print" => Some(Self::Print),
            "high" | "high-quality" | "highquality" | "quality" => Some(Self::HighQuality),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FastPreview => "fast-preview",
            Self::Screen => "screen",
            Self::Print => "print",
            Self::HighQuality => "high-quality",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RenderProfile;

    #[test]
    fn parses_profile_aliases() {
        assert_eq!(
            RenderProfile::parse_name("fast-preview"),
            Some(RenderProfile::FastPreview)
        );
        assert_eq!(
            RenderProfile::parse_name("SCREEN"),
            Some(RenderProfile::Screen)
        );
        assert_eq!(RenderProfile::parse_name("print"), Some(RenderProfile::Print));
        assert_eq!(
            RenderProfile::parse_name("highquality"),
            Some(RenderProfile::HighQuality)
        );
        assert_eq!(RenderProfile::parse_name("unknown"), None);
    }

    #[test]
    fn reports_stable_profile_names() {
        assert_eq!(RenderProfile::FastPreview.as_str(), "fast-preview");
        assert_eq!(RenderProfile::Screen.as_str(), "screen");
        assert_eq!(RenderProfile::Print.as_str(), "print");
        assert_eq!(RenderProfile::HighQuality.as_str(), "high-quality");
    }
}
