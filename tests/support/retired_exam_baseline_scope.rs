/// 사용자 승인으로 제거한 시험지의 기존 원장 행이 자동 수집으로 되살아나지 않게 한다.
/// 새 이름의 관계형 회귀 검사와 독립 PDF 시각 검증에는 적용하지 않는다.
pub fn is_retired(path: &std::path::Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let root_prefix = format!("{}/", env!("CARGO_MANIFEST_DIR").replace('\\', "/"));
    let relative = normalized.strip_prefix(&root_prefix).unwrap_or(&normalized);
    include_str!("../fixtures/retired_exam_baseline_inputs.txt")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .any(|line| line == relative)
}
