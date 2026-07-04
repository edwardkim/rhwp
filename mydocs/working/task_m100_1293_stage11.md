# Task 1293 Stage 11: shape987 기본 근방 미주 흐름 과다 분석

## 목적

Stage10에서 큰 구분선 블록 계열은 쪽수 정합을 맞췄지만,
`2024-11-practice-shape987`은 PDF 21쪽, SVG/render tree 22쪽으로 1쪽 과다 상태가 남았다.

이 샘플은 이름 기준 `구분선 위 9mm / 미주 사이 8mm / 구분선 아래 7mm`이며, 한컴 기본값에 가까운
미주 모양이다. 따라서 Stage10의 큰 위/아래 여백 분기처럼 20mm 계열에만 적용되는 보정으로는
해결하면 안 된다.

## 현재 관찰

Stage10 sweep 결과:

| target | PDF | SVG | render tree | 주요 후보 |
|---|---:|---:|---:|---|
| `2024-11-practice-shape987` | 21 | 22 | 22 | p11/p13/p17 frame overflow, p19 이후 line/red drift |

대표 산출물:

- `output/task1293_stage10_sample_check_all/2024-11-practice-shape987/contact_sheet.png`
- `output/task1293_stage10_sample_check_all/2024-11-practice-shape987/analysis/metrics.json`
- `output/task1293_stage10_sample_check_all/2024-11-practice-shape987/render_tree/`

## 분석 가설

1. 미주 separator block 자체가 큰 값은 아니므로, 구분선 위/아래 field 적용 방향이 아니라
   기존 compact/default 미주 flow cap이 너무 보수적으로 동작했을 수 있다.
2. p11/p13/p17의 overflow 후보가 실제 한컴처럼 다음 단으로 넘길 대상인지, 아니면 sweep의 frame
   검출 오탐인지 먼저 확인해야 한다.
3. p19 이후 대량 equation/text overlap 후보는 페이지가 하나 밀린 뒤 비교 축이 어긋난 결과일 수
   있으므로, 최초 분기 페이지를 찾아야 한다.
4. `betweenNotesMm=7.997`은 기본 7mm보다 약간 크지만 20mm 계열이 아니다. 새 미주 묶음 간격과
   구분선 아래 6.999mm가 중복 소비되는지 확인한다.

## 분석 결과

`output/task1293_stage11_sample_check_all` 기준으로 `shape987`은 PDF 21쪽, SVG/render tree 22쪽이었다.
첫 페이지 목록 차이는 12쪽 전후가 아니라 11쪽 하단에서 시작했다.

- Stage10 기준 `2024-11-practice-shape987` p12 오른쪽 단은 `pi=571` 일부만 올라가고, 이후 문항이 다음
  페이지로 밀렸다.
- 같은 문단 내부의 LINE_SEG vpos가 되감기며 단 하단에서 split 후보를 만들었는데, `advance_for_fit`으로
  새 단에 들어간 뒤에도 이전 단에서 계산한 `internal_rewind_split`이 남아 있었다.
- 이 stale split 때문에 새 단 첫 문단을 다시 쪼개 한컴보다 미주 흐름이 한 쪽 늦어졌다.
- split을 지우자 `shape987`은 21쪽으로 맞았지만, `구분선위20/미주사이0/구분선아래20`은 다시 22쪽으로
  밀렸다. 원인은 새 보정이 `미주사이=0` 같은 기본 이하 간격에도 적용된 것이었다.

따라서 Stage11 보정은 두 계열을 분리했다.

- `compact_endnote_between_notes_flow`: 기존처럼 기본 이하 미주 사이와, 구분선 위 여백에 흡수되는 기본 근방
  미주 사이를 모두 포함한다. 자체 vpos span fit처럼 넓은 compact 흐름 판단에 사용한다.
- `endnote_has_absorbed_between_notes_gap`: `미주사이 > 기본값`이면서 `구분선아래 <= 기본값`, `구분선위 > 0`,
  `미주사이 <= 구분선위`인 경우만 true로 본다. 이 값은 stale internal rewind split 제거와 조기 단 넘김
  임계값에만 사용한다.

이렇게 분리하면 `9/8/7` 샘플은 한컴처럼 21쪽으로 압축되고, `20/0/20`처럼 미주 사이가 0인 큰 구분선
샘플은 Stage10 흐름을 유지한다.

## 진행 계획

1. `shape987`의 page break 차이가 처음 발생하는 페이지를 `dump-pages`, render tree, compare PNG로
   찾는다.
2. 최초 분기 직전 단에서 미주 번호 묶음의 separator/current_height/advance_px 로그를 확인한다.
3. 같은 문서의 PDF 21쪽과 rhwp 21~22쪽 tail을 비교해, 과다 페이지가 실제 extra blank/tail인지
   중간부터 누적된 line height drift인지 분류한다.
4. 보정은 공식 미주 모양 필드와 공통 흐름 로직으로 설명되는 경우에만 적용한다.

## 검증 대기

- [x] `cargo fmt --all -- --check`
- [x] `python3 -m py_compile scripts/task1274_visual_sweep.py`
- [x] `cargo build --bin rhwp`
- [x] `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- [x] `2024-11-practice-shape987`, `2024-11-practice-above20-between0-below20` pair sweep
  - `shape987`: 21/21/21
  - `above20-between0-below20`: 21/21/21
- [x] 2024-11 신규 샘플 8종 쪽수 회귀 sweep

| target | PDF | SVG | render tree |
|---|---:|---:|---:|
| `2024-11-practice-shape987` | 21 | 21 | 21 |
| `2024-11-practice-above0-between0-below0` | 21 | 21 | 21 |
| `2024-11-practice-above0-between7-below2` | 21 | 21 | 21 |
| `2024-11-practice-above0-between7-below20` | 21 | 21 | 21 |
| `2024-11-practice-above0-between20-below2` | 22 | 22 | 22 |
| `2024-11-practice-above20-between0-below20` | 21 | 21 | 21 |
| `2024-11-practice-above20-between7-below2` | 21 | 21 | 21 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23 | 23 | 23 |

- [x] 2024-09 회귀 sweep
  - `2024-09-between20`: 24/24/24
  - `2024-09-below20-above20`: 23/23/23

대표 산출물:

- `output/task1293_stage11_pair_check/summary.json`
- `output/task1293_stage11_sample_check_all_v2/summary.json`
- `output/task1293_stage11_2024_09_regression/summary.json`
