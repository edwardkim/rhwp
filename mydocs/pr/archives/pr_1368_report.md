# PR #1368 처리 보고서 — 미주 높이 모델 SSOT 리팩터 v1

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1368 |
| 제목 | Task #1363: 미주 높이 모델 SSOT 리팩터 — 다단 미주 col0 본문 초과(#1357) 해소 |
| 작성자 | `planet6897` |
| 관련 이슈 | #1363, #1357, #1336 |
| PR base | `devel` (`430d5edc`) |
| 원 PR head | `41f348a` |
| 처리 기준 | `local/devel` |
| 통합 방식 | PR 커밋 6개 cherry-pick + maintainer fix + contributor 문서 archive 정리 |
| 리뷰 문서 커밋 | `eb54d714` |
| 정리 커밋 | `5a087891` |
| 처리 보고서 커밋 | `6b4d635b` |
| devel merge | `5c79fde9` |
| PR 처리 코멘트 | https://github.com/edwardkim/rhwp/pull/1368#issuecomment-4677353807 |
| PR close | `2026-06-11T05:05:45Z` |
| 처리 판정 | 조건부 수용 가능 |
| #1363 처리 | 즉시 close 보류 |
| 후속 PR | #1371 — 측정 SSOT A3 opt-in |

## 2. 처리 내용

작업지시자 리뷰 보고서 승인 후 PR #1368의 원 커밋 6개를 현재 `local/devel` 위에
cherry-pick했다.

원 PR 커밋:

```text
074ba3e8 Task #1363: 미주 높이 모델 SSOT 리팩터 — 다단 미주 col0 본문 초과(#1357) 해소
395e7344 Task #1363 v2: 후보 A(누적기↔렌더러 vpos 공유) 수행계획서
28cdd4e6 Task #1363 v2 Stage1: vpos_adjust 입력 분해 + 아키텍처 경로 확정
91811185 Task #1363 v2 Stage2: 미주 다단 누적 SSOT 시뮬레이션 배선 (A2)
e3eeeb41 Task #1363 v2 Stage3: fit/split 점진 SSOT 통합 음성 결과
41f348a4 Task #1363: cargo fmt 적용 (CI Format check #1368 수정)
```

devel 반영 커밋:

```text
6fba06f6 Task #1363: 미주 높이 모델 SSOT 리팩터 — 다단 미주 col0 본문 초과(#1357) 해소
74cedc0d Task #1363 v2: 후보 A(누적기↔렌더러 vpos 공유) 수행계획서
b257fe7b Task #1363 v2 Stage1: vpos_adjust 입력 분해 + 아키텍처 경로 확정
c7a6f748 Task #1363 v2 Stage2: 미주 다단 누적 SSOT 시뮬레이션 배선 (A2)
c3c527e4 Task #1363 v2 Stage3: fit/split 점진 SSOT 통합 음성 결과
20e8aa56 Task #1363: cargo fmt 적용 (CI Format check #1368 수정)
```

리뷰에서 지적한 maintainer fix를 적용했다.

- `src/renderer/typeset.rs`
  - `EnSsotLevel` 문서 주석의 기본값 설명을 실제 동작인 `B` 기준으로 정정
  - 상세 문서 경로를 archive 이동 후 경로로 정정

Contributor 작업 문서와 산출물은 PR 처리 문서와 분리하기 위해 archive로 이동했다.

- `mydocs/plans/archives/task_m100_1363.md`
- `mydocs/plans/archives/task_m100_1363_v2.md`
- `mydocs/working/archives/task_m100_1363_stage1.md`
- `mydocs/working/archives/task_m100_1363_stage2.md`
- `mydocs/working/archives/task_m100_1363_stage3.md`
- `mydocs/working/archives/task_m100_1363_stage4.md`
- `mydocs/working/archives/task_m100_1363_stage5.md`
- `mydocs/working/archives/task_m100_1363_v2_stage1.md`
- `mydocs/working/archives/task_m100_1363_v2_stage2.md`
- `mydocs/working/archives/task_m100_1363_v2_stage3.md`
- `mydocs/report/archives/task_m100_1363_report.md`
- `mydocs/report/archives/task1363_ssot_diff_stage3.tsv`
- `mydocs/report/archives/task1363_ssot_diff_stage4.tsv`

## 3. 변경 내용

`src/renderer/typeset.rs`:

- 미주 para 누적 경로에 `EnSsotLevel`과 `RHWP_EN_SSOT` 단계 플래그 추가
- 기본값 `B`에서 다음 두 divergence를 적용
  - 내부 vpos rewind para의 acc를 `line_advances_sum` 기반으로 보정
  - TAC 그림 미주 누적을 겹침 가정에서 순차 적층으로 전환
- `RHWP_EN_SSOT_DEBUG` 계측 추가
- A2 opt-in 실험 경로 `simulate_endnote_column_bottom_y()` 추가

`tests/issue_1082_endnote_multicolumn_drift.rs`:

- 대상 샘플 overflow 가드를 `60px`에서 `5px`로 강화

`scripts/task1363_ssot_diff.py`:

- `RHWP_EN_SSOT` 레벨별 overflow와 잔차 비교 하니스 추가

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

로컬 검증:

| 명령 | 결과 |
|---|---|
| PR 커밋 6개 cherry-pick | 통과, 충돌 없음 |
| `git diff --check origin/devel..HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture` | 통과, 5 passed |
| `python3 scripts/task1363_ssot_diff.py --level B --baseline legacy` | 통과, 대상 `3-09'24 sep20/20` overflow `0.0`, `Δoverflow -50.1` |
| `CARGO_INCREMENTAL=0 cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과, 72 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib` | 통과, 1644 passed / 0 failed / 6 ignored |
| `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings` | 통과 |
| `CARGO_INCREMENTAL=0 cargo check --lib --target wasm32-unknown-unknown -j 2` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과, `Done in 2m 03s` |

작업트리:

| 항목 | 결과 |
|---|---|
| `git status --short --branch` | 검증 직후 clean on `local/devel`; 이후 보고서와 주문서 문서 변경만 작성 |

## 5. 남는 범위

이번 PR은 대상 샘플의 page-height overflow 지표를 `50.1px -> 0.0px`로 개선한다. 다만
작업지시자 확인 중 `output/poc/pr1368-sep2020-default/3-09월_교육_통합_2024-구분선아래20구분선위20_022.svg`
에서 추가 잔여 문제가 확인되었다.

- `"한편, ..."` 문단은 render tree 기준 `pi=1156`, `단 0`에 배치
- page 22 body bottom은 `1092.3`, 해당 문단 baseline은 `y≈1100.6`
- debug overlay 기준 `s0:pi=1156 y=1090.3`은 body bottom 바로 위이며, 문단 높이 약 `18px`를
  고려하면 정상 조판에서는 다음 단으로 넘어가야 함
- 다음 문단 `pi=1157` `"이므로"`도 `y≈1118.6`에 남아 `LAYOUT_OVERFLOW 30.4px` 로그 발생
- 현재 엔진은 `pi=1158`에서야 단 전환하므로, 단 전환 판정이 최소 1~2문단 늦음
- 따라서 body bottom / 한컴 페이지네이션 정합은 아직 완전히 닫히지 않음

이 잔여 문제는 PR 내부 `task_m100_1363_v2.md`가 명시한 p17/p21 콘텐츠 단·페이지 배치 미해결
범위와 같은 성격이다. 작업지시자 확인 결과, 이 범위는 후속 PR #1371에서 다루는 것으로 보이므로
이번 PR #1368은 v1 부분 개선 범위로 완료 처리한다.

## 6. 판정

**조건부 수용 가능**.

이 PR은 미주 높이 모델의 일부 divergence를 안전하게 좁히고, 대상 샘플의 page-height overflow
지표를 제거한다. 대형 미주 회귀 테스트, lib 전체 테스트, clippy, WASM 빌드까지 통과했으므로
v1 부분 개선으로 수용할 수 있다.

단, 이 PR로 #1363 전체를 종결하면 안 된다. body bottom 기준 overflow와 p17/p21 콘텐츠 배치
잔여가 확인되므로, #1363은 유지한다. 해당 잔여는 후속 PR #1371 검토 범위로 넘긴다. #1357 close
상태도 이번 수용만으로 추가 변경하지 않는다.

## 7. 후속 절차

처리 완료:

- [x] `mydocs/pr/pr_1368_report.md` 및 주문서 갱신 커밋 — `6b4d635b`
- [x] `local/devel`을 `devel`에 no-ff merge — `5c79fde9`
- [x] `origin/devel` push — `5c79fde9`
- [x] PR #1368에 처리 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1368#issuecomment-4677353807
- [x] PR #1368 close — `2026-06-11T05:05:45Z`
- [x] #1363 close 보류, 잔여는 PR #1371에서 검토
