// 사용자가 별도 작업으로 이관한 문서는 공통 회귀 코퍼스에서도 검사하지 않는다.

use std::path::Path;

const TRANSFERRED_EXAMS: &str = include_str!("../fixtures/transferred_exam_regressions.txt");

pub(crate) fn sample_is_in_scope(path: &Path) -> bool {
    // 논리 corpus 경로에는 samples/ 접두사가 없을 수도 있다.
    let normalized = path.to_string_lossy().replace('\\', "/");
    let name = normalized.rsplit('/').next().unwrap_or("");
    if name.starts_with("hwp3-sample16")
        || include_str!("../fixtures/deferred_document_regressions.txt")
            .lines()
            .any(|retired| retired == name)
    {
        return false;
    }
    let absolute_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples");
    let relative = path
        .strip_prefix("samples")
        .or_else(|_| path.strip_prefix(&absolute_root));
    let Ok(relative) = relative else {
        return true;
    };
    let relative = relative.to_string_lossy().replace('\\', "/");
    !TRANSFERRED_EXAMS.lines().any(|exam| exam == relative)
}
