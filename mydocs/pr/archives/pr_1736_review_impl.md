# PR #1736 처리 계획 — #1733 tail-before-vpos-reset over-pagination 추가 완화

## 대상

- PR: #1736
- 작성자: @planet6897
- 관련 이슈: #1733
- 문서 작성 시점 PR head: `9f541df2b48b8c5899267f7237e835710c853d3e`
- 처리 판단: 수용 및 merge 완료
- merge commit: `342199af1be055f573f65aca3a48e3579feea726`
- PR 후속 comment: https://github.com/edwardkim/rhwp/pull/1736#issuecomment-4854743131

## 커밋

1. `4eba6367fca7913b8d73384e74b21b3179bff1a7`
   - `Task #1733 (부분): tail-before-vpos-reset over-pagination 추가 완화 (250→245)`
   - 실제 기능/보고서 변경 커밋
2. `9f541df2b48b8c5899267f7237e835710c853d3e`
   - `Merge branch 'devel' into pr/devel-1733`
   - 최신 `devel` 동기화용 merge commit

## 검토 단계

### Stage 1. PR 메타 확인

- base branch: `devel`
- draft: false (작성 시점 참고값)
- mergeable: `MERGEABLE` (작성 시점 참고값)
- mergeStateStatus: `BLOCKED` (GitHub required check `Build & Test` 진행 중인 작성 시점 참고값)
- maintainerCanModify: `true`
- 규모: 2 files, +88/-1

### Stage 2. merge 시뮬레이션

완료.

- `upstream/devel` 기준 `local/pr1736` 병합 시뮬레이션
- 충돌 없음
- 검증 후 merge-test 브랜치와 staged merge 상태 정리 완료

### Stage 3. 변경 내용 검토

완료.

- `src/renderer/typeset.rs`에 tail-before-vpos-reset 빈 문단 관통 가드 추가
- tail-before-vpos-reset fit 판정에서 20px overflow tolerance를 1회만 적용
- `mydocs/report/task_m100_1733_report.md` 추가

### Stage 4. 로컬 검증

완료.

- cargo 검증 전 `target` 하위 항목 삭제
- `CARGO_INCREMENTAL=0 cargo test --lib`: 2044 passed, 0 failed, 7 ignored
- `CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `CARGO_INCREMENTAL=0 cargo test --test svg_snapshot`: 8 passed

### Stage 5. 페이지 수 검증

완료.

- `samples/task1725/text_footnote_tail_overpagination.hwpx`: 245쪽
- `samples/task1725/text_footnote_tail_overpagination.hwp`: 244쪽
- `samples/byeolpyo1.hwp`: 4쪽
- `samples/byeolpyo4.hwp`: 26쪽
- `samples/task1718/table_giant_cell_overfill.hwp`: 42쪽

### Stage 6. merge 전 대기 조건

- GitHub Actions `Build & Test` 최종 success 확인 완료
- 작업지시자 승인 완료

### Stage 7. merge 및 후속 코멘트

완료.

- `gh pr merge 1736 --repo edwardkim/rhwp --merge --admin`
- merge commit: `342199af1be055f573f65aca3a48e3579feea726`
- `git merge --ff-only upstream/devel` 로 로컬 `devel` 동기화 완료
- PR 후속 comment 완료: https://github.com/edwardkim/rhwp/pull/1736#issuecomment-4854743131
- #1733은 부분 해결 이슈이므로 close하지 않고 open 유지

## merge 후 필수 후속 처리

`mydocs/manual/pr_review_workflow.md` 기준으로 처리한다.

1. merge 직전 최신 GitHub Actions와 head SHA 재확인: 완료
2. `gh pr merge 1736 --repo edwardkim/rhwp --merge --admin`: 완료
3. `git checkout devel && git pull --ff-only upstream devel`: 완료
4. 리뷰 문서를 `mydocs/pr/archives/`로 이동: 후속 문서 PR에 포함
5. `mydocs/orders/20260701.md` 오늘할일에 #1736 처리 내용 반영: 후속 문서 PR에 포함
6. #1733 이슈는 부분 해결이므로 close하지 않고 잔여 +3쪽 후속 범위 유지: 완료
7. PR 감사 코멘트에는 250→245 확인, HWPX/HWP page count 구분, 잔여 #1733 계속 추적을 기록: 완료
8. 리뷰 문서/오늘할일은 문서-only PR로 처리 후 merge: 진행 예정

## 후속 코멘트 요지

- PR head 기준 HWPX 샘플 245쪽 확인
- 기준 HWP 파일은 244쪽으로 별도 확인
- byeolpyo1/byeolpyo4/#1718 대표 샘플 무회귀 확인
- 로컬 `test --lib`, `clippy --all-targets`, `svg_snapshot` 통과
- #1733은 부분 해결이며 잔여 +3쪽은 계속 추적
