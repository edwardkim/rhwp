# Task #697 Stage 1 — 결함 재현 + 가설 확정 분석 보고서

- 이슈: [#697](https://github.com/edwardkim/iop/rhwp/issues/697)
- 단계: Stage 1 (분석, **소스 무변경**)
- 브랜치: `local/task697`

## 1. 결함 재현 + 시각 측정

| 페이지 | RMSE (-fuzz 5%) | 시각 차이 |
|---|---|---|
| p1 | **26.4%** | 표 자체 외곽은 본문 영역에 정상 배치되나, 행 6(`사업개요`) 셀 내부 컨텐츠가 26개 문단 전체 압축되어 작은 글씨로 그려짐 |
| p2 | **22.6%** | 행 6 일부 (`split_start=459.7`) + 행 7 ; 행 6의 셀 내부 split이 정합과 어긋나 텍스트가 중복/위치 어긋남 |

명령:
```
rhwp export-svg samples/inner-table-01.hwp -o /tmp/rhwp_diff/inner_svg
rsvg-convert -d 100 -p 100 .../inner-table-01_001.svg -o ...
pdftoppm -r 100 pdf/inner-table-01-2022.pdf .../inner_pdf -png
compare -metric RMSE -fuzz 5% inner_pdf-1.png inner-table-01_001.png
```

## 2. IR 단서 — 셀[11] paragraph vpos 리셋

`samples/inner-table-01.hwp` 의 표 (8행×2열, `쪽나눔=RowBreak`, padding=(510,510,141,141)):

| 행 | 셀 IDs (rs/cs) | 행 높이 (HU) | 100dpi px |
|---|---|---|---|
| 0 | [0]+[1] | 2714 | 38 |
| 1 | [2]+[3] | 2997 | 42 |
| 2 | [4]+[5] | 2997 | 42 |
| 3 | [6] (cs=2) | 2710 | 38 |
| 4 | [7](rs=2)+[8] | 11371 | 158 |
| 5 | [7] cont. + [9] | 8214 | 114 |
| 6 | [10]+**[11] (paras=26, h=48776)** | 48776 | **677** |
| 7 | [12]+[13] | 26504 | 368 |
| **합** | | | **1477** |

본문 영역(`body_area.h = 876.9px`) 보다 큼 → 두 페이지 분배 필요.

**셀[11] p[0..25] vpos 패턴** (HU):
```
p[0]   vpos=0       (1300 lh)
...
p[19]  vpos=30520   (= 30520 + 1300 + 1300 = 33120 끝)
p[20]  vpos=0       ← 리셋 발생
p[21]  vpos=1300
...
p[25]  vpos=7000    (= 7000 + 1300 = 8300 끝)
```

→ 한컴이 **편집 시 산출한 페이지 분할점**을 셀 내부에 인코딩한 것.
- p[0..19] : 1페이지 영역 — 셀 안에서 33120 HU(459.7px) 차지
- p[20..25] : 2페이지 영역 — 셀 안에서 8300 HU(115px) 차지

## 3. 정합 (PDF) 분배 검증

| 페이지 | 한글 2022 PDF | 행/셀 분배 |
|---|---|---|
| p1 | 행 0..5 정상 + 행 6 위쪽 일부 | 행 0..5 (432px) + 행 6 위 (셀[11] p[0..19] = 459.7px) ≈ **891px** ≈ 본문 876.9px |
| p2 | 행 6 아래쪽 + 행 7 | 행 6 아래 (셀[11] p[20..25] = 115px) + 행 7 (368px) = 483px |

**결론**: 한컴 정합은 **(B) 셀 내부 split** — `cell[11]` 의 vpos 리셋 위치에서 분할. (A) 행 단위 통째 이월 가설은 합 1477px > 본문 877px 두 배 분배 시 행 7 누락이 강제되어 PDF 결과(2 페이지 fit)와 모순.

## 4. rhwp 페이지네이션 결정 — 정합 의도 확인

`src/renderer/pagination/engine.rs` L1744-1757:
```rust
if next_can_intra_split && mt.is_row_splittable(r) {
    ...
    if avail_content_for_r >= MIN_SPLIT_CONTENT_PX
        && avail_content_for_r >= min_first_line
        && remaining_content >= MIN_SPLIT_CONTENT_PX
    {
        end_row = r + 1;
        split_end_limit = avail_content_for_r;  // 459.7
    }
}
```

`dump-pages` 출력:
```
p1: PartialTable rows=0..7 cont=false split_start=0.0   split_end=459.7
p2: PartialTable rows=6..8 cont=true  split_start=459.7 split_end=0.0
```

`split_end=459.7` 은 실제로 cell[11] p[19] 끝 (33120 HU = 459.7px) 과 일치 — **페이지네이션 결정 단계는 정합**.

`mt.is_row_splittable(r=6)` 가 true 를 반환하여 intra-row split 발동 → 정상 동작.

## 5. 결함 위치 확정 — `layout/table_partial.rs` L113-130

```rust
if split_end_content_limit > 0.0 {
    let last_row = end_row.saturating_sub(1);
    if last_row < row_count {
        let mut max_split_h = 0.0f64;
        for cell in &table.cells {
            if cell.row_span == 1 && cell.row as usize == last_row {
                let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
                let cell_h = split_end_content_limit + pad_top + pad_bottom;  // ← 셀 높이만 줄임
                if cell_h > max_split_h { max_split_h = cell_h; }
            }
        }
        if max_split_h > 0.0 {
            row_heights[last_row] = max_split_h;  // ← row_heights[6] = 478px
        }
    }
}
```

이 블록은 **`row_heights[last_row]` 만 split_end_content_limit + padding 으로 덮어쓴다**. 실제 SVG 검증:

```
SVG cell positions (inner-table-01_001.svg):
  행 0: y=132.26 h=36.19   ← 행 0~5 정상 높이 보존
  행 1: y=168.45 h=39.96
  행 2: y=208.41 h=39.96
  행 3: y=248.37 h=36.13
  행 4+5: y=284.50 h=261.13
  행 6 split: y=545.64 h=463.48   ← cell_h = 459.7 + padding 적용됨
  행 7: 미표시
```

표 외곽 배치 자체는 정합 의도 (`row_heights[6] = 478px`).

**그러나 셀 내부 paragraphs 렌더 경로**:

`split_end_content_limit > 0.0 && nested table 가 있는 셀` 분기는 `table_partial.rs` L548 `else if has_nested_table && is_in_split_row && split_end_content_limit > 0.0` 등에 존재.

**paragraph-only 셀 (예: cell[11], 26 paras, nested table 없음) 의 split_end 분기는 누락**.

→ cell[11] 의 26개 문단 전체가 줄어든 cell_h(463.48px) 안에 모두 그려짐 → **압축 회귀**.

p2 도 마찬가지: `split_start_content_offset > 0.0` 인 paragraph-only 셀에 대한 처리는 L82-110 에 존재 (`compute_cell_line_ranges` 사용)하지만, **split end (1페이지 내) paragraph-only 셀 처리가 비대칭**.

## 6. 정정 방향 확정 — (B) 셀 내부 split 정합화

**(B) 채택**: `table_partial.rs` 에서 paragraph-only 큰 셀의 split_end 처리 경로 추가/정합.

방향 (A) 행 단위 이월은:
- 한컴 정합과 모순 (PDF 2 페이지 fit 불가)
- Task #474 가 이미 RowBreak 정책 도입 → 본 결함은 그 후속 영역
- 회귀 위험 큼 (다른 큰 셀 표 전부 영향)

방향 (B) 채택 근거:
- 페이지네이션 결정은 정합 (split_end=459.7 정확)
- IR 단서 (셀 내부 vpos 리셋) 가 한컴이 셀 내부 split 을 산출했음을 보여줌
- 결함 영역이 `table_partial.rs` 한 함수 내 분기 누락으로 좁혀짐 (회귀 위험 작음)

## 7. Stage 2 작업 영역 (잠정)

`src/renderer/layout/table_partial.rs` 정밀 변경:

1. **L113-130 (split_end 행 높이 산출)**: paragraph-only 셀에 대해 row_heights 외에 — 셀 내부 paragraph 렌더 시 적용할 split offset/limit 메타데이터 함께 보존
2. **L322-376 (cell 렌더 분기)**: `is_split_end_row && !has_nested_table` 케이스에 split offset/limit 적용 추가 (현재는 nested table 케이스만 분기 존재)
3. **paragraph-only 셀의 cell-internal split rendering**:
   - p1 (split_end): 셀 paragraphs 중 `vpos < split_end` 인 부분만 렌더 (= p[0..19])
   - p2 (split_start): 셀 paragraphs 중 `vpos >= 0 (리셋 후)` 인 부분만 렌더 (= p[20..25])
4. 단위 테스트 추가:
   - `pagination/tests.rs` `test_inner_table_01_split` (split_end limit 결정 검증)
   - `layout/table_partial.rs` `test_paragraph_only_cell_internal_split` (셀 paragraph 분할 렌더 검증)

## 8. 영향 회귀 검증 fixture (Stage 4 에서 수행)

- `samples/inner-table-01.hwp` (본 결함 fixture, 양 페이지)
- `samples/k-water-rfp.hwp` (큰 표, 27페이지)
- `samples/issue_265.hwp`, `samples/hwp3-sample.hwp` (페이지 16)
- Task #474 fixture (RowBreak 표 분할)
- Task #362 fixture (페이지보다 큰 nested table)
- Task #324 v3 fixture (split_start nested table)

## 9. 다음 단계

Stage 2 — 구현 계획서 (`mydocs/plans/task_m100_697_impl.md`) 작성 → 승인 요청.

---

승인 요청: 본 분석 결과 + 정정 방향 (B) 기준으로 Stage 2 구현 계획서 작성 진행해도 되는지 확인 부탁드립니다.
