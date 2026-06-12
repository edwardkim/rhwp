# PR #1362 검토 — 미주 제목 앞 gap 이중계상 정정 v2

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1362 |
| 작성자 | `planet6897` |
| 상태 | open / draft 아님 |
| base | `devel` |
| head | `planet6897:fix/endnote-title-gap-double-1355-v2` |
| 관련 이슈 | #1355 |
| 변경 규모 | 4 files, +213 / -0 |
| mergeable | `MERGEABLE`, `BEHIND` |

## 2. 변경 요약

해설(미주) 영역에서 문제 제목 앞 여백이 이중으로 계산되어 문항 본문이 아래로 드리프트하던 문제를 정정한다.

대상 샘플:

- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`

핵심 변경:

- `src/renderer/layout.rs`
  - 미주 제목 배치에서 `vpos_adjust` 직후 보정 조건 추가
  - 직전 미주 문단이 textless, 즉 수식 전용 tail이고
  - `flow_advance`가 기존 미주 gap과 거의 같으며
  - 현재 제목의 saved LINE_SEG vpos가 직전 bottom보다 크게 점프할 때만
  - 제목 y를 흐름 위치(`y_before_vpos`)로 되돌려 gap을 한 번만 남김
- `tests/issue_1355_endnote_title_gap_double.rs`
  - p18 좌측 첫 미주 제목 baseline y가 이중계상 위치로 회귀하지 않는지 가드
- `mydocs/plans/task_m100_1355.md`
- `mydocs/report/task_m100_1355_report.md`

PR 본문상 v1(PR #1356)은 `flow_advance >= gap` 단독 게이트로 정상 gap과 이중계상을 구분하지 못해 회귀를 만들었고, v2는 `textless + saved-vpos jump`를 추가 판별 신호로 사용한다.

## 3. GitHub 상태

GitHub Actions:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL Analyze (javascript-typescript) | pass |
| CodeQL Analyze (python) | pass |
| CodeQL Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

PR 대화:

- 추가 코멘트 없음

관련 이슈:

- #1355는 open 상태
- PR 본문에 `closes #1355`가 있어 일반 merge면 자동 close 가능성이 있으나, 프로젝트 처리 관례상 close 여부는 후속 확인 필요

## 4. 로컬 검토 방식

검토 기준:

```text
origin/devel @ 6cc5f0d8
PR head     @ 5067f3f9
```

로컬 브랜치:

```text
local/pr1362-upstream   @ 5067f3f9
local/pr1362-merge-test @ d85d9c49
```

적용 방식:

- `origin/devel` 기준 검증 브랜치 생성
- PR 단일 커밋 `5067f3f9` cherry-pick
- 충돌 없음

## 5. 로컬 검증

실행 완료:

```bash
git diff --check origin/devel..HEAD
cargo fmt --all -- --check
cargo test --test issue_1355_endnote_title_gap_double
cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20
cargo test --test issue_1139_inline_picture_duplicate issue_1189_2022_oct_page11_endnote_question_gaps_match_pdf -- --exact
cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame -- --exact
cargo test --test issue_1139_inline_picture_duplicate issue_1284_2022_oct
cargo test --test issue_1139_inline_picture_duplicate issue_1284_2022_sep_page17_question27_starts_at_pdf_top -- --exact
cargo test --lib
cargo clippy -- -D warnings
cargo test --test svg_snapshot
```

결과:

| 명령 | 결과 |
|---|---|
| `git diff --check` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| 신규 `issue_1355` 테스트 | 통과, 1 passed |
| `issue_1284_2024_between20` | 통과, 5 passed |
| 2022 oct/sep 미주 tail 관련 필터 | 통과, 7 passed |
| `cargo test --lib` | 통과, 1622 passed / 0 failed / 6 ignored |
| `cargo clippy -- -D warnings` | 통과 |
| `cargo test --test svg_snapshot` | 통과, 8 passed |

시각 판정용 SVG:

- `output/poc/pr1362-endnote-gap/3-09월_교육_통합_2024-구분선아래20구분선위20_018.svg`
- `output/poc/pr1362-endnote-gap/3-09월_교육_통합_2024-구분선아래20구분선위20_019.svg`

## 6. 검토 의견

차단 이슈는 발견하지 못했다.

수용 가능한 이유:

- PR이 수정하는 runtime 코드는 `layout.rs`의 미주 제목 배치 보정에 한정된다.
- 기존 `current_is_endnote_question_title`, `endnote_flow`, bottom-fit, backtrack, preserved-gap 조건 뒤에 추가 게이트가 붙어 영향 범위가 좁다.
- v1 회귀로 지목된 케이스를 의식해 `flow_advance` 단독이 아니라 `prev textless + saved_delta_hu > 5000`을 함께 요구한다.
- 신규 가드가 실제 문제 페이지 p18의 제목 y 위치를 직접 감시한다.
- 기존 미주/수식 tail 회귀군과 `svg_snapshot`, `cargo test --lib`, clippy가 모두 통과했다.

주의점:

- `saved_delta_hu > 5000`은 계측 기반 임계값이다. 조건을 좁게 묶었으므로 당장 차단 사유는 아니지만, 추후 다른 미주 문서에서 textless tail + 큰 saved-vpos jump가 정상 gap인 사례가 나오면 재검토가 필요하다.
- contributor 문서가 활성 `mydocs/plans`, `mydocs/report`에 추가된다. 수용 시 archive 정책에 맞춰 이동할지 결정해야 한다.
- PR은 `BEHIND` 상태이나 최신 `origin/devel` 기준 cherry-pick은 충돌 없이 적용됐다.

## 7. 권장 처리

권장: **수용 가능**.

권장 절차:

1. 작업지시자 시각 판정.
2. 시각 판정 통과 시 `local/devel`에 PR 커밋 반영.
3. contributor 작업 문서 archive 정리 여부 결정.
4. 최종 검증:
   - `git diff --check`
   - `cargo fmt --all -- --check`
   - `cargo test --test issue_1355_endnote_title_gap_double`
   - `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20`
   - `cargo test --test svg_snapshot`
5. 처리 보고서 작성.
6. `origin/devel` push.
7. PR #1362에 메인테이너 코멘트 작성 후 close.
8. Issue #1355 close 여부 확인 후 필요 시 수동 close.

## 8. 승인 요청

위 검토 결과 기준으로 PR #1362 수용 절차를 진행해도 되는지 승인 요청한다.
