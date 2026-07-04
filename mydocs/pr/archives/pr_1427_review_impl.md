# PR #1427 리뷰 처리 계획

## 목적

Task 1282 구현 PR을 Collaborator 절차에 맞게 검증하고, 리뷰 문서와 오늘할일을 처음부터 archive 경로에 포함한 상태로 PR head에 반영한다.

## 처리 단계

1. PR 메타데이터 확인
   - PR #1427 URL, base/head, draft 여부, merge state 확인 완료.
   - PR head는 `jangster77:task_m100_1282`이다.

2. 로컬 검증 결과 반영
   - `cargo build --release`
   - `cargo test --release --lib`
   - `cargo test --profile release-test --tests`
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`
   - issue 전용 Rust 테스트, WASM 빌드, Studio E2E, Studio production build

3. 시각 증적 정리
   - Stage6 Studio 스크린샷과 Stage11 한컴 PDF 비교 이미지를 PR 설명과 최종 보고서에 반영했다.
   - 대표 증적은 `comparison_restrict.png`, `comparison_no_restrict.png`, `pdf_restrict.png`, `pdf_no_restrict.png`이다.

4. 리뷰 문서/오늘할일 커밋
   - `mydocs/pr/archives/pr_1427_review.md`
   - `mydocs/pr/archives/pr_1427_review_impl.md`
   - `mydocs/orders/20260617.md`
   - 문서 전용 변경이므로 `git diff --check`로 공백 오류와 경로 범위를 확인한다.

5. 원격 push
   - 현재 PR #1427 head가 fork branch이므로 같은 head인 `origin task_m100_1282`에 문서 커밋을 push한다.
   - push 후 PR diff에 archive 리뷰 문서와 오늘할일이 포함됐는지 확인한다.

6. GitHub Actions 재확인
   - 문서 커밋 push 뒤 required check가 다시 실행된다.
   - 모든 check가 통과하면 merge 가능 상태로 판단한다.

## 주의 사항

- PR 리뷰 문서는 active `mydocs/pr/` 경로를 거치지 않고 archive 경로에 바로 작성한다.
- 문서 커밋을 push한 뒤 단순히 GitHub Actions 통과 여부를 문서에 추가하기 위한 재-push는 하지 않는다.
- GitHub 이슈/PR 코멘트는 초안을 작업지시자에게 보여주고 승인받은 뒤 등록한다.
