# PR #1354 처리 보고서 — 수식 PUA 조건부 막대 U+E04D 매핑

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1354 |
| 작성자 | `planet6897` |
| 관련 이슈 | #1343 |
| 검토 브랜치 | `local/pr1354-upstream` |
| 통합 방식 | 현재 `local/devel` 기준 PR 단일 커밋 cherry-pick + contributor 문서 archive 정리 |
| 원 PR head | `24156ad0` |
| 반영 커밋 | `e34e4d22` |
| 문서 정리 커밋 | `4af2fc7c` |
| devel merge | `b6c3277c` |
| PR close | `2026-06-10T02:34:04Z` |
| Issue #1343 close | `2026-06-10T02:34:23Z` |

## 2. 처리 내용

PR #1354의 본질 커밋 `24156ad0`을 현재 `local/devel` 위에 cherry-pick했다.

반영 커밋:

- `e34e4d22` — `Task #1343: 수식 PUA 조건부 막대(U+E04D) → '|' 매핑`

Contributor 작업 문서는 프로젝트 PR 처리 문서와 분리하기 위해 archive로 이동했다.

- `mydocs/plans/archives/task_m100_1343.md`
- `mydocs/plans/archives/task_m100_1343_impl.md`
- `mydocs/report/archives/task_m100_1343_report.md`
- `mydocs/working/archives/task_m100_1343_stage2.md`
- `mydocs/working/archives/task_m100_1343_stage3.md`

## 3. 변경 내용

- `src/renderer/equation/symbols.rs`
  - 수식 도메인용 PUA 매핑 테이블 `EQUATION_PUA` 추가
  - `U+E04D`를 표준 조건부 막대 `"|"`로 매핑
  - `lookup_equation_pua(ch)` 조회 함수 추가
- `src/renderer/equation/tokenizer.rs`
  - 기존 non-ASCII `Text` 폴백 직전에 PUA 매핑 조회 추가
  - 매핑된 PUA는 기존 `|`와 같은 `TokenType::Symbol` 토큰으로 처리
  - `test_pua_conditional_bar` 회귀 테스트 추가

매핑이 없는 PUA 문자는 기존 non-ASCII `Text` 폴백을 유지하므로 알 수 없는 PUA의 동작은 바뀌지 않는다.

## 4. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

Cherry-pick 후 로컬 검증:

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `git diff --check HEAD~2..HEAD` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib test_pua_conditional_bar -- --nocapture` | 통과, 1 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib renderer::equation::tokenizer -- --nocapture` | 통과, 37 passed |
| `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과, `Done in 1m 52s` |

SVG 확인:

| 항목 | 결과 |
|---|---|
| 명령 | `cargo run --bin rhwp -- export-svg 'samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx' -o output/poc/pr1354-equation-pua -p 15` |
| 산출물 | `output/poc/pr1354-equation-pua/3-09월_교육_통합_2024-구분선아래20구분선위20_016.svg` |
| `U+E04D`/두부 문자 잔존 | 0건 |
| 조건부 막대 `|` text node | 2건 (`P(A|B)`, `P(B|A)` 위치) |
| 작업지시자 시각 판정 | 통과 |

## 5. 판정

**수용 가능**.

이번 변경은 HWP 수식 스크립트에 PUA로 저장된 조건부 확률 막대를 수식 tokenizer 단계에서 표준
기호로 정규화하는 좁은 수정이다. 수식 렌더 파이프라인 안에서만 적용되고, 생성 토큰은 기존 `|`
기호 처리 경로를 그대로 재사용한다.

대상 샘플 16페이지 SVG에서 `U+E04D`와 두부 문자가 사라지고 조건부 막대 `|`가 생성됨을 확인했다.
작업지시자 시각 판정도 통과했다. GitHub Canvas visual diff와 로컬 WASM 빌드도 통과했으므로
rhwp-studio 경로 수용 리스크도 낮다고 본다.

남는 주의점은 non-ASCII 연속 문자열 중간에 PUA가 섞인 특수 입력이다. 이번 이슈의 수식 스크립트는
Latin/명령어 중심이라 영향이 없고, 향후 PUA 매핑 테이블이 늘어날 경우 Text 루프 안에서도 PUA를
끊는 보강을 후속으로 검토할 수 있다.

## 6. 후속 절차

처리 완료:

- [x] `mydocs/pr/pr_1354_report.md` 및 주문서 갱신 커밋 — `ef7e547c`
- [x] `local/devel` → `devel` no-ff merge — `b6c3277c`
- [x] `origin/devel` push — `b6c3277c`
- [x] PR #1354에 처리 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1354#issuecomment-4665923360
- [x] PR #1354 close — `2026-06-10T02:34:04Z`
- [x] Issue #1343 close — `2026-06-10T02:34:23Z`
