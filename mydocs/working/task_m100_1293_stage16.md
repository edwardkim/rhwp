# Task 1293 Stage 16: shape987 p19/p20 page flow 후보 분석

## 목적

Stage15에서 p18의 `pi=853` split 흐름은 PDF와 맞췄지만, sweep에는 여전히
p11/p18/p19/p20 frame 후보가 남는다. 이 중 p19/p20은 red marker drift와 함께 나타나
단순 glyph bleed가 아니라 다음 문항 흐름 자체가 밀렸을 가능성이 크다.

## 현재 기준

- 직전 커밋: `e9b53af6 task 1293: 미주 단일 line rewind 분할 보정`
- 기준 산출물: `output/task1293_stage15_shape987_p18_check/summary.json`
- 대상 샘플: `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp`
- Stage15 기준 `shape987`: `frame=[11, 18, 19, 20]`, `title=[]`, `order=[]`

## 분석 결과

p11/p18/p19/p20의 frame 후보는 모두 `rhwp_outside_frame_max_y=1102`, frame 하단 `1097`로
5px 범위의 하단 glyph bleed였다. compare PNG에서도 문항 흐름이 frame 밖으로 크게 밀린
상태가 아니라 글자 descender/anti-aliasing이 하단 border 아래로 잡히는 패턴이다.

p20의 `equation_text_overlap` 후보도 실제 수식 겹침이 아니라 render tree의 TextRun bbox가
글자 ink 높이가 아닌 line-height를 사용해 adjacent line과 겹쳐 잡힌 오탐이었다.

p20은 별도 renderer 미세 보정도 필요했다. `문29)` 제목은 한컴/PDF처럼 왼쪽 컬럼 하단에
남아야 하지만, 본문 첫 줄 `pi=956`까지 같이 남기면 하단 overflow가 된다.
`compact_endnote_own_vpos_span_fits`가 true여도 non-default compact 미주 본문 tail이 실제
frame을 넘는 경우에는 split 후보로 다시 넘겨야 한다.

## 수정 내용

- `scripts/task1274_visual_sweep.py`
  - frame 하단 6px 이내의 glyph bleed는 metric에는 기록하되 `frame_overflow_pixels` 플래그에서 제외한다.
  - equation/text overlap 판단 시 TextRun bbox 높이를 16px ink 높이로 제한하고, y-overlap이 4px 이상인 경우만 후보로 남긴다.
- `src/renderer/typeset.rs`
  - non-default compact 미주에서 `compact_endnote_own_vpos_span_fits`가 true여도, 본문 tail이
    현재 컬럼 95% 이후에서 전체 문단 기준 frame을 넘으면 fit split 판단을 유지한다.
  - p20에서 `문29)` 제목만 왼쪽 하단에 남고 `pi=956` 본문은 오른쪽 컬럼으로 이어진다.

## 확인 결과

`output/task1293_stage16_shape987_renderer_and_sweep_check/summary.json` 기준:

- 페이지 수: 21/21/21
- `frame_overflow_pages`: `[]`
- `equation_text_overlap_pages`: `[]`
- `question_title_text_overlap_pages`: `[]`
- `line_order_overlap_pages`: `[]`
- 남은 후보: red marker drift, line band drift

시각 확인:

- `output/task1293_stage16_shape987_renderer_and_sweep_check/2024-11-practice-shape987/compare/compare_020.png`
- p20에서 `문29)` 제목은 PDF처럼 왼쪽 컬럼 하단에 남고, 본문은 다음 컬럼으로 이어진다.

남은 red/line drift는 실제 문항 흐름 후보를 찾기 위한 신호로 유지한다.
다음 스테이지에서는 refined sweep 기준으로 red/line 후보 중 한컴/PDF와 실제로 다른 페이지를 고른다.

## 검증 예정

- `python3 -m py_compile scripts/task1274_visual_sweep.py`: 통과
- `cargo build --bin rhwp`: 통과
- `cargo fmt --all -- --check`: 통과
- `target/debug/rhwp dump-pages ... -p 18 --respect-vpos-reset`: 일반 dump와 동일
- `target/debug/rhwp dump-pages ... -p 19 --respect-vpos-reset`: 일반 dump와 동일
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-shape987 --out output/task1293_stage16_shape987_renderer_and_sweep_check --rhwp-bin target/debug/rhwp`: 완료
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 52개 통과
