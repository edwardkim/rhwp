# Task #295 3단계 — 수정 + 단위 검증 보고서

## 가설 확정 (3-1)

임시 `eprintln!`으로 `layout_table_item` 진입 시 표 속성과 `y_offset` 변화를 추적하여 가설을 정정·확정했다.

```
DBG_TASK295 layout_table_item pi=22 ci=0 is_tac=false vert=Page wrap=TopAndBottom
            valign=Bottom v_off=141 h=10102 y_in=147.4 body=[147.4..1360.6] page_h=1489.1
DBG_TASK295 layout_table_item pi=22 ci=1 ... y_in=1516.1
DBG_TASK295 layout_table_item pi=27 ci=0 ... y_in=1371.8
```

정정된 진단:

- 1단계 보고서에서 `vert=Paper`로 추정했으나 실제 `pi=22 ctrl[0]`은 `vert=Page` (덤프에서 본 `vert=Paper/101954`는 바탕쪽 객체였음).
- pi=22 ctrl[0]은 `vert=Page valign=Bottom`의 페이지 하단 앵커 푸터 표.
- 처리 후 `y_offset`이 `147.4 → 1516.1`로 점프 (표 하단 좌표).
- 후속 항목 pi=22 ci=1, pi=27 ci=0 등이 모두 본문 하단 부근에서 시작 → 좌단 콘텐츠 압축·겹침.

기존 `renders_above_body` 분기는 `VertRelTo::Paper` + `tbl_y < body.y` 조건으로 머리말 자리 표만 out-of-flow 처리. **vert=Page** 및 **본문 아래** 케이스가 모두 누락.

## 수정 (3-2)

`src/renderer/layout.rs::layout_table_item`의 두 분기 확장:

1. `renders_above_body` → `renders_outside_body` (≈1995행)
   - vert: `Paper` → `Paper | Page`
   - 위치: `tbl_y < body.y` → `tbl_y < body.y || tbl_y + tbl_h > body_bottom`
2. `is_above_body` → `is_outside_body` (≈2100행)
   - 동일 확장 (표 하단 간격/spacing_after 추가 여부)

판정 기준은 표 상단이 본문 위(머리말 영역)이거나 표 하단이 본문 아래(꼬리말 영역)에 걸치면 본문 흐름과 분리. 두 조건 모두 거짓일 때(=본문 영역 내부)만 기존 in-flow 동작 유지.

## 단위 검증 (3-3)

### LAYOUT_OVERFLOW

| 시점 | 12쪽 | exam_math.hwp 전체 | exam_math_no.hwp 전체 |
|------|------|--------------------|----------------------|
| 수정 전 | 18건 | (다수) | (다수) |
| 수정 후 | **0건** | **0건** | **0건** |

### 12쪽 좌단 항목 y 좌표

| 항목 | 수정 전 | 수정 후 | 비고 |
|------|---------|---------|------|
| pi=22 (단나누기+푸터) | 1371.5 | 147.4 | 단 시작 (푸터는 out-of-flow로 절대좌표 배치) |
| pi=23 (29번 본문) | **1340.1** ❌ | **178.7** ✅ | 본문 상단 부근 |
| pi=24 (빈) | 1345.3 | 256.0 | |
| pi=25 (본문) | 1345.3 | 282.2 | |
| pi=26 (빈) | 1345.3 | 363.4 | |
| pi=27 (표+수식) | 1371.8 | 390.0 | |
| pi=28..33 (우단) | 147.4..497.3 | 147.4..497.3 | 변화 없음 |

### 시각 비교

- `mydocs/working/task_m100_295_p12_pdf.png` — PDF 정답
- `mydocs/working/task_m100_295_p12_before.png` — 수정 전 (좌단 붕괴)
- `mydocs/working/task_m100_295_p12_after.png` — 수정 후 (좌단 정상)

좌단 본문이 정상적으로 위에서 아래로 흐르고, 푸터 박스/페이지번호 박스/우단 콘텐츠 모두 정상 위치 유지.

### 회귀 점검 — 머리말 표가 있는 페이지

`exam_math.hwp` 1쪽: `vert=Page` 머리말 표(2025학년도 대학수학능력시험 문제지, "제 2 교시", "1") + 풋터 "1/20" 모두 정상 위치 유지. 좌·우단 본문 흐름 정상.

### 기타 샘플

| 샘플 | LAYOUT_OVERFLOW |
|------|-----------------|
| `equation-lim.hwp` | 0건 |
| `text-align-2.hwp` | 0건 |

### cargo test

```
test result: ok. 983 passed; 0 failed; 1 ignored
test result: ok. 14 passed; 0 failed
test result: ok. 25 passed; 0 failed
test result: ok. 6 passed; 0 failed   (svg_snapshot)
total: 1028 passed, 0 failed
```

## 추가 수정 — pi=27 wrap=Square 호스트 잔여 문제

좌단 붕괴 수정 후 드러난 세 가지 잔여 문제를 동일 타스크 범위에서 일괄 수정.

### 문제 1: 표 머리행 누락 / 호스트 본문 누락

`layout_table_item`이 wrap=Square 표에 대해 `layout_wrap_around_paras`를 호출할 때 `!wrap_around_paras.is_empty()` 조건으로 가드됨. pi=27처럼 후속 wrap 문단이 없는 자가 wrap host의 경우 함수 자체가 호출되지 않아 호스트 본문이 전혀 렌더링되지 않음.

**수정** (layout.rs ≈2086): 가드 제거 — Square wrap이면 wrap_around_paras 비어 있어도 호출.

### 문제 2: 호스트 본문 다중 줄 누락

`layout_wrap_around_paras` 호스트 텍스트 렌더링이 다중 LINE_SEG 문단에서 첫 줄만 렌더링하도록 하드코딩됨 (`text_end_line = start_line + 1`).

**수정** (layout.rs ≈2509): 모든 텍스트 줄을 wrap 영역에 렌더링.

### 문제 3: Square 표가 좌측에 강제 배치 (halign 무시)

`layout_table_item`의 `tbl_inline_x` 계산이 wrap=Square이면 무조건 `col_area.x`(좌측). pi=27은 halign=Right이지만 좌측에 배치되어 wrap 텍스트와 겹침.

**수정** (layout.rs ≈1980): wrap=Square + halign 분기 추가 (Left/Right/Center).

### 결과

12쪽 좌단이 PDF와 거의 일치:
- 표 머리행 "X | P(0≤Z≤X)" + 4 데이터 행 모두 표시
- 호스트 본문 "P(15≤X≤20)+P(15≤Y≤20)의 값을 다음 표준정규분포표를 이용하여…" 5줄 모두 표시
- 표가 우측에 위치, 본문이 좌측에 wrap

## 잔여 별건 (본 타스크 외)

부수 문제(머리말 페이지번호 4↔2)는 별도 이슈 분리 권고 (1단계 보고서 명시).

## 변경 파일

- `src/renderer/layout.rs`
  - `renders_outside_body`(≈1995), `is_outside_body`(≈2110): vert=Page까지 + 본문 위/아래 양쪽
  - `tbl_inline_x` Square 분기(≈1978): halign에 따라 좌/중/우
  - 어울림 호출 가드 제거(≈2090): wrap_around_paras 비어 있어도 호출
  - `layout_wrap_around_paras` 호스트 텍스트(≈2505): 다중 줄 모두 렌더링

## 임시 산출물

- `output/exam_math_p12/`, `mydocs/working/task_m100_295_p12_*.png`
