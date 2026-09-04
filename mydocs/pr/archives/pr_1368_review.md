# PR #1368 검토 — 미주 높이 모델 SSOT 리팩터 v1

- PR: https://github.com/edwardkim/rhwp/pull/1368
- 제목: Task #1363: 미주 높이 모델 SSOT 리팩터 — 다단 미주 col0 본문 초과(#1357) 해소
- 작성일: 2026-06-11
- 작성자: `planet6897`
- 관련 이슈: #1363, #1357, #1336
- base: `devel` (`430d5edc`)
- head: `planet6897:local/task1363` (`41f348a`)
- 검토 브랜치: `local/pr1368-upstream`
- 최신 devel 적용 검증 브랜치: `local/pr1368-merge-test`

## 1. 요약 판단

**조건부 수용 가능**으로 판단한다.

PR은 미주 다단 누적 경로에서 `compute_en_metrics()`와 실제 렌더 경로의 divergence 중
안전하게 분리 가능한 A/C만 기본 적용한다. 대상 샘플의 page-height overflow 지표는
legacy 대비 `50.1px -> 0.0px`로 내려가고, 미주 질문 흐름 회귀군 72개 및 lib 전체 테스트도
통과했다. 변경 방향과 검증 밀도는 수용 가능한 수준이다.

단, 이 PR을 **#1363 전체 종결**로 보기는 어렵다. PR 내부 v2 계획서도 p17/p21 콘텐츠 단·페이지
배치가 미해결이라고 명시하고, 로컬 export에서도 body bottom 기준 `LAYOUT_OVERFLOW` 로그가
아직 남는다. 따라서 수용 시에는 "v1 부분 개선 수용"으로 처리하고 #1363은 유지하거나 후속 이슈로
분리하는 것이 안전하다.

수용 전 또는 수용 커밋에서 정리할 점:

- `src/renderer/typeset.rs`의 `EnSsotLevel` 주석이 "미설정 시 legacy"라고 되어 있으나 실제 기본값은 `B`
- contributor 작업 문서와 TSV 산출물은 archive로 이동
- #1363 close 여부는 보류

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `MERGEABLE` |
| mergeStateStatus | `BEHIND` |
| 변경량 | 16 files, +8371 / -69 |
| 작성자 | `planet6897` |
| closing issues | 없음 |

커밋:

- `074ba3e8` — Task #1363: 미주 높이 모델 SSOT 리팩터
- `395e7344` — Task #1363 v2 수행계획서
- `28cdd4e6` — Task #1363 v2 Stage1
- `91811185` — Task #1363 v2 Stage2 A2 시뮬레이션 배선
- `e3eeeb41` — Task #1363 v2 Stage3 음성 결과
- `41f348a4` — cargo fmt 적용

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

## 3. 변경 검토

### 3.1 코드 변경

`src/renderer/typeset.rs`:

- `EnSsotLevel`과 `RHWP_EN_SSOT` 단계 플래그 추가
  - `legacy/off`: 기존 누적 경로
  - `A`: 내부 vpos rewind para의 acc를 `line_advances_sum` 기반으로 보정
  - `B`: 기본값, A + TAC 그림 미주 순차 적층
  - `on`: 현재 B와 동일한 예약 tier
  - `A2`: opt-in 실험 tier
- `RHWP_EN_SSOT_DEBUG` 계측 추가
- `compute_en_metrics()`에서 내부 rewind 문단의 acc를 SSOT 방향으로 보정
- TAC 그림 미주 경로에서 기존 `max(rewind_start + adv)` 겹침 가정을 B 이상에서 `+= adv` 순차 적층으로 전환
- A2 opt-in용 `simulate_endnote_column_bottom_y()` 추가

`tests/issue_1082_endnote_multicolumn_drift.rs`:

- sep20/20 대상 샘플 가드를 기존 `REG_LIMIT 60px`에서 `SEP2020_TIGHT 5px`로 강화

`scripts/task1363_ssot_diff.py`:

- `RHWP_EN_SSOT` 레벨별 overflow/잔차 측정 하니스 추가

### 3.2 개선 효과

로컬 하니스 결과:

```text
=== RHWP_EN_SSOT=B ===
=== baseline RHWP_EN_SSOT=legacy ===

exam                          overflow  |ssot_res|  rewind#  Δoverflow
3-09'23 hwp                        0.0       431.1        8       +0.0
3-09'23 hwpx                       0.0       431.1        8       +0.0
3-09'22 hwp                        0.0       244.4        5       +0.0
3-10'22 hwp                        0.0       155.6        3       +0.0
3-11'22 hwp                        0.0       309.7        7       +0.0
3-09'24 sep20/20 (대상)              0.0       239.6        8      -50.1
```

이 지표 기준으로는 대상 샘플의 page-height overflow가 제거된다.

### 3.3 남는 범위

대상 샘플 직접 export:

```text
cargo run --quiet --bin rhwp -- export-svg samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp -o /private/tmp/pr1368-sep2020-default
```

결과:

- 23페이지 SVG 내보내기 성공
- 다만 body bottom 기준 로그는 잔존
  - `LAYOUT_OVERFLOW: page=21 ... para=1156 ... overflow=10.0px`
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=1157 ... overflow=30.4px`
  - `LAYOUT_OVERFLOW: page=21 ... para=1157 ... overflow=30.4px`

추가 확인:

- 시각 판정 SVG:
  `output/poc/pr1368-sep2020-default/3-09월_교육_통합_2024-구분선아래20구분선위20_022.svg`
- debug overlay:
  `output/poc/pr1368-sep2020-debug/3-09월_교육_통합_2024-구분선아래20구분선위20_022.svg`
- render tree:
  `output/poc/pr1368-sep2020-render-tree/render_tree_022.json`
- 사용자 지적 위치의 `"한편, ..."` 문단은 render tree 기준 `pi=1156`, `단 0`에 배치된다.
  SVG 좌표도 `x≈34.0, y≈1100.6`이므로 페이지 내부 좌표 기준으로는 오른쪽 단이 아니라
  왼쪽 단이다.
- page 22 body area는 `y=90.7, h=1001.6`으로 body bottom이 `1092.3`이다. 따라서
  `"한편, ..."` baseline은 body bottom을 약 `8~10px` 초과한다. 다음 문단 `pi=1157`
  `"이므로"`도 `y≈1118.6`에 남아 overflow `30.4px` 로그가 발생한다.
- debug overlay의 `s0:pi=1156 y=1090.3`은 body bottom `1092.3` 바로 위다. 문단 자체가
  약 `18px` 높이를 소비하므로 정상 조판이라면 `pi=1156`은 다음 단으로 넘어가야 한다. 현재
  결과는 `pi=1158`에서야 단 전환이 발생하므로, 단 전환 판정이 최소 1~2문단 늦다.
- `RHWP_EN_SSOT_DEBUG=1` 기준 누적기는 `pi=1156`을 `978.7 + 18.0 = 996.7`로 판단해
  available `1001.6` 안에 들어간다고 본다. 그러나 실제 draw 좌표는 body bottom을 넘는다.
  `pi=1157`은 `996.7 + fit 14.4 > 1001.6`인데도 같은 단에 남는다.

즉 PR의 "overflow 0"은 `scripts/task1363_ssot_diff.py`와 `issue_1082` 테스트가 쓰는
page-height overflow 지표 기준이다. body bottom / 한컴 페이지네이션 정합 전체가 닫혔다고
읽으면 안 된다.

PR 내부 `task_m100_1363_v2.md`도 다음 미해결 범위를 명시한다.

- p17 `pi=894` "C×C" 단 배치
- p21 `pi=1127` 위치
- fit/split 결정의 holistic 재작성 필요

## 4. 로컬 검증

최신 `local/devel` 기준 검증 브랜치: `local/pr1368-merge-test`

| 명령 | 결과 |
|---|---|
| PR 커밋 6개 cherry-pick | 통과, 충돌 없음 |
| `git diff --check local/devel..HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture` | 통과, 5 passed |
| `python3 scripts/task1363_ssot_diff.py --level B --baseline legacy` | 통과, 대상 `-50.1` |
| `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과, 72 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib` | 통과, 1644 passed / 0 failed / 6 ignored |
| `CARGO_INCREMENTAL=0 cargo check --lib --target wasm32-unknown-unknown -j 2` | 통과 |
| 대상 샘플 `export-svg` | 통과, 23 SVG 생성, body overflow 로그 잔존 |

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| 미주 다단 누적 경로 변경 | 중간~높음 | typeset core 경로. 다만 A/C만 기본 적용하고 대형 회귀 테스트 통과 |
| #1363 전체 종결 오판 | 높음 | v2 문서와 export 로그상 body/page 배치 잔여 존재 |
| `EnSsotLevel` 주석 불일치 | 낮음 | 주석은 legacy 기본이라고 하나 실제 기본은 B |
| A2 실험 경로 포함 | 낮음~중간 | opt-in이지만 알려진 실패 실험 코드가 함께 들어옴. 기본값에는 영향 없음 |
| contributor 문서 활성 폴더 잔류 | 중간 | 수용 시 archive 이동 필요 |

## 6. 권장 수용 절차

작업지시자 승인 후:

1. PR 커밋 6개를 `local/devel`에 cherry-pick
2. `EnSsotLevel` 주석의 기본값 설명을 `B` 기준으로 정정하는 maintainer fix 적용
3. contributor 작업 문서와 TSV 산출물을 archive로 이동
4. 검증
   - `cargo fmt --check`
   - `git diff --check`
   - `CARGO_INCREMENTAL=0 cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture`
   - `CARGO_INCREMENTAL=0 cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
   - `CARGO_INCREMENTAL=0 cargo test --lib`
   - `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings`
   - `CARGO_INCREMENTAL=0 cargo check --lib --target wasm32-unknown-unknown -j 2`
5. 처리 보고서 작성
6. 승인 시 `devel` no-ff merge, push, PR #1368 close
7. #1363은 즉시 close하지 말고 작업지시자 판단에 따라 유지 또는 후속 이슈 분리

## 7. 승인 요청

위 검토 결과 기준으로 PR #1368을 "v1 부분 개선" 범위로 조건부 수용 절차를 진행해도 되는지
승인 요청한다.
