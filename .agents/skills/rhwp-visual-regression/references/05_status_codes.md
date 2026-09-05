# 05 — 상태 코드

`status_str` (`render_geom_diff.rs`) 우선순위:

1. 쪽 수가 다르면 `PAGE_MISMATCH`
2. 하드 구조 불일치 페이지가 있으면 `STRUCT_MISMATCH`
3. maxDisp > 임계면 `OVER`
4. TextRun ±1 구조만 있으면 `WARN_TEXTRUN`
5. 그 외 `PASS`

`LOAD_FAIL` 은 비교 전에 파일을 못 연 배치 행의 상태다.

| status | hard | 텍스트 exit | --json exit |
| --- | --- | --- | --- |
| PASS | 아니오 | 0 | 0 |
| WARN_TEXTRUN | 아니오 | 0 | 0 |
| OVER | 예 | 1 | 3 |
| STRUCT_MISMATCH | 예 | 1 | 3 |
| PAGE_MISMATCH | 예 | 1 | 3 |
| LOAD_FAIL | 예 | 1 | 1 (측정 실패 우선) |

`status_is_hard_failure` = PASS/WARN_TEXTRUN 이 아닌 것.

에이전트 판정은 hard 여부와 별개다. STRUCT 는 hard 이지만 F03 이면
문서 회귀가 아니다.
