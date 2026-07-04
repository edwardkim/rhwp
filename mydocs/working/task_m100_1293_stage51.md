# Task 1293 Stage 51: issue_1139 focused 회귀 가드 복구

## 목적

Stage50에서 전체 sweep의 구분선 gap 지표를 보강한 뒤, 구현 계획서의 focused 검증을 다시
시작했다. `issue_1139_inline_picture_duplicate` 전체 테스트에서 기존 미주 회귀 테스트 일부가
실패했으므로, stage별 증상 보정이 기본 미주 설정을 침범한 지점을 좁혀 복구한다.

## 실패 증상

`cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 초회 실행에서 다음 항목이
실패했다.

- `issue_1139_endnote_spacing_reference_files_match_hancom_page_counts`
- `issue_1189_2022_oct_page11_endnote_question_gaps_match_pdf`
- `issue_1209_2022_nov_page14_question22_keeps_hancom_endnote_gap`
- `issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame`
- `issue_1274_2022_oct_page16_question30_title_keeps_first_line`

## 원인 분석

### 기본 7mm 미주 tail 회귀

`issue_1274_2022_oct_page16_question30_title_keeps_first_line`를 기준으로 bisect한 결과
`f1f4b28e task 1293: compact 미주 하단 FullParagraph overflow 보정` 이후 기본 7mm 미주에서도
`late_compact_text_tail_overflow_risk`가 과하게 적용됐다.

Stage 목적은 compact profile의 위험한 하단 텍스트 tail을 다음 column으로 넘기는 것이었지만,
기본 `미주 사이 7mm` + compact separator 아래 간격이 실제 한컴 기준으로 허용하는 단일 문항
제목 tail까지 넘겼다.

### Stage29 vpos rewind crossing 범위 회귀

`issue_1139_endnote_spacing_reference_files_match_hancom_page_counts`를 기준으로 bisect한 결과
`de648144 task 1293: 보이는 구분선 큰 미주 rewind fit 보정` 이후 `2024-09-between20` 계열
page count가 회귀했다.

Stage29 문서의 채택 범위는 page-bottom TAC picture/shape-only 사례였지만, 실제 조건은
`large_between_notes_gap_before_rewind && has_visible_endnote_separator`에 거의 전역으로 걸려
일반 텍스트 미주에서도 이전 content vpos crossing을 강제로 분리했다.

## 수정 방향

- `late_compact_text_tail_overflow_risk`의 첫 column 분리 조건은 큰 separator block, 비기본
  `미주 사이`, 기본 7mm의 문29 late-tail, 또는 기본 7mm의 큰 3줄 body tail에서만 적용한다.
- risk로 판정된 큰 body tail은 `split_endnote_to_fit`보다 column advance를 우선한다. 2022-10
  문28 본문처럼 line box가 큰 3줄 문단을 한 줄만 단 하단에 남기면 render line box overflow가
  발생하기 때문이다.
- 기본 compact 미주에서 `구분선 아래`가 작은 경우, 첫 column 하단의 단일 문항 제목 tail이
  line height 기준으로 허용 범위에 들어오고 column 하단 92% 이후인 경우 같은 column에 유지한다.
- Stage29의 `local_vpos_rewind_crosses_prev_content`는 보이는 구분선 + 큰 미주 사이에서
  column-top rewind를 제외하고, column 높이 22.5% 이후의 mid/tail crossing만 적용한다.

## 현재까지 통과한 focused 재현

- `cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_oct_page16_question30_title_keeps_first_line -- --exact --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1189_2022_oct_page11_endnote_question_gaps_match_pdf -- --exact --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame -- --exact --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1209_2022_nov_page14_question22_keeps_hancom_endnote_gap -- --exact --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_spacing_reference_files_match_hancom_page_counts -- --exact --nocapture`

## 남은 검증 계획

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo test --test issue_1050_footnote_serialize -- --nocapture`
- `cargo test --lib compact_endnote -- --nocapture`
- `cargo fmt --all -- --check`
- `git diff --check`
- 영향 대상 sweep:

```bash
rm -rf output/task1293_stage51_regression_sweep
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-09-below20 \
  --target 2022-10 \
  --target 2022-11-practice \
  --target 2024-11-practice-above0-between20-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --out output/task1293_stage51_regression_sweep \
  --rhwp-bin target/debug/rhwp
```

## 최종 검증 결과

### focused 테스트

- 통과: `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 52 passed
- 통과: `cargo test --test issue_1050_footnote_serialize -- --nocapture`
  - 7 passed
- 통과: `cargo test --lib compact_endnote -- --nocapture`
  - 29 passed
- 통과: `cargo fmt --all -- --check`
- 통과: `git diff --check`
- 확인: `rg -n "RHWP_ENDNOTE_STAGE51_DEBUG|STAGE51" src/renderer/typeset.rs || true`
  - 임시 디버그 문자열 없음

### 영향 대상 sweep

실행 전 `target/debug/rhwp`가 stale binary가 되지 않도록 `cargo build --bin rhwp`를 수행했다.

```bash
rm -rf output/task1293_stage51_regression_sweep_final
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-09-below20 \
  --target 2022-10 \
  --target 2022-11-practice \
  --target 2024-11-practice-above0-between20-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --out output/task1293_stage51_regression_sweep_final \
  --rhwp-bin target/debug/rhwp
```

핵심 게이트 결과:

| target | page count | overflow | frame | title | equation | order | separator drift |
|---|---:|---:|---:|---:|---:|---:|---:|
| `2024-09-between20` | 24/24/24 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-09-below20` | 23/23/23 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2022-10` | 18/18/18 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2022-11-practice` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |

참고 후보:

- `visual_metrics` 기준 red marker drift 후보 47건, line band drift 후보 58건,
  large ink region drift 후보 75건, content bottom drift 후보 43건은 남아 있다.
- 이번 stage51은 focused 회귀 복구와 renderer overflow/log 후보 제거가 목적이므로, 위 참고 후보는
  PR/후속 시각 검증에서 기존 sweep 후보로 계속 추적한다.

## 판단

Stage50 이후 focused 테스트를 다시 시작하면서 드러난 기본 7mm 미주 회귀와 large-between-note
overflow 회귀를 동시에 복구했다. 현재 evidence 기준으로 stage51 범위의 focused 테스트와 영향 대상
sweep 핵심 게이트는 통과했다.
