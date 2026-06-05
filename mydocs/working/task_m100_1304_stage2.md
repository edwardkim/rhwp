# 단계별 완료 보고서 — Task M100-1304 Stage 2 (후속): 시그마 상·하한 가로 정렬

이슈: [#1304](https://github.com/edwardkim/rhwp/issues/1304) · 브랜치: `local/task1304`

## 1. 배경

Stage 1 수정으로 `sum_k=1 ^6` 하한이 `k=1` 전체로 복원된 뒤, 시각 검토에서
"시그마 인자가 왼쪽으로 조금 밀려있음" 피드백. 첨자(상·하한)가 ∑ 기호 중심보다 좌측에 정렬됨.

## 2. 근본 원인

연산자 폭 추정이 layout 과 render 에서 불일치.

- **layout** (`estimate_text_width('∑')`) = `0.8·op_fs` → max_w/첨자 중앙정렬 기준.
- **render** (`estimate_op_width('∑')`) = `0.6·op_fs` → ∑ 중앙정렬에 사용.

∑ 폭을 0.6 으로 과소추정 → `op_x = (max_w - 0.6·op_fs)/2` 가 과대 → ∑ 가 우측으로 치우치고, max_w 중앙에 정렬된 첨자가 상대적으로 좌측에 보인다.
하한이 ∑ 보다 넓을 때(`k=1`)만 가시화되며, Stage 1 으로 하한이 넓어지면서 드러났다(브레이스 표기 본문 수식에도 잠재).

## 3. 수정

연산자 중앙정렬 폭을 layout 과 동일한 단일 진실 소스로 통일.

- `layout::estimate_text_width` 를 `pub(crate)` 로 노출.
- `svg_render` / `canvas_render` 의 big-op 중앙정렬을 `estimate_op_width`(0.6) → `super::layout::estimate_text_width(symbol, op_fs, false)` 로 교체.
- 중복·미사용된 `estimate_op_width`(svg_render, canvas_render) 제거.

변경 파일: `layout.rs`, `svg_render.rs`, `canvas_render.rs`.

## 4. 검증

`rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 10` 문18 첫 ∑ 가로 중심(픽셀 측정, `(` 제외):

| 요소 | 수정 전 | 수정 후 |
|------|--------|--------|
| ∑ 중심 | 71 | 51.7 |
| 상한 `6` | 56 (15 좌) | 54.6 (≈정렬) |
| 하한 `k=1` | 47 (24 좌) | 51.6 (≈정렬) |

- 문18 4줄 모든 ∑ 상·하한이 기호 중앙 정렬, 권위 PDF 와 일치.
- `cargo test --lib` 통과(0 failed), `cargo clippy --lib` 경고 없음, 빌드 경고 0.

## 5. 결론

첨자 좌측 치우침 해소. 연산자/첨자가 동일 폭 기준을 공유해 가로 중심이 일치한다.
