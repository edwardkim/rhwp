"""사용자가 별도 작업으로 분리한 시험지의 Gym 입력 선택 경계."""

from pathlib import Path
import unicodedata

# 원본은 보존하고 과제·코퍼스 실행에서만 제외한다. 같은 이름의 복사본도
# 자동 코퍼스 선택으로 시험지 검사에 다시 들어가지 않도록 제외한다.
EXCLUDED_EXAM_INPUT_NAMES = frozenset({
    '111exam_social.hwp',
    '21_언어_기출_편집가능본.hwp',
    '3-09월_교육_통합_2022.hwp',
    '3-09월_교육_통합_2022.hwpx',
    '3-09월_교육_통합_2023.hwp',
    '3-09월_교육_통합_2023.hwpx',
    '3-09월_교육_통합_2024-격자기준종이.hwp',
    '3-09월_교육_통합_2024-격자기준쪽.hwp',
    '3-09월_교육_통합_2024-구분선아래20.hwp',
    '3-09월_교육_통합_2024-구분선아래20구분선위20.hwp',
    '3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx',
    '3-09월_교육_통합_2024-미주사이20.hwp',
    '3-10월_교육_통합_2022.hwp',
    '3-10월_교육_통합_2022.hwpx',
    '3-11월_실전_통합_2022.hwp',
    '3-11월_실전_통합_2022.hwpx',
    '3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp',
    '3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwpx',
    '3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp',
    '3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwpx',
    '3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp',
    '3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwpx',
    '3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwp',
    '3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwpx',
    '3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp',
    '3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwpx',
    '3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp',
    '3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwpx',
    '3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp',
    '3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwpx',
    '3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp',
    '3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwpx',
    'eq-002.hwp',
    'eq-002.hwpx',
    'exam-kor-1p.hwp',
    'exam-kor-1p.hwpx',
    'exam-kor-2p.hwp',
    'exam-kor-2p.hwpx',
    'exam-kor-3p.hwp',
    'exam-kor-3p.hwpx',
    'exam-kor-4p.hwp',
    'exam-kor-4p.hwpx',
    'exam_eng.hwp',
    'exam_kor.hwp',
    'exam_kor.hwpx',
    'exam_math.hwp',
    'exam_math_8.hwp',
    'exam_math_no.hwp',
    'exam_science.hwp',
    'exam_social-p1.hwp',
    'exam_social-p1.hwpx',
    'exam_social.hwp',
    'exam_social.hwpx',
    'exambank_math_equations_min.hml',
})
_EXCLUDED_NORMALIZED = frozenset(
    unicodedata.normalize("NFC", name).casefold()
    for name in EXCLUDED_EXAM_INPUT_NAMES
)


def is_excluded_exam_input(path):
    """원본을 읽거나 변경하지 않고 지정 파일의 이름만 판별한다."""
    name = unicodedata.normalize("NFC", Path(path).name).casefold()
    return name in _EXCLUDED_NORMALIZED
