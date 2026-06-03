# Task #965 Stage 5 — 시각 검증 + 최종 작업 정리

## 1. 시각 검증 (작업지시자 확인)

- sample16 page 18 WMF 다이어그램:
  - Before: 박스 내부 한글 텍스트 박스 하단 라인 걸침 (Windows 서버군, Unix 서버군, PE6450 등)
  - After: 박스 내부 정상 위치 ✓ 한컴 viewer 정합

## 2. 최종 변경 요약

### 2.1 코드 변경
`src/wmf/converter/svg/mod.rs` 3 영역 (~60 lines):
- `ext_text_out` (~813-833): baseline y shift 정합 (VTA_TOP=+ascent, VTA_BOTTOM=-descent, VTA_BASELINE=0)
- `text_out` (~1541-1556): 동일 baseline 보정 (PR #918 미포함, 본 task 추가)
- `set_text_align` (~2186-2208): vertical bits (0x0018 mask) 정확 분기

### 2.2 PR #918 대비
- PR #918 (closed, 5082 additions): LO emfio + WASM raster + woff2 + DX + POLYPOLYGON + Stage 33-A
- 본 task #965 (~60 lines): **Stage 33-A 의 핵심 height/baseline 보정만** 추출
- PR #918 의 부작용 (다양한 영역 변경) 회피 + root cause fix 만 도입

### 2.3 문서 추가
- `mydocs/plans/task_m100_965.md`
- `mydocs/plans/task_m100_965_impl.md`
- `mydocs/plans/task_m100_965_impl_v2.md`
- `mydocs/working/task_m100_965_stage1.md`
- `mydocs/working/task_m100_965_stage4.md`
- `mydocs/working/task_m100_965_stage5.md`
- `mydocs/report/task_m100_965_report.md`
- `mydocs/orders/20260517.md` 갱신

## 3. PR 구성

- base = upstream/devel
- head = jangster77:local/task965
- 코드: 단일 파일 (svg/mod.rs) 3 영역
- 문서: 본 task 의 plans + working + report + orders
