# Task 1293 Stage 77: 20mm 미주 변형 qflow 잔여 분석

## 목적

Stage76에서 `3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
20쪽 문30 제목 흐름은 qflow 후보에서 제거됐다.

이번 stage는 같은 계열의
`3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp`에서 남은
qflow `[10, 20]`을 분석한다. 단순한 미주 사이 수치 보정이 아니라, 문단 내부
`lineSegArray -> line_seg -> ComposedLine/TextRun -> TAC control` 소속과
다음 미주 제목 tail 여부를 같이 확인한다.

추가 반례 샘플:

- `samples/수식-문자처럼취급-아님.hwp`
- `pdf/수식-문자처럼취급-아님.pdf`

이 샘플은 문단 안에 수식 컨트롤이 있어도 dump 결과가 `tac=false`이다. 따라서
`Control::Equation` 타입 자체를 글자처럼 취급하면 안 되고, `eq.common.treat_as_char`
값까지 확인해야 한다.

## 확인 기준

- 20mm 미주 사이 값은 실제로 큰 경계 gap을 만들어야 하므로, Stage76의 0/0/0 tail
  허용을 무조건 재사용하지 않는다.
- 문항 번호가 아닌 다음 미주 경계, line_seg 저장 vpos, Equation TAC textRun 여부를
  기준으로 분기를 만든다.
- Equation TAC textRun 여부는 `Control::Equation(_)`만으로 보지 않고
  `eq.common.treat_as_char=true`일 때만 인정한다.
- 0/0/0 target의 `qflow=[]`, 페이지 수 `21/21/21`은 회귀시키지 않는다.
- `issue_1139_inline_picture_duplicate` focused test를 유지한다.

## 검증 계획

1. `output/task1293_stage76_title_tail`의 20mm target `metrics.json`에서 page10/page20 qflow
   marker 위치를 확인한다.
2. `RHWP_ENDNOTE_LINE_DEBUG=1 RHWP_ENDNOTE_ADVANCE_DEBUG=1`로 해당 페이지의 미주 title/tail
   흐름을 확인한다.
3. 수식 반례 샘플을 dump/export하여 `tac=false` 수식이 비TAC 경로로 남는지 확인한다.
4. 수정이 필요하면 최소 분기로 반영한다.
5. focused test와 두 target sweep을 재실행한다.

## 현재 확인

- `target/debug/rhwp dump samples/수식-문자처럼취급-아님.hwp` 결과:
  `수식 ... size=5910x5445 tac=false`
- `target/debug/rhwp dump-pages samples/수식-문자처럼취급-아님.hwp -p 0` 결과:
  문단 텍스트와 같은 paragraph/control 안에 있으나 pagination item은 별도 `Shape`로 나온다.
- 따라서 `line_has_tac_equation_control`, `line_is_equation_tac_text_run_only` 계열 helper는
  반드시 `eq.common.treat_as_char`를 확인해야 한다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 미주 pagination helper의 Equation TAC 판정을 `Control::Equation(_)` 타입 단독에서
    `eq.common.treat_as_char=true` 조건으로 좁혔다.
  - Stage76의 zero-profile 수식 tail 보정은 TAC 수식에만 적용되도록 유지했다.
- `src/renderer/layout/paragraph_layout.rs`
  - paragraph layout의 equation-only TAC line 판정도 `eq.common.treat_as_char`를 확인하도록
    동일하게 정리했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - `samples/수식-문자처럼취급-아님.hwp`를 회귀 샘플로 추가했다.
  - 수식 컨트롤이 `tac=false`로 파싱되고, 페이지 배치에서도 별도 `Shape` item으로 남는지
    검증한다.
- `mydocs/manual/memory/project_equation_always_tac.md`
  - 기존 “수식은 항상 TAC” 기억을 폐기하고, `Control::Equation`과
    `eq.common.treat_as_char`를 분리해서 판단하도록 정정했다.

## 검증

- `cargo fmt --all`: 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 53개 통과.
- `cargo build --bin rhwp`: 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage77_equation_tac_guard --rhwp-bin target/debug/rhwp`
  - `2024-11-practice-above0-between0-below0`: SVG/PDF/render tree `21/21/21`, qflow `[]`.
  - `2024-11-practice-above0-between20-below2`: SVG/PDF/render tree `22/22/22`, qflow `[10, 20]` 유지.

## 다음 stage로 넘길 잔여

- 20mm 변형의 qflow `[10, 20]`은 이번 수식 TAC 반례 정정과 별개로 남아 있다.
- page20은 RHWP marker 6개/PDF marker 3개이며, RHWP가 미주 경계를 더 잘게 쪼갠다.
- 다음 stage에서는 `between_notes=20mm` 경계에서 line_seg 저장 vpos와 실제 한컴/PDF marker gap
  차이를 기준으로 다시 좁힌다.
