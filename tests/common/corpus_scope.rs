// 사용자가 별도 작업으로 이관한 시험지는 공통 회귀 코퍼스에서도 검사하지 않는다.

use std::path::Path;

const TRANSFERRED_EXAMS: &str = include_str!("../fixtures/transferred_exam_regressions.txt");

pub(crate) fn sample_is_in_scope(path: &Path) -> bool {
    let absolute_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples");
    let relative = path
        .strip_prefix("samples")
        .or_else(|_| path.strip_prefix(&absolute_root));
    let Ok(relative) = relative else {
        return true;
    };
    let relative = relative.to_string_lossy().replace('\\', "/");
    let name = relative.rsplit('/').next().unwrap_or(&relative);
    if include_str!("../fixtures/deferred_document_regressions.txt")
        .lines().any(|deferred| deferred == name)
    {
        // 별도 PR에서 회귀를 다시 추가하는 문서는 자동 수집에서도 제외한다.
        return false;
    }
    if name.starts_with("hwp3-sample16.") || name.starts_with("hwp3-sample16-") {
        // sample16 원본·변환본은 별도 PR에서 회귀를 관리한다.
        return false;
    }
    !TRANSFERRED_EXAMS.lines().any(|exam| exam == relative)
}
