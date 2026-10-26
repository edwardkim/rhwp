// 사용자가 다른 작업에서 재추가할 시험지 입력을 공용 회귀 corpus에서도 제외한다.
// 원본 문서와 기준 PDF는 보존하며 의료 검진 문서는 시험지로 분류하지 않는다.

pub fn is_exam_fixture(path: &std::path::Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let name = normalized.rsplit('/').next().unwrap_or("");
    // sample16 전용 검사와 별도 PR로 이관한 문서는 공용 corpus에서 수집하지 않는다.
    name.starts_with("hwp3-sample16")
        || include_str!("../fixtures/deferred_document_regressions.txt")
            .lines()
            .any(|retired| retired == name)
        || name.starts_with("exam_")
        || name.starts_with("exam-")
        || name.starts_with("eq-002.")
        || name.starts_with("21_언어_기출_편집가능본.")
        || name.starts_with("exambank_math_equations_min.")
        || (name.starts_with("3-")
            && (name.contains("월_교육_통합_") || name.contains("월_실전_통합_")))
}
