# Task 1293 Stage 76: 0/0/0 미주 20쪽 문항 흐름 분석

## 목적

Stage75에서 `3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`의
18쪽 qflow 후보는 제거됐지만, sweep 결과 20쪽이 아직 PDF와 크게 다르다.

이번 stage는 page20의 문30 및 그림 흐름을 문항 번호 hard-code 없이 문단 내부
`lineSegArray -> line_seg -> ComposedLine/TextRun -> TAC control` 소속으로 분석한다.
수식은 별도 floating object가 아니라 `Control::Equation(_)`인 TAC textRun으로 보고,
그림/shape TAC와 분리해서 판단한다.

## 확인 기준

- `line_seg.text_start`, `vertical_pos`, `line_height`, `line_spacing`과 `ComposedLine.char_start`를
  함께 확인한다.
- 현재 단에서 넘겨야 하는 것은 문단 전체인지, 특정 line range인지, 또는 TAC 그림/수식이 속한
  line인지 분리한다.
- 0/0/0 profile 전용 분기를 만들더라도 20mm 변형 샘플의 기존 qflow `[10, 20]`은 악화시키지 않는다.
- Stage75에서 고친 18쪽 qflow가 되살아나면 수정 방향을 되돌린다.

## 검증 계획

1. `dump-pages -p 19`와 `RHWP_ENDNOTE_LINE_DEBUG=1 RHWP_ENDNOTE_ADVANCE_DEBUG=1`로 20쪽의
   문30/그림 관련 paragraph와 line_seg 소속을 확인한다.
2. `render_tree_020.json`, `compare_020.png`, `metrics.json`의 qflow 후보를 대조한다.
3. 원인이 확정되면 최소 수정 후 focused test를 실행한다.
   - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
4. target sweep을 다시 실행한다.
   - `2024-11-practice-above0-between0-below0`
   - `2024-11-practice-above0-between20-below2`

## 진행 기록

- page20 로그에서 문29 마지막 tail은 `text=""`지만 `tac=[...:eqh2205]`인
  수식 TAC 단일 `line_seg`로 확인했다.
  - `note=29 ep=20`, `line_segs=1`, `runs_empty=true`, `tac=[0@Some(0):eqh2205]`
  - `current=970.17`, `available=1001.56`, `en_fit=29.40`, `total=35.43`
  - line height 자체는 좌측 단에 들어가지만, 기존 로직은 `late_text_tail=true`로
    판단해 다음 단으로 이동한다.
- 1차 가설로 0/0/0 + 보이는 구분선 + 단일 수식 TAC tail을 현재 단에 남기도록
  시도했으나 폐기했다.
  - focused test는 통과했지만 target sweep에서
    `2024-11-practice-above0-between0-below0`가 SVG 20쪽 / PDF 21쪽으로 깨졌다.
  - qflow 후보가 `[11, 12, 13, 14, 15, 16, 17, 18, 19]`로 대량 증가했다.
  - 결론: "수식 TAC tail은 글자처럼 취급한다"는 방향은 맞지만, tail line만
    전역적으로 현재 단에 남기는 식은 page count를 무너뜨린다.
- 다음 분석 방향:
  - page20의 문제는 q29 마지막 수식 한 줄만이 아니라, q29 앞쪽 line_seg 누적이
    PDF보다 늦게 흘러 문30 제목이 우측 상단으로 밀리는 현상이다.
  - 문단 전체 높이/단 이동이 아니라 `lineSegArray -> line_seg -> TextRun/TAC`
    소속별 누적 차이를 좁힌다.
  - 특히 q29 `p417:note7`, `note9`의 TAC 그림 line과 `note19~20`의 수식 TAC line이
    현재 단 높이에 어떻게 반영되는지 비교한다.
- 추가 로그에서 q29 마지막 tail과 q30 제목의 실제 흐름을 확정했다.
  - q29 마지막 tail: `note=29 ep=20`, `text=""`, `tac=[0@Some(0):eqh2205]`,
    `line_segs=1`, `comp_lines=1`, `fmt_lines=1`.
  - 기존에는 이 줄이 `late_text_tail=true`로 분류되어 `advance_fit=true`가 되었고,
    q30 제목이 우측 단 상단으로 이동했다.
  - 한컴/PDF는 q29 마지막 수식 line을 좌측 단 tail에 남기고, q30 제목 한 줄도 같은
    좌측 단 하단 tail로 남긴 뒤, q30 그림/본문을 우측 단 상단으로 넘긴다.
- 수정 내용:
  - `line_is_equation_tac_text_run_only()`를 추가해 보이는 텍스트는 없지만 해당
    `line_seg`에 `Control::Equation(_)` TAC만 있는 textRun 줄을 구분했다.
  - endnote loop를 index-aware로 바꿔 현재 미주의 마지막 paragraph에서 다음 미주의
    첫 제목 paragraph를 확인할 수 있게 했다.
  - 0/0/0 + 보이는 구분선 profile에서 현재 미주의 마지막 줄이 Equation TAC textRun이고,
    다음 미주의 제목 한 줄까지 bottom bleed 허용 범위에 들어가면 `late_text_tail`로 인한
    fit advance를 억제한다.
  - 다음 미주 제목 tail은 line spacing까지 완전 fit해야 한다는 기존 첫 단 조건을 완화했다.
    제목 자체의 line height가 `ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX` 안에 들어가면
    한컴처럼 현재 단 tail로 허용한다.
- 검증 결과:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
    - 52 passed.
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage76_title_tail --rhwp-bin target/debug/rhwp`
    - `2024-11-practice-above0-between0-below0`: SVG/PDF/render tree `21/21/21`, `qflow=[]`, `frame=[]`, `equation_text_overlap=[]`.
    - `2024-11-practice-above0-between20-below2`: SVG/PDF/render tree `22/22/22`, 기존 `qflow=[10,20]` 유지, `frame=[]`, `equation_text_overlap=[]`.
  - `compare_020.png`에서 q30 제목이 RHWP/PDF 모두 좌측 단 하단 tail에 위치하는 것을 확인했다.
