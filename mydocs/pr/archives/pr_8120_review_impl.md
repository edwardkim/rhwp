# PR #8120 self-review·병합 실행 순서

## 확정 범위와 승인

2026-10-21 작업지시자가 “self-review 로 머지 진행합니다”라고 지시했습니다.
검토 대상 code head는 `7ee8bfb6323a27b7d865e8ae287064278879032d`, base는
`2e3b4f880bd11d35398e6ea0100846611f13a72d`입니다.
현재 소스·테스트 보정은 없고, [검토 결과](pr_8120_review.md)의 기존 실행 증거를 재사용합니다.

| 단계 | 상태와 실행 경계 |
|---|---|
| 구현·로컬 검증 | 완료; 제품d1d3091c5 / 전체 회귀168e96b86 / 동작 불변 lint979188617을 구분 |
| PR 등록·CI 보정 | 완료;81de97603 등록 뒤 E2E/기록만 바꾼7ee8bfb63에서 Full CI·실제 Render Diff 성공 |
| self-review | 완료; 공개370파일의 commit/해시·TSV, 실제 소비 경로·대표 PNG와 승인 예외 대조 |
| 기록 trailing | 이 review·review_impl·오늘할일·최종 보고만 single-parent commit으로 추가. 별도 devel 병합 없음 |
| push 전 gate | 최신 base/head 고정, merge-tree exit0, tree의 공백·문서 링크·오늘할일 보존 확인. 원격 base/head 재조회 |
| 최종 CI·병합 | trailing exact-head의 실제 fast-pass·필수 aggregate/상태를 확인한 뒤 승인된 일반 squash merge. 실패/대기/충돌 시 병합하지 않음 |
| 후속 | merge SHA·devel 반영과 duration 갱신 확인, PR 후속 comment. #7998은 Refs이며 별도 close 승인 전 유지; #8106–#8110 유지 |
| 안전 정리 | 영구 증거·원본·실패 재현 자료 보존 후 작업 전용 clean worktree/branch/output 정리. root WIP·다른 작업·공유 target 보호 |

문서 trailing에 source/test/fixture/workflow를 섞지 않습니다. CI가 완료되기 전 병합 사실을
문서에 미리 쓰지 않으며, 실제 확정 SHA·URL·정리 결과는 후속 PR comment에 남깁니다.
병합 전 실패는 미병합 상태로 되돌아가 원인을 처리하고, 병합 뒤 제품 되돌리기가 필요하면
별도 revert PR과 승인으로 처리합니다. 다른 작업 branch를 reset하거나 history를 재작성하지 않습니다.
