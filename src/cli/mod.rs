//! rhwp 프로세스 어댑터의 명령 표면.
//!
//! #5511은 handler를 이동하기 전에 실제 dispatch와 외부 자기서술의 관계부터
//! 고정한다. 하위 모듈은 application/service 계층이 아니며 도메인 로직을 소유하지
//! 않는다.

pub(crate) mod batch;
pub(crate) mod catalog;
pub(crate) mod commands;
pub(crate) mod integrity;
pub(crate) mod metadata;
pub(crate) mod outputs;
pub(crate) mod protocol;
pub(crate) mod queries;

/// `--compat 2022|2024` 값을 `hangul2024_compat` 세션 설정으로 옮긴다.
///
/// 축이 4세대가 아니라 이분인 것은 실측 결과다 — 한글 2018·2020·2022 는 사실상 같은
/// 조판 엔진이고 2024 만 갈린다(3자 대조에서 2020↔2022 5건 vs 2020↔2024 258건,
/// `mydocs/report/hangul_version_oracle_r1_20260807.md` 8절).
///
/// 값은 세션 설정이며 파서가 확정하는 provenance 가 아니다. 저장 버전(`lastSavedWith`)
/// 으로 자동 선택하지 않는다 — 저장 버전은 "이 문서가 2024 규칙을 필요로 하는가"를
/// 예측하지 못한다(전수 실측: 두 버전이 다르게 조판하는 254건 중 2024 저장은 0건).
pub(crate) fn parse_compat_generation(value: &str) -> Option<bool> {
    match value {
        "2022" => Some(false),
        "2024" => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_compat_generation;

    #[test]
    fn compat_generation_accepts_only_measured_axis() {
        assert_eq!(parse_compat_generation("2022"), Some(false));
        assert_eq!(parse_compat_generation("2024"), Some(true));
        // 2018·2020 은 2022 와 같은 엔진이라 별도 세대를 만들지 않는다.
        assert_eq!(parse_compat_generation("2020"), None);
        assert_eq!(parse_compat_generation("2018"), None);
        assert_eq!(parse_compat_generation(""), None);
    }
}
