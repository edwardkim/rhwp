# Task #324 Stage 2 보고서 — 코드 수정 시도 + 추가 분석

**기준일**: 2026-04-25
**브랜치**: `local/task324`
**단계**: 코드 수정

---

## 1. 시도한 수정안

### 1차 패치 — `compute_cell_line_ranges` 가시성 검사 추가

```rust
// has_table_in_para 분기에 limit 초과 시 atomic 단위 미룸
} else if content_limit > 0.0 {
    result.push((line_count, line_count));   // 다음 페이지로
    continue;
}
```

### 2차 패치 — `layout_partial_table` 렌더 스킵 분기 추가

`start_line >= end_line` 인 중첩 표 문단을 split_end_row 에서도 스킵하도록 분기 추가.

## 2. 디버그 트레이스 결과

`RHWP_DBG_T324` 환경변수 게이트로 `compute_cell_line_ranges` 호출 시 인자/상태 출력:

```
[T324] pi=28 para_h=63.6 offset_rem=0.0 limit_rem=0.0 content_offset=0.0 content_limit=443.0   ← 페이지 1 split-end
[T324] pi=28 para_h=63.6 offset_rem=0.0 limit_rem=∞   content_offset=443.0 content_limit=0.0   ← 페이지 2 split-start
```

## 3. 근본 원인 — 더 깊은 문제 발견

당초 가설(`compute_cell_line_ranges` 의 가시성 분기 결함)은 **단편적**임이 확인됨. 실제 문제는 **3-층 결함**:

### 3-1. cell.h vs 실제 콘텐츠 높이 불일치

- 셀[73] r=19, c=0: `cell.h = 63539 HWPUNIT (847.2px)` (모델 선언값)
- 실제 paragraphs 누적 콘텐츠: p[0] vpos=0 ~ p[28] end=30564 HWPUNIT (≈407.5px)
- **차이 ≈ 440px (셀 내 상당량의 빈 공간)**

→ pagination 엔진은 `row_heights[19] = 847px` 로 본다. avail_for_rows ≈ 845px 와 맞지 않아 분할 결정. 그러나 실제 콘텐츠는 page 1 에 통째로 들어감.

### 3-2. line-by-line 누적 vs vpos 누적 불일치

`compute_cell_line_ranges` 가 `limit_remaining` 을 line 단위 (`line_height + line_spacing + spacing_before/after`) 로 차감.

- p[0..27] 까지 vpos 기준 종료 위치: 25376 HWPUNIT (≈338px)
- 하지만 line 누적은 content_limit=443px 를 모두 소진 (`limit_rem=0.0` at p[28])
- 차이 ≈ 100px → spacing_before/spacing_after 가 vpos 와 이중 계산되는 것으로 추정

### 3-3. content_offset 기반 가시성 결정의 결함

페이지 2 진입 시 `content_offset=443`. p[28] vpos=338px, height=63.6px → 위치 338..401.6px.

- 401.6px < 443px → 위치상 page 1 영역에 속함
- 하지만 page 1 에서는 limit 초과로 표시되지 않음(1차 패치 후) ← 주관 의도
- page 2 에서도 visibility check 가 `para_h <= content_offset (63.6 ≤ 443)` 으로 \"이미 지나간\" 판정 → 표시 안 됨

→ 결국 **두 페이지 모두에서 누락**되는 회귀 발생.

## 4. 검증 결과 (2차 패치 후)

```
output/svg/task324_baseline/form-002_001.svg  269 KB  (인너 표 page 1 노출)
output/svg/task324_after/form-002_001.svg     228 KB  (인너 표 제거됨, -41KB)
output/svg/task324_baseline/form-002_002.svg  267 KB  (인너 표 없음)
output/svg/task324_after/form-002_002.svg     267 KB  (변화 없음 — 인너 표 추가 안 됨)
```

**즉 인너 표가 페이지 1 에서 사라졌으나 페이지 2 로 옮겨가지 않음.** 회귀.

## 5. 결론 및 제안

이 이슈는 단순 가시성 패치로 해결되지 않음. **3가지 영역의 변경이 필요**:

1. **셀 분할 결정 자체 재검토** (`pagination/engine.rs`): `row_heights` 기반 split 결정에서, 셀의 실제 콘텐츠 높이(`paragraphs` 누적)를 고려하여 \"split 불필요\" 케이스 식별
2. **content_offset/limit 의 미터링 통일**: line-by-line 누적과 vpos 누적의 일치성 보장 (spacing_before/after 이중 계산 제거 또는 vpos 기반 단일 미터링)
3. **atomic block (중첩 표) 의 페이지 결정 정책**: split 경계와 무관하게 \"가장 가까운 한 페이지\"에 배치

이는 Task #324 의 단일 패치 범위를 넘는 **페이지네이션 엔진 리팩토링** 영역으로, Epic 309 (LINE_SEG vpos 우선 모드 전환) 와 직접 연결됨.

## 6. 권고 사항

- 본 Task #324 는 **분석/조사 결과 보고로 마무리** 하고, 실제 수정은 Epic #309 의 후속 task 로 분리
- 또는 Task #324 의 범위를 \"임시 우회\" 로 축소: cell.h 와 콘텐츠 높이 차이가 큰 경우 split 자체를 회피하는 가드 추가
- 어느 방향으로 진행할지 작업지시자 결정 필요

## 7. 작업 상태

- 코드 수정: **모두 revert** (`git checkout src/renderer/layout/table_layout.rs src/renderer/layout/table_partial.rs`)
- 작업 트리: 클린 상태 (디버그 코드 포함되지 않음)

---

**다음 단계 제안**: 위 6항의 두 옵션 중 작업지시자 선택 후 진행.
