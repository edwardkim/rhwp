# PR #7567 실행 순서 — RowBreak 선행 보정과 후속 범위

- 관련 이슈: [#7470](https://github.com/edwardkim/rhwp/issues/7470), PR: [#7567](https://github.com/edwardkim/rhwp/pull/7567).
- 기준 판정과 증적은 [self-review](pr_7567_review.md)에 둔다. 이 문서는 단계·commit 소유만 기록한다.
- 작업지시자 승인: 최소 공통 메트릭을 선행 PR에 포함, 2026-10-04 시각 수용·push·Open PR 생성·리뷰 기록 추가.
- 원래 공백·지도 후보 `bac75f50ee4e57839f4c0ac7a259165acf0e3509`는 clean 상태로 별도 보존한다.

| commit | 범위 |
| --- | --- |
| `f69775daf3` | 최소 공통 메트릭 기초 + RowBreak·중첩 흐름 소유 복원 |
| `b119d784ac` | 저장 조각의 흐름 경계 한 번 소비 |
| `43822b6ae7` | 정식 원본 기반 소유 검사 6개; 추가 전 시각 선행 조건 충족 |
| `e09bf25853` | fit·paint의 공통 source frame. 중간 실패는 작업 기록에 보존 |
| `9d2c3a66f9` | 원본 source frame의 독립 닫힘 근거 |
| `4fda213b77` | 단일 행의 전체 두 프레임 닫힘. 이 시점 전체 회귀 4 FAIL이어서 보류 |
| `6be1a7d6c0` | 첫 선언 행의 실제 소유 판정; 교육과정 후반 행 회귀 해소 |
| `7380b29a2c` | 최신 devel `8497729b4f` 통합, 최종 source와 전체 검증 |
| `24956cfac6`, `5b24c9ad8b` | 공개 PNG·검증 기록·작업지시자 시각 수용; source/test 변경 없음 |

1. 완료: local regression/lint/fresh WASM/Visual Sweep → 작업지시자 시각 판정 → upstream 새 branch push → Open #7567.
2. 완료: 제출 후보 CI 29 success/5 skipped, archive review/오늘할일 작성, merge-tree 공백·링크·기록 보존. 다음: 문서 후행 push.
3. 남음: 후행 head required checks와 mergeability → 별도 merge 승인 → merge/후속 comment/이번 PR 전용 자산 정리.
4. 이후: 원래 보정 후보를 병합된 최신 devel에 정렬 → 영향 페이지 Native/fresh WASM·한컴 PDF 재검증 → 별도 PR.

실행 취소 시 미병합 PR의 상태 변경은 별도 승인 대상으로 유지한다. 기존 #7544·원 contributor 본문·원본 입력·
기준 PDF·비공개 글꼴·공유 `target/pr-review`·보존 후보를 되돌리거나 삭제하지 않는다.
