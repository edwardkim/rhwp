---
kind: pr-review
status: active
pr: 5674
---

# PR #5674 검토: kevin9327 조회 CLI 54개 누적 통합

## 접수

- PR: [#5674](https://github.com/edwardkim/rhwp/pull/5674) `feat(agent): kevin9327 조회 CLI 54개를 누적 통합한다`
- 준비자: `@jangster77`, 원 작성자: `@kevin9327`
- base: `devel` (`c2a36398dba0eb46eb0744933ae9e70cd0d6f10d`)
- code candidate: `829bb4cbb007c9bf1f5b5a23a1089ea2336f8a3e`
- 작성 시점 상태: Open, non-draft, `MERGEABLE`; code candidate CI 진행 중이라 `BLOCKED`는 미완료 check에 의한 상태다.
- base route: `maintainer_general.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`, `rework_and_exceptions.md`

## 변경 범위와 계보

- 최신 `upstream/devel` 위 가시성 검토 브랜치 `review/kevin9327-q-cli-round2-20260819`에서 원 PR [#5659](https://github.com/edwardkim/rhwp/pull/5659), [#5662](https://github.com/edwardkim/rhwp/pull/5662), [#5663](https://github.com/edwardkim/rhwp/pull/5663), [#5664](https://github.com/edwardkim/rhwp/pull/5664), [#5668](https://github.com/edwardkim/rhwp/pull/5668)을 `git cherry-pick -x`로 적용했다.
- 원 작성자 커밋은 각각 `b1ce8f697`, `2afd526a6`, `12ab7afcb`, `730ef07b0`, `942853ea7`로 보존된다.
- 단건 읽기 전용 query CLI 4개와 `rhwp-q-kit` 하위 query 50개가 범위다. 모든 신규 회귀 원본은 `tests/cases`에만 두었고, 파생 suite·manifest·Cargo target은 stage하지 않았다.
- 메인터너 보정 `403da0a93`은 등록된 q-kit 하위 명령이 `--help`와 `-h`에서 종료 코드 0과 usage를 반환하도록 만들고, 전역 목록의 50개 명령 전체를 실행하는 계약 테스트를 추가한다.

## 검토 결과

**차단 결함 없음.** q-kit은 전역 help와 하위 명령 help의 종료 코드가 달라 자동 탐색 계약을 깨고 있었으나, dispatcher 보정과 전수 contract로 해소했다. 나머지 CLI는 기존 `DocumentCore` 읽기 질의만 노출하며 편집·저장 경로를 추가하지 않는다.

## 검증 증적

### 로컬

- `node scripts/rust-test-suite-manifest.mjs --prepare` 뒤 manifest `--check` 통과
- `node scripts/rust-unit-test-tiers.mjs --check` 통과: 4,225 tests / 298 modules
- `cargo fmt --all -- --check`, `cargo clippy --all-targets --target-dir target/review-kevin9327-q-cli-round2-20260819 -- -D warnings` 통과
- q-kit focused integration contract 6/6, 단건 query CLI JSON smoke 4/4, q-kit 하위 명령 help 50/50 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/review-kevin9327-q-cli-round2-20260819 --tests --no-fail-fast`: **7,978/7,978 통과**, 38 skipped
- `git diff --check` 통과

### 시각 검증

조회 전용 Rust CLI와 integration contract 변경만 포함하므로 renderer·layout·visual fixture 검증은 적용하지 않는다.

## 다음 게이트

이 self-review·구현 기록·오늘할일만 담은 single-parent trailing docs commit을 현재 source branch에 push한다. 이후 최신 trailing head의 GitHub Actions, mergeability, required aggregate가 성공하면 병합한다. 병합 뒤 원 PR·연결 issue comment/close와 branch·worktree·전용 target 정리는 `post_merge.md` 순서로 수행한다.
