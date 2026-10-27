// 사용자가 다른 PR에서 회귀 검사를 다시 추가할 문서의 공통 수집 경계.

pub fn is_deferred_document(path: &std::path::Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    include_str!("../fixtures/deferred_regression_documents.txt")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .any(|deferred| match deferred.strip_suffix('*') {
            Some(prefix) => name.starts_with(prefix),
            None => name == deferred,
        })
}
