/// 원본을 보존하면서 이관 이슈가 지정한 렌더링 축의 활성 입력 목록을 분리한다.
/// parser·serializer 및 이관을 철회한 정상 회귀에는 적용하지 않는다.
pub fn is_deferred(path: &std::path::Path, axis: &str) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let root_prefix = format!("{}/", env!("CARGO_MANIFEST_DIR").replace('\\', "/"));
    let relative = normalized.strip_prefix(&root_prefix).unwrap_or(&normalized);
    include_str!("../fixtures/deferred_visual_regression_inputs.tsv")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .any(|line| {
            let mut columns = line.split('\t');
            columns.next().is_some_and(|entry| {
                relative == entry || normalized.ends_with(&format!("/{entry}"))
            }) && columns
                .next()
                .is_some_and(|axes| axes.split(',').any(|value| value == axis))
        })
}
