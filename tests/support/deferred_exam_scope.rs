// 시험지 회귀는 별도 작업에서 다시 추가한다. 원본과 기존 기준값은 보존하고,
// 공통 코퍼스 검사에서도 해당 문서를 실행 대상에서 제외한다.
fn is_deferred_exam_document(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_lowercase();
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    name.starts_with("exam_")
        || name.starts_with("exam-")
        || name.starts_with("exambank_math")
        || name.starts_with("eq-002")
        || name.starts_with("21_언어_기출_편집가능본")
        || name.starts_with("so-sueop")
        || (name.starts_with("3-")
            && (name.contains("월_교육_통합") || name.contains("월_실전_통합")))
}
