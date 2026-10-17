// 시각 미달 이슈로 옮긴 원본도 비시각 형식·IR 보존 계약에서는 계속 검사한다.
// 이 목록은 렌더링/쪽수 회귀의 수집 경로에 사용하지 않는다.

use std::path::{Path, PathBuf};

const PRESERVED_HWP5_INPUTS: &[(&str, &str)] = &[
    (
        "hwpctl_API_v2.4.hwp",
        "mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp",
    ),
    (
        "issue5679/10857_delegation_rules.hwp",
        "mydocs/pr/assets/issue7700/delegation-rules/10857_delegation_rules.hwp",
    ),
    (
        "task2287/1342000_edu_curriculum_map.hwp",
        "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp",
    ),
];

pub(crate) fn hwp5_path(logical_name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    PRESERVED_HWP5_INPUTS
        .iter()
        .find_map(|(name, preserved)| (*name == logical_name).then(|| root.join(preserved)))
        .unwrap_or_else(|| root.join("samples").join(logical_name))
}

pub(crate) fn append_preserved_hwp5_inputs(inputs: &mut Vec<(PathBuf, String)>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for &(logical_name, _) in PRESERVED_HWP5_INPUTS {
        let path = hwp5_path(logical_name);
        assert!(path.is_file(), "보존한 형식 검사 원본이 없다: {logical_name}");
        assert!(
            !root.join("samples").join(logical_name).exists(),
            "형식 원본이 활성/보존 경로에 중복 등록됐다: {logical_name}"
        );
        assert!(
            inputs.iter().all(|(_, name)| name != logical_name),
            "형식 원본이 수집 목록에 중복됐다: {logical_name}"
        );
        inputs.push((path, logical_name.to_owned()));
    }
}
