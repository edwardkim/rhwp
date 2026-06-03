# Task #705 Stage 5 — 회귀 검증 + 최종 보고

## 산출물

- `src/renderer/layout/integration_tests.rs` — 추가 회귀 가드 2건 (국립국어원 + KTX)
- `mydocs/working/task_m100_705_stage5.md` (본 보고서)
- `mydocs/report/task_m100_705_report.md` (최종 결과 보고서)

## 회귀 검증 결과

### 1. 단위/통합 테스트 sweep

```
cargo test --release --lib
test result: ok. 1123 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

→ **0 fail**. 본 task 의 신규 회귀 가드 6건 (test_705_*) 포함.

### 2. clippy

```
cargo clippy --release --lib
Finished `release` profile [optimized] target(s) in 11.10s
```

→ **0 warning**.

### 3. aift.hwp 페이지 카운트 무변화

```
rhwp dump-pages samples/aift.hwp
문서 로드: samples/aift.hwp (77페이지)
```

→ **77 페이지** (Stage 0 측정값과 일치).

### 4. 198 sample sweep — 파서 결과 무변화

```
=== scan_cell_pagehide: 198 파일 sweep ===
  본문 PageHide 합계: 95
  셀 안 PageHide 합계: 13
    - 6필드 모두 true: 1
    - page_num 만 true: 10
  영향 샘플: 6
```

→ Stage 0 측정값과 정확 일치 (분포 무변화). 본 task 의 정정은 **수집 + 적용** 만 변경, **파서 결과 무영향**.

### 5. SVG smoke check (aift.hwp 5 페이지)

| 페이지 | 분류 | bg rect | footer 텍스트 | 정합 |
|--------|------|---------|--------------|------|
| 1 (정상) | page_hide=None | 1 (`#ffffff`) | 3 (page_num 등) | ✓ |
| **2 (감추기 6필드)** | 셀[167] full6 | **0** (`hide_fill` 가드) | 9 (본문 표 35×27 cell 텍스트, footer 영역까지 차지) | ✓ (background 가드 동작) |
| 4 (목차) | s2/p[34] body | 1 | **0** (page_num 미표시) | ✓ |
| 5 (별첨) | s2/p[54] body | 1 | **0** | ✓ |
| 6 (본문 시작) | page_hide=None | 1 | 3 | ✓ |

### 6. 회귀 가드 신규 추가 (Stage 1 의 4건 + Stage 5 의 2건)

| # | 테스트 | 영역 |
|---|--------|------|
| 1 | `test_705_aift_page2_cell_pagehide_collected` | aift p2 page_hide.is_some |
| 2 | `test_705_aift_page2_cell_pagehide_six_fields` | 6 필드 모두 true (메인테이너 권위 측정) |
| 3 | `test_705_aift_page3_cell_pagehide_collected` | aift p3 page_hide.is_some |
| 4 | `test_705_aift_cell_pagehides_total_count` | 본문 2 + 셀안 2 = 최소 4 매핑 |
| 5 | `test_705_kor2022_cell_pagehide_collected` | 국립국어원 본문 1 + 셀안 1 |
| 6 | `test_705_ktx_cell_pagehide_collected` | KTX 본문 1 + 셀안 1 |

→ 6건 모두 GREEN.

## 종합 결과

### 결함 3건 정정 매트릭스

| # | 위치 | 결함 → 정정 | Stage |
|---|------|-----------|-------|
| 1 | `pagination/engine.rs:519-544` + `typeset.rs:2120` | `page_hides` 수집이 본문 paragraph 만 → 셀 안 paragraph 재귀 추가 | 2 |
| 2 | `layout.rs:411-422` | `build_page_background()` + `build_page_borders()` 가드 부재 → `hide_fill`/`hide_border` 가드 추가 | 3 |
| 3 | `main.rs:1665-1670` | dump 셀 안 controls 의 PageHide 분기 부재 → 분기 추가 | 4 |

### Stage 0 의 발견 검증

aift.hwp page 2 의 셀[167]/p[3] PageHide:
- **메인테이너 권위 측정**: 6 필드 모두 true (header/footer/master/border/fill/page_num)
- **본 환경 파서 측정** (`inspect_705`): 정확 일치 ✓
- **dump 출력** (Stage 4): 정확 일치 ✓
- **layout 가드** (Stage 3): page 2 SVG 에 bg_rect 미존재로 검증 ✓

PR #640 (Task #637) 의 H2 가설 ("셀 내부 PageHide") 기각의 측정 누락 (본 환경 결함 #1 으로 인한) 도 확인 — **셀 안 PageHide 13건 (Stage 0 sweep)** 이 실제로 존재하나 결함 #1 으로 무시된 영역.

## Stage 5 진입 후 추가 작업

회귀 가드 강화 차원에서 통합 테스트 2건 추가 (국립국어원 + KTX). 모두 GREEN.

## 최종 보고서 진입

본 Stage 5 완료 후 `mydocs/report/task_m100_705_report.md` 작성.

## 관련

- 수행 계획서: `mydocs/plans/task_m100_705.md`
- 구현 계획서: `mydocs/plans/task_m100_705_impl.md`
- Stage 0~4 보고서: `mydocs/working/task_m100_705_stage{0..4}.md`
- 본 보고서: `mydocs/working/task_m100_705_stage5.md`
- 최종 보고서 (작성 예정): `mydocs/report/task_m100_705_report.md`
