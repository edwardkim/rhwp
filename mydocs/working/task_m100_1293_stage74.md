# Task 1293 Stage 74: 0/0/0 미주 새 문항 tail fit 보정

## 목적

Stage73 진단에서 `3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
12쪽의 문16 시작이 한컴/PDF와 다르게 오른쪽 단으로 넘어가는 원인을 확인했다.

현재 로직은 2단 첫 단에서 새 미주가 시작될 때 `current_height > available * 0.88`이면
`advance_for_new_endnote=true`로 다음 단으로 넘긴다. 하지만 0/0/0 profile에서는 구분선 주변
여백과 미주 사이가 없으므로, 문16처럼 새 미주 전체가 남은 단 높이에 들어가는 경우 한컴은 현재
단 하단에 남긴다.

## 기준

- 문항 번호를 직접 보지 않는다.
- 문단 전체 수치 조정이 아니라 미주 문단의 `lineSegArray -> line_seg -> TextRun/TAC` 측정값을
  사용한다.
- `Control::Equation(treat_as_char)` 수식은 글자처럼 취급되는 TAC textRun으로 보고,
  이미 Stage72/73에서 정합한 줄 소유권과 `format_paragraph`의 line advance를 사용한다.
- 새 미주 전체가 남은 단 높이에 들어갈 때만 현재 단에 남긴다. 들어가지 않으면 기존 advance
  보호 로직을 유지한다.

## 검증 계획

1. focused test: `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
2. target sweep:
   - `2024-11-practice-above0-between0-below0`
   - `2024-11-practice-above0-between20-below2`
3. `dump-pages -p 11`에서 0/0/0 샘플 12쪽 문16이 현재 단 하단에 남는지 확인한다.

## 구현

- 0/0/0 profile에서 새 미주가 시작될 때 현재 미주 전체의 저장 `line_seg` vpos span이
  현재 단에 들어가면 `advance_for_new_endnote`를 막는다.
- 현재 단에 이미 쌓인 미주 PageItem의 `line_seg` span을 계산해, 순차 누적 `current_height`가
  저장 vpos 흐름보다 과대일 때 0/0/0 profile에서만 `current_height`를 span 기준으로 되돌린다.
- `PartialParagraph`는 실제 포함된 line range만 span에 반영한다.
- 되감김이 있는 미주는 sequential line advance 합계가 아니라 저장 vpos span을 우선한다. 이 케이스는
  한컴이 수식/TAC를 글자처럼 취급하면서 줄 위치를 저장 vpos로 재배치한 결과이므로, 수식을 별도
  floating object 높이로 다시 더하지 않는다.

## 검증 결과

- `cargo fmt --all && cargo build --bin rhwp && cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 통과: 52 passed.
- `dump-pages samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp -p 11`
  - 12쪽 왼쪽 단은 문16 전체와 문17 앞부분까지 남는다.
  - 오른쪽 단은 문17 tail 뒤 문18로 이어져 PDF compare의 큰 흐름과 맞아졌다.
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage74_vpos_span_tail --rhwp-bin target/debug/rhwp`
  - `2024-11-practice-above0-between0-below0`: page count 21/21, `qflow=[18]`.
    기존 `[11,12,13,19,20]`에서 page18 하나로 감소했다.
  - `2024-11-practice-above0-between20-below2`: page count 22/22, `qflow=[10,20]`.
    Stage72 수준을 유지해 20mm 샘플 회귀는 없다.
- 남은 문제:
  - 0/0/0 샘플 18쪽은 그래프/문항 흐름 위치가 PDF와 다르다. 다음 stage에서 그림/TAC 포함
    line_seg 흐름으로 별도 분석한다.
