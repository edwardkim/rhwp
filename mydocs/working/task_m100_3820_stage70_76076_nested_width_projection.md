---
kind: analysis
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 70 — 76076 near-fit nested table width projection

## 문제

`76076_regulatory_analysis.hwp` p34의 1×1 비-TAC nested table은 저장 폭 36,572HU
(96dpi 약 487.6px)보다 넓은 부모 셀(약 506.2px)에 들어 있다. 현
`RenderNormalizationOverlay`는 저장 폭이 부모 셀의 90% 이상이면 부모 폭으로 확장한다.

직접 PDF 대조에서는 이 확장이 정답이 아니었다. PDF의 continuation 가용폭은 437.3px로
저장폭 경로와 일치했고, 확장 경로의 442.3px는 줄 끝을 한 글자씩 뒤로 밀었다.

## 검증 순서

1. p33--p36 PDF와 직접 비교해 near-fit projection을 제거했을 때 표 폭·문단 줄 경계·조각
   경계가 회복되는지 확인한다.
2. 원본 HWP와 HWP5-origin HWPX(`samples/issue1891/76076_regulatory_analysis.hwpx`)를 모두
   확인한다.
3. 기존 #2308 geometry pin이 PDF 사실과 충돌하면, 숫자를 보수하기보다 PDF 기준의 폭·텍스트
   경계 assertion으로 교체한다.
4. `issue_1891`, `overflow_cell_baseline`, #3308 narrow table gate를 실행해 범위 밖 손실이 없는지
   확인한다.

## 불변 조건

- 진짜 좁은 비-TAC nested table의 선언 폭·가운데 배치(#3308)는 유지한다.
- source IR은 변경하지 않고 render-only derived state만 바꾼다.
- p34 `근거설명`의 text paint가 nested-table 우측선 밖으로 나가지 않아야 한다.

## 결과

`RenderNormalizationOverlay`의 near-fit stretch 하한을 1.0으로 올려, 부모 셀보다
좁은 비-TAC nested table이 저장 폭을 그대로 사용하게 했다. 이 변경은 source IR을
변경하지 않으며 `nested_table_width_scale()`이 1.0을 돌려주는 render-only 계약이다.

- 원본 HWP p34 nested fragment: `x=213.7`, `w=487.6`, `y=77.1`, `h=426.9` px.
- HWP5-origin HWPX도 같은 p34 fragment 수치를 재현했다.
- PDF bbox 기준 continuation 폭 437.3px와 저장폭 경로가 일치했다. 과거 parent-width
  projection은 442.3px로 한 글자씩 늦은 줄바꿈과 잘못된 395.2px fragment를 만들었다.
- 기존 #2308 geometry pin은 PDF가 아닌 과거 projection 수치를 고정하고 있었으므로
  `426.9px` 및 `487.6px` PDF oracle로 교체했다.

## 검증

- `cargo fmt --check`
- `cargo test --profile release-test --lib renderer::render_normalization::tests -- --nocapture`
- `cargo test --profile release-test --test issue_2308_render_normalized_derived_state -- --nocapture`
- `cargo test --profile release-test --test issue_1891 -- --nocapture`
- `cargo test --profile release-test --test issue_3308_nested_table_width -- --nocapture`
- `cargo test --profile release-test --test overflow_cell_baseline` — exit code 0
- `scripts/visual_sweep.py` p33--p36: 원본 HWP 및 HWP5-origin HWPX 각각 82 SVG pages,
  4/4 selected pages complete.

## 증적

`mydocs/pr/assets/task_m100_3820_stage70_76076_nested_width_projection/`에 원본 HWP
review p33--p36/contact sheet, HWP/HWPX sweep summary 및 p34 render tree를 보관했다.
