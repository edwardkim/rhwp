---
kind: pr-review-implementation
status: validated-local
source-prs: [5659, 5662, 5663, 5664, 5668]
---

# kevin9327 조회 CLI 누적 통합 구현·검토 기록

## 적용 순서

1. 최신 `upstream/devel@14e4d01af`에서 `review/kevin9327-q-cli-round2-20260819`를 만들었다.
2. #5659, #5662, #5663, #5664, #5668 head를 순서대로 `git cherry-pick -x`로 적용했다.
3. 원 작성자·원본 SHA가 남은 누적 커밋 위에서 q-kit 하위 명령 help 계약을 메인터너 보정으로 추가했다.

## 메인터너 보정 이유

`rhwp-q-kit`의 전역 `--help`에는 50개 명령이 표시되지만 `<명령> --help`는 handler의 미지 플래그 처리로 넘어가 종료 코드 2가 됐다. 단건 q CLI와 일관된 탐색 표면을 제공하려면 등록된 하위 명령 모두가 자체 usage와 종료 코드 0을 제공해야 한다.

dispatcher에서 `--help`와 `-h`를 먼저 처리하고, integration contract는 전역 목록에서 추출한 50개 명령을 전부 실행한다. 따라서 이후 새 명령을 목록에 등록하면 help 계약에서 빠질 수 없다.

## 산출물 정책

`rust-test-suite-manifest --prepare`가 만든 generated suite, manifest, Cargo test target은 review worktree 검증 산출물이다. 이 통합 PR에는 stage하지 않았고, source PR은 `tests/cases` 원본만 포함한다.

## 검증 결과

- suite manifest prepare/check: 통과
- source unit tier 정책: 4,225 tests / 298 modules, 통과
- formatter와 clippy: 통과
- q-kit focused contract: 6/6 통과
- 단건 CLI JSON smoke: 4/4 통과
- q-kit 하위 명령 help smoke: 50/50 통과
- 전체 release-test nextest: 7,978/7,978 통과, 38 skipped

## 병합 후 체크리스트

1. 누적 통합 PR의 최신 head 원격 CI 성공을 확인한다.
2. 누적 통합 PR을 병합한다.
3. 원본 PR #5659, #5662, #5663, #5664, #5668과 관련 issue #5656, #5657, #5658, #5661, #5667에 체리픽 수용 결과를 댓글로 남기고 close한다.
4. `review/kevin9327-q-cli-round2-20260819` 원격·로컬 브랜치, review worktree, 전용 target을 절차에 따라 정리한다.
