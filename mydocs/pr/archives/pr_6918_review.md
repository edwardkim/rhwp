# PR #6918 검토

## 판정: 승인

**#6888의 담당자 표 복원 범위는 수용 가능하다.** 집중 테스트, 통합 회귀와 1쪽 직접 시각
대조를 통과했다. GitHub approve/merge 승인을 뜻하지 않으며 최신 원격 CI와 작업지시자 승인 게이트는 별도다.

## 원 PR과 검토 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/6918
- 이슈: https://github.com/edwardkim/rhwp/issues/6888
- 원 커밋: `6a2ada933352520fef7ae5478d030b7aa121b2c9`, `8db9e0019b31af7a920a70a41fb9d08ae92db6bb`.
- 로컬 체리픽: `e968bca32`, `df7e661e1`. 원 PR 사전 리뷰어는 `jangster77`.
- 원 head CI는 최초 선정 시 그린이었다. 최신 통합 PR CI는 아직 실행하지 않았다.

조판과 배치가 공용 `topbottom_float_displaced_below_following_flow`를 사용한다.
다음 문단의 저장 바닥보다 아래에 놓인 비-TAC TopAndBottom 개체의 높이를 앞선 흐름에
중복 계상하지 않는 변경이다. 담당자 표의 위치/본문 경계와 도형 위치 유지 집중 테스트 2건이 통과했다.

공용 함수와 sample README 일부의 '틈 0' 설명은 원 PR이 폐기한 초기 조건을 설명한다.
실제 구현은 틈 0을 요구하지 않고 밴드 상단과 다음 문단 바닥을 비교한다.
기여자가 보고한 10,000건 코퍼스 A/B를 이번에 재실행한 것으로 기록하지 않는다.

## 직접 시각 판정

[시각 대조 기록](pr_6914_6918_visual_sweep.md)의 1쪽에서 담당자 표가 본문 안에 완전히
표시되고, 하단 도형은 표 아래에 유지되는 것을 확인했다. 자동 후보 0/1쪽,
pixel match 86.82853%, ink/proxy 15.84985%다. 글꼴·자간·제목 굵기 등의 차이가 남으므로
문서 전체 fidelity를 승인한 것은 아니다. 원 PR의 목표인 표 소실/흐름 계상 문제에 한정한 판정이다.

## 검증한 통합 상태

- 검토일: 2026-09-09, macOS arm64.
- 브랜치: `review/planet6897-6914-6918-20260909`.
- 기준 devel: `ad84192839eb7b8534715ab085dd91d69a5c4a38`.
- 통합 HEAD: `3d382f53f9167a9d18b74bfc07820e4eeb6d5700`.
- HEAD 위의 미커밋 #6914 보정을 포함했다. `src/renderer/layout.rs`의 검증 blob은 `d65d0fd850972913ca77955c2d0ec5f49821a3fa`다. HEAD만으로 보정 상태를 재현할 수 있다고 주장하지 않는다.
- 원 PR 4개, 원 커밋 6개를 `cherry-pick -x`로 적용했다. 원 저자와 출처를 보존했다.
- 테스트 후 추가 제품 코드 변경은 없다. 아래 문서는 검증 결과 기록이다.

## 실제 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| 전체 Rust 회귀 | **9,337 통과, 0 실패, 46 skip**, 78 binaries, slow 4건 |
| 전체 회귀 실행 시간 | 368.376초; 사전 컴파일 5분 51초는 별도 |
| 4개 PR 집중 테스트 | **9 통과, 0 실패** |
| Rust test manifest | 1,233 sources, 28 suites + 20 exceptions, **48/48 targets** |
| Source-side test tier | **4,205 tests / 298 modules**, 기준선 증가 없음 |
| Rust format | `cargo fmt --all -- --check` 통과 |
| WASM | locked wrapper 빌드 성공, wasm-pack 총 19.71초 |
| Canvas Render Diff | **3문서 3쪽 통과**, 기존 허용 비율 0.05% 유지 |
| diff whitespace | 문서 갱신 전 제품 변경의 `git diff --check` 통과 |

전체 회귀 명령:

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast \
  --status-level fail --final-status-level fail
```

Nextest run ID: `596c294b-057f-4c3a-9d86-d70deef2a4e0`.
미지원 설정 키 `profile.ci-duration-observation.junit.report-skipped` 경고가 있었으나
명령은 종료 코드 0으로 끝났다. 46개 skip을 실행/통과 건수에 포함하지 않았다.

집중 테스트는 `node scripts/run-rust-test.mjs <module> -- --cargo-profile release-test --target-dir target/pr-review --test-threads 8`로 실행했다.

| module | 실행 / 통과 |
| --- | --- |
| `issue_6900_table_tail_gap_ladder` | 2 / 2 |
| `issue_6888_displaced_float_flow_charge` | 2 / 2 |
| `issue_6922_legacy_vtchart_guard` | 3 / 3 |
| `issue_6868_nested_list_orphan_field_end` | 2 / 2 |

WASM 및 브라우저 검증:

```bash
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg
VITE_PORT=7714 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
  npm --prefix rhwp-studio run e2e:render-diff:ci
```

WASM SHA-256: `6f9599864db782de2bb61740474c95c4c7257b397f1b6aaa0180030b0b775de4`.
WASM binding 도구의 사전 빌드 다운로드 경고 뒤 fallback으로 완료됐다.
이번에는 Clippy 3종, 전체 Studio E2E, 원격 통합 PR CI를 실행하지 않았다.
로컬 통과를 원격 required check 통과로 대체하지 않는다. 생성 suite와 원시 로그는 커밋 대상이 아니다.

## Merge 후 contributor PR comment 계획

작업지시자 승인과 실제 최종 CI 완료 뒤, 직접 merge가 아닌 체리픽 수용 사실과 merge SHA를
기록한다. 1쪽만 대조했고 자동 후보 0개라는 사실, 위 지표 및 전체 fidelity 한계를 명시한다.

`![#6918 1쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6914_6937_20260909/pr6918-p001.png)`

[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)과
최종 merge SHA의 review 링크를 함께 적는다. UTF-8 body file로 승인 후 한 번 게시하고 API로 재조회한다.
원 fork branch는 보존한다. 이번 작업에서는 remote push, PR 생성, approve, merge, close를 하지 않았다.

## 최종 제출 검증 갱신 (2026-09-09)

- 제출 code candidate는 `ae89a7b9d72dabce1a1a705b5fcd6d53d6baa953`이다. contributor 체리픽 6개 뒤에 #6914 메인터너 보정을 별도 commit으로 고정했다. 뒤따르는 기록 commit은 제품 코드를 바꾸지 않는다.
- 앞선 검토 시점의 Clippy 미실행 기록은 중간 상태다. 이번 최종 준비에서 아래 검사를 순차 실행했고 모두 exit code 0으로 통과했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
```

- 기존 동일 제품 코드의 focused 9개, 전체 회귀 9,337개 통과(실패 0, skip 46, 8 threads), WASM 빌드 및 Canvas Render Diff 3개 페이지 결과를 재사용했다. 전체 회귀를 이번 제출 준비에서 중복 실행하지 않았다.
- #6914는 보정 완료 후 수용 가능, #6918·#6934·#6937은 각 문서에 명시한 변경 범위에 한해 승인이다. #6922 전체 차트 렌더링과 #6868 cross-section 잔여까지 해결했다는 의미는 아니다.
- 한컴 첨부 PDF 원본은 `samples/issue6900/pdf/`와 `samples/issue6888/pdf/`에 보존한다. SHA-1이 같은 검토용 `pdf/` 사본 두 개는 제거했다. PDF 1.4/1.6으로 한컴 생성본을 배제하지 않는 재사용 지침을 함께 반영한다.
- 원 PR 4개의 최신 head가 기록한 source SHA와 같음을 확인했다. 통합 PR의 최신 head 원격 CI 및 병합·후속 처리는 아직 미완료다. 원 PR과 관련 이슈를 미리 닫지 않는다.
