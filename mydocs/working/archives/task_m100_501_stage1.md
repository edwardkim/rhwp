# Task #501 Stage 1 — 회귀 본질 진단 보고서

## 진단 결과

### 표 자체는 정합

`samples/mel-001.hwp` 2쪽 s0:pi=22 (8행×12열 인원현황 표):

| 항목 | rhwp 출력 | HWP IR 정합 |
|------|----------|-------------|
| 표 시작 y | 637.4 | vpos=93039 정합 |
| 표 전체 높이 | 146.13px | cell.h=10960 HU = 38.7mm = 146.6px ✓ |
| 표 너비 | 630.4px | 47280 HU = 166.8mm = 631.0px ✓ |

→ **표 자체 높이/너비/위치 모두 정합**. dump-pages 출력 동일.

### 회귀 본질 = rowspan rs=2 셀의 vertical-center 정렬 결함

작업지시자 정합: \"표높이는 정상. 셀의 높이가 문제입니다.\"

**셀 IR 구조**:

| 셀 | r,c | rs,cs | h (HU) | text | paragraphs |
|----|-----|-------|--------|------|-----------|
| [0] | 0,0 | 1,3 | 1980 | \"직급별\\n구 분\" | 2 |
| [10] | 1,0 | **2,2** | **2560** | \"합계\" | 1 |
| [11] | 1,2 | 1,1 | 1280 | \"정 원\" | 1 |
| [21] | 2,2 | 1,1 | 1280 | \"현 원\" | 1 |
| 본부 셀 (rs=2) | 3,0 | **2,2** | 2560 | \"본부\" | 1 |
| ... | | | | | |

### SVG 출력 분석 — text baseline 위치

표 시작 y=637.4 부터 각 행 baseline 위치:

| 행 | 콘텐츠 | baseline y | 위치 정합 |
|----|-------|-----------|----------|
| r=0 line1 | \"직급별\" (셀[0] p[0]) | 650.61 | 행 0 영역 (637.4~663.8) 내 |
| r=0 line2 | \"구 분\" (셀[0] p[1]) | 659.95 | 행 0 영역 내 |
| **합계 (rs=2 셀[10])** | \"합계\" (라벨) | **691.76** | r=2 영역 위치 ★ |
| r=1 정원 | \"정 원\" (셀[11]) | 698.42 | r=2 영역 (681~698) **하단 초과** |
| **본부 (rs=2)** | \"본부\" (라벨) | **707.59** | r=4 영역 |
| r=3 정원 | \"정 원\" (셀 본부행) | 698.42 → 다음 위치 | — |
| r=4 현원 | \"현 원\" | 727.20 | r=4 영역 |

### 결정적 발견 — vertical-center 결함

**rowspan rs=2 셀 (\"합계\", \"본부\" 등) 의 텍스트 baseline 이 두 번째 차지 행 (r=2 또는 r=4) 영역에 위치**.

정상 (vertical-center) 이라면:
- \"합계\" 라벨 → 두 행 (r=1, r=2) 중앙 baseline (≈ 행1과 행2 중간)
- \"본부\" 라벨 → 두 행 (r=3, r=4) 중앙

회귀 (현재 rhwp):
- \"합계\" 라벨 → r=2 행 영역 → r=1 의 \"정 원\" + 숫자 라벨 위에 가려짐 (혹은 r=1 행 자체가 미렌더)
- \"본부\" 라벨 → r=4 행 영역 → r=3 의 \"정 원\" 라벨 위에 가려짐

→ **회귀 = rowspan 셀의 텍스트 vertical 정렬에서 두 번째 행 영역 (r=last) 으로 이동**. 정상은 첫 번째 행 (r=first) 또는 두 행 합산 영역의 중앙.

### IR 의 valign / vert-align 정합

dump-pages 의 셀 valign (HWP IR) 확인 필요:
- 셀[10] (합계, rs=2): IR valign=Top (또는 Middle?)
- 셀[11] (정 원, rs=1): valign=...

회귀가 valign=Middle 인 rowspan 셀이 두 번째 행 영역에만 그려지는 것이라면:
- 셀의 visible 영역이 (r=last, r=last+rs-1) 로 잘못 계산
- 또는 셀의 y_offset 이 r=last 의 y_top 으로 시작

## 회귀 origin 후보 영역 좁히기

archives 의 본 영역 정정 이력 + 회귀 본질 정합:

| 영역 | 후보 정합 |
|------|----------|
| **Task #44** (분할 표 셀 세로 정렬 — 중첩 표 높이 반영 + 분할 행 Top 강제) | rowspan 영역 — 핵심 의심 |
| **Task #347** (셀 첫 줄 y 에 LineSeg.vertical_pos 적용 + valign 통합) | 셀 첫 줄 y 계산 — 핵심 의심 ★ |
| Task #214 (TypesetEngine 전환) | 셀 path 변경 |
| Task #324 (compute_cell_line_ranges) | line range 측정 |

핵심 의심 — **Task #347** (\"표/그림 절대 좌표 계산 5건 통합 수정\" + \"셀 첫 줄 y 에 LineSeg.vertical_pos 적용 (vertical_align 통합)\"). 영역 정합:
- vertical_align 통합 → rowspan 셀의 vertical-center 처리 영역 변경
- 셀 첫 줄 y → vpos 적용 → rowspan 셀의 y_offset 시작점 영역

## 다음 단계 (Stage 2)

1. Task #347 의 \"vertical_align 통합\" 변경 영역 정독
2. layout_table 의 rowspan 셀 vertical-center 처리 path 분석
3. 회귀 origin commit 확정
4. 정정 방향 결정 → 구현 계획서 (`task_m100_501_impl.md`) 작성 + 승인

## 검증 자료

- `output/mel-001_002.svg` (디버그 오버레이 SVG, 576KB)
- dump 출력 (셀 IR 구조 정합)
- dump-pages 출력 (표 영역 정합)
