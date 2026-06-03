# Task #656 Stage 1: 본질 정밀 측정 + 회귀 베이스 영역 구축 — 완료 보고서

## 작업 영역

소스 변경 0. typeset/layout drift 정밀 측정 + 회귀 검증 베이스 영역 수집.

## 측정 영역

### 1. typeset/layout drift 정량 (RHWP_TYPESET_DRIFT=1)

**단단 일반 (k-water-rfp.hwp p3) — Task #359 영역**:

| diff (fmt_total - vpos_h) | 빈도 | 본질 |
|---------------------------|------|------|
| +9.6 px | 142 | trail_ls (line_spacing) 가 typeset 누적에 포함, vpos 진행에는 미포함 |
| +10.4 ~ +15.7 px | 다수 | 동일 |
| +3.2 px | 25 | 단행 + line_spacing 적은 항목 |

→ **본질 식별**: typeset 의 `fmt.total_height` 누적이 vpos 측정 영역보다 항상 +3.2 ~ +15.7px 크다. 이는 N items 누적 시 N × trail_ls 만큼 layout 진행과 어긋난다.

**현재 정합 베이스 (k-water-rfp p3)**:
- typeset 단단 advance = `total_height` (trail_ls 포함, Task #359)
- layout 본문 advance = `lh + ls 모든 줄` (trail_ls 포함, Task #452)
- → 양 영역 정합 → **LAYOUT_OVERFLOW=0**

**다단 (exam_eng.hwp p8)**:
- col 0: 160 항목, col 1: 141 항목
- diff +6.6 ~ +15.7px (단단과 동일 본질)
- typeset 다단 advance = `height_for_fit` (trail_ls 제외, Task #391)
- layout 본문 advance = `lh + ls 모든 줄` (trail_ls 포함, Task #452)
- → **잠재 어긋남** (1 건 LAYOUT_OVERFLOW: page=7 pi=298 overflow=6.1px)

### 2. 분할 표 영역 정밀 (synam-001.hwp p15)

**대상**: PartialTable pi=140, 8x2, RowBreak, rows=6..7

```
body_area: x=37.8 y=75.6 w=718.1 h=990.3
PartialTable   pi=140 ci=0  rows=6..7  cont=true  8x2  vpos=9340
split_start=1280.6 split_end=965.4
abs_limit = 1280.6 + 965.4 = 2246.0
```

**LAYOUT_OVERFLOW (현 베이스, devel head, epsilon 미적용)**:

```
LAYOUT_OVERFLOW: page=11, col=0, para=140, type=PartialTable, 
  y=1071.6, bottom=1065.8, overflow=5.8px
```

→ Task #485 PR 베이스에서 epsilon (2.0px) 으로 흡수했던 영역 (실제 5.8px 초과).

**셀 13 의 영역 정보**: paras=85, h=239230 HU (매우 큰 셀)

**TYPESET_DRIFT_PI 미캡처 영역**: pi=140 은 표 (`Control::Table`) 라서 일반 fit/place 분기 (line 859 `typeset_paragraph`) 가 아닌 다른 경로 (`typeset_table` 영역) 진입. drift 측정은 `compute_cell_line_ranges` 영역으로 별도 진단 필요 → Stage 2 진입 시 정밀.

### 3. 광범위 회귀 베이스 영역

| 샘플 | 페이지 영역 | LAYOUT_OVERFLOW 건수 | 본질 |
|------|-------------|---------------------|------|
| synam-001.hwp (전체 35p) | 베이스 | 25 건 (page=1, 11, 23, 33, 36, 38, 52, 57, 66, 68, 69 등) | 분할 표 + 단독 항목 영역 |
| kps-ai.hwp (전체 80p) | 베이스 | 12 건 | Task #362 회귀 영역 잔여 |
| k-water-rfp.hwp p3 | Task #359 베이스 | 0 건 | 정합 깨끗 |
| exam_eng.hwp p8 | Task #391 베이스 | 1 건 | 다단 잔여 |

LAYOUT_OVERFLOW_DRAW (실제 그리기 overflow) 도 다수 발견:
- synam-001: pi=17, pi=74, pi=141 (분할 표 직후 텍스트), pi=515, pi=753 등

### 4. 자동 회귀 테스트 베이스

```
cargo test --release
test result: ok. 1141 passed; 0 failed; 2 ignored
```

기타 test 영역도 모두 통과. **Stage 1 자동 회귀 베이스 = 1141+ 통과, 0 실패**.

## 베이스 영역 저장 위치

- SVG 베이스: `output/svg/baseline_656/{synam-001|kps-ai|k-water-rfp-p3|exam_eng-p8|synam-001-p15}/`
- 진단 로그: `output/diagnostic/baseline_656/*.log`

## Stage 2 진입 영역 영향 평가

### typeset advance 통일 (단단/다단 → height_for_fit) 의 영역 영향

**단독 변경 영역 점검** (typeset 만 변경, layout 미변경):

- 단단: typeset 누적 -3.2 ~ -15.7px/item → layout (lh+ls 누적) 과 어긋남
- N items 누적 시 layout 진행이 typeset 추정보다 크다
- → **LAYOUT_OVERFLOW 재발 위험** (k-water-rfp p3 회귀 영역)

**양 영역 동시 변경** (Stage 2 권장 경로):

- typeset advance = `height_for_fit` (lh 누적, trail_ls 제외)
- layout advance = `lh 모든 줄` (trail_ls 제외)
- → 양 영역 정합 회복 + trail_ls 모델 영역 자체 제거

### 분할 표 영역 영향

- typeset 의 `split_end_content_limit = avail_content` 추정과 layout 의 `compute_cell_line_ranges` 의 line_h 누적 어긋남 → 본질 5.8px overflow
- 양 advance 모델 통일 시 → 분할 표 영역도 자연 정합 (epsilon 영역 불요)

### 정밀 변경 영역 식별

| 위치 | 변경 영역 | Stage |
|------|-----------|-------|
| `src/renderer/typeset.rs:991, 1027, 1043` | `if col_count > 1 { height_for_fit } else { total_height }` → `height_for_fit` | Stage 2 |
| `src/renderer/layout/paragraph_layout.rs:2640-2652` | 본문 단락 마지막 줄 `y += line_height + ls` → `y += line_height` (셀 영역과 동일) | Stage 2 |
| `src/renderer/layout.rs:1474-1505` (vpos correction) | `prev_vpos_end = vpos + lh + ls` 영역 | Stage 3 (조건부) |
| `src/renderer/layout/table_layout.rs (compute_cell_line_ranges)` | epsilon 도입 회피 입증 (변경 없이) | Stage 4 |

## Stage 1 결과 영역 결론

1. typeset/layout drift 의 본질 = **trail_ls 처리 영역의 두 모델 어긋남** 정량 입증 (~9.6px/item)
2. 현 정합 베이스 영역은 **trail_ls 포함 모델** (typeset 단단 = total_height, layout = lh+ls). 본 타스크는 **trail_ls 제외 모델** (height_for_fit, lh 만) 로 통일.
3. 분할 표 영역 (synam-001 p15) 의 5.8px overflow 도 동일 본질의 발현. epsilon 영역 자연 해소 가능 영역.
4. Stage 2 진입 영역 = **typeset + layout 양 영역 동시 변경 필수** (단독 변경 시 회귀).
5. 자동 회귀 베이스 = 1141+ 통과, 0 실패. Stage 2 진입 후 동일 영역 입증 필수.

## Stage 2 진입 승인 요청 영역

작업지시자 결정 영역:
1. Stage 1 결과 영역 적정성
2. Stage 2 변경 영역 (typeset + layout 동시 변경) 진입 영역 승인
