# PR #7122 검토

- 검토일: 2026-09-14
- PR: https://github.com/edwardkim/rhwp/pull/7122
- 작성자: dependabot[bot], base devel. reviewer edwardkim 추가(기존 jangster77 유지).
- 경로: maintainer_general + intake_and_review + local_validation + multi_pr_update_branch + rework_and_exceptions; 렌더 의존성에는 visual_fixture_evidence 참고.
- 원 head: `267cb4c8e97098714154c2ec2803ed7de6d85f27`.
- 검토 base: `037e4906a93e99896daa145a5ee5517824bfeaf4`.
- 각 PR의 current-base merge-tree 및 여섯 건 누적 cherry-pick 모두 충돌 없음.
- 여섯 건 검토 순서: #7121 → #7122 → #7125 → #7126 → #7127 → #7128. #7121 실패 분리 후 나머지 다섯 건을 같은 순서로 재구성했다. 각 원 commit을 -x로 보존했고 원 PR branch는 변경하지 않았다. 기능상 선행 PR 의존성은 없으나 Cargo.lock 공통 변경을 누적으로 확인했다.
- 다섯 건 검증 후보: `4ff101bee6f187b20bda044d3182d64a6f2e95a0`.

## 검토·검증

Cargo.lock만 변경하며 core_detect, multiversion, multiversion-macros, multiversion_no_op, simdutf8을 새로 잠근다. 기존 rustversion·scopeguard도 encoding_rs 의존성으로 추가된다.
공식 crate README Release Notes에서 0.8.40의 MSRV 1.88, legacy decoder buffer 경계 보정, ASCII 가속 재작성, 0.8.41의 multiversion 0.9 / syn 3 정합 변경을 확인했다. 저장소 toolchain 1.93.1은 새 MSRV를 충족한다.
[공식 변경 기록](https://github.com/hsivonen/encoding_rs/blob/v0.8.41/README.md#release-notes).
실제 사용처는 HWP3 Johab/EUC-KR 변환, OLE chart, WMF charset, 문서 질의이다.
다섯 건 누적 후보에서 Johab/OLE/WMF 집중 테스트 58건 통과(필터 밖 4,010건 미실행).
명령: `cargo nextest run --locked --lib --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review -E 'test(parser::hwp3::johab) | test(ole_chart) | test(wmf)' --no-fail-fast`.
로그: `output/dependabot-20260914/encoding-focused.log`. Host nextest 0.9.137은 권장 0.9.140보다 낮다는 비차단 경고가 있었으며 테스트 exit 0이다. 이 로컬 버전과 #7127이 CI에 설치하는 버전을 혼동하지 않는다.
조판 원칙: 조판 조건·golden·baseline 변경 없음. 원격 전체 검증과 인코딩 계약 집중 검증을 구분한다.
검증 입력: 기존 commit의 unit test 및 매핑 상수이며 신규 sample 없음.

## 원격 CI 및 검증 경계

- [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34800351212/job/103841642356): SUCCESS.
- [Lint (fmt, clippy, WASM check)](https://github.com/edwardkim/rhwp/actions/runs/34800351381/job/103841673396): SUCCESS.
- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/34800351381/job/103841673378): SUCCESS.
- [Frontend package gates](https://github.com/edwardkim/rhwp/actions/runs/34800351381/job/103841673423): SUCCESS.
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34800351381/job/103844786691): SUCCESS.

- 원 head의 진행/실패 check 없음. source·test·fixture 보정 없이 current-base merge clean이므로 원격 전체 CI를 재사용하고 로컬 집중 검증만 추가한다. SKIPPED를 실행 성공으로 세지 않는다.
- frontend 테스트에서 사용한 pkg는 기존 로컬 WASM SHA256 `61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`를 복사한 것이다. Rust 의존성 갱신을 반영해 새로 빌드한 WASM이라는 주장은 하지 않는다.
- primary devel·다른 작업의 worktree·원격 source 변경 없음. 원격 push·GitHub review/comment·merge는 하지 않았다.

## 판정

**승인**. 이는 기술 검토 판정이며 병합 권한 승인은 별도다.

병합 직전에 원 head·CI·current-base 충돌을 재확인한다. 나머지 다섯 건은 한 번의 일괄 병합 승인 대상으로 보고하고 #7121은 분리한다.

## 병합 후 기록

- 작업지시자의 5건 일괄 병합 승인으로 원 head 그대로 병합했다.
- merge SHA: `5fbb63dc4c133de76c7cffb8489918d9bc893112`; 시각: 2026-09-14T07:32:29Z (UTC).
- 각 병합 직전 최신 devel fetch·merge-tree·exact head·성공 CI를 재확인했다. GitHub의 일시적 mergeable UNKNOWN은 재조회하여 MERGEABLE로 바뀐 뒤에만 진행했다.
- 5건 완료 뒤 devel `c3a200464ad09e843b4b3bfc08ba2469910d5280`과 로컬 검증 후보 `4ff101bee6f187b20bda044d3182d64a6f2e95a0`의 제품 tree가 동일함을 git diff로 확인했다.
- 별도 관련 issue 없음. maintainer 직접 반영으로 이 archive review와 오늘할일만 한 운영 기록 commit에 보존한다. 원격 Dependabot branch는 이번 작업에서 만들지 않았으므로 삭제하지 않는다.
- 이 merge의 duration 갱신 34818334483: success. 일괄 병합 중 #7125·#7127의 중간 갱신은 cancelled였고 마지막 devel의 [34818448033](https://github.com/edwardkim/rhwp/actions/runs/34818448033)은 success다. CI 검증 실패로 취급하거나 재실행하지 않았다.
- 5개 merge 모두 Close Issues workflow success이며 별도 관련 issue 없음. CI·CodeQL·Adapter·Proptest 등 검증 workflow를 병합 뒤 새로 시작하지 않았다.
- 검토 로그는 기본 작업공간 `output/dependabot-20260914/`에 복사해 보존했다. 전용 worktree·로컬 review branch 및 이번 fetch ref는 후속 종료 시 제거하며 공유 `target/pr-review`와 다른 작업은 유지한다.
