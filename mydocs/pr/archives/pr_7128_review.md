# PR #7128 검토

- 검토일: 2026-09-14
- PR: https://github.com/edwardkim/rhwp/pull/7128
- 작성자: dependabot[bot], base devel. reviewer edwardkim 추가(기존 jangster77 유지).
- 경로: maintainer_general + intake_and_review + local_validation + multi_pr_update_branch + rework_and_exceptions; 렌더 의존성에는 visual_fixture_evidence 참고.
- 원 head: `47d930d3929083e9e04c125ce9d0679325082e01`.
- 검토 base: `037e4906a93e99896daa145a5ee5517824bfeaf4`.
- 각 PR의 current-base merge-tree 및 여섯 건 누적 cherry-pick 모두 충돌 없음.
- 여섯 건 검토 순서: #7121 → #7122 → #7125 → #7126 → #7127 → #7128. #7121 실패 분리 후 나머지 다섯 건을 같은 순서로 재구성했다. 각 원 commit을 -x로 보존했고 원 PR branch는 변경하지 않았다. 기능상 선행 PR 의존성은 없으나 Cargo.lock 공통 변경을 누적으로 확인했다.
- 다섯 건 검증 후보: `4ff101bee6f187b20bda044d3182d64a6f2e95a0`.

## 검토·검증

Cargo.lock에서 skia-safe와 skia-bindings 두 패키지 version/checksum만 함께 변경했다. Cargo.toml의 semver 범위 안이며 native-skia 선택 기능에 해당한다.
[공식 release](https://github.com/rust-skia/rust-skia/releases/tag/0.153.3)는 Skia m153-0.101.2 갱신과 CLANGCC/CLANGCXX 옵션이 포함된 Yocto cross build 복구를 기록한다. 단순 build script만의 변경이라고 축소하지 않는다.
원격 native Skia job의 실제 테스트 step 성공을 API로 확인했다. 로컬 다섯 건 후보에서도 native Skia 집중 49건 통과(필터 밖 4,076건 미실행).
명령: `cargo nextest run --locked --lib --features native-skia --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review -E 'test(renderer::skia)' --no-fail-fast`.
실제 skia-safe 0.153.3을 컴파일하여 glyph/text/image/clip/shape/그룹 replay 계약을 실행했다. 로그: `output/dependabot-20260914/skia-focused.log`.
Host nextest 0.9.137 < 권장 0.9.140의 비차단 경고는 유지했고 테스트 exit 0이다. 광범위 native 및 PDF 통합 검증은 원 head CI를 재사용하며 이번 로컬 집중 결과를 전체 검증으로 확대하지 않는다.
조판 원칙: rhwp layout·fixture·golden·baseline 변경 없음. native raster/PDF 의존성 영향과 WASM CanvasKit 경로를 구분한다. 별도 시각 개선 주장은 하지 않는다.
검증 입력: 기존 commit의 native 계약 테스트. 신규 sample/정답지 없음.

## 원격 CI 및 검증 경계

- [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34800366626/job/103841746821): SUCCESS.
- [Lint (fmt, clippy, WASM check)](https://github.com/edwardkim/rhwp/actions/runs/34800366814/job/103842633172): SUCCESS.
- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/34800366814/job/103842633196): SUCCESS.
- [Frontend package gates](https://github.com/edwardkim/rhwp/actions/runs/34800366814/job/103842633167): SUCCESS.
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34800366814/job/103845313843): SUCCESS.

- 원 head의 진행/실패 check 없음. source·test·fixture 보정 없이 current-base merge clean이므로 원격 전체 CI를 재사용하고 로컬 집중 검증만 추가한다. SKIPPED를 실행 성공으로 세지 않는다.
- frontend 테스트에서 사용한 pkg는 기존 로컬 WASM SHA256 `61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`를 복사한 것이다. Rust 의존성 갱신을 반영해 새로 빌드한 WASM이라는 주장은 하지 않는다.
- primary devel·다른 작업의 worktree·원격 source 변경 없음. 원격 push·GitHub review/comment·merge는 하지 않았다.

## 판정

**승인**. 이는 기술 검토 판정이며 병합 권한 승인은 별도다.

병합 직전에 원 head·CI·current-base 충돌을 재확인한다. 나머지 다섯 건은 한 번의 일괄 병합 승인 대상으로 보고하고 #7121은 분리한다.

## 병합 후 기록

- 작업지시자의 5건 일괄 병합 승인으로 원 head 그대로 병합했다.
- merge SHA: `c3a200464ad09e843b4b3bfc08ba2469910d5280`; 시각: 2026-09-14T07:33:56Z (UTC).
- 각 병합 직전 최신 devel fetch·merge-tree·exact head·성공 CI를 재확인했다. GitHub의 일시적 mergeable UNKNOWN은 재조회하여 MERGEABLE로 바뀐 뒤에만 진행했다.
- 5건 완료 뒤 devel `c3a200464ad09e843b4b3bfc08ba2469910d5280`과 로컬 검증 후보 `4ff101bee6f187b20bda044d3182d64a6f2e95a0`의 제품 tree가 동일함을 git diff로 확인했다.
- 별도 관련 issue 없음. maintainer 직접 반영으로 이 archive review와 오늘할일만 한 운영 기록 commit에 보존한다. 원격 Dependabot branch는 이번 작업에서 만들지 않았으므로 삭제하지 않는다.
- 이 merge의 duration 갱신 34818448033: success. 일괄 병합 중 #7125·#7127의 중간 갱신은 cancelled였고 마지막 devel의 [34818448033](https://github.com/edwardkim/rhwp/actions/runs/34818448033)은 success다. CI 검증 실패로 취급하거나 재실행하지 않았다.
- 5개 merge 모두 Close Issues workflow success이며 별도 관련 issue 없음. CI·CodeQL·Adapter·Proptest 등 검증 workflow를 병합 뒤 새로 시작하지 않았다.
- 검토 로그는 기본 작업공간 `output/dependabot-20260914/`에 복사해 보존했다. 전용 worktree·로컬 review branch 및 이번 fetch ref는 후속 종료 시 제거하며 공유 `target/pr-review`와 다른 작업은 유지한다.
