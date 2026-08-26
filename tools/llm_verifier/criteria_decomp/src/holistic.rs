//! 총점 채점이 원자 실패를 가리는지. 총점 자체는 내보내지 않는다.

/// 가림 임계: 통과 원자가 전체의 절반 이상이면 총점이 실패를 덮을 수 있다.
pub const HOLISTIC_HIDE_NUM: u64 = 1;
pub const HOLISTIC_HIDE_DEN: u64 = 2;

/// `atom_pass` 가 거짓이고, 묶음에서 통과 비율이 절반 이상이며,
/// 원자가 둘 이상일 때만 총점이 이 실패를 가린다.
pub fn holistic_would_hide(atom_pass: bool, bundle_pass_count: u64, bundle_total: u64) -> bool {
    if atom_pass || bundle_total < 2 {
        return false;
    }
    bundle_pass_count.saturating_mul(HOLISTIC_HIDE_DEN)
        >= bundle_total.saturating_mul(HOLISTIC_HIDE_NUM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passing_atom_is_never_hidden() {
        assert!(!holistic_would_hide(true, 5, 5));
        assert!(!holistic_would_hide(true, 4, 5));
    }

    #[test]
    fn majority_pass_hides_one_fail() {
        assert!(holistic_would_hide(false, 4, 5));
        assert!(holistic_would_hide(false, 2, 4));
    }

    #[test]
    fn clear_bundle_fail_is_not_hidden() {
        assert!(!holistic_would_hide(false, 1, 5));
        assert!(!holistic_would_hide(false, 0, 3));
    }

    #[test]
    fn singleton_bundle_cannot_hide() {
        assert!(!holistic_would_hide(false, 0, 1));
        assert!(!holistic_would_hide(false, 0, 0));
    }
}
