# PR #6934 검토

## 판정: 승인

**레거시 차트 식별 조건 개선이라는 제한된 범위에서 수용 가능하다.** 실제 차트 렌더링 또는
#6922 전체 해결을 승인한 것은 아니다. 최신 원격 CI와 작업지시자 승인 gate는 별도다.

## 원 PR과 코드 검토

- 원 PR: https://github.com/edwardkim/rhwp/pull/6934
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/6922
- 원 head: `893be62716f05378251e93264d6805bd67dd0d06`; 로컬 체리픽: `01093eed4`.
- 최초 선정 시 일반 workflow는 성공했지만 CodeQL aggregate가 `NEUTRAL`이었다.
- CodeQL 사유는 devel의 JavaScript/TypeScript 및 Python 비교 구성 2개 누락이다.
  취약점 발견으로 실패한 것은 아니나 성공 또는 expected skip으로 바꾸어 기록하지 않는다.
- 사용자의 명시적 추가 포함 지시에 따라 체리픽했다.

`src/ole_chart/parser.rs`는 근거가 부족했던 `w1 == w2` 조건만 제거한다.
헤더 종류, 개체 시작 offset 범위, `VtDataGrid` 표지 조건은 유지한다.
집중 테스트는 서로 다른 두 word 인식, 기존 같은 word 형상 유지, 표지 없는 입력 거부
3건 모두 통과했다. 전체 회귀에서도 실패가 없었다.

## 수용 범위와 잔여 문제

이 PR만으로 원 문서의 차트는 그려지지 않는다. 후속 그리드 해석의
`NumberCellCountMismatch`/`legacy HWP chart data grid shape not recognized`는
미해결이다. 기여자의 OLE 1,046개 전수 조사와 75개 차트 표지 일치 수치는 참고 주장으로만
확인했으며 이번에 독립 재실행하지 않았다.

이번 검증은 합성 입력의 파서 식별 계약이다. 새 시각 산출물은 만들지 않았으며,
렌더링 개선 PR처럼 PDF 동일성 또는 차트 표시 성공을 주장하지 않는다.

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

## Merge 후 contributor PR/issue comment 계획

작업지시자 승인 및 실제 최종 CI 후, 식별 개선만 체리픽 수용했다는 점과 집중 3건 및
전체 회귀 결과, merge SHA, 실제 PR/devel CI 링크를 기록한다. 시각 증적은 해당 없음으로 적는다.
원 PR의 closing issue references는 조회 당시 비어 있었다. #6922는 차트 렌더링 문제가 남으므로
이 PR 수용만으로 전체 해결/close 처리하지 않는다. 승인된 UTF-8 body file comment를 한 번 게시하고
API로 body를 재조회한다. 원 fork branch는 보존한다.

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
