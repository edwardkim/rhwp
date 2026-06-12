# 구현계획서 — Task #1357: 미주 다단 오버플로 정밀 수정 (권장 ① 경로)

- **이슈**: #1357 (M100)
- **브랜치**: `local/task1357`
- **근거**: Stage1(`task_m100_1357_stage1.md`) — typeset `current_height` ~91px 과소누적,
  default col0 tail 에 정확한 렌더-y 예측 부재. under-fill 회귀는 오버플로 바운드로
  미포착 → **시각 회귀 세트 선구축 후 수정**.

## 회귀 대상 (exam 5종, 기준 PDF 보유)
- 3-09월_교육_통합_2022 / 2023 / 3-10월_교육_통합_2022 / 3-11월_실전_통합_2022 /
  3-09월_교육_통합_2024-구분선아래20구분선위20 (`.hwp`)

## 단계 (4단계)

### Stage A — 시각 회귀 하니스 + 베이스라인
- 스크립트: 각 exam 전 페이지에 대해 native SVG→PNG, 기준 PDF→PNG(동일 폭) 렌더
- 페이지·컬럼별 **최하단 잉크 y(채움 높이)** 측정
- 메트릭: `our_fill_y - pdf_fill_y` (양수=오버필/초과, 음수=언더필)
- **베이스라인 저장**(수정 전 our 값) + PDF 대비 현재 편차표
- 산출물: 하니스 스크립트 + `_stage{A}.md` 베이스라인 표

### Stage B — 정밀 수정 구현
- `typeset.rs`: default-between-notes **col0 tail(ep_idx>0)** 케이스에 HeightCursor
  시뮬레이션 기반 렌더-y 예측 블록 추가(기존 `large_between_*_render_overflows` 동형),
  예측 overflow 시 `advance_column_or_new_page`
- 가장 좁은 게이트(default_between_notes_gap && col0 && current_height>0.85·avail &&
  !rewind && split=None && visible)로 한정

### Stage C — 검증 (하니스 + 테스트)
- 하니스 재측정: **AFTER vs BEFORE(our)** diff → 변경된 페이지만 PDF 대비 개선/회귀 판정
  - 목표: p21 오버플로 해소(PDF 정합), **타 exam 전 페이지 무회귀**(편차 악화 0)
- `issue_1082` 5 passed 유지 + `cargo test` 전체
- LAYOUT_OVERFLOW 경고 전 문서 0 또는 베이스라인 대비 감소
- 산출물: `_stage{C}.md` 검증 표

### Stage D — 보고
- `_report.md` + 하니스 스크립트 보존(재현 가능)

## 롤백 기준
하니스에서 **어느 exam 페이지든 PDF 편차가 베이스라인보다 악화(under-fill 포함)** 시
즉시 롤백하고 바운드 유지로 전환.
