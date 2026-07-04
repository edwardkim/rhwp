# Task 1293 Stage 15: shape987 p18 단일 line head overflow 분석

## 목적

Stage14에서 `shape987` p14 새 미주 제목 tail advance는 PDF 기준에 맞췄다.
남은 frame 후보 중 p18은 이전 단계에서 확인한 `pi=853` 단일 line head 분할 문제와
연관되어 있어, 먼저 이 페이지의 실제 흐름을 분석한다.

## 현재 기준

- 직전 커밋: `75d17f50 task 1293: 미주 새 제목 tail advance 보정`
- 기준 산출물: `output/task1293_stage14_shape987_final2_check/summary.json`
- 대상 샘플: `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp`
- Stage14 기준 `shape987`: `frame=[11, 18, 19, 20]`, `title=[]`, `order=[]`
- p18 후보: `rhwp_outside_frame_pixels=208`, `content_bottom_delta_px=25`

## 분석 결과

`pi=853`은 저장된 line segment가 `vpos-rewind@line1`으로 끊긴다.
한컴/PDF는 첫 줄 `"(나)를 고려하기 위해 ..."`만 왼쪽 컬럼 하단에 두고,
나머지 줄은 오른쪽 컬럼 상단으로 이어진다.

Stage14 기준 rhwp는 non-default compact 미주에서 `internal_rewind_split == Some(1)`을
일괄 해제해 `pi=853` 전체를 왼쪽 컬럼 하단에 배치했다. 이 때문에 p18 하단과 다음 컬럼
시작이 PDF보다 어긋났다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 단일 line internal rewind라도 첫 줄이 현재 컬럼에 들어가고, 전체 문단은 하단을 넘으며,
    다음 컬럼이 남아 있으면 split을 보존한다.
  - 기존처럼 fit만으로 생긴 무리한 한 줄 split은 계속 해제해 p14/p21 회귀를 막는다.

## 확인 결과

`dump-pages -p 17` 기준 p18은 아래처럼 바뀐다.

```text
단 0 ... PartialParagraph  pi=853  lines=0..1
단 1 ... PartialParagraph  pi=853  lines=1..4
```

`output/task1293_stage15_shape987_p18_check/summary.json` 기준:

- 페이지 수: 21/21/21
- `frame_overflow_pages`: `[11, 18, 19, 20]`
- `question_title_text_overlap`: 없음
- `line_order_overlap`: 없음

p18은 split 흐름이 PDF와 맞아졌지만, sweep상 하단 5px glyph bleed 후보는 아직 남는다.
다음 스테이지에서는 p19/p20의 실제 page flow 후보와 함께 남은 frame 후보를 계속 본다.

## 검증 예정

- `cargo build --bin rhwp`: 통과
- `target/debug/rhwp dump-pages ... -p 17`: `pi=853` split 확인
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-shape987 --out output/task1293_stage15_shape987_p18_check --rhwp-bin target/debug/rhwp`: 완료
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 52개 통과
