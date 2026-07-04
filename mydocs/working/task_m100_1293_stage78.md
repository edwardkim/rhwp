# Task 1293 Stage 78: 20mm 미주 qflow 잔여 보정

## 목적

Stage77에서 수식 컨트롤 TAC 판정을 `eq.common.treat_as_char` 기준으로 정정했다.
이제 남은 대상은 `3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp`의
qflow `[10, 20]`이다.

## 현상

- `output/task1293_stage77_equation_tac_guard` 기준:
  - page count: SVG/PDF/render tree `22/22/22`
  - frame overflow: 없음
  - equation/title/order overlap: 없음
  - qflow: `[10, 20]`
- 20쪽은 RHWP marker가 6개, PDF marker가 3개이다.
  RHWP가 20mm 미주 사이 경계를 PDF/Hancom보다 더 잘게 나누거나, 이전 page tail을
  너무 빨리 소비하는 흐름이다.

## 분석 원칙

- 수치만 줄이는 방식은 금지한다.
- `lineSegArray -> line_seg`의 저장 vpos, paragraph 내부 rewind, 다음 미주 경계의
  실제 `between_notes=20mm` 예약 여부를 확인한다.
- 수식은 `eq.common.treat_as_char=true`일 때만 TAC textRun으로 본다.
- 0/0/0 target의 qflow `[]`와 page count `21/21/21`은 유지한다.

## 검증 계획

1. page10/page20 전후의 dump-pages와 `RHWP_ENDNOTE_LINE_DEBUG`,
   `RHWP_ENDNOTE_ADVANCE_DEBUG`를 비교한다.
2. PDF compare/annotated PNG에서 실제 marker gap 차이를 다시 확인한다.
3. 최소 분기로 20mm 미주 사이 경계 처리만 보정한다.
4. `issue_1139_inline_picture_duplicate` focused test와 두 target sweep을 재실행한다.

## 분석 결과

- page20 qflow의 직접 원인은 직전 page19 마지막 미주 문단이었다.
  - 대상: note30 ep45, `pi=894`, text=`이므로 이고 이다.`
  - 최종 render tree에는 다음 미주 경계 때문에 `line_seg.gap=5669`(20mm)가 붙는다.
  - 그러나 fit 판단 시점의 formatter 로그는 같은 문단을 `seg_ls=452`(약 1.6mm)로 계산하고 있었다.
  - 결과적으로 page19 하단에서 `cur=960.68`, `total=33.63`으로 fit된다고 판단했지만,
    실제 20mm 경계 gap을 포함하면 frame을 넘는다.
- 수정은 상수 축소가 아니라, 다음 미주가 존재하는 현재 미주의 마지막 문단에서
  `between_notes > 기본 7mm`이고 저장 gap보다 큰 경우 그 초과분을 하단 overflow 판단에만 반영했다.
  - 문단 전체 높이/advance에 항상 더하지 않는다.
  - 따라서 2024-09 `미주사이20`처럼 기존에 맞던 중간 흐름은 유지하고,
    page 하단에서만 실제 경계 gap이 frame fit을 깨는지 판단한다.
- page10 잔여 qflow는 실제 compare PNG상 문항 흐름 자체 차이보다 sweep의 marker 추출 오탐이었다.
  - 기존 sweep은 한 page 전체에서 red marker row band를 묶어 좌/우단 같은 y대 marker를 병합했다.
  - marker 추출을 좌/우단별로 수행하고, qflow는 marker 개수 차이가 3개 이상인 강한 후보만 표시하도록 보수화했다.
  - `between_notes_marker_gap`도 좌단 마지막 marker에서 우단 첫 marker로 넘어갈 때 음수 gap이 생기지 않도록 단별 gap만 비교하게 했다.

## 검증 결과

- `cargo fmt --all`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 53개 통과
- `cargo build --bin rhwp`: 통과
- `python3 -m py_compile scripts/task1274_visual_sweep.py`: 통과
- sweep:
  - 명령:
    `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage78_column_marker_sweep2 --rhwp-bin target/debug/rhwp`
  - `2024-11-practice-above0-between0-below0`: SVG/PDF/render tree `21/21/21`, qflow `[]`
  - `2024-11-practice-above0-between20-below2`: SVG/PDF/render tree `22/22/22`, qflow `[]`
  - page20 dump에서 `pi=894`가 page20 맨 위로 넘어와 PDF/Hancom의 이전 미주 tail 흐름과 맞는다.
