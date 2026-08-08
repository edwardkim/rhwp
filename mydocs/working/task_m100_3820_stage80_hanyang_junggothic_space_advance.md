---
kind: implementation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 80 — 한양중고딕 PDF space advance 보정

## 입력과 근거

Stage 79의 직접 한컴 PDF p35 glyph-box 대조는
`samples/76076_regulatory_analysis.hwp`의 RowBreak outer-table `(row=4,col=2)` 본문에서
RHWP가 `…반죽된 용`까지 한 줄에 넣고, PDF는 `…반죽된`에서 끊는 차이를 재현했다.
저장 `LINE_SEG`가 없는 paragraph이므로 renderer가 새로 line-wrap한 결과다.

한글 advance는 PDF `15.04px`, RHWP `14.96px`로 같지만, 공백 gap은 PDF `9.92px`,
RHWP `7.75px`다. 현재 U+0020 branch는 face별 generated hmtx를 쓰지 않고 전역 반각
`512/1024 em`을 강제한다. PDF glyph black-box 사이 간격을 단순히 layout advance로
환산한 첫 추정 `670/1024 em`은 첫 줄은 맞췄지만 다음 줄의 `인근에`를 밀어내므로 채택하지
않았다. 동일 paragraph의 **연속 두 줄**을 직접 oracle로 삼아 reflow 경계를 재보정한 값은
`550/1024 em`이다.

## 변경 범위와 금지 사항

- `한양중고딕` primary face의 U+0020에만 `550/1024 em` layout advance를 적용한다.
- 자동 생성 `font_metrics_data.rs` 및 `HYGothic-Medium`, 다른 HY/Hanyang face, table padding,
  RowBreak height는 수정하지 않는다.
- paint font의 실제 TTF hmtx가 아니라 한컴 PDF의 line-decision을 따르는 layout calibration임을
  코드 주석과 unit test에 남긴다.

## 완료 조건

1. p35 body first run이 `…반죽된`에서 끊기고 다음 run이 `용기를`로 시작한다.
2. Stage 78이 복원한 p35 table tail/p36 next-row geometry가 유지된다.
3. direct PDF p35/p36 evidence 및 overflow-cell baseline, 전체 release-test, fmt, clippy를
   다시 검증한다. 픽셀 수치만으로 PDF 동일을 주장하지 않는다.

## 구현 및 focused 검증

`src/renderer/layout/text_measurement.rs`의 embedded U+0020 분기에
`hanyang_junggothic_pdf_space_width()`를 추가했다. `한양중고딕` 원명만
`550/1024 em`을 받고, 실제 HYG face인 `HY중고딕` 및 다른 한양 face는 기존 `em/2`를
유지한다. 자동 생성 `font_metrics_data.rs`는 수정하지 않았다.

단위 회귀는 세 face의 U+0020 width를 고정한다. integration 회귀는 p35의 연속 run을
`…반죽된` / `용기를 … 인근에`로 고정하고, p35 tail과 p36 `11.영향평가 여부` 위치도
Stage 78의 값으로 함께 확인한다.

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task-3820-stage80-hanyang-space \
  cargo test --profile release-test --test issue_3820_rowbreak_rowspan_band -- --nocapture

2 passed; 0 failed
```

직접 PDF sweep (`--pages 35-36 --dpi 180`)의 증적은
`mydocs/pr/assets/task_m100_3820_stage80_hanyang_junggothic_space_advance/`에 보관한다.

| page | PDF line contract | pixel match | ink match |
|---|---|---:|---:|
| 35 | `…반죽된` / `용기를 … 인근에` | 90.28891% | 20.40964% |
| 36 | p35 row tail 뒤 `11.영향평가 여부` 재개 | 92.10189% | 33.14411% |

`review_035.png`·`review_036.png`은 PDF와 SVG paint font가 달라 raster ink 지표가 낮게
나온다는 사실을 보인다. 이 수치는 최종 시각 판정이 아니며, 이 단계에서는 대상 셀의 two-line
wrap 및 table fragment geometry만 direct oracle로 수용했다.

## 최종 게이트 (2026-08-08)

모든 명령은 `CARGO_INCREMENTAL=0`,
`CARGO_TARGET_DIR=target/task-3820-stage80-hanyang-space`로 실행했다.

| gate | 결과 |
|---|---|
| focused `issue_3820_rowbreak_rowspan_band` | 2 passed; 0 failed |
| `overflow_cell_baseline` | 678 samples, 17 non-zero documents, 691 lines; passed (기준선 증가 없음) |
| `cargo fmt --check` | passed |
| `cargo test --profile release-test --tests` | exit 0; 모든 unit/integration/visual baseline 통과 |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | passed |

따라서 이번 Stage 80은 p35의 `…반죽된` / `용기를 … 인근에` 직접 PDF 줄바꿈 계약과
p35/p36 표 조각 경계를 회복하면서 기존 전체 회귀 기준도 보존한다. PDF paint font와 SVG의
잔여 raster 차이는 별도 시각 fidelity 범위로 남기며, 이 보정의 성공 근거로 과장하지 않는다.
