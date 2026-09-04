# Task #1749 최종 보고서 — saved bounds 신뢰의 페이지-마지막 증거 조건 (쪽 경계 overfill 수정)

## 요약
hwpdocs 페이지·PI 대조 S1(overfill) 원인 규명에서 확정한 결함 수정. 단일줄 문단 fit 의
저장 LINE_SEG 신뢰(`saved_single_line_bottom_fits`, fe6de3ef 도입)가 **누적좌표 문서**
(저장 vpos 가 페이지 경계에서 리셋되지 않는 결재문서 계열)에서 누적높이 초과를 우회해
쪽 경계 overfill 을 만들던 문제. 대표 36371084: 1쪽 used 1011.8px > 본문 990.2px →
**981.4px 해소, MATCH 전환**. 동일 계열 보너스 개선 2건.

## 원인
- pi18: 누적높이 검사 탈락(998.7 > 986.2px)인데 저장 bounds(985.7px ≤ avail)로 통과.
- 저장 bounds 신뢰는 "저장 vpos 가 페이지 배정을 인코딩한다"는 전제 위에 있는데, 누적좌표
  문서(pi19 vpos=74902 가 리셋 없이 본문높이 74265HU 초과)는 전제 불성립.
- 도입 커밋(fe6de3ef)의 보호 케이스는 **문서 마지막 문단** 합성 테스트(Paginator 경로) —
  "다음 실줄 없음/리셋"으로 결함 케이스와 구분 가능.

## 수정
`src/renderer/typeset.rs`: `saved_flow_marks_page_last` 신설 — 다음 문단의 첫 실줄이
없거나(문서/구역 끝) 현재 줄보다 작은 vpos 로 리셋(새 쪽)될 때만 saved bounds 신뢰.
누적좌표 문서는 누적높이 판정으로 복귀. (+단위테스트 4케이스, 통합테스트 1)

## 검증
| 항목 | 결과 |
|------|------|
| 재현·코퍼스 원본 (한글 OLE) | PI_MISMATCH → **MATCH** (2=2쪽) |
| 보너스 개선 | 36396987 MATCH 전환, 36367156 총쪽수 정합(PAGE_DELTA→PI_MISMATCH n=2) |
| cargo test --lib | 2051 passed / 0 failed (fe6de3ef 계열 합성 테스트 포함) |
| 통합 8크레이트 | 101 passed / 0 failed |
| 페이지 게이트 (byeolpyo1/4·승강기·1700/1745/1750·국제고속선기준) | 4/26/42/1/3/5/251쪽 무회귀 |
| 코퍼스 mismatch 재검증 / MATCH 표본 150 | 악화 0 / 150 유지 |
| rustfmt / clippy --lib | 통과 |

## 게이트 조사 중 별도 발견 (PR #1746 코멘트 기록)
국제고속선기준 245→251쪽의 원인은 #1745 의 **대량 문단 흡수(135개, 부속서 4 전체) 해소** —
콘텐츠 복원이며 회귀 아님. 251 이 새 기준선. 상세: `task_m100_1749_stage3.md`.

## 한계 / 후속
- 리셋형 lineseg 인데 한글 라이브와 어긋나는 문서(생성기 lineseg 부정확)는 본 가드 범위 밖 —
  메트릭 정밀화(#1720 계열).
- S1/S2 잔여 razor-thin(rhwp 자체 기준 정상 fit) 미해결 — 개체 단위 높이 오라클 필요.

## 산출물
- 소스: `src/renderer/typeset.rs` (+ `tests/issue_1749_saved_bounds_cumulative.rs`)
- 재현: `samples/task1749/saved_bounds_cumulative_vpos.hwpx` + README
- 검증 TSV: `output/poc/hwpdocs_pipage/{bad39,match150}_recheck_1749.tsv`,
  격리 실험 `gosun_{upstream,head1749}.txt`
