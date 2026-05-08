# Task #697 Stage 2 — 구현 계획서

큰 셀 cross-page split 결함 정정 — `inner-table-01.hwp` (한글 2022 PDF mismatch)

- 이슈: [#697](https://github.com/edwardkim/rhwp/issues/697)
- 단계: Stage 2 (구현 계획서, 소스 무변경)
- 브랜치: `local/task697`
- 선행: Stage 1 분석 보고서 `mydocs/working/task_m100_697_stage1.md` (승인 완료)

## 1. 결함 본질 — vpos 리셋과 line metric 누적의 mismatch

Stage 1 에서 가설을 (B) 셀 내부 split 정합화로 확정. 본 단계에서 **결함의 산술적 본질**을 추가 검증:

### 1.1 한컴 IR 인코딩

`samples/inner-table-01.hwp` 의 cell[11] (행 6, c=1, h=48776 HU = 677.4px):

| 영역 | paragraph 범위 | vpos 누적 끝 (HU) |
|---|---|---|
| 1페이지 영역 | p[0..19] | 33120 (= 30520+1300+1300) → **459.7px** |
| 2페이지 영역 | p[20..25] (vpos 리셋) | 8300 (= 7000+1300) → **115.3px** |

→ 한컴은 셀 내부에서 **vpos 리셋 위치(p[20] 시작)** 를 페이지 분할점으로 인코딩.

### 1.2 rhwp 의 누적 산출 방식

`src/renderer/layout/table_layout.rs::compute_cell_line_ranges`:
```rust
let mut cum: f64 = 0.0;
for line in &comp.lines {
    let h = hwpunit_to_px(line.line_height, self.dpi);
    let ls = hwpunit_to_px(line.line_spacing, self.dpi);
    let line_h = if !is_cell_last_line { h + ls } else { h };
    ...
    cum = line_end_pos;
}
```

— **line_height + line_spacing** (composed metric) 누적 사용. vpos (LINE_SEG.vertical_pos) 무시.

### 1.3 산출 검증

cell[11] SVG p1 출력의 텍스트 y 좌표 분포: y=562.25 ~ 986.52 (총 424.27px)

| 누적 방식 | 끝 위치 | abs_limit (459.7) 대비 |
|---|---|---|
| vpos 누적 (한컴 인코딩) | 459.7px (p[19] 끝) | 정확히 일치 — 1페이지 fit |
| line_height+line_spacing 누적 (rhwp) | ~424px (전체 26 paras) | abs_limit 미만 → cut 안 됨 |

→ **rhwp 누적이 vpos 누적보다 작아 abs_limit 안에 26 paras 전체가 들어감** → 모두 visible 처리 → cell 내부 압축/오버플로우 회귀.

이는 `vpos 리셋` 영역(p[20..25]) 이 한컴 측에서는 다음 페이지 영역이지만, rhwp 측에서는 동일 페이지 내 연속 라인으로 취급된 결과.

## 2. 정정 방향 (B) 구체화

### 옵션 (B-1) — vpos 리셋 인식 cut

`compute_cell_line_ranges` 가 paragraph 의 LINE_SEG.vpos 가 직전 paragraph 끝보다 **작아지는** 위치를 검출하여 cut 발생점으로 사용.

**장점**: 한컴 인코딩과 직접 정합. 결함 본질에 정확히 매칭.
**단점**: vpos 리셋이 컬럼 변경 시에도 발생 — 컬럼 분할과 페이지 분할 구분 필요.

### 옵션 (B-2) — vpos 누적 기반 cut

`compute_cell_line_ranges` 의 누적 metric 을 line_height+line_spacing 대신 **LINE_SEG.vpos + LINE_SEG.line_height** 로 전환. paragraph 간 spacing 도 vpos 차분으로 흡수됨.

**장점**: 한컴 IR vpos 그대로 사용 — 가장 정합.
**단점**: vpos 리셋 시 cumulative 가 거꾸로 가므로 별도 처리 필요. 다른 호출처 (height_measurer, calc_visible_content_height_from_ranges) 일관성 확보 필요.

### 옵션 (B-3) — IR 단계 page-break 마킹

파서(`src/parser/hwp5/`) 에서 vpos 리셋 위치를 검출하여 paragraph 에 `page_break_before` 플래그 부여. 레이아웃은 이 플래그를 기반으로 cut.

**장점**: 의미가 명시적. 다른 결함에도 활용 가능.
**단점**: 파서 변경 → 회귀 위험 큼. HWP3/HWPX 파서 동시 정합 필요.

### 채택 — (B-1) + 점진 도입

**1차 변경**: `compute_cell_line_ranges` 에 vpos 리셋 검출 분기 추가 — paragraph 시작 시 직전 누적 끝과 LINE_SEG[0].vpos 비교, 리셋 발생 시 그 paragraph 부터 다음 페이지 영역으로 처리.

이는 **cell-internal split 의 한정된 영역에서만 동작** — 즉 paragraph 가 page-break 신호를 가질 때만 적용. 다른 케이스에 영향 없음.

**(B-2)/(B-3) 는 후속 이슈로 분리** — 본 타스크 범위 외.

## 3. 변경 영역 (정밀)

### 3.1 핵심 변경 — `src/renderer/layout/table_layout.rs`

#### `compute_cell_line_ranges` (L2271)

추가: paragraph 진입 시 vpos 리셋 검출 + cum 의 page-break 정합 보정.

```rust
// 진입 직전: 이 paragraph 의 LINE_SEG[0].vpos 가 이전 paragraph 끝의 누적 vpos 보다
// 작으면 (= 컬럼/페이지 리셋), cum 을 abs_limit 까지 강제 진행 시켜 limit 초과 처리 발동.
let para_first_vpos_hu = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(0);
let prev_cum_vpos_hu = prev_para_end_vpos_hu;  // 추적 변수 (직전 para 의 vpos+line_height 끝)
let is_vpos_reset = pi > 0 && para_first_vpos_hu < prev_cum_vpos_hu;
if is_vpos_reset && has_limit && cum < abs_limit {
    // page-break 신호 검출: 이 paragraph 부터 다음 페이지 영역
    cum = abs_limit;  // 다음 paragraph 들이 limit 초과로 cut 되도록
}
```

**조건**:
- `pi > 0` (첫 paragraph 는 리셋 비교 대상 없음)
- `has_limit` (split_end 컨텍스트에서만 — split_start/일반 케이스 영향 없음)
- vpos 리셋 발생

**산출물 변화**: line_ranges 의 (start, end) 값이 vpos 리셋 paragraph 부터 (n, n) (스킵) 으로 산출됨.

### 3.2 대칭 정정 — split_start 케이스

`compute_cell_line_ranges` 가 split_start_content_offset > 0 케이스에서도 vpos 리셋 후 paragraph 들의 위치 인식 필요. 동일 로직으로 정합 가능 (paragraph 시작 시 vpos 리셋 검출 시 cum 재산출).

본 케이스는 p2 SVG 결과가 p1 보다 정상 (RMSE 22.6%) 인 것으로 보아 부분적으로 작동. 1차 변경은 split_end 만 적용, split_start 는 이후 검증 후 정합.

### 3.3 누적 끝 추적 변수 추가

`compute_cell_line_ranges` 에 `prev_para_end_vpos_hu` 추적:
```rust
let mut prev_para_end_vpos_hu: i32 = 0;
// ... paragraph 루프 끝에서:
if let Some(last_seg) = para.line_segs.last() {
    prev_para_end_vpos_hu = last_seg.vertical_pos + last_seg.line_height;
}
```

### 3.4 동시 호출처 정합 — `calc_visible_content_height_from_ranges`

`compute_cell_line_ranges` 가 vpos 리셋 영역 paragraph 들을 (n, n) 으로 마킹하면, 본 함수도 자동으로 그 paragraph 들을 height 합산에서 제외 → 별도 변경 불필요.

### 3.5 `src/renderer/layout/table_partial.rs` 영향

L113-130 (`split_end_content_limit > 0.0` 행 높이 산출) 는 변경 없음 — `row_heights[last_row] = max_split_h` 그대로.

L322-380 (셀 렌더 분기) 는 line_ranges 가 (n, n) 으로 산출되므로 자동으로 vpos 리셋 영역 paragraph 들 스킵 → 별도 변경 불필요.

## 4. 단위 테스트 추가

### 4.1 `src/renderer/layout/table_layout.rs` 또는 별도 tests 모듈

- `test_compute_cell_line_ranges_vpos_reset_split_end`: 합성 fixture (paragraphs 5개, p[3] 에서 vpos 리셋, split_end_content_limit=2페이지 영역 시작) → line_ranges = `[(0,1), (0,1), (0,1), (1,1), (1,1)]` (p[3..5] 스킵) 검증
- `test_compute_cell_line_ranges_no_reset_no_change`: 리셋 없는 기존 케이스 → 동작 변경 없음 검증
- `test_compute_cell_line_ranges_reset_no_limit`: 리셋 있지만 split 컨텍스트 아님 → 동작 변경 없음 검증

### 4.2 통합 fixture 테스트 — `src/renderer/layout/integration_tests.rs`

- `test_inner_table_01_p1_cell_internal_split`: `samples/inner-table-01.hwp` 를 export-svg 후 cell[11] 의 SVG 텍스트 노드 y 분포가 cell area (y=545.64~1009.12) 안에서 p[0..19] 영역 (~459.7px) 안에만 분포 검증

## 5. 회귀 검증 fixture (Stage 4)

영향 영역 — 변경 후 RMSE 비교:

| fixture | 회귀 위험 |
|---|---|
| `samples/inner-table-01.hwp` | **타겟** — RMSE -fuzz 5% 이내 목표 |
| `samples/k-water-rfp.hwp` | 큰 표 다중 페이지 — 동일 결함 가능성 |
| `samples/issue_265.hwp` | hwp3 sample — 동일 |
| `samples/hwp3-sample.hwp` | 동일 |
| Task #474 fixture | RowBreak 표 분할 |
| Task #362 fixture | 페이지보다 큰 nested table — 별도 vpos 처리 분기 |
| Task #324 v3 fixture | split_start nested table |
| Task #431 fixture | abs_limit 단위 정합 |

검증 절차:
1. `cargo test` 전체 통과
2. fixture 별 export-svg → 100dpi PNG → PDF (한글 2022) 와 RMSE 비교
3. RMSE 변화량 ±2% 이내 (또는 본 결함 fixture 만 개선, 다른 fixture 회귀 없음)

## 6. 진행 단계 (Stage 3 — 구현)

1. **Stage 3-1**: `compute_cell_line_ranges` 에 vpos 리셋 검출 추가 + 단위 테스트 (4.1)
2. **Stage 3-2**: `inner-table-01.hwp` p1/p2 SVG ↔ PDF RMSE 검증 + 통합 테스트 (4.2)
3. **Stage 3-3**: 회귀 fixture 검증 (5절)

각 sub-stage 완료 후 단계별 보고서 (`task_m100_697_stage{N}.md`) 작성 → 승인.

## 7. Stage 4 — 검증 + 최종 보고서

- 모든 회귀 fixture RMSE 정합 확인
- `mydocs/report/task_m100_697_report.md` 작성

## 8. 비목표

- (B-2) vpos 누적 기반 cut 전면 전환 — 후속 이슈
- (B-3) 파서 단계 page-break 마킹 — 후속 이슈
- p2 split_start 정합 (현재 부분 작동) — 본 타스크에서 발견된 결함만 정정
- 다른 표 결함 (#688 등)

---

승인 요청: 본 구현 계획 (특히 (B-1) 채택, `compute_cell_line_ranges` vpos 리셋 검출 분기 추가) 기준으로 Stage 3-1 (구현) 진행해도 되는지 확인 부탁드립니다.
