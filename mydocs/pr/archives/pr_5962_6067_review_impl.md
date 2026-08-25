# PR #5962-#6067 CI green open PR 통합 검토 기록

- 작성일: 2026-08-25
- branch: `review/open-ci-green-20260825`
- 기준선: `upstream/devel@898e75930a6c`
- code candidate: `1748b5cf33cb`

## 라우팅

- base route: `maintainer_general.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`,
  `visual_fixture_evidence.md`
- loaded documents: `mydocs/manual/pr_review_workflow.md`, `mydocs/manual/pr_review/local_validation.md`

## 포함 PR

최신 open PR 목록에서 draft가 아니고 CI가 완료됐으며 failure가 없는 PR만 누적 cherry-pick했다.

- #5962, #5963, #5970, #5997, #6012, #6020, #6022, #6027, #6033, #6038, #6043, #6047
- #6048, #6050, #6051, #6052, #6054, #6058, #6061, #6062, #6064, #6065, #6066, #6067

## 제외 PR

- #5953, #6059: Draft라 정식 검토 대상에서 제외.
- #6019: CI는 완료됐지만 `postmelee` reviewer의 `CHANGES_REQUESTED`가 남아 있어 제외.
- #6056: 최신 CI에 failure가 남아 있어 제외.
- #6068: 최신 CI에 failure가 남아 있어 제외. contributor가 unit-tier 실패 원인을 comment로 남겼으나,
  최신 open PR 상태가 green이 아니므로 통합 후보가 아니다.
- #6069: 검토 중 이미 merge되어 open PR 목록에서 제외됐다. `upstream/devel@898e75930a6c`에 포함된 상태에서
  통합 branch를 rebase했다.

## PR 코멘트 검토 요약

- #5962, #5963: 과거 Draft 안내 comment만 있었고 현재 non-draft라 차단 아님.
- #5970: 이전 Archive B/C 실패 지적 뒤 contributor가 destination 축 보정과 양방향 비교를 수정했다. 최신
  head CI green.
- #5997: 이전 overflow baseline 실패 지적 뒤 contributor가 최신 `devel`로 rebase하고 중복분을 제거했다.
  최신 head CI green.
- #6012: 이전 `issue_3637` 실패와 충돌 지적 뒤 contributor가 최신 `devel` 병합과 순차 block 보존을 설명했다.
  통합 conflict도 같은 원칙으로 해결했다.
- #6065: unit-tier 실패와 IR field sweep baseline 실패가 contributor 후속 commit으로 해소됐다.
- #6066, #6067: Codex usage limit 자동 comment만 있으며 차단 review는 없다.

## 메인터너 보정

- `src/renderer/layout/table_layout.rs` conflict 해소 후 생긴 rustfmt 공백만 정리했다.
- 보정 commit: `1748b5cf33cb` (`style: 통합 체리픽 포맷을 정리`)

## 로컬 검증

- `cargo fmt --all -- --check`: 통과.
- `git diff --check`: 통과.
- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`: 18 tests 통과.
- `node scripts/rust-test-suite-manifest.mjs --prepare && node scripts/rust-test-suite-manifest.mjs --check`: 통과.
- `node --test scripts/tests/rust-unit-test-tiers.test.mjs`: 12 tests 통과.
- `node scripts/rust-unit-test-tiers.mjs --check`: 4221 tests 기준 통과.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  `8350 passed, 43 skipped`.
- `npm --prefix rhwp-studio test`: 1082 passed, 1 skipped.
- `npm --prefix rhwp-studio run e2e:undo-depth`: stack 260, snapshot slot 0, 연속 undo 260 통과.
- `python3 scripts/tests/test_undo_depth_e2e_workflow.py`: 5 tests OK.
- `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`: 통과.
- `npm --prefix rhwp-studio run build:no-hwpctrl`: 통과.

## rebase 후 확인

#6069 merge로 `upstream/devel`이 `242c104bd`에서 `898e75930a6c`로 이동해 통합 branch를 rebase했다.
rebase는 충돌 없이 완료됐다. rebase 후 빠른 정합 검사는 다음과 같다.

- `cargo fmt --all -- --check`: 통과.
- `git diff --check upstream/devel...HEAD`: 통과.
- `node scripts/rust-test-suite-manifest.mjs --prepare && node scripts/rust-test-suite-manifest.mjs --check`: 통과.
- `node scripts/rust-unit-test-tiers.mjs --check`: 통과.

## 판정

포함 24건은 통합 수용 가능. 통합 PR 생성은 작업지시자 사전 승인 후 진행한다.
