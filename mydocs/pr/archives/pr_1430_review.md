# PR #1430 리뷰 기록

## PR 정보

- PR: https://github.com/edwardkim/rhwp/pull/1430
- 제목: `task 1428: 개체 속성 비율 유지와 누름틀 편집 정합화`
- 연결 이슈: https://github.com/edwardkim/rhwp/issues/1428
- base: `edwardkim/rhwp:devel`
- head: `edwardkim/rhwp:task_m100_1428`
- 상태: Open, Draft 아님
- 작성자: `jangster77`
- 작성 시점: 2026-06-18 00:22 KST
- 규모: 30 files, +868 / -130
- mergeable: `MERGEABLE`
- merge state: `BLOCKED` (GitHub Actions pending)

## 변경 범위

- rhwp-studio 개체 속성 기본 탭에 `비율 유지` 설정을 추가했다.
- `비율 유지` OFF에서는 너비와 높이를 독립 입력하고, ON에서는 기존 비율에 맞춰 반대 축을 보정한다.
- `비율 유지`는 HWP/HWPX 저장 속성이 아니라 `rhwp-settings`에 저장되는 Studio 사용자 UI 설정으로 처리했다.
- 개체 속성 및 주요 모달이 외부 클릭만으로 닫히지 않도록 정리했다.
- 누름틀 고치기 완료 후 포커스/캐럿 복귀, 빈 guide 클릭, 경계 바깥 클릭, 인접 누름틀 guide hit-test, 누름틀 붙여넣기 후 입력 위치를 보정했다.
- `tests/issue_258_clickhere_form_mode.rs`와 `rhwp-studio/tests/user-settings.test.ts`에 회귀 테스트를 추가했다.

## 로컬 검증

PR 생성 전 개발 clone에서 최종 통과를 확인했다.

```text
git diff --check upstream/devel..HEAD
cargo fmt --check
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo clippy --all-targets -- -D warnings
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npm run build
```

결과:

- `rhwp-studio` 테스트: 75 passed
- `cargo test --release --lib`: 1830 passed, 6 ignored
- `cargo test --profile release-test --tests`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `rhwp-studio` production build: 통과

## Collaborator 사전 검증

- PR head fetch: `upstream/task_m100_1428`
- merge 시뮬레이션: `pr1430-merge-test`에서 `git merge upstream/devel --no-commit --no-ff`
- 결과: `Already up to date`, 충돌 없음
- 리뷰 문서/오늘할일 추가 전 `git diff --check upstream/devel..HEAD`: 통과

## 문서 처리

`pr_review_workflow.md` 규칙에 따라 리뷰 문서는 처음부터 archive 경로에 작성했다.

- `mydocs/pr/archives/pr_1430_review.md`
- `mydocs/pr/archives/pr_1430_review_impl.md`

오늘할일은 collaborator 처리 기록으로 함께 추가한다.

- `mydocs/orders/20260618.md`

이 문서 커밋은 PR head인 `upstream/task_m100_1428`에 직접 push해 PR diff에 포함시킨다.

## 리뷰 결론

로컬 필수 검증과 merge 시뮬레이션 기준으로는 merge 준비가 가능하다.

다만 리뷰 문서/오늘할일 커밋을 PR head에 포함한 뒤 GitHub Actions가 다시 실행되어야 한다. 해당 push 이후 required check가 모두 통과하면 merge 가능으로 판단한다.

GitHub issue auto-close는 API의 `closingIssuesReferences`가 비어 있는 것으로 확인됐다. merge 후 #1428 close 상태를 수동 확인해야 한다.
