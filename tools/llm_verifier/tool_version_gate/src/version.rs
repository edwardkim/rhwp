//! 영수증 `toolVersion` 과 검증기 바이너리 버전의 식별.

/// 도구 버전 한 값. 호환 범위가 아니라 식별 문자열이다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolVersion {
    raw: String,
    identity: String,
}

impl ToolVersion {
    pub fn parse(raw: impl AsRef<str>) -> Self {
        let raw = raw.as_ref().to_string();
        let identity = raw.trim().to_string();
        Self { raw, identity }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn is_empty(&self) -> bool {
        self.identity.is_empty()
    }

    /// trim 후 바이트가 같으면 같은 바이너리로 본다. semver 범위가 아니다.
    pub fn same_identity(&self, other: &Self) -> bool {
        !self.is_empty() && !other.is_empty() && self.identity == other.identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_only_identity() {
        let a = ToolVersion::parse("0.8.4");
        let b = ToolVersion::parse("  0.8.4\t");
        assert!(a.same_identity(&b));
    }

    #[test]
    fn build_meta_is_different_binary() {
        let a = ToolVersion::parse("0.8.4");
        let b = ToolVersion::parse("0.8.4+git.abc");
        assert!(!a.same_identity(&b));
    }

    #[test]
    fn v_prefix_is_different_binary() {
        let a = ToolVersion::parse("0.8.4");
        let b = ToolVersion::parse("v0.8.4");
        assert!(!a.same_identity(&b));
    }

    #[test]
    fn empty_after_trim_is_empty() {
        assert!(ToolVersion::parse("   ").is_empty());
        assert!(ToolVersion::parse("").is_empty());
    }
}
