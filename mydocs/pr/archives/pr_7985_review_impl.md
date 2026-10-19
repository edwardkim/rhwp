# PR #7985 적용 기록

## 신원과 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/7985
- 원 author: donggyun112 <ssddgg99@daum.net>.
- 원 head `5dd0c86ac1a86c977e9d80a7fe263ac60358f720` → 적용 `c575ae384aa4ac343d1b64531f2490206ee9ccfe` (`-x` 원 SHA 보존).
- 최신 base `7d01ef610f378157b79d36ef23e8ed4a3c8ab051` · 검증 head `2dee7730306dcc7f57615c87c25cb09af3c90f63`.
- 기본 작업공간 `review/donggyun112-20261017`에서 검토했으며 별도 worktree는 만들지 않았다.
- [최종 검토](pr_7985_review.md) · [누적 계획](../../plans/pr_review_donggyun112_20261017.md).

## 충돌과 보정

manifest 충돌은 #7977 HF/FN 설명과 #7983 신규 항목을 모두 유지했다.
메인터너 제품 보정 `607f1f7b515553d2c1b618448d345b1d72e8e3e3`는 #7985의 `onInput`에서 실제 빈 누름틀 정규화보다
선택 허용 검사를 먼저 수행한다. #7978의 여섯 scalar 수정은 원 기여 그대로다.
기존 가운뎃점 E2E의 manifest 누락 한 행을 보완했고 기존 검사의 조판 정확성을 새로 주장하지 않는다.
추가 #7988의 실제 UI Unicode 치환·undo/redo 회귀는 `2dee7730306dcc7f57615c87c25cb09af3c90f63`에 별도로 보강했다.

## 검증과 리베이스

[공통 검증 결과](pr_7985_review.md#공통-로컬-검증)에 실제 명령·수치·WASM 해시와 한계를 기록했다.
초기 2개 PR만의 성공과 WASM은 이력이며 최종 3개 PR 검증으로 대체했다.
검증 중 upstream의 #7995 바인딩·문서가 추가되어 clean 상태에서 rebase했다. Rust 입력 Git object는
동일하다. 컴파일 중 중단한 이전 전체 회귀는 PASS로 계산하지 않았고 최신 head에서 다시 완료했다.

## 통합 PR code CI와 남은 후속 처리

[통합 PR #7997](https://github.com/edwardkim/rhwp/pull/7997)의 정확한 code head
`fd3d97b0b9effa2d0d4a1f65f47e3475ba4ec5c8`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/39041517840) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39041517836) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39041517829) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/39041517834) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/39041517854) · [Controller](https://github.com/edwardkim/rhwp/actions/runs/39042954892)를 확인했다.
집계 Build & Test·언어별 CodeQL·Canvas·Adapter·Proptest와 정책 status가 완료됐고
실패·진행 중 check는 없다. GHAS CodeQL의 neutral과 영향 정책상 skip을 실제 성공 worker와 구분한다.
PR 생성 전 source PR CI를 재사용한 결과가 아닌 이 통합 PR의 실제 실행이다.

사용자는 CI 모니터링 이후 병합·후속 처리를 승인했다. 이 기록은 동일 PR의 문서-only trailing이며
제품·테스트·workflow·fixture·baseline은 변경하지 않는다. 최신 base와 merge tree의 문서 링크·
오늘할일 보존을 확인하고 push한 뒤, trailing head의 실제 fast-pass·required aggregate·mergeability를
다시 확인한다. 아직 실행 전인 trailing CI·merge·이슈 종료·정리는 완료로 표시하지 않는다.

원본 로컬 영수증·로그의 종료 후 보존 경로는 `/home/tsjang/rhwp-pr-evidence/pr-7997/`다.
저장소의 ignored `output/pr-review/donggyun112-20261017/`는 실행 당시 경로이며,
최종 보존 파일을 해시 확인한 뒤 `post_merge.md`에 따라 root `output/` 전체를 정리한다.
공용 `target/pr-review`와 contributor fork는 보존한다.
