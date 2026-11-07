# PR #8711 검토 — 선행 정의 슬롯과 HWP 저장 재열기

## 최종 판정

**머지 보류 — 메인테이너 직접 검증 중.** 원 제출 head의 Full CI는 녹색이며 최신 devel merge-tree는 clean이다. 집중 회귀·Native/fresh WASM 원본/저장/재열기·독립 Print 대조와 최근 그림 편집 기능 보존을 완료한 뒤 판정을 확정한다. 원격 댓글·push·merge는 아직 승인 범위가 아니다.

## 접수와 경로

- 대상 [PR #8711](https://github.com/edwardkim/rhwp/pull/8711), 작성자 postmelee(COLLABORATOR), 원 기여자 semanticist21의 [PR #8226](https://github.com/edwardkim/rhwp/pull/8226)을 대체하는 통합 PR.
- 제출 head `57f897a3bd844aadd883d7afb31def4904acb8ca`, upstream branch `review/pr-8226-save-recheck`.
- 최신 검토 base `13775409e57a9550cfb31866d40bdf319ab54bac`, 제출 검증 base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`.
- reviewer edwardkim 지정 후 requested_reviewers API에서 확인했다. source head를 유지한다.
- base route: maintainer_general. modifiers: intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, rework_and_exceptions(대형 증거 묶음), post_merge. collaborator가 만든 통합 PR의 credit·원 PR 후속 계획도 대조한다.
- review branch: `review/pr-8711-leading-definition-slots`, worktree `/home/edward/mygithub/rhwp-worktrees/pr-8711-leading-definition-slots`. 기본 devel 및 외부 pr-8296 작업공간은 보존한다.

## 실행 범위

1. 원 #8226 → 보존한 체리픽 → Storage 호환 `03300db4a` → 머리말/꼬리말 수용 `24bb81d03` → 추가 관계 회귀 `effcb0d60` → 독립 Print·원본/저장본의 출처를 대조한다. #4680·#4687 전체 종료는 이 PR의 주장이 아니다.
2. 최신 base와 clean merge한 실제 tree에서 leading definition 슬롯·필드 소유·저장 read-only·반복 재열기, 공통 줄/표 계획 소비를 검토한다.
3. local_validation 4.3.0에 따라 source/test/fixture 보정 없이 정확한 제출 head의 녹색 Full CI를 재사용한다. 로컬 전체/Native 광범위 회귀는 중복하지 않으며, focused·fresh WASM·시각 검증은 실제 실행한다. 새 보정이 필요하면 재사용 조건을 다시 판단한다.
4. 원 1쪽 입력과 HWP 저장/재열기(직접 API·CLI)를 독립 Mac 한컴 Print에 Native/fresh WASM Canvas/SVG로 대조한다. 원 제공 결과를 메인테이너 실행으로 표시하지 않는다.
5. 새 devel에 추가된 #8566·#8595·#8632의 실제 공통 호출 경로·회귀·전쪽 출력과 저장/소유 계약을 대조한다. 관련 13쪽과 편집 저장 경계를 같은 조건으로 확인한다.
6. 전쪽 TSV·직접 확대 판독·입력/PDF Git blob·producer source/해시를 기록하고 원격 처리안을 구체화한다. CI 취소·재실행·보안 재분류·원 PR close는 별도 승인 없이 수행하지 않는다.
