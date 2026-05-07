# Task #656 Stage 3: compute_cell_line_ranges break 조건 본질 정정 — 완료 보고서

## 작업 영역

후보 B (분할 표 영역 만 정정) 진입. `compute_cell_line_ranges` 의 break 비교 시 마지막 visible 줄의 trail_ls 제외 (셀 마지막 줄 미렌더 모델과 일관).

## 진단 영역 측정 결과 (synam-001 p15)

진단 출력 (RHWP_CELL_DRIFT=1) 으로 정확한 어긋남 origin 식별:

```
CELL_DRIFT_BREAK: pi=61 li=0 cum=2229.33 line_h=23.47 line_end_pos=2252.80 
                  abs_limit=2245.97 delta=+6.83 
                  content_offset=1280.56 content_limit=965.41
                  h=14.67 ls=8.80
```

**본질**:
- `cum + h = 2229.33 + 14.67 = 2244.00 ≤ abs_limit (2245.97)` → 줄 자체는 들어감
- `cum + h + ls = 2252.80 > abs_limit` → trail_ls 까지 포함 시 6.83px 초과

→ break 비교 시 **마지막 visible 줄의 trail_ls (= 8.80px) 가 포함**되어 그 줄이 미렌더 처리. 그러나 셀 영역 마지막 줄의 trail_ls 는 다음 줄 직전 spacing 이므로 **페이지 끝에서는 미렌더** — 비교 영역에서도 제외해야 일관.

이는 Task #485 의 epsilon (2.0px) 영역의 본질 정정 — 휴리스틱 마진 영역이 아닌 본질적 모델 영역.

## 변경 영역

### `src/renderer/layout/table_layout.rs:2355-2371` (1 영역)

```rust
// [Task #656] break 비교 시 마지막 visible 줄의 trail_ls 제외.
// - cum 누적은 line_h (h+ls) 그대로 (이전 줄들의 ls 는 다음 줄 직전 spacing 이므로 렌더)
// - break 비교는 line_break_pos = cum + h (이 줄의 ls 제외) 로 비교
//   → 이 줄이 visible 시 마지막 줄이면 trail_ls 미렌더 영역, abs_limit 안에 들어감
// typeset 의 `split_end_limit = avail_content` 추정 영역과 정합. 셀 영역
// `is_cell_last_line` 분기 의 trail_ls 미렌더 모델과 동일 본질.
// (Task #485 의 epsilon 영역 의 본질 정정 — 휴리스틱 마진 없이 일관된 모델.)
let line_break_pos = cum + h;
if has_limit && line_break_pos > abs_limit {
    // limit 초과 → 이 줄과 이후 모든 콘텐츠 차단
    break;
}
```

## 회귀 검증 결과

### 본질 영역 (분할 표) 자연 해소 입증

| 영역 | 베이스 | Stage 3 | 평가 |
|------|--------|---------|------|
| **synam-001 p15 PartialTable OVERFLOW** | **5.8px overflow** | **0** | **본질 정정 ✓** |
| **form-002 page-0 분할 표 마지막 visible 줄** | 26 글자 미렌더 (클립) | **렌더 ★** | **본질 정정 ✓** |

### 광범위 회귀 영역 비교

| 샘플 | 베이스 (페이지/OVERFLOW) | Stage 3 | 평가 |
|------|--------------------------|---------|------|
| synam-001 (35p) | 25 OVERFLOW | 25 | 변화 없음 (분할 표 외 영역 보존) |
| kps-ai (80p) | 12 OVERFLOW | 12 | 변화 없음 |
| k-water-rfp (27p) | 3 OVERFLOW | 3 | 변화 없음 |
| exam_eng p8 | 1 OVERFLOW | 1 | 변화 없음 |
| aift (77p) | 13 OVERFLOW | 13 | 변화 없음 |
| biz_plan (6p) | 1 OVERFLOW | 1 | 변화 없음 |
| exam_science (4p) | 1 OVERFLOW | 1 | 변화 없음 |
| hwp-multi-001 (10p) | 1 OVERFLOW | 1 | 변화 없음 |

→ **분할 표 영역 외 회귀 0**. 본 영역 정정이 분할 표 break 영역만 정확히 정정.

### 자동 회귀 (cargo test)

```
test result: ok. 1141 passed; 0 failed; 2 ignored
```

**1 건 골든 영역 갱신 필요**: `tests/golden_svg/form-002/page-0.svg`

- 베이스: 840 `<text>` 요소
- Stage 3: 866 `<text>` 요소 (+26 글자, "ㅇPFC 나노산소운반체의 최적제조공정개발 및 GMP실증")
- 본 추가 영역은 **본질 정정의 의도된 효과** (분할 표 첫 페이지의 마지막 visible 줄 클립 해소)

작업지시자 시각 판정 후 `UPDATE_GOLDEN=1 cargo test --test svg_snapshot` 영역 갱신.

## 본질 영역 비교 (Task #485 vs Task #656)

| 영역 | Task #485 (origin/pr/task-485) | Task #656 Stage 3 |
|------|-------------------------------|-------------------|
| 정정 영역 | break 조건에 epsilon = 2.0px | break 조건에서 trail_ls 제외 |
| 본질 | 휴리스틱 마진 (임시방편) | 일관 모델 (셀 마지막 줄 미렌더 정합) |
| 영역 흡수 | epsilon ≥ 어긋남 영역 시 정정 | 모든 trail_ls 어긋남 자연 정정 |
| 회귀 영역 | epsilon < 어긋남 영역 시 회귀 | trail_ls 모델 외 영역만 회귀 (현 측정 0) |
| 폰트 의존성 | 폰트 변경 시 epsilon 부족 가능 | 폰트 무관 (모델 일관) |

→ 본 정정 영역이 Task #485 의 epsilon 영역을 **본질적으로 대체**.

## Task #485 PR 처리 영역

origin/pr/task-485 의 epsilon 변경분 영역:
- `compute_cell_line_ranges` 의 break 조건에 `+ 2.0` epsilon 추가
- out-of-order 정정 (limit_reached 플래그)

본 Stage 3 영역 정정은 epsilon 영역을 자연 해소. **Task #485 PR 의 epsilon 영역만큼은 본 타스크 머지 후 close 가능**. 단 out-of-order 정정 (limit_reached 플래그) 영역은 본 타스크 영역 외 (별도 점검 필요).

작업지시자 결정 영역:
1. Task #485 PR 의 out-of-order 정정 영역 본 타스크 영역 진입 여부 (Stage 4 진입 영역)
2. Task #485 PR close 영역 결정

## Stage 4 진입 영역 후보

본 Stage 3 의 결과 (분할 표 본질 정정 + 회귀 0) 가 본 타스크 본질 영역의 핵심 영역 달성. Stage 4 진입 영역:

### 후보 4-A: 광범위 회귀 검증 + 최종 보고

- 추가 시각 회귀 점검 (분할 표 샘플 영역, 작업지시자 시각 판정)
- 골든 영역 갱신 (form-002/page-0)
- 최종 보고서 작성
- 본 타스크 종결

### 후보 4-B: Task #485 의 out-of-order 정정 영역 진입

- Task #485 PR 의 out-of-order 정정 (limit_reached 플래그) 영역 본 타스크 진입
- 본 영역의 본질 점검 후 정정 진입

### 권장: 후보 4-A

본 Stage 3 영역 정정이 본 타스크 본질 영역의 핵심 영역을 처리. out-of-order 영역은 별도 영역 (Task #485 의 회귀 영역) 으로 본 타스크 영역 외면 가능.

## Stage 4 진입 승인 요청

작업지시자 결정 영역:
1. 본 Stage 3 의 본질 정정 영역 적정성
2. Golden 영역 (form-002/page-0) 갱신 영역 — 시각 판정 영역
3. Stage 4 진입 영역 (4-A 권장)
