# PR #1441 리뷰 처리 계획

## 목적

Task #1440 PR #1441을 Collaborator 절차에 맞춰 검토하고, 리뷰 문서와 오늘할일을 archive 경로 기준으로 PR head에 반영한 뒤 GitHub Actions 재확인 후 merge한다.

## 처리 단계

1. PR 메타데이터 확인
   - PR #1441 URL, base/head, draft 여부, mergeable 상태 확인 완료.
   - base는 `edwardkim/rhwp:devel`, head는 `edwardkim/rhwp:task_m100_1440`이다.
   - head가 원본 저장소 브랜치이므로 문서 커밋은 `upstream/task_m100_1440`에 직접 push한다.

2. 변경 범위 검토
   - 35쪽 그림 어울림 LineSeg 보정, 6쪽 문단 테두리 박스 회귀 보정, 문단 테두리 연결 속성 보존이 중심이다.
   - 시각 검증 자료와 사용자 제공 HWPX/PDF 샘플이 PR에 포함됐다.
   - 관련 이슈는 `Closes #1440`으로 PR 본문에 명시돼 있다.

3. 로컬 검증
   - `cargo build --release`: 통과
   - `cargo test --release --lib`: 통과 (`1842 passed; 0 failed; 6 ignored`)
   - `cargo test --profile release-test --tests`: 통과
   - `cargo fmt --check`: 통과
   - `cargo clippy --all-targets -- -D warnings`: 통과
   - `npm --prefix rhwp-studio run build`: 통과
   - `cargo test --test issue_1440_onsamiro_picture_wrap`: 통과 (`4 passed`)
   - `cargo test --release --lib renderer::layout::integration_tests::tests::test_547_passage_text_inset_match_pdf_p4`: 통과

4. 리뷰 문서/오늘할일 커밋
   - `mydocs/pr/archives/pr_1441_review.md`
   - `mydocs/pr/archives/pr_1441_review_impl.md`
   - `mydocs/orders/20260619.md`
   - 문서 전용 변경이므로 `git diff --check`와 변경 파일 범위 확인으로 검증한다.

5. 원격 push
   - 작업지시자에게 변경 범위와 검증 결과를 보고하고 명시 승인 후 `upstream/task_m100_1440`에 push한다.
   - push 후 PR diff에 archive 리뷰 문서와 오늘할일이 포함됐는지 확인한다.

6. GitHub Actions 재확인
   - 문서 커밋 push 후 required checks가 재실행되면 완료를 기다린다.
   - 모든 required checks가 통과하면 merge 절차로 넘어간다.

7. 후속 처리
   - `Closes #1440` 자동 close 여부를 확인한다.
   - auto-close가 실패하면 workflow 기준에 따라 이슈 close 코멘트 초안을 준비한다.
   - merge 후 `upstream/devel`을 fetch하고 로컬 기준 브랜치를 동기화한다.
   - 임시 원격 브랜치 `task_m100_1440` 삭제 여부를 확인한다.

## 주의 사항

- 리뷰 문서는 active `mydocs/pr/` 경로를 거치지 않고 archive 경로에 바로 작성한다.
- 문서 커밋 push 후 CI 통과 여부만 추가하려고 새 문서 커밋을 다시 push하지 않는다.
- GitHub PR/issue 코멘트는 초안을 작업지시자에게 보여주고 승인받은 뒤 등록한다.
