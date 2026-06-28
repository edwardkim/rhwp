# PR #1629 처리 계획

## 대상

- PR: #1629
- 제목: task 1623: 셀 테두리 대각선 UI와 렌더링 정합 개선
- base: `devel`
- head: `task_m100_1623`
- 관련 이슈: #1623

## 커밋

- `967170f6c5028010e727de91be2d9774679f0c44` — `task 1623: 셀 테두리 대각선 UI와 렌더링 정합 개선`
- `7939b4173662715025ff6e372ee29b656faf85e7` — `Merge branch 'devel' into task_m100_1623` (GitHub update branch)
- `568ffc983c328b3056a21396be2b7edc4cbce20b` — #1629 merge commit
- 본 문서와 오늘할일 갱신은 작업지시자 지시에 따라 #1629 merge 후 별도 `mydocs/**` 문서 전용 PR로 처리한다. 후속 문서 PR 커밋 SHA는 push 후 확인한다.

## Stage 1. 구현 커밋 준비

- `upstream/devel` 최신 fetch 후 rebase 완료.
- 코드/샘플/테스트/계획 문서를 단일 구현 커밋으로 정리.
- `task_m100_1623` 브랜치를 원본 저장소 `upstream`에 push.
- PR #1629 생성 완료.
- GitHub update branch 반영 후 로컬 브랜치를 `upstream/task_m100_1623`으로 fast-forward 완료.

## Stage 2. 검증

완료한 로컬 검증:

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `git diff --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --doc`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `wasm-pack build --target web --out-dir pkg`
- `cargo test --test svg_snapshot`

## Stage 3. 별도 문서 PR

- `mydocs/pr/archives/pr_1629_review.md`
- `mydocs/pr/archives/pr_1629_review_impl.md`
- `mydocs/orders/20260628.md`

작업지시자 후속 지시에 따라 #1629에는 review 문서를 추가하지 않고, #1629 merge 후 `mydocs/**` 문서 전용 PR로 분리한다. 이 문서 묶음은 merge 완료 사실, issue close 결과, 검증 결과를 사후 운영 기록으로 남긴다.

## Stage 4. 원격 CI 확인

- PR head 최신 커밋 기준 GitHub Actions 통과 여부를 확인했다.
- 최종 확인값: Build & Test, CodeQL, Render Diff 통과.
- 이 PR은 코드/테스트/샘플/렌더링 fixture 변경을 포함하므로 `mydocs/**` fast-pass 예외 대상이 아니다.

## Stage 5. merge 전 확인

- 최신 `mergeable` / `mergeStateStatus` 재확인: `MERGEABLE` / `CLEAN`.
- 작업지시자 최종 merge 승인 확인.
- #1629 merge 완료: `568ffc983c328b3056a21396be2b7edc4cbce20b`.
- merge 후 #1623 자동 close 실패 확인. 수동 close/comment 완료.

## 후속 후보

- `Crooked=2` 계열 꺾인 대각선의 한컴 2024 시각 표현 정합은 후속 이슈 후보로 분리한다.
