# PR #6914 검토

## 판정: 메인터너 보정 됨, 수용 가능

**메인터너 보정 적용, 테스트 통과 및 원 PR이 보고한 3문서 악화의 A/B 해소 확인을 완료했다.**
목표 문서의 출처 줄 복원을 유지하면서 악화 보고 3문서에서 대조군 대비 새 이상 신호가 없음을
확인했다. 이 판정은 검토 범위의 수용 가능 판정이며, GitHub approve/merge 승인 또는
10,000문서 코퍼스 전체의 무회귀를 뜻하지 않는다.

## 원 PR과 보정

- 원 PR: https://github.com/edwardkim/rhwp/pull/6914
- 이슈: https://github.com/edwardkim/rhwp/issues/6900
- 원 head: `fd2bfdb38e35921b46033c0fea4a88f0794d47d8`; 로컬 체리픽: `a23236713`.
- 원 PR 사전 리뷰어는 `jangster77`로 지정했다.
- CI 그린 제한의 예외 포함은 사용자 지시다. 실패를 성공으로 분류해 포함한 것이 아니다.

원 조건 `next_ladder_y <= y_offset + 0.5`는 저장 위치가 표 하단보다 임의로 멀리 위에
있어도 후행 간격을 제거한다. 중복 간격이라는 근거가 있는 경우로 한정하기 위해
`(next_ladder_y - y_offset).abs() <= 0.5`로 좁혔다. 순방향 저장 위치 조건과 기존 환산
허용치는 유지했고, 파일명 예외나 baseline 상향은 넣지 않았다.
출처 줄 위치 및 표 마지막 두 행 음성 대조 2건이 모두 통과했다.
테스트 파일 도입부의 '한쪽 방향' 설명은 원 PR 설명이며 보정 후 실제 조건은 양방향 일치다.

## 시각 검토와 원격 CI 실패 구분

[직접 시각 대조 기록](pr_6914_6918_visual_sweep.md)의 4쪽에서 출처 줄은 본문 안에 있고
꼬리말 로고에 가리지 않는다. 표 마지막 행도 유지된다. 자동 후보 0/1쪽이며, pixel match
84.04956%, ink/proxy 45.34051%다. 글꼴·굵기·자간 등 다른 차이는 남으므로 전체 fidelity
통과라는 뜻이 아니다.

[원 Render Diff 실패](https://github.com/edwardkim/rhwp/actions/runs/34265493828/job/102193756594)는
한컴 PDF가 아니라 KTX legacy/layer 비교였다. 원 artifact의 표 경계 차이는 12,049/891,662픽셀
(1.35130%, 최대 채널 차이 36)이었다. 최신 로컬 WASM에서는 242/891,662픽셀
(0.02714%, 최대 채널 차이 162)로 기존 0.05% 비율 gate를 통과했다.
다른 두 문서는 차이 0이었다. Mac Chrome과 과거 Linux CI의 환경 차이가 있으므로 이 결과만으로
원격 실패의 원인이 오탐이었다거나 원격 CI까지 해소됐다고 단정하지 않는다.

## 보류 사유 해소: 악화 보고 3문서의 독립 A/B

기여자가 보고한 10,000건 A/B에서 악화됐던 3개 원본을 Mac의
`/Users/tsjang/Downloads/korea_downloads/prism_downloads/`에서 찾아 직접 대조했다.
추가 제품 코드 수정 없이 기존 양방향 일치 조건 보정의 효과를 확인했다.

| 문서 식별자 | 전수 쪽수 A / B | overflow A / B | overlap A / B | text-overlap A / B | 원 악화 쪽 SVG |
| --- | --- | --- | --- | --- | --- |
| 1051000-201800093 | 158 / 158 | 19 / 19 | 0 / 0 | 0 / 0 | 1쪽 바이트 동일 |
| 1051000-201700077 | 113 / 113 | 29 / 29 | 1 / 1 | 8 / 8 | 75쪽 바이트 동일 |
| 1411000-201000028 | 137 / 137 | 16 / 16 | 0 / 0 | 7 / 7 | 128쪽 바이트 동일 |

- A: 통합 HEAD `3d382f53f9167a9d18b74bfc07820e4eeb6d5700`에서 #6914 체리픽
  `a23236713`만 임시 worktree에서 `revert --no-commit`한 대조군. #6918·#6934·#6937은 유지했다.
- B: 같은 통합 HEAD 및 미커밋 양방향 일치 조건 보정. 이전 전체 회귀와 직접 시각 대조에
  사용했던 바이너리를 복사해 고정했다.
- A 바이너리 SHA-256: `3ac883b7548f9e7bc6cc1782f99ca075434342d5b15b2ae435e10f7334e584c4`.
- B 바이너리 SHA-256: `dadf5abe13b441aa9379d5f8cc61846b676e67abe431f1ca5c27ddbaf7638972`.
- A layout blob: `ba6162370ea1222bfc5ad379ce79e5fd2a998170`.
- B layout blob: `d65d0fd850972913ca77955c2d0ec5f49821a3fa`.
- **3문서 전체 408쪽의 `layout-anomaly --json` 결과가 객체 전체로 동일**했다.
  총량만 같아서 개별 악화가 상쇄된 것으로 판정하지 않았다.
- off-canvas는 각 문서 A/B 모두 0이다. empty-page는 앞 두 문서 0/0, 마지막 문서 1/1로
  기존 신호가 유지된다. 기존 overflow/overlap/text-overlap을 정상 또는 해결로 덮어쓰지 않는다.
- 악화 보고 1·75·128쪽은 `export-svg --font-style -p <0-based>` 결과도 바이트 단위로
  동일했다. 이 검증은 한컴 정본과의 일치 검증이 아니라 해당 변경의 악화 방지 검증이다.
- #6900 목표 4쪽의 출처 줄 및 표 행 유지 집중 테스트, 직접 PDF 대조, 전체 회귀
  9,337건은 동일한 B 코드 상태의 앞선 검증 결과를 유지한다. 이번에 전체 회귀를 다시
  실행했다고 기록하지 않는다.

원본과 대상 쪽 SVG의 SHA-256:

| 문서 | 원본 SHA-256 | A/B 공통 대상 쪽 SVG SHA-256 |
| --- | --- | --- |
| 1051000-201800093 | `2c31518d8b7b33b4e374e540fc7c4ac7833c42b9e68831b76d34af8c0df5fb54` | `843a7bfaff279b7223ff818e3a836b73a4c50d7c1f1b1de22685a3b413f1ff07` |
| 1051000-201700077 | `44f15e3758c62940f843058864f02b9e8049d7b63c90bda042a8b47d1bb60392` | `b082761817e541acb22eb3d4cb4af88de200955221fd1a12098bc3ba3a4184b3` |
| 1411000-201000028 | `4b7fe3bfe2fda07f513d40cb65101232a772e39adb3520402a8097c793a2bdaf` | `3b1d9e567150c0e1aa2e5bd9398eb372068dc08aeff75c9e96b76a8896a16594` |

실행은 저장소 루트에서 같은 기본 허용치(overflow 1px, overlap 2px)를 사용했다.

```bash
/tmp/rhwp-6914-ab-20260909/base-rhwp layout-anomaly "<원본문서>" --json
/tmp/rhwp-6914-ab-20260909/candidate-rhwp layout-anomaly "<원본문서>" --json
/tmp/rhwp-6914-ab-20260909/base-rhwp export-svg "<원본문서>" -p <0|74|127> --font-style -o <A-output>
/tmp/rhwp-6914-ab-20260909/candidate-rhwp export-svg "<원본문서>" -p <0|74|127> --font-style -o <B-output>
```

6개 전수 anomaly 실행과 6개 지정 쪽 SVG export는 모두 정상 종료했다.
입력 목록·A/B JSON·SVG·대조군 재구성 patch는 `/tmp/rhwp-6914-ab-20260909`에만 두며
커밋하지 않는다. 사용한 임시 대조 worktree는 이 작업의 변경만 되돌린 뒤 제거했다.

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

최신 통합 PR/devel CI 및 작업지시자 승인 전에는 comment/close하지 않는다. 승인 뒤
체리픽 수용 사실, 별도 메인터너 보정, 실제 CI, merge SHA 및 위 3문서 408쪽 A/B 결과를 기록한다.
비교 범위는 4쪽 1개이며 [시각 기록](pr_6914_6918_visual_sweep.md)의 수치와 한계를 함께 적는다.
다음 이미지를 merge SHA로 고정해 코멘트에서 직접 표시한다.

`![#6914 4쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6914_6937_20260909/pr6914-p004.png)`

UTF-8 body file로 승인 후 한 번 게시하고 API로 body를 재조회한다. 원 fork branch는 보존한다.

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
