# Task #656: typeset/layout height 측정 모델 통일 — 최종 보고서

## 요약

`compute_cell_line_ranges` 의 break 비교 시 마지막 visible 줄의 trail_ls 제외 — 셀 마지막 줄 미렌더 모델 (`is_cell_last_line`) 과 일관 영역. Task #485 의 epsilon (2.0px) 휴리스틱 영역을 본질적으로 대체.

**1 영역 변경 + 1 골든 갱신** 으로 분할 표 영역 클립 본질 정정 + 광범위 회귀 0 입증.

## 본 타스크의 본질

이슈 #656 의 본질:
- Task #485 의 epsilon = 2.0px 가 임시방편 (typeset 의 split_end_limit 추정과 layout 의 line_h 누적 어긋남 흡수용)
- 본 타스크는 그 어긋남의 본질을 일관 모델 영역으로 정정

선행 시도 (Task #331 단일 모델 통합) 는 회귀로 revert 됨. 본 타스크는 단계적 접근.

## 진행 영역 요약

| Stage | 영역 | 결과 | 영역 |
|-------|------|------|------|
| Stage 1 | 본질 정밀 측정 + 회귀 베이스 영역 구축 | drift 정량, 베이스 영역 수집 | af9238f7 |
| Stage 2 | typeset/layout advance 모델 통일 시도 (단단/다단 통일 + 본문 마지막 줄 trail_ls 제외) | **회귀 발생** (kps-ai +50, k-water-rfp +41) → 후퇴 | 543ba094 |
| Stage 3 | `compute_cell_line_ranges` break 조건 본질 정정 (trail_ls 제외) | **본질 정정 + 회귀 0** | 72fe32f4 |
| Stage 4 | 광범위 회귀 검증 + 골든 갱신 + 최종 보고 | 본 보고서 + golden 영역 1건 | (본 영역) |

Stage 2 의 회귀는 본 타스크의 본질 영역 (typeset/layout advance 광범위 통일) 의 광범위함을 입증. 후보 B (분할 표 영역 만 정정) 진입으로 본질 정정 영역 도달.

## Stage 3 변경 영역 정밀

### 1 영역 변경 (`src/renderer/layout/table_layout.rs:2355-2371`)

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
    break;
}
```

### 1 골든 갱신 (`tests/golden_svg/form-002/page-0.svg`)

- 베이스: 840 `<text>` 요소
- 갱신: 866 `<text>` 요소 (+26 글자, "ㅇPFC 나노산소운반체의 최적제조공정개발 및 GMP실증")
- 본 추가는 분할 표 첫 페이지의 마지막 visible 줄이 클립되었던 영역의 본질 정정 효과

## 진단 측정 결과 (synam-001 p15)

```
CELL_DRIFT_BREAK: pi=61 li=0 cum=2229.33 line_h=23.47 line_end_pos=2252.80 
                  abs_limit=2245.97 delta=+6.83 
                  content_offset=1280.56 content_limit=965.41
                  h=14.67 ls=8.80
```

**본질**:
- `cum + h = 2244.00 ≤ abs_limit (2245.97)` → 줄 자체는 들어감
- `cum + h + ls = 2252.80 > abs_limit` → trail_ls 까지 포함 시 6.83px 초과

→ break 비교 시 trail_ls (8.80px) 가 포함되어 그 줄이 미렌더. 본 정정으로 trail_ls 제외 비교 → 그 줄 visible.

## 회귀 검증 결과

### 본질 영역 (분할 표) 자연 해소

| 영역 | 베이스 | Stage 3 | 평가 |
|------|--------|---------|------|
| **synam-001 p15 PartialTable OVERFLOW** | 5.8px overflow | **0** | **본질 정정 ✓** |
| **form-002 page-0 분할 표 마지막 visible 줄** | 26 글자 클립 | **렌더** | **본질 정정 ★** |

### 광범위 회귀 영역 (베이스 vs Stage 3 비교)

| 샘플 | 베이스 (페이지/OVERFLOW) | Stage 3 | 평가 |
|------|--------------------------|---------|------|
| synam-001 (35p) | 25 OVERFLOW | 25 | 변화 없음 |
| aift (77p) | 13 | 13 | 변화 없음 |
| biz_plan (6p) | 1 | 1 | 변화 없음 |
| exam_science (4p) | 1 | 1 | 변화 없음 |
| exam_eng (8p) | 13 | 13 | 변화 없음 |
| kps-ai (80p) | 12 | 12 | 변화 없음 |
| k-water-rfp (27p) | 3 | 3 | 변화 없음 |
| hwp-multi-001 (10p) | 1 | 1 | 변화 없음 |

→ **분할 표 영역 외 회귀 0**. 본 정정이 분할 표 break 영역만 정확히 정정.

### 자동 회귀

```
cargo test --release: 1141 passed; 0 failed; 2 ignored
```

골든 영역 갱신 (form-002/page-0) 후 svg_snapshot 모두 통과.

## 본질 영역 비교 (Task #485 vs Task #656)

| 영역 | Task #485 (origin/pr/task-485) | Task #656 |
|------|-------------------------------|-----------|
| 정정 영역 | break 조건에 epsilon = 2.0px 추가 | break 비교 시 trail_ls 제외 |
| 본질 | 휴리스틱 마진 (임시방편) | 일관 모델 (셀 마지막 줄 미렌더 정합) |
| 영역 흡수 | epsilon ≥ 어긋남 영역 시 정정 | 모든 trail_ls 어긋남 자연 정정 |
| 폰트 의존성 | 폰트 변경 시 epsilon 부족 가능 | 폰트 무관 (모델 일관) |
| 회귀 영역 | epsilon < 어긋남 영역 시 회귀 | trail_ls 모델 외 영역만 회귀 (현 측정 0) |

→ 본 타스크 정정이 Task #485 의 epsilon 영역을 **본질적으로 대체**.

## Task #485 PR 처리 영역

### 영역

origin/pr/task-485 의 4 commit:
- 3965b519: Stage 1 본질 정밀 측정
- 5f56c667: Stage 2a out-of-order 정정 (limit_reached 플래그)
- 53effd17: Stage 2b boundary epsilon (2.0px) 적용
- 8188e29f: Stage 3 회귀 검증
- 7e432692: Stage 4 최종 보고서 + orders

### 영역 본질 비교

본 Task #656 정정이 흡수하는 영역:
- **53effd17 (Stage 2b boundary epsilon)** — 본 타스크 정정의 본질 영역. Task #656 으로 자연 대체

본 Task #656 영역 외 (Task #485 별도 영역):
- **5f56c667 (Stage 2a out-of-order 정정)** — `limit_reached` 플래그 영역. break 후 out-of-order 처리 (`compute_cell_line_ranges` 의 다른 영역). Task #656 영역 외.

### 작업지시자 결정 영역

| 영역 후보 | 영역 |
|-----------|------|
| **A: Task #485 PR close** | 본 Task #656 머지 후 Task #485 PR 의 epsilon 영역 자연 해소. PR close + 사유 보고. out-of-order 영역 별도 처리 (별도 이슈 영역) |
| **B: Task #485 PR 영역 cherry-pick (out-of-order 영역만)** | Task #485 의 5f56c667 (Stage 2a out-of-order 정정) 만 별도 처리 |
| **C: Task #485 PR 영역 외면** | Task #485 PR 의 영역 그대로 영역. 본 Task #656 영역 머지 후 Task #485 PR 영역은 영역 외면 (자연 close) |

권장: **C** — 본 Task #656 정정이 Task #485 의 epsilon 영역을 본질 대체. out-of-order 영역은 본 정정 후 발현 영역 점검 후 별도 이슈 영역 결정 (현재까지 회귀 0 영역으로 본질 영역 외 발현 미식별).

## 오늘할일 갱신

`mydocs/orders/20260507.md` — Task #656 완료 영역 추가.

## 머지 영역

- 작업지시자 시각 판정 통과 시 → `local/devel` 머지 → `devel` 머지 → push origin devel
- Issue #656 close 영역
- Task #485 (origin/pr/task-485) PR 처리 영역 (작업지시자 결정 영역)

## 결론

본 타스크의 본질 정정 영역 도달:
1. **분할 표 셀 마지막 visible 줄 클립 본질 정정** (synam-001 p15, form-002 page-0)
2. **Task #485 의 epsilon 휴리스틱 영역을 일관 모델 영역으로 대체**
3. **광범위 회귀 0** + 자동 회귀 0
4. **Stage 2 시도 (광범위 단일 모델 통합) 후퇴 영역 입증** — 본 영역의 본질이 광범위함, 분할 표 영역 만 정정 영역이 적정
