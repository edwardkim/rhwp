# Task 384: pagination current_height vs layout y_offset 체계적 불일치 (B-002)

## 상태: 분석 진행 중

## 재현 파일
- `samples/bodo-02.hwp` — pi=21: pagination current_h=850.8 vs layout 합산 ~635px (215px 과대)

## 분석 결과

### 1. 문단별 높이 차이 추적

pagination에 디버그 삽입하여 각 항목의 `para_height` / `table_total_height`를 추적:

```
| pi | type | pagination | layout overlay | 차이 |
|----|------|-----------|----------------|------|
| 0  | 표   | 54.5      | 47.0           | +7.5 |
| 1  | 문단 | 1.9       | 1.3            | +0.6 |
| 2  | 문단 | 1.9       | 1.3            | +0.6 |
| 3 ci=0 | TAC표 | 28.4  | 20.9          | +7.5 |
| 3 ci=1 | 자리차지표 | 85.3 | 82.1       | +3.2 |
| 4  | 문단 | 11.2      | 8.0            | +3.2 |
| 5  | 문단 | 26.1      | 18.7           | +7.4 |
| 6  | 문단 | 26.1      | 18.7           | +7.4 |
| 7  | 문단 | 84.0      | 56.0           | +28.0|
| 10 | 표   | 74.3      | 68.3           | +6.0 |
| 11 | 문단 | 84.0      | 56.0           | +28.0|
| 14 | 표   | 96.9      | 83.4           | +13.5|
| 16 | 문단 | 112.0     | 74.7           | +37.3|
```

### 2. 일반 문단 차이의 원인: `corrected_line_height`

**시도**: `corrected_line_height` 조건을 `raw_lh < 1.0`으로 변경 → LINE_SEG가 있는 HWP에서는 보정 미적용
**결과**: 일반 문단 차이가 사라짐
**하지만**: 기존 테스트 5개 실패 (페이지 분할 결과 변경)

**추가 조사**: layout도 `corrected_line_height`를 사용하지 않고 LINE_SEG lh를 그대로 사용
**결론**: `corrected_line_height` 보정은 pagination에서만 적용되어 layout과 차이 발생

**시도 2**: `raw_lh > 0.0`이면 보정 건너뛰기 → **755 테스트 전체 통과**
**하지만**: bodo-02의 pi=21 분할 여전히 발생 — 일반 문단 차이는 해소되었지만 표 높이 차이 유지

### 3. 표 높이 차이의 원인: `host_spacing`

비-TAC 표의 `table_total_height = effective_height + host_spacing`에서:
- `host_spacing = before + sa + outer_bottom + host_line_spacing`
- `host_line_spacing = LINE_SEG.last().line_spacing`

layout에서의 "표 아래 간격":
- `spacing_after` + `line_spacing` (또는 `line_height` if ls=0)

두 계산이 대체로 일치하지만, `effective_height` 자체가 다를 수 있음:
- pagination: `MeasuredTable.total_height` (Task 381에서 셀 내용 높이 비교 확장)
- layout: `compute_table_y_position` 기반 실제 렌더링 높이

### 4. dump-pages `h` vs 실제 layout `y_advance`

dump-pages의 `h` 값은 **lh만 합산** (ls 미포함) — 참고용 수치
실제 layout `y_advance`는 `lh + ls` 합산 (마지막 줄 포함)
pagination `para_height`도 `lh + ls` 합산

→ **dump-pages h와의 비교는 부정확**, pagination vs layout 직접 비교 필요

### 5. 마지막 줄 ls 제외 시도

**시도**: pagination `lines_total`에서 마지막 줄 ls 제외
**결과**: 테스트 3~5개 실패 (엔터 반복으로 페이지 넘김 테스트)
**원인**: layout도 마지막 줄 ls를 포함하므로 제외하면 불일치
**결론**: 마지막 줄 ls 제외는 올바르지 않음

## 미해결 사항

1. **`corrected_line_height` 제거**: `raw_lh > 0.0`이면 보정 건너뛰기는 안전 (755 테스트 통과)
   하지만 이것만으로는 bodo-02 pi=21 분할 미해결

2. **표 `effective_height` 차이**: MeasuredTable.total_height와 실제 렌더링 높이 차이 원인 미확인

3. **`host_spacing` 정확도**: 비-TAC 표의 host_spacing 계산이 layout과 정확히 일치하는지 미검증

## 추가 분석 (2차)

### 6. effective_height vs common.height 차이

디버그 결과 (`[EFF-H]`):
```
pi=14: effective=92.9 common=83.4 diff=+9.6
pi=21: effective=81.6 common=68.3 diff=+13.3
pi=28: effective=59.3 common=41.7 diff=+17.6
pi=37: effective=120.3 common=78.7 diff=+41.6
```

원인: Task 381에서 `resolve_row_heights`의 MeasuredTable 경로에 셀 내용 높이 비교를 추가.
셀 내용이 cell.height보다 크면 행 높이를 확장 → MeasuredTable.total_height 증가.

**하지만 layout도 같은 `resolve_row_heights`를 사용하므로 확장된 높이로 렌더링한다.**
→ effective_height를 common.height로 제한하면 layout과 불일치 + 테스트 실패

### 7. layout y_offset도 과대

overlay에서 pi=21 y=1034.0 > body_area 하단(1028.1) → **layout도 body 영역을 넘어섬!**
pagination과 layout 모두 높이가 과대 — 표 아래 간격(`gap`)이 과대 추가되는 문제.

layout line 1935-1938:
```rust
let gap = if seg.line_spacing > 0 { seg.line_spacing } else { seg.line_height };
y_offset += hwpunit_to_px(gap, self.dpi);
```
비-TAC 표 아래에 매번 gap 추가. 다음 문단의 spacing_before와 겹치면 이중.

### 8. host_spacing 구성 비교

pagination: `host_spacing = before + sa + outer_bottom + host_line_spacing`
layout: `spacing_after + gap(ls or lh)`

두 계산이 대체로 일치하지만, pagination이 `before(sb)`를 포함하는 반면
layout은 표 **위** 간격에서 spacing_before를 별도 처리.
→ pagination에서 sb가 host_spacing에 포함되면 이중 계산 가능

## 분할 정복 계획

| 단계 | 대상 | 내용 | 상태 |
|------|------|------|------|
| A | corrected_line_height | raw_lh>0이면 보정 건너뛰기 | ✅ 완료 |
| B | 표 아래 gap 이중 | layout의 gap + 다음 문단 spacing 겹침 조사 | 미착수 |
| C | host_spacing sb 이중 | pagination의 before(sb)가 이중인지 조사 | 미착수 |
| D | effective_height 정합성 | Task 381 확장과 pagination/layout 일치 검증 | 조사 완료 (일치 확인) |
| E | 통합 검증 | bodo-01, bodo-02, kps-ai, 기존 테스트 | 미착수 |
