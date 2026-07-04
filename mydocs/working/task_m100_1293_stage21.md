# Task 1293 Stage 21: 큰 구분선 잔여 overflow 문단 이월 분석

## 목적

Stage20에서 `2024-11-practice-above20-between0-below20`의 쪽수와 frame/equation/title/order
후보는 맞췄지만, PDF와 직접 비교하면 p15/p18/p20의 문항 흐름이 아직 크게 다르다.
미주는 수치 보정으로 억지로 압축하면 다시 overwrap이 생기므로, 이번 단계에서는
`LAYOUT_OVERFLOW` 로그를 단순 제거하지 않고 실제 PDF 흐름과 비교해 공식 미주 모양
공통 로직에서 남은 drift를 줄인다.

## 기준 산출물

- `output/task1293_stage20_target_after_cap_narrow/summary.json`
- `output/task1293_stage20_target_after_cap_narrow/2024-11-practice-above20-between0-below20/analysis/metrics.json`
- `output/task1293_stage20_target_after_cap_narrow/2024-11-practice-above20-between0-below20/compare/compare_015.png`
- `output/task1293_stage20_target_after_cap_narrow/2024-11-practice-above20-between0-below20/compare/compare_018.png`
- `output/task1293_stage20_target_after_cap_narrow/2024-11-practice-above20-between0-below20/compare/compare_020.png`
- `output/task1293_stage20_target_after_cap_narrow/2024-11-practice-above20-between0-below20/render_tree/render_tree_015.json`
- `output/task1293_stage20_target_after_cap_narrow/2024-11-practice-above20-between0-below20/render_tree/render_tree_018.json`
- `output/task1293_stage20_target_after_cap_narrow/2024-11-practice-above20-between0-below20/render_tree/render_tree_020.json`

## 현재 관찰

- 대상 문서의 설정은 `구분선 위=20mm`, `미주 사이=0mm`, `구분선 아래=20mm`이다.
- page count는 SVG/PDF/render tree 모두 `21/21/21`로 맞다.
- frame/equation/title/order 후보는 비어 있지만, 다음 overflow 로그가 남아 있다.
  - 0-based page=14, 실제 표시 15쪽 col1 `pi=753` partial paragraph, 최대 `82.4px` overflow
  - 0-based page=17, 실제 표시 18쪽 col0 `pi=875`, `pi=876`, 최대 `40.5px` overflow
  - 0-based page=19, 실제 표시 20쪽 col0 `pi=966`, `pi=967`, 최대 `46.2px` overflow
- 이 로그는 frame 밖 픽셀 검출에서는 약하게 보이거나 누락될 수 있지만, 렌더러 내부 기준으로는
  실제 문단이 단 하단을 넘긴 상태다.
- compare PNG 기준으로도 p15/p18/p20은 PDF와 문항/그림 배치가 다르다.
  - p15: `문29/문30` 위치와 풀이 본문 분배가 PDF와 다르다.
  - p18: rhwp는 `문30` 풀이가 길게 남고, PDF는 같은 쪽에서 `문23~문25`까지 진행한다.
  - p20: rhwp는 큰 그림/문29/문30 분배가 PDF와 다르며 large ink region drift가 크다.
- `dump-note-shape` 기준 target raw 값은 파일명과 일치한다.
  - `separatorAbove=19.999mm`, `betweenNotes=0.0mm`, `separatorBelow=19.999mm`
  - 따라서 현재 남은 차이는 설정 파싱 누락보다 미주 flow/vpos/line-spacing 정합 문제로 본다.

## Stage21 진행 메모

- `height_cursor.rs`에서 `betweenNotes=0`인 compact 미주 새 문항 제목은 저장 vpos가 만드는 큰
  forward gap을 쓰지 않고 직전 문단 뒤의 순차 위치를 유지하도록 보정했다.
  - 확인 로그: p15 `pi=692`가 `y_in=249.04`, `result=249.04`, `compact_new_note=true`로
    처리되어 Stage20의 큰 제목 gap이 제거된다.
  - 검증: `cargo test compact_endnote_zero_between_question_title_caps_forward_gap --lib`,
    `cargo build --bin rhwp`, target sweep 수행.
- target sweep 결과 page count는 `21/21/21`로 유지되고 frame/title/order/equation 후보는 비어 있다.
- 최종 산출물:
  - `output/task1293_stage21_title_gap_final_sweep/summary.json`
  - `output/task1293_stage21_title_gap_final_sweep/2024-11-practice-above20-between0-below20/compare/compare_015.png`
  - `output/task1293_stage21_title_gap_final_sweep/2024-11-practice-above20-between0-below20/compare/compare_018.png`
  - `output/task1293_stage21_title_gap_final_sweep/2024-11-practice-above20-between0-below20/compare/compare_020.png`
- 다만 p15 `pi=753` partial paragraph overflow는 남아 있다.
  - render tree 기준 body bottom은 `1092.3px`, `pi=753` 첫 줄 시작은 `1090.5px`라 정상 줄 높이로는
    들어가지 않는다.
  - typeset은 이 문단을 `lines=0..5`로 p15에 남기고 `lines=5..15`를 p16으로 넘긴다.
  - 이 차이는 단순 `remaining_height` 계산보다 실제 renderer line y가 더 아래로 누적된 데서 온다.
- 최종 sweep의 잔여 내부 overflow:
  - p15(display) `pi=753`, line 1~4, 최대 `82.4px`
  - p18(display) `pi=875/876`, 최대 `40.5px`
  - p20(display) `pi=966/967`, 최대 `46.2px`

## 폐기한 접근

두 가지 조판 guard를 임시 적용해 검토했지만 모두 원복했다.

1. `internal_rewind_split`의 head가 현재 단에 fit하지 않으면 즉시 다음 단으로 advance
   - `pi=753` overflow는 사라졌지만 앞쪽 `pi=705~707`이 새로 overflow되어 단 흐름이 더 깨졌다.
   - 원인: 문단 내부 vpos reset은 단 하단 split뿐 아니라 같은 단 내부 저장 재배치에도 나타나므로,
     rewind 존재만으로 advance하면 과이월된다.
2. 단 하단에서 첫 줄이 넘는 일반 미주 문단을 공통 advance
   - p10/p11/p16 등 앞 페이지에서 대량 overflow가 새로 생겼다.
   - 원인: typeset의 `current_height`와 layout의 `HeightCursor` 보정 y가 이미 다르므로,
     조판 단계에서만 tail을 밀면 렌더러 vpos base가 더 크게 어긋난다.
3. `pi=753`의 `internal_rewind_split`을 취소해 문단 전체를 p16으로 advance
   - p15 overflow는 사라졌지만 p16에서 `pi=775~784`가 frame 아래로 밀려 새 overflow가 생겼다.
   - 원인: `pi=753`은 내부 vpos rewind가 있는 긴 문단인데, full paragraph로 넘기면 현재
     accumulation이 실제 formatter 총 높이와 맞지 않아 뒤 문항을 과도하게 같은 단에 배치한다.
4. 0mm 새 미주 제목 threshold를 낮춰 p16의 문24/문25를 다음 단으로 넘김
   - p16 시작 자체가 더 앞당겨져 `pi=764~772`가 새로 overflow되었다.
   - 결론: threshold 조절은 overwrap 제거에는 단기적으로 보일 수 있으나 PDF/Hancom 흐름과 반대로
     누적 drift를 키운다.

따라서 다음 수정은 `typeset.rs`의 단순 advance가 아니라 `HeightCursor`의 compact endnote
vpos 보정과 line-spacing/between-notes 소비 위치를 기준으로 좁혀야 한다.

## 남은 원인 가설

- 0mm 미주 사이에서는 `HeightCursor`의 새 문항 제목 gap은 개선됐지만, partial paragraph split
  계산은 아직 renderer의 실제 line y를 반영하지 못한다.
- 특히 `split_endnote_to_fit`/`internal_rewind_split`은 typeset의 formatter 높이로 줄 수를 결정하고,
  renderer는 이전 문단들의 실제 line/equation 배치와 `HeightCursor` vpos state를 따라 y를 누적한다.
- 해결 방향은 문단을 통째로 넘기는 threshold가 아니라, partial split 후보 산정 시 renderer와 같은
  “현재 단의 실제 다음 line y”를 계산하거나, renderer line y와 typeset line count가 어긋나는
  compact endnote segment만 별도로 재측정하는 것이다.

## 분석 계획

1. `dump-pages`로 `pi=753/875/876/966/967`의 문단 유형, line count, 미주 번호, column
   배치를 확인한다.
2. render tree bbox에서 overflow 문단의 실제 y 위치와 다음 page/column 시작 문단을 대조한다.
3. PDF PNG와 compare PNG를 같이 보며 한컴 기준으로 같은 문단이 현재 단에 남는지 다음 단으로
   넘어가는지 확인한다.
4. 공통 로직 후보:
   - `HeightCursor`가 compact endnote에서 stale forward vpos를 formatter 높이로 cap할 때,
     직전 문단의 trailing line spacing과 `미주 사이=0` 경계를 중복 제거하지 않는지 확인한다.
   - `구분선 아래 20mm`는 첫 미주 내용 시작에만 소비되어야 하며, 새 단/쪽 재개 때 같은 separator
     block처럼 다시 소비되면 안 된다.
   - 문단 내부 vpos rewind는 split 신호일 수도 있지만, 단 하단 fit 판정 없이 조판 단계에서
     advance하면 과이월되므로 render y 보정과 함께 판단한다.

## 검증 계획

- `cargo build --bin rhwp`
- `cargo fmt --all -- --check`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage21_target_check --rhwp-bin target/debug/rhwp`
- 회귀 target:
  - `2024-11-practice-shape987`
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-09-below20-above20`
