// 사용자가 다른 작업에서 재추가할 시험지 입력을 공용 회귀 corpus에서도 제외한다.
// 원본 문서와 기준 PDF는 보존하며 의료 검진 문서는 시험지로 분류하지 않는다.

pub fn is_exam_fixture(path: &std::path::Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    name.starts_with("exam_")
        || name.starts_with("exam-")
        || name.starts_with("eq-002.")
        || name.starts_with("21_언어_기출_편집가능본.")
        || name.starts_with("exambank_math_equations_min.")
        || (name.starts_with("3-")
            && (name.contains("월_교육_통합_") || name.contains("월_실전_통합_")))
}
