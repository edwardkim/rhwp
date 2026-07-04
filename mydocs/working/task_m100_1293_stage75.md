# Task 1293 Stage 75: 0/0/0 미주 18쪽 문항 흐름 분석

## 목적

Stage74에서 `3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`의
12쪽 새 미주 tail fit 문제는 줄였지만, sweep 결과 18쪽의 문항 흐름이 아직 PDF와 다르다.

이번 stage는 문단 전체 높이 보정이 아니라, 문단 내부 `lineSegArray`에서 `line_seg` 단위로
좁혀서 어떤 `TextRun`과 글자처럼 취급해야 하는 TAC 수식/그림이 해당 줄에 포함되는지 확인한다.
수식은 별도 floating object가 아니라 `Control::Equation(common.treat_as_char)`인 textRun 흐름으로
판단한다.

## 기준

- 문항 번호 hard-code 없이 `line_seg.text_start`, 문단 control offset, TAC 여부로 흐름을 판단한다.
- PDF/한컴과 다르게 단이 넘어가는 원인을 먼저 확인하고, 원인이 확정된 뒤 최소 수정한다.
- Stage74에서 안정화한 12쪽 흐름과 20mm 샘플은 회귀시키지 않는다.

## 검증 계획

1. `dump-pages`와 `RHWP_ENDNOTE_LINE_DEBUG=1`로 18쪽의 문항/그래프/TAC 줄 소유권을 확인한다.
2. `export-render-tree` 또는 sweep 산출물의 render tree로 문항별 bbox와 column 위치를 확인한다.
3. 수정 후 focused test를 실행한다.
   - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
4. target sweep을 다시 실행한다.
   - `2024-11-practice-above0-between0-below0`
   - `2024-11-practice-above0-between20-below2`

## 진행 기록

- `src/renderer/typeset.rs`의 Stage75 추가 분기를 문단 단위 수식 검사에서
  `line_seg -> ComposedLine -> tac_control_indices_for_line()` 기준으로 좁혔다.
  - 수식은 프로젝트 기억과 렌더 경로 계약에 맞게 `Control::Equation(_)` 전체를 글자처럼
    취급되는 TAC textRun으로 본다.
  - 새 문항 title tail 판단은 첫 `line_seg`의 실제 textRun 또는 해당 줄에 배정된 TAC 수식이
    있을 때만 허용한다.
  - local rewind 예외도 문단 전체 `controls`가 아니라 현재 0번 line에 TAC 수식이 배정됐는지
    확인한다.
- 18쪽에서 좌측 단 하단에 있던 text 없는 TAC 그림/shape 줄은 0/0/0 profile + 보이는 구분선 +
  단 하단 bleed 조건에서 다음 단으로 advance하도록 했다.
  - 이 분기는 `para_is_treat_as_char_picture_only()`와 단일 line 조건에 묶어 수식 TAC textRun에는
    적용하지 않는다.
- broad local rewind 예외는 18개 focused test 회귀를 만들었으므로 사용하지 않았다. 현재 예외는
  0/0/0 profile, 보이는 구분선, 현재 단 tail fit, 그리고 해당 line의 TAC 수식 소속까지 확인한다.

## 검증 결과

- `cargo fmt --all`: 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 52개 통과.
- target sweep:
  - 명령:
    `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage75_lineseg_text_run --rhwp-bin target/debug/rhwp`
  - `2024-11-practice-above0-between0-below0`: SVG/PDF 21/21, `qflow=[20]`.
    Stage74의 `qflow=[18]` 대비 18쪽 문항 흐름 후보가 제거됐다.
  - `2024-11-practice-above0-between20-below2`: SVG/PDF 22/22, `qflow=[10, 20]`.
    20mm 변형 샘플은 Stage74 수준을 유지했다.
- 직접 확인:
  - `output/task1293_stage75_lineseg_text_run/2024-11-practice-above0-between0-below0/compare/compare_018.png`
    에서 18쪽 문항 흐름 후보는 제거됐다.
  - `compare_019.png`는 qflow 후보는 아니지만 line/large drift가 남아 있다.
  - `compare_020.png`는 아직 PDF와 큰 흐름 차이가 있고 `qflow=[20]`으로 남았다.

## 남은 문제

- 0/0/0 샘플 20쪽은 RHWP/PDF의 문30 및 그림 흐름이 크게 다르다. 다음 stage에서 page20의
  `lineSegArray -> line_seg -> TextRun/TAC` 소속을 다시 좁힌다.
