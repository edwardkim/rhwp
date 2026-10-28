# PR #2401 검토 — 거대 셀 드래그 선택 page-tree 범위 제한 (postmelee, #2215)

- collaborator 정식 타스크 산출물 (계획서·단계 3·보고서 동봉), Closes #2215
- 본질: ①정확성 — 분할 문단 페이지 경계에서 leading/trailing 커서를 같은
  PageRenderTree 에서 짝지어 이좌표계 결합 차단 ②성능 — 두 끝점 page hint 로
  후보 페이지 한정 (115쪽 전체 탐색 → 1~2쪽). hint 는 **선택적 성능 힌트** —
  부재/무효/미해소 전부 FullFallback 수렴 (fail-safe).

## 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| Rust 전체 스위트 / fmt / clippy --all-targets | 0 실패 / OK / 0 |
| 신규 회귀 issue_2215 (378줄) | 4/4, red→green(원복 시 2 FAIL) |
| studio tsc / npm test | OK / 406/406 |
| CI (해소 head) | 전 항목 green |

기여자 실측: 드래그 p95 0.8~3.4ms, warm long task 0, UI 114→115 수동 해소
확인. 후속 클릭 오인은 #2400 분리 (스코프 규율).

## 충돌 처리

mydocs/orders/20260719.md 표 양측 추가 — 합집합 해소를 포크 head 에 push
(#2386 LFS 함정 없이 통과).

## 처리 결과 (2026-07-19)

merge 완료(admin) + 감사 코멘트. #2215 는 close-issues 자동 close.
