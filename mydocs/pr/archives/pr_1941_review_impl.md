# PR #1941 Review Impl

## Stage 1. PR 생성

완료.

- 코드 커밋: `54e3baeda task 1939: HWP5-origin HWPX LineSeg marker 보존`
- PR: https://github.com/edwardkim/rhwp/pull/1941
- base: `devel`
- head: `task/m100-1939-76076-renderdiff`

## Stage 2. 변경 검토

완료.

- HWP5-origin HWPX export marker가 TAC 표 높이 보정에서 실제 LineSeg처럼 변형되는 경로를 확인했다.
- 보정은 `missing_lineseg_placeholder` 단일 문단에 한정해 TAC 높이 보정에서 제외한다.
- 특정 샘플명/페이지 번호/임의 계수 분기가 아니라 `LineSeg` marker의 IR 의미에 기반한다.

## Stage 3. 검증

완료.

- `render-diff --via hwpx`: 76076 샘플 PASS
- `issue_1939`: 통과
- `issue_1891`: 통과
- `clippy --test issue_1939`: 통과
- `git diff --check`: 통과
- 작업지시자 full integration 검증: `cargo test --profile release-test --tests` 통과

## Stage 4. 옵션 1 문서 반영

진행 중.

- `mydocs/pr/archives/pr_1941_review.md`
- `mydocs/pr/archives/pr_1941_review_impl.md`
- `mydocs/orders/20260705.md`

위 문서를 PR head에 추가 커밋으로 포함한다.

## Stage 5. merge 전 조건

- GitHub Actions 최신 head 기준 통과 확인
- 작업지시자 최종 승인

## Stage 6. merge 후 후속 처리

`mydocs/manual/pr_review_workflow.md` 7장에 따라 순차 처리한다.

- merge SHA 확인
- #1939 auto-close 여부 확인
- auto-close 여부와 관계없이 #1939에 검증 코멘트 남김
- PR #1941 코멘트
- `devel` sync
- 로컬/원격 PR head 브랜치 정리
