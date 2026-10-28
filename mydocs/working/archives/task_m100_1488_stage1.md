# Task M100 #1488 Stage 1 — 진단 결과

- 브랜치: `local/task_m100_1488`
- 작성일: 2026-06-25
- 단계: Stage 1 (진단, 코드 무수정)

## 1. 재현 현황

| 항목 | 값 |
|------|-----|
| 샘플 | `samples/rowbreak-problem-pages.hwpx` |
| 정답지 | `pdf/rowbreak-problem-pages-2024.pdf` = **18페이지** |
| clean devel(`4538a02c`) | **22페이지** (4페이지 초과) |
| 이슈 제기 시점(`e678104`) | 24페이지 |

관측 `LAYOUT_OVERFLOW`: sec0 p2/p3/p6, sec1 p1/p2/p14 (PartialTable/Table).

## 2. 근본 원인 A — 오버레이 vpos 리셋 하드 브레이크 남발 (주범, 확정)

문제 표: **섹션 1, 문단 28, control[0]** = `1행×1열 RowBreak 표`, 셀 1개에 18문단, 셀 h=86740 HU(≈1157px, 페이지 가용 ≈954px → 2페이지 분할이 정상).

`RHWP_TABLE_DRIFT=1` 진단 (pi=28 sec=1):

```
fragment 1: consumed=185.6  avail=186.5
fragment 2: consumed=84.9   avail=954.8   ← 954px 가용에 85px만 배치 후 페이지 넘김
fragment 3: consumed=32.8   avail=954.8
fragment 4: consumed=61.8   avail=954.8
fragment 5: consumed=61.8   avail=954.8
fragment 6: consumed=61.8   avail=954.8
fragment 7: consumed=918.0  avail=954.8   ← 실제 본문
```

### 메커니즘 (코드 경로)

- 셀 내부 빈 문단 `p[6..15]`(text_len=0)는 vpos 가 직전 문단 끝보다 작은 **오버레이(역방향) 리셋**.
  추가로 각 빈 문단의 두 줄(ls[0],ls[1])이 **동일 vpos**(겹침)라 줄 사이에도 리셋.
- `cell_units`(`table_layout.rs`)가 이 리셋을 `hard_break_before=true`로 표시
  (`reset_before` 4284행 / `line_reset_before` 4312행).
- `advance_row_cut`(4399행)는 `j>start && u.hard_break_before`에서 **가용 예산과 무관하게 즉시 컷 종료**.
- → 리셋(오버레이)마다 fragment 1개 = 거의 빈 페이지 1장. 본문(p[0..5])·도식(p[16..17])이
  같은 셀을 공유하는 오버레이 구조가 세로로 펼쳐져 여분 페이지 5장 + 페이지 2 본문·도식 겹침으로 나타남.

### 핵심 판단

`hard_break_before`는 Task #993에서 **가시 텍스트 문단 사이**의 진짜 페이지 분할 경계를 위해 도입됨
(`test_advance_row_cut_vpos_reset_hard_break`는 `text_para`(가시 텍스트) 간 리셋을 검증).
**빈(비가시) 오버레이 스페이서 문단이 만든 리셋까지 하드 브레이크로 처리하는 것이 과적용(false positive)**이다.
[[tech_trailing_model_no_ssot]] 교훈에 따라 광범위 통일 대신 **비가시 유닛 리셋만 하드 브레이크에서 제외**하는
조건부 게이트가 적절하다.

## 3. 근본 원인 B — PartialTable fragment 높이 페이지 초과

- `LAYOUT_OVERFLOW`(sec0 p2/p3/p6, sec1 p1/p2/p14)는 분할 표 fragment 가 페이지 바닥을 넘는 케이스.
- 이슈의 10p/23p 하단 표 잘림에 대응. 근본 원인 A 보정으로 fragment 경계가 재정렬되면 일부 자동 해소
  가능성이 있으나, 잔여분은 Stage 3에서 fragment 높이/가용 계산 보정으로 처리.

## 4. 시각 결함 ↔ 근본 원인 매핑

| 이슈 결함(24p 기준) | 분류 | 처리 단계 |
|----------|------|----------|
| 17~22p 여분 빈 페이지 | 근본 A | Stage 2 |
| 2p 본문·도식 겹침 | 근본 A (동일 셀 오버레이) | Stage 2 |
| 10p/23p 하단 표 잘림 | 근본 B (LAYOUT_OVERFLOW) | Stage 3 |
| 7p 셀 텍스트 행 겹침 | PartialTable 셀 렌더(분할 표) — A/B 보정 후 재판정 | Stage 3 |
| 12p 파란 콜아웃 잘림 | 1x1 분할 표(pi=13류) — A/B 보정 후 재판정 | Stage 3 |
| 16p 상단 컷오프 | 근본 A 연속(빈 페이지 인접) | Stage 2 |

모든 결함이 RowBreak/PartialTable cut 분할 계열에 수렴한다. 7p/12p 는 별도 신규 로직이 아니라
분할 표 렌더 산물이므로 본 타스크 범위에 포함하되, A/B 보정 후 잔여 여부로 추가 판정한다.

## 5. Stage 2 방향

`cell_units`에서 **비가시(빈 텍스트) 문단/줄이 만든 vpos 리셋은 `hard_break_before`로 표시하지 않는다.**
가시 텍스트 문단 간 리셋(Task #993 의도)은 보존. 이로써 오버레이 스페이서가 페이지를 강제 분할하지 않고
가용 예산까지 정상 패킹되어 여분 빈 페이지가 제거된다. 전체 cargo test + baseline 으로 회귀 검증.
