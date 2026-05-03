# Task #549 최종 결과 보고서 — 사전 분석 결과로 close

**제목**: 좁은 셀 inline TAC 표 cell padding 분석 → wrap_text_x 본질 cause 확인
**브랜치**: `local/task549` (merge 안함)
**이슈**: https://github.com/edwardkim/rhwp/issues/549 (close)
**후속 이슈**: #550 (신규 등록 예정)

---

## 1. 요약

페이지 14 [28~30] 박스 [A] inline TAC 표의 visible 결함 (body text 와 table_right
overlap) 의 본질 cause 를 cell padding hack 으로 잘못 진단. Stage 2 fix 적용 시
시각 변화 없음 (shrink 보상 효과).

본질 재분석 결과 cause 가 다른 영역 (wrap_text_x 또는 table_x 산식) 에 있어 Task
\#549 scope 재정의 필요. **#549 close, #550 신규 등록**.

## 2. 분석 결과

### 2.1 visible 결함 (실측)

| 항목 | 우리 SVG | PDF 한컴 2010 |
|------|---------|----------------|
| cell[2] right (table 끝) | 628.12 px | 113.76 pt |
| body "반" left | 626.22 px | 122.27 pt |
| gap | **-1.90 px (overlap)** | **+8.51 pt (= margin.right 850 HU) ✓** |

### 2.2 본질 cause (수정된 진단)

PDF: body text 시작 = `table_right + outer_margin.right` (850 HU)
우리: body text 시작 = `col_x + IR cs` (= 3455 HU) → table 안 1.90 px overlap

table_right 계산 (`compute_table_x_position` host_margin_left 적용) 또는
wrap_text_x 산식 차이로 990 HU shift 발생.

### 2.3 Stage 0 잘못된 진단 사유

initial 분석에서 PDF cell-rel "[" position (3.28 px) vs 우리 (2.32 px) 의 1 px
차이만 보고 cell padding 문제로 단정. body text overlap (실제 visible 결함) 을
간과.

## 3. 부산물

| 파일 | 보존 여부 |
|------|---------|
| `examples/scan_aim_cells.rs` | 보존 (광범위 cell padding 분석 도구) |
| `examples/inspect_cell.rs` | 보존 (table cell IR inspect 도구) |
| `examples/find_A_pi.rs` | 보존 (paragraph 검색 도구) |
| `src/renderer/layout/integration_tests.rs` (test_549) | **revert** (Task #550 에서 재작성) |
| `mydocs/plans/task_m100_549.md` | 보존 (분석 기록) |
| `mydocs/working/task_m100_549_stage{0..2}.md` | 보존 (분석 단계 기록) |
| `mydocs/report/task_m100_549_report.md` | 본 보고서 |

## 4. Task #550 신규 scope 권고

### 4.1 본질 진단

PDF 정합 기준: body text 시작 x = table_right + outer_margin.right.

후보 fix 위치:
1. `layout.rs:2548-2551` wrap_text_x 산식: cs 대신 `table_right + margin.right`
   직접 사용
2. `compute_table_x_position` Square wrap 분기: host_margin_left 적용 재검토
3. IR cs 의 실제 의미 확인 (parser 검증)

### 4.2 RED 테스트

`test_549_cell_inline_brackets_centered_p14` (현재 #[ignore]) 를 Task #550 에서
재작성: body text "반" 위치 vs table_right gap = outer_margin.right ±2 px 검증.

### 4.3 광범위 사전 평가

scan_aim_cells.rs 결과: 990 cells 가 hack 발동 (Task #347). Task #550 본질 fix 시
hack 영향과 무관할 가능성 큼 (cell padding 과 wrap_text_x 는 다른 산식).

## 5. revert 항목

### 5.1 코드

- `src/renderer/layout/table_layout.rs:791-798` `resolve_cell_padding` aim=false
  분기 — 옵션 B 적용 → revert 완료 (1122 단위 테스트 통과 확인)
- `src/renderer/layout/integration_tests.rs` `test_549_cell_inline_brackets_centered_p14`
  — Task #550 에서 재작성 예정 (현재 #[ignore], merge 안함)

### 5.2 분석 도구 (보존)

`examples/` 의 scan_aim_cells.rs, inspect_cell.rs, find_A_pi.rs 는 향후 분석 시
유용하므로 보존.

## 6. 승인 요청

옵션 B (Task #549 close + Task #550 신규) 진행 OK?

승인 후:
1. `local/task549` 브랜치 폐기 (또는 분석 보존용 보관)
2. Issue #549 close (본 보고서 URL 첨부)
3. Issue #550 등록 (위 4. 신규 scope)
4. orders 갱신
