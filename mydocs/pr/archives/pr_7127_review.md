# PR #7127 검토

- 검토일: 2026-09-14
- PR: https://github.com/edwardkim/rhwp/pull/7127
- 작성자: dependabot[bot], base devel. reviewer edwardkim 추가(기존 jangster77 유지).
- 경로: maintainer_general + intake_and_review + local_validation + multi_pr_update_branch + rework_and_exceptions; 렌더 의존성에는 visual_fixture_evidence 참고.
- 원 head: `993528b413a350ae50e7aea23984631b7ebbde98`.
- 검토 base: `037e4906a93e99896daa145a5ee5517824bfeaf4`.
- 각 PR의 current-base merge-tree 및 여섯 건 누적 cherry-pick 모두 충돌 없음.
- 여섯 건 검토 순서: #7121 → #7122 → #7125 → #7126 → #7127 → #7128. #7121 실패 분리 후 나머지 다섯 건을 같은 순서로 재구성했다. 각 원 commit을 -x로 보존했고 원 PR branch는 변경하지 않았다. 기능상 선행 PR 의존성은 없으나 Cargo.lock 공통 변경을 누적으로 확인했다.
- 다섯 건 검증 후보: `4ff101bee6f187b20bda044d3182d64a6f2e95a0`.

## 검토·검증

nextest archive build/run workflow의 action SHA 두 줄만 변경했다. 실행 순서·권한·트리거·캐시 정책은 변경하지 않는다.
공식 v2.87.11 tag가 PR의 `9534c84618278caac52cb373bb164ed464dbd8af` commit과 일치함을 GitHub API로 확인했다.
[공식 diff](https://github.com/taiki-e/install-action/compare/v2.87.5...v2.87.11)에는 각 도구 manifest 갱신이 주로 포함된다. cargo-nextest manifest에 0.9.144 항목이 추가됐지만 latest/0.9 alias는 0.9.143을 유지하므로 action patch 변경을 nextest 본체 강제 갱신으로 해석하지 않는다.
로컬 `node --test scripts/tests/ci-workflow-evidence.test.cjs scripts/tests/nextest-target-duration-policy.test.mjs`: 25 PASS.
원격 CI에서 archive A/B/C/D build와 shard 실행이 모두 성공했다.
로그: `output/dependabot-20260914/workflow-tests.log`.
조판 원칙: 비해당(CI 설치 의존성만 변경). 입력 확인: 기존 추적 workflow와 contract test 그대로 사용하며 sample 없음.

## 원격 CI 및 검증 경계

- [Lint (fmt, clippy, WASM check)](https://github.com/edwardkim/rhwp/actions/runs/34800365463/job/103842699278): SUCCESS.
- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/34800365463/job/103842699184): SUCCESS.
- [Frontend package gates](https://github.com/edwardkim/rhwp/actions/runs/34800365463/job/103842699147): SUCCESS.
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34800365463/job/103845032284): SUCCESS.

- 원 head의 진행/실패 check 없음. source·test·fixture 보정 없이 current-base merge clean이므로 원격 전체 CI를 재사용하고 로컬 집중 검증만 추가한다. SKIPPED를 실행 성공으로 세지 않는다.
- frontend 테스트에서 사용한 pkg는 기존 로컬 WASM SHA256 `61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`를 복사한 것이다. Rust 의존성 갱신을 반영해 새로 빌드한 WASM이라는 주장은 하지 않는다.
- primary devel·다른 작업의 worktree·원격 source 변경 없음. 원격 push·GitHub review/comment·merge는 하지 않았다.

## 판정

**승인**. 이는 기술 검토 판정이며 병합 권한 승인은 별도다.

병합 직전에 원 head·CI·current-base 충돌을 재확인한다. 나머지 다섯 건은 한 번의 일괄 병합 승인 대상으로 보고하고 #7121은 분리한다.

## 병합 후 기록

- 작업지시자의 5건 일괄 병합 승인으로 원 head 그대로 병합했다.
- merge SHA: `02e5633be7bd57909a65272cd4eca1b6324178e4`; 시각: 2026-09-14T07:33:49Z (UTC).
- 각 병합 직전 최신 devel fetch·merge-tree·exact head·성공 CI를 재확인했다. GitHub의 일시적 mergeable UNKNOWN은 재조회하여 MERGEABLE로 바뀐 뒤에만 진행했다.
- 5건 완료 뒤 devel `c3a200464ad09e843b4b3bfc08ba2469910d5280`과 로컬 검증 후보 `4ff101bee6f187b20bda044d3182d64a6f2e95a0`의 제품 tree가 동일함을 git diff로 확인했다.
- 별도 관련 issue 없음. maintainer 직접 반영으로 이 archive review와 오늘할일만 한 운영 기록 commit에 보존한다. 원격 Dependabot branch는 이번 작업에서 만들지 않았으므로 삭제하지 않는다.
- 이 merge의 duration 갱신 34818438183: cancelled. 일괄 병합 중 #7125·#7127의 중간 갱신은 cancelled였고 마지막 devel의 [34818448033](https://github.com/edwardkim/rhwp/actions/runs/34818448033)은 success다. CI 검증 실패로 취급하거나 재실행하지 않았다.
- 5개 merge 모두 Close Issues workflow success이며 별도 관련 issue 없음. CI·CodeQL·Adapter·Proptest 등 검증 workflow를 병합 뒤 새로 시작하지 않았다.
- 검토 로그는 기본 작업공간 `output/dependabot-20260914/`에 복사해 보존했다. 전용 worktree·로컬 review branch 및 이번 fetch ref는 후속 종료 시 제거하며 공유 `target/pr-review`와 다른 작업은 유지한다.
