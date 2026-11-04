mod corpus_deferred_document_scope {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/deferred_documents.rs"
    ));
}

// 사용자가 별도 작업으로 이관한 문서는 공통 회귀 코퍼스에서도 검사하지 않는다.

use std::path::Path;

const TRANSFERRED_EXAMS: &str = include_str!("../fixtures/transferred_exam_regressions.txt");

pub(crate) fn sample_is_in_scope(path: &Path) -> bool {
    if corpus_deferred_document_scope::is_deferred_document(path)
        || path.file_name().and_then(|name| name.to_str()).is_some_and(|name| {
            matches!(name, "2025 행정업무운영 편람(최종).hwp" | "2025 행정업무운영 편람(최종).hwpx")
        })
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
