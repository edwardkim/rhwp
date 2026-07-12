# PR #2016 검토 — Issue #2015: HWPX 부동 RowBreak 표 vert_offset 이중계상 수정

- 2026-07-07 / planet6897 / 연작 4/7 / Closes #2015 (발원지 ②는 findings 인계)

## 요지
#1811(pre-emit host)이 전진시킨 host_h를 typeset 예산과 layout 배치가 vert_offset으로
재차감/재적용 → 이중계상 91.2px 오버플로우. 수정: `pre_emitted_host_heights` 맵을
TypesetState→PaginationResult→LayoutEngine 전파, host pre-emit 문단에 한해
`(vert_off − host_h).max(0)` 감액. 비대상 문서는 host_h=0으로 종전 동일.

## 검토
two-path 정합(typeset 예산 + layout 배치 동시 정정 — feedback_fix_scope_check_two_paths
패턴을 스스로 준수). 신규 핀 issue_2015 + 기존 issue_1749 핀 갱신. 결합 게이트 통과.
## 판단: 머지 권고.

## 결합 게이트 (devel `cbebe4b6` + 연작 7건 시간순 통합, 2026-07-07)
fmt 통과 / clippy 0 / `--tests` **2,924 통과·실패 0** / OVR 5샘플 **추가 변동 0**
(기지 #1936발 3건 동일) / 순차 통합 충돌 0건
