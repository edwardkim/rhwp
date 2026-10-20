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
    !TRANSFERRED_EXAMS.lines().any(|exam| exam == relative)
}
