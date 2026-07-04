# PR #1435 리뷰 처리 계획

## 목적

Task 1352 구현 PR을 Collaborator 절차에 맞게 검증하고, 리뷰 문서와 오늘할일을 archive 경로 기준으로 PR head에 반영한다.

## 처리 단계

1. PR 메타데이터 확인
   - PR #1435 URL, base/head, draft 여부, mergeable 상태 확인 완료.
   - PR head는 `edwardkim:task_m100_1352`이다.

2. 로컬 검증 결과 반영
   - `git diff --check upstream/devel..HEAD`
   - `cargo build --release`
   - `cargo test --release --lib`
   - `cargo test --profile release-test --tests`
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `wasm-pack build --target web --out-dir pkg`

3. 시각 증적 정리
   - 수정 전 `upstream/devel`과 수정 후 task branch 산출물을 비교했다.
   - 대표 증적은 `before_after_header_cell_compare.png`, `before_after_page1_compare.png`이다.
   - 완료 보고서에 회귀 유입 분석과 비교 좌표를 기록했다.

4. 리뷰 문서/오늘할일 커밋
   - `mydocs/pr/archives/pr_1435_review.md`
   - `mydocs/pr/archives/pr_1435_review_impl.md`
   - `mydocs/orders/20260618.md`
   - 문서 전용 변경이므로 `git diff --check`로 공백 오류와 경로 범위를 확인한다.

5. 원격 push
   - 현재 PR #1435 head가 원본 저장소 브랜치이므로 `upstream task_m100_1352`에 직접 push한다.
   - push 후 PR diff에 archive 리뷰 문서와 오늘할일이 포함됐는지 확인한다.

6. GitHub Actions 재확인
   - 문서 커밋 push 뒤 required check가 다시 실행된다.
   - 모든 check가 통과하면 merge 가능 상태로 판단한다.

## 주의 사항

- PR 리뷰 문서는 active `mydocs/pr/` 경로를 거치지 않고 archive 경로에 바로 작성한다.
- 문서 커밋을 push한 뒤 단순히 GitHub Actions 통과 여부를 문서에 추가하기 위한 재-push는 하지 않는다.
- GitHub 이슈/PR 코멘트는 초안을 작업지시자에게 보여주고 승인받은 뒤 등록한다.
