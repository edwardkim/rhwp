---
kind: working
task: 4042
stage: 2
status: draft
last_verified: 2026-08-05
---

# Task #4042 Stage 2 구현계획서 — 버그 B: 다중열 행 내부(intra-row) 분할

## 0. 위치

- 브랜치: `local/task4042`
- 선행 완료·커밋: 중앙정렬(`bf0cf05b8`), 버그 A 폭 재단(`69b8e6e45`)
- 이 단계: 부모 셀이 편집영역 하단까지 채우고, 큰 중첩 표 행을 쪽 경계에서
  줄 단위로 분할해 다음 쪽으로 이어지게 한다. 쪽수 24→17 목표.

## 1. 확정 근인 (워크플로 + 코드 정독)

`table_layout.rs:5842` path-1: 텍스트 없는 문단의 단일 중첩 표(≥2행)를 **행 단위
원자 CellUnit**(`nested_row=Some(ri)`, height=rh)으로 분해한다. 조문대비표 제9조 ②
행(605px, 우열 12줄+)은 **한 개의 원자 유닛**이라, `advance_row_cut`(6825
`h + u.height > avail_height → break`)이 남은 288px에 못 넣고 통째 이월한다.
결과: 페이지 2 하단 288px 낭비 + 페이지 3 상단에 같은 행 재렌더(겹침).

한컴: 그 행을 쪽 경계에서 **줄 단위로 분할**(좌우 열이 각각 하단까지 채우고
페이지 3로 이어짐). = 작업지시자 규정 "부모 셀이 페이지 끝까지 채우고 다음
페이지에서 이어짐".

## 2. 접근 (작업지시자 승인: 줄 단위 행 분할)

원자 행 유닛을 **줄 단위 sub-row 유닛**으로 대체한다. `advance_row_cut`은 sub-row
유닛 사이에서 컷하므로 별도 로직 변경 없이 행 내부에서 잘린다. 렌더는 기존
`NestedTableSplit.offset_within_start`(start_row 내부 픽셀 오프셋 부분 렌더)를
재사용한다.

### 2.1 재사용 자산 (이미 존재)
- `NestedTableSplit.offset_within_start` (444): start_row 내부 픽셀 오프셋 부분 렌더.
- `calc_nested_split_rows(row_heights, cell_spacing, offset, space)` (462): 픽셀
  오프셋/공간 → 행 범위 + offset_within_start 변환.
- `CellUnit.nested_row` (435): 유닛 ↔ 중첩 행 매핑.

### 2.2 신설/변경

1. **다중열-인지 행 줄 분해기** (신설, `table_layout.rs` path-1 내)
   `nested_row_line_fragments(nt, ri, styles) -> Vec<f64>`:
   행 ri 의 각 열 셀 문단을 `recompose_for_cell_width` 로 줄 분해 → 두(또는 n)
   열의 줄을 content-y 로 병합해 후보 컷 y 생성 → 각 fragment 높이 = 그 구간
   **열별 max**. **불변식: Σ fragment = rh (드리프트 0)** — 마지막 fragment 에
   잔차 귀속. cell_units 캐시(셀 포인터 키) 내부에서 1회 계산 → O(1) 재사용.

2. **path-1 유닛 방출 변경** (5863 루프)
   행 ri 를 단일 유닛 대신, `nested_row_line_fragments(nt, ri)` 로 얻은 각
   sub-fragment 를 splittable CellUnit 으로 push. 각 유닛 `nested_row=Some(ri)`
   유지 + sub-row 내부 위치(픽셀 오프셋) 기록. om_top/spacing 은 첫 fragment,
   om_bot/spacing 은 마지막 fragment 에만.
   **게이트**: 행이 실제로 쪼개질 수 있어야 의미 — `page_break==CellBreak` 이고
   `rh` 가 유의미(≥ 최소 2줄 이상)한 행만. `rh` 가 작은 행은 종전대로 단일 유닛
   (미세 fragment 회귀 방지).

3. **컷 → NestedTableSplit 매핑 확장** (`table_partial.rs` nested_cut_rows 소비)
   sub-row 유닛에서 컷하면, 시작 sub-fragment 의 행 내부 픽셀 오프셋을
   `offset_within_start` 로, 남은 공간을 `calc_nested_split_rows` 에 넘겨 부분 행
   렌더를 구성. start_row 의 이미-렌더된 상단 픽셀을 스킵.

4. **렌더 검증**: start_row 를 offset 만큼 잘라 이어 그리는 경로가 mixed_nested
   fragment(path-2 1×1)에서 이미 동작하는지 확인 후, 다중열에 동일 적용.

## 3. 위험과 게이트

- **드리프트 0**: Σ fragment = rh 아니면 HWPX 왕복 정합(#2148/#2169) 회귀.
- **열별 컷 y**: 현행 vs 개정안 줄 수 상이 → 열별 max 취함("더 긴 열 기준 컷").
- **미세 fragment 회귀**(form-002·issue_1891·76076): 게이트로 작은 행 제외.
- **셀 배경/테두리**: sub-row 경계에 가짜 가로 테두리 금지(세로선만 이어짐).
- **advance_row_cut 상호작용**: tiny_fragment_waste·hard_break 흡수 휴리스틱이
  finer 유닛과 어긋나지 않는지 골든 실측.

## 4. 검증 게이트 (순서)

1. 42065 p2 표 하단이 body 하단(~1009)까지, p2/p3 큰 행 67px 중복 소멸(SVG 실측).
2. 드리프트 0 계측 (Σ fragment = rh).
3. 골든: issue_1073·issue_1891·form-002·76076·21298295(#2097)·86712(#2279) + 42065.
4. overflow_cell_baseline 래칫 + 42065 쪽수(24→17 방향)를 **한컴2020 PDF 1:1 대조**
   후에만 스냅샷 갱신(환각의 테스트 제도화 방지).
5. 최종 시각 판정 (작업지시자, 한컴 편집기).

## 5. 구현 순서

A. `nested_row_line_fragments` 신설 + 드리프트 0 단위 테스트.
B. path-1 유닛 방출 변경 (게이트 포함).
C. 컷→NestedTableSplit offset_within_start 매핑.
D. 렌더 부분 행 확인·정정.
E. 실측·골든·시각 판정.

## 6. 진행 상황 (2026-08-05)

### Step A — 완료·검증 (WIP, 미커밋)

`nested_row_line_fragments(nt, ri, rh, styles) -> Vec<f64>` 신설
(`table_layout.rs`, `nested_table_mixed_fragment_heights` 앞). 각 열 셀 문단을
`recompose_for_cell_width` 로 줄 분해 → 줄 하단 y 를 합집합 → 밴드 fragment,
잔차를 마지막에 귀속. **RHWP_DIAG_ROWFRAG 실측: 전 행 drift=0.000**, 특히
조문대비표 ② 행 `rh=605.0 → 20 fragment, sum=605.0`. 알고리즘 확정.

### Step B·C — 배선 시도·부분 작동, dedup 미완으로 되돌림 (WIP 패치 백업)

배선 구현(`scratchpad/bugB_wip_full.patch`, 343줄)을 실측한 결과:

**작동한 것**
- 게이트 확정: 한컴 오라클(42065 PDF p2·p3) 실측으로 **RowBreak 표도 큰 행을
  인트라-로우 분할**함이 확정됐다 — p3 개정안 열 상단이 "2. 신고사항…"으로 ② 행의
  계속이고 현행 열 상단은 비어있음(② 좌열은 p2 에서 끝남). rhwp 의 "RowBreak=
  인트라-로우 없음" 해석은 한컴과 다르다. 게이트 = `page_break != None`.
- **fill-to-bottom 작동**: 게이트 확대 후 조문대비표 ② 행이 p2 하단까지 채워졌고
  쪽수 24→23 감소. sub-row 유닛(② 행 605px → 20 fragment)이 advance_row_cut 에서
  p2 를 채우도록 소비됨을 확인.

**막힌 것 — 첫 페이지 split 경로와 continuation 경로의 불일치(중복)**
- ROWSPLIT 계측: p3 continuation 은 `content_offset=379.7` 이 정확히 row 3(②) 시작에
  떨어져 `offset_within_start=0` → p3 가 ② 행을 **처음부터 재렌더**.
- p2 첫 등장의 조문대비표 split 은 `nested_row_split_from_cut`(내 함수)을 **안 타고**
  `table_partial.rs:1365 nested_h > available_h` 경로(calc_nested_split_rows(row_heights,
  cs, 0, available_h))로 처리된다. 이 경로는 available_h(잔여 288px)에 맞춰 rows 0–2 만
  할당하는데, **렌더는 그와 무관하게 ② 행 상단을 클립해 표시**한다.
- 결과: p2 가 ② 행 상단을 보이고 p3 가 ② 행 전체를 다시 보여 **② 중복**. 원래(빈
  하단)보다 나쁜 시각 회귀라 배선을 되돌렸다.

**진짜 남은 일 (dedup)**: 첫 페이지(첫 등장) split 도 continuation 과 **동일한 sub-row
콘텐츠-오프셋 모델**을 타게 해, p2 가 소비한 ② 행 sub-unit 만큼을 p3 의
`offset_within_start` 로 넘겨야 한다. 현재 두 경로(1365 available_h 휴리스틱 vs
컷-유닛 기반)가 공존해 경계가 어긋난다. 이 통일이 Step D 의 본질이다.

### (참고) 초기 설계 메모 — mixed_nested 재사용 검토

**핵심 제약**: path-1 이 `nested_row` 유닛 모델을 쓰는 이유는 `mixed_nested`
경로의 **offset-좌표 문제를 회피**하기 위함이다.
- `mixed_nested_split_from_cut`(7774)은 `unit.height` 합으로 offset 을 구하고
  `calc_nested_split_rows(resolve_row_heights, cs, offset, visible)` 로 행 매핑한다.
- 그런데 유닛 height 에는 om_top/ncs/spacing 이 접혀 있어 `resolve_row_heights`
  (순수 행 높이) 좌표와 어긋난다 → offset 이 om_top 만큼 부풀어 행 매핑 드리프트.
- path-1 은 이를 피하려 `nested_row_range_from_cut_units`(7919, 행 번호 직접
  읽기, offset 미사용)를 쓴다. mixed_nested 로 라우팅하면 이 문제를 상속하고,
  `mixed_nested_split_from_cut` 을 고치면 path-2(1×1, #2007·form-002)를 깰 위험.

**권장 배선 (병렬 매핑, nested_row 모델 유지)**:
1. path-1 방출: sub-row 유닛을 `nested_row=Some(ri)` 로 유지하되, **순수 콘텐츠
   fragment 높이를 `mixed_nested_content_height` 에**, om/ncs 접힌 총높이를
   `height` 에 저장. 게이트: `page_break==CellBreak`(splittable 표만).
2. 신규 `nested_row_split_from_cut`: 컷 (lo,hi)에서 start_row 의 **이미 보인
   콘텐츠 합**(mixed_nested_content_height 기준, om/ncs 제외)을
   `offset_within_start` 로 산출 → `NestedTableSplit` 반환. 순수 콘텐츠 좌표라
   `compute_cell_line_ranges(offset, budget)` 와 정합.
3. `table_partial.rs:684` 소비부가 이 split 을 우선 사용.
4. 게이트로 CellBreak 아닌 표는 종전 원자 경로 유지(무회귀).

**미확정/위험**: nested_table cell_spacing·om 이 0 이 아닌 표에서 콘텐츠 좌표
분리의 정확도, CellBreak 다중행 표 전반의 골든(#1073·#1891·form-002) 파급.
Step B·C 는 골든 게이트를 동반한 신중한 반복이 필요 — 마라톤 세션 말미 무검증
투입 금지(미봉책 회피). 다음 집중 세션에서 배선·검증.

### 커밋 상태
- 중앙정렬 `bf0cf05b8`, 버그 A `69b8e6e45` (검증·커밋 완료).
- Step A 분해기: WIP(미커밋), 패치 백업 `scratchpad/bugB_stepA_decomposer.patch`.
