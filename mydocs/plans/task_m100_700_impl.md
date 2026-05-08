# Task #700 Stage 2 — 구현 계획서

옵션 C 채택: `compute_cell_line_ranges` cum 절대 동기화 + 셀별 가드

- 단계: Stage 2 (구현 계획서, 소스 무변경)
- 브랜치: `local/task700`
- 선행: Stage 1 분석 보고서 (승인 완료)

## 1. 변경 영역

### 1.1 핵심 — `src/renderer/layout/table_layout.rs::compute_cell_line_ranges`

**현재 (Task #697 적용 후)**:
```rust
if pi > 0 && has_limit && cum < abs_limit {
    let prev_para = &cell.paragraphs[pi - 1];
    let prev_end_vpos_hu = prev_para.line_segs.last()
        .map(|s| s.vertical_pos + s.line_height)
        .unwrap_or(0);
    let cur_first_vpos_hu = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(0);
    if prev_end_vpos_hu > 0 && cur_first_vpos_hu < prev_end_vpos_hu {
        cum = abs_limit;
    }
}
```

**변경 (옵션 C 적용)**:
```rust
if pi > 0 {
    let cell_first_vpos = cell.paragraphs.first()
        .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos))
        .unwrap_or(-1);
    let cur_first_vpos = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1);
    let prev_para = &cell.paragraphs[pi - 1];
    let prev_end_vpos = prev_para.line_segs.last()
        .map(|s| s.vertical_pos + s.line_height)
        .unwrap_or(-1);

    // 셀별 가드: 셀 첫 paragraph 의 LINE_SEG[0].vpos == 0 (한컴 정상 인코딩 케이스)
    if cell_first_vpos == 0 && cur_first_vpos >= 0 && prev_end_vpos > 0 {
        if cur_first_vpos < prev_end_vpos {
            // vpos 리셋 — page-break 신호 (Task #697)
            if has_limit && cum < abs_limit {
                cum = abs_limit;
            }
        } else {
            // 정상 누적 — cum 을 LINE_SEG.vpos 절대값으로 동기화 (전진만 허용)
            let target_cum = hwpunit_to_px(cur_first_vpos, self.dpi);
            if target_cum > cum {
                cum = target_cum;
            }
        }
    }
}
```

### 1.2 변경 본질

- 차분 누적 (`delta`) 대신 **절대 vpos 동기화** — form-002 Stage 3-2 회귀 (paragraph 사이 spacing mismatch 누적) 회피
- 셀별 가드 (`cell_first_vpos == 0`) — 한컴 정상 인코딩 케이스만 적용
- `target_cum > cum` 조건 — cum 만 전진 (감소 금지). line metric 가 vpos 보다 큰 paragraph 가 있어도 영향 없음

### 1.3 가드의 의미

`cell_first_vpos == 0`: 셀 첫 paragraph 의 첫 LINE_SEG.vpos 가 0 인 경우. 이는 한컴이 셀 시작점을 기준으로 vpos 누적 인코딩한 표준 케이스. 다른 케이스 (예: 셀 첫 vpos != 0) 는 컬럼 리셋 등 다른 의미일 수 있어 동기화 적용 안전하지 않음.

검증 fixture 군 모두 `cell_first_vpos == 0` 인 한 안전. 만약 cell first vpos != 0 인 셀이 있다면 가드 미통과 → 변경 무효 → 회귀 없음.

## 2. 단위 테스트 추가

`src/renderer/layout/tests.rs` (또는 신규 파일):

1. **`test_compute_cell_line_ranges_cum_vpos_sync`**: 합성 fixture — 셀 첫 paragraph vpos=0, p[1] vpos > line metric (예: vpos=2000, line_h=500), content_limit=1500. → cum 동기화로 p[1] 가 limit 초과 처리되는지 검증
2. **`test_compute_cell_line_ranges_cum_vpos_sync_no_guard`**: cell first vpos != 0 인 셀 → 동기화 분기 미진입, 동작 변경 없음
3. **`test_compute_cell_line_ranges_vpos_reset_with_sync`**: 정상 동기화 + vpos 리셋 발생 → cum 절대 동기화 후 리셋 시 abs_limit 강제 (Task #697 분기)

## 3. 회귀 fixture 검증 (Stage 4)

| Fixture | 위험도 |
|---|---|
| `samples/inner-table-01.hwp` p1/p2 | 타겟 — `- 전사...` p2 정상 표시 + p1 cut 위치 정합 |
| `samples/hwpx/form-002.hwpx` p1 (cell[73] paras=29) | 회귀 검출 — vpos 동기화 시 paragraph 누락 없음 |
| `samples/k-water-rfp.hwp` (27p) | 큰 표 다중 페이지 |
| `samples/issue_265.hwp`, `samples/hwp3-sample.hwp` | hwp3 sample |
| Task #474, #362, #324v3, #485 fixture | abs/effective_limit 영역 |
| svg_snapshot 7 tests | golden snapshot |

## 4. 진행 단계

### Stage 3-1 — 옵션 C 구현

`compute_cell_line_ranges` 변경 적용 + 단위 테스트.

산출: `mydocs/working/task_m100_700_stage3_1.md`

### Stage 3-2 — fixture 검증 + 회귀 분석

inner-table-01 RMSE 측정 + form-002 등 회귀 fixture 군 RMSE 비교. 만약 회귀 발생 시 가드 정합화.

산출: `mydocs/working/task_m100_700_stage3_2.md`

### Stage 4 — 최종 검증 + 보고서

전체 cargo test 통과 + 시각 정합 확인 + `mydocs/report/task_m100_700_report.md`

## 5. 비목표

- 옵션 A (height_measurer 변경) — 본 task 에서 채택 안 함
- 옵션 B (paragraph y 시각 배치) — Stage 3-2 RMSE 결과 부족 시 별 sub-stage 로 검토 (회귀 가드 정합 후)
- pagination engine 의 split_end_content_limit 산출 정합 — 후속 별 task

## 6. 위험 분석

| 위험 | 대응 |
|---|---|
| form-002 회귀 (Stage 3-2 와 같음) | 가드 (`cell_first_vpos == 0` + `target_cum > cum`) 로 차분 누적 방식 회피 |
| 다른 fixture 의 cell 에서 cum 절대 동기화로 paragraph 누락 | Stage 3-2 RMSE 광범위 비교로 검출 |
| line_ranges 산출이 변경됐으나 paragraph y 시각 배치는 여전히 line metric 으로 → cell area 안에서 위치 어긋남 | 시각 RMSE 영향 — 후속 별 sub-stage 또는 task 로 분리 |

---

승인 요청: 본 구현 계획 (옵션 C) 기준으로 Stage 3-1 (구현) 진행해도 되는지 확인 부탁드립니다.
