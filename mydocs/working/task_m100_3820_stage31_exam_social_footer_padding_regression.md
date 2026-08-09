---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 31 — exam_social master footer padding 회귀 복구

## 재현과 기준선

- 입력: `samples/hwpx/exam_social.hwpx`
- 새 Hancom 기준: `pdf/exam_social-current-2020.pdf`
  - `PrintToPDFEx`, 4 pages, SHA-256
    `33b2d6b32385f96ff45819e62d7ad93c944777bd9654ade785c76f0daada899a`
- 기존 기준: `pdf/exam_social-2022.pdf`

`CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 cargo test --profile
release-test --tests`는 `issue_1100_hwpx_master_page_footer_page_number_is_preserved`에서
실패했다. 동일 test는 최신 `upstream/devel` (`0102a7ae2`)에서 3/3 통과하고, 이번 Stage 30
직전 commit `b07b13ab5`에서도 동일하게 실패했다. 따라서 76076 Stage 30 변경이 아니라
기존 누적 보정의 회귀다.

새 PDF p2와 기존 2022 PDF 모두 footer의 좌측 페이지 번호 `2`를 보존한다. 600dpi raster의
footer frame을 기준으로 현재 rhwp의 glyph는 약 1.8pt 오른쪽이며, 기존 #1100의
`x=483.7733` gate가 현 출력 `x=486.8`보다 기준에 가깝다. 고정 gate를 완화하거나 새 PDF를
근거 없이 맞추는 대신, padding 문맥을 복구해야 한다.

## 원인과 수정 방침

Stage 29가 `aim=false` + 작은 저장 cellMargin(510HU)을 HWP5 비글자 중첩 표에서 사용할 수 있게
했지만, 호출 조건 `depth > 0`은 `Header`/`Footer`/`MasterPage` 내부 표도 포함했다. 이 master
footer는 #2195의 일반 `aim=false` 규칙대로 table `inMargin`(283HU)을 써야 한다. 예외를 body의
비글자 중첩 표로 제한하고 header/footer/master page는 원 규칙으로 둔다.

## 결과와 증적

- body 중첩 표 조건에 `Header`/`Footer`/`MasterPage` 제외를 추가했다. footer는 다시
  table `inMargin`을 사용하며 SVG p2의 auto-number anchor가 `x=483.7733, y=1406.76`로
  복구됐다.
- [Hancom 2020 PDF p2](../pr/assets/task_m100_3820_stage31_exam_social_footer_padding_regression/reference_p002.png)와
  [rhwp 수정 후 p2](../pr/assets/task_m100_3820_stage31_exam_social_footer_padding_regression/rhwp_p002_after.png)
- `issue_1100_exam_social_hwpx_header` — 3 passed
- Stage 30 `issue_2308_render_normalized_derived_state` — 3 passed
- `issue_2007_nested_cell_pagination` — 9 passed
- `issue_3595_nested_split_row_identity` — 2 passed

전체 release-test, Native Skia 3종, fmt, clippy는 이 Stage commit 뒤 PR 준비 게이트에서
순차 실행한다.
