---
PR: #714
제목: Task #712 — wrap=TopAndBottom 음수 vert_offset 표 침범 정정 (closes #712)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
base / head: devel / pr-task712
mergeStateStatus: BEHIND
mergeable: MERGEABLE — 충돌 0건
CI: ALL SUCCESS
변경 규모: +1034 / -6, 10 files (소스 2 + 통합 테스트 1 + 보고서 5 + plans 2)
검토일: 2026-05-09
---

# PR #714 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #714 |
| 제목 | Task #712 — wrap=TopAndBottom 음수 vert_offset 표 침범 정정 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터 |
| base / head | devel / pr-task712 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — `git merge-tree` 충돌 0건 (auto-merging layout.rs + table_partial.rs) |
| CI | ALL SUCCESS |
| 변경 규모 | +1034 / -6, 10 files (소스 2 + 통합 테스트 1 + 보고서 5 + plans 2) |
| 커밋 수 | 6 (Stage 0/1/2 + Stage 1 RED + Stage 2-3 GREEN + Stage 4-5 검증 + Stage 6 보고서) |
| closes | #712 |

## 2. 결함 본질 — `HwpUnit=u32` 영역 의 signed 캐스트 누락

### 2.1 결함 메커니즘
- `HwpUnit` 타입 = `u32`
- `vertical_offset` 음수 (예: -1796 HU) 영역 의 unsigned 비트표현 = `0xFFFFF8FC` = `4294965500u32`
- `> 0` 게이트 영역에서 unsigned 양수로 통과 → 후속 `as i32` 캐스트 영역에서 음수 영역 적용 → 표가 위로 점프
- 비-Partial 경로 (`table_layout.rs:1069+`) 에는 `raw_y.max(y_start)` 클램프 영역으로 음수 무력화. **Partial 경로** 에는 클램프 부재 영역 → 결함 노출

### 2.2 결함 시각화
`samples/2022년 국립국어원 업무계획.hwp` 영역 (작업지시자 안내 영역의 페이지 31 영역, 본 환경 영역 의 동적 페이지 탐색):
- pi=585: 1×3 인라인 TAC 제목 표 ("붙임 / / 과제별 추진일정"), wrap=TopAndBottom
- pi=586: 12×5 일정 표, treat_as_char=false, wrap=TopAndBottom, vert=문단(**-1796 HU** 음수)
- pi=586 외곽 상단 y = 124.93 px → pi=585 안쪽으로 **~15.94 px 침범**

## 3. PR 의 정정 — signed 비교 14 라인

### 3.1 `src/renderer/layout/table_partial.rs:59-78` (Partial 경로, 본질)

```rust
// [Task #712] HwpUnit=u32 이라 `vertical_offset > 0` 는 음수 비트표현
// (예: -1796 HU = 0xFFFFF8FC = 4294965500u32) 도 양수로 통과시켜
// 후속 `as i32` 캐스트에서 음수가 적용 → 표가 위로 점프, 직전 인라인
// 표 영역 침범. 비-Partial 경로(table_layout.rs:1069+)는 동일 분기에
// `raw_y.max(y_start)` 클램프가 있어 음수 무력화. Partial 경로에는
// 클램프가 없으므로 게이트를 signed 비교로 정정해 동등 효과.
let vert_off_signed = table.common.vertical_offset as i32;
let y_start = if !is_continuation && !table.common.treat_as_char
    && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(table.common.vert_rel_to, VertRelTo::Para)
    && vert_off_signed > 0   // [Task #712] signed 비교
{
    y_start + hwpunit_to_px(vert_off_signed, self.dpi)
} else {
    y_start
};
```

### 3.2 `src/renderer/layout.rs:2687+` (비-Partial 경로 게이트 동기)

```rust
// [Task #712] 두 경로 동기화 — signed 비교
&& (t.common.vertical_offset as i32) > 0
```

→ **`feedback_image_renderer_paths_separate` 권위 룰 정합** (Partial / 비-Partial 두 경로 동기 정정).

### 3.3 `is_continuation=true` 가드 무영향 확증

분할 표 연결 페이지 영역 (pi 가 직전 페이지 영역에서 분할되어 이어지는 영역) 영역에는 `vert_offset` 미적용 영역. 본 정정 영역 영향 부재. 광범위 sweep 회귀 0.

## 4. 회귀 가드 (`tests/issue_712.rs`, +88 LOC, 신규)

### 4.1 동적 페이지 탐색 영역
```rust
// pi=585 / pi=586 가 등장하는 페이지 인덱스는 빌드의 pagination 결과에 의존:
// - Task #643 미적용 (stream/devel, 본 회귀 테스트 작성 시점): page_index 35
// - Task #643 적용 (PR #644 merge 후): page_index 30
// 페이지 인덱스를 하드코딩하지 않고 pi=585/586 를 가진 페이지를 동적으로 탐색한다.
```

→ **본 환경 영역 의 페이지네이션 변동 영역 (PR #711 머지 후 35 페이지) 영역 자동 적응**. 작업지시자 안내 영역 (36 → 31, 본 환경 영역 35 페이지) 정합.

### 4.2 검증 영역
```rust
assert!(
    pi586_top >= pi585_bottom - 0.5,   // 0.5 px 허용 오차
    "pi=586 12x5 표가 pi=585 1x3 표 안쪽으로 침범..."
);
```

## 5. 영향 범위

### 5.1 무변경 영역
- `is_continuation=true` 분할 표 연결 페이지 영역
- `vertical_offset >= 0` 영역 (signed 양수 / 0)
- 비-TopAndBottom wrap 영역
- TAC 표 영역 (treat_as_char=true)
- 단일 단 영역 / 다단 영역 모두 무관

### 5.2 변경 영역 (영향 좁힘)
- Partial 경로 + 비-Partial 경로 의 `vertical_offset > 0` 게이트만 정정
- 음수 vert_offset 영역 의 게이트 통과 차단

→ **위험 매우 낮음**. 14 라인 본질 변경 + signed 캐스트 정합 + 두 경로 동기.

## 6. 페이지네이션 변동 영역 안내

작업지시자 안내 (2026-05-09):
> "이전 PR 처리 후 samples/2022년 국립국어원 업무계획.hwp 의 페이지네이션이 달라져 PR에 언급된 36페이지는 31 페이지에 위치합니다."

본 환경 직접 측정:
- rhwp = PDF 권위 = **35 페이지** (PR #711 머지 후)
- Issue #712 제목 영역 이미 갱신 완료: "...p31..."
- PR 본문 영역 의 "36 페이지" 영역 — 본 환경 영역 의 동적 페이지 탐색 영역 으로 자동 적응

→ 시각 판정 영역 시 작업지시자 환경 영역 의 페이지 영역 (31) ↔ 본 환경 영역 의 페이지 영역 (35) 영역 차이 영역 가능성 — pi=585/586 영역 자체 가 권위 영역.

## 7. 충돌 / mergeable

- `mergeStateStatus: BEHIND` (PR base = `215abb52`, devel HEAD = `49f94e3b`, 38 commits 뒤처짐)
- `git merge-tree --write-tree` 실측: **CONFLICT 0건** (auto-merging layout.rs + table_partial.rs)

## 8. 처리 옵션

### 옵션 A — 6 commits 단계별 보존 cherry-pick + no-ff merge (추천)

PR 의 TDD 절차 (Stage 0~6) 정합. PR #694/#693/#695/#699/#706/#707/#710/#711 패턴 일관.

```bash
git branch local/task714 49f94e3b
git checkout local/task714
git cherry-pick edcc7d58^..47ca1178
git checkout local/devel
git merge --no-ff local/task714
```

→ **옵션 A 추천**.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release --test issue_712` — 1 PASS (회귀 가드)
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets` clean
- [ ] **광범위 sweep** — 7 fixture / 170 페이지 회귀 0 확증

### 시각 판정 게이트 (선택)
- 본 PR 은 음수 vert_offset 영역 의 침범 정정 — 시각 영향 영역 좁음 (pi=586 영역 한정)
- 작업지시자 영역 의 직접 시각 판정 영역 시 영역:
  - `samples/2022년 국립국어원 업무계획.hwp` page 31 (작업지시자 환경) / page 35 (본 환경) 영역 의 pi=586 12×5 일정 표 영역 정합
  - 직전 pi=585 1×3 제목 표 영역 침범 부재
  - 다른 페이지 영역 회귀 부재

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 |
| `feedback_hancom_compat_specific_over_general` | 영향 좁힘 — `is_continuation=true` 영역 + `vertical_offset >= 0` 영역 무영향 |
| `feedback_image_renderer_paths_separate` | Partial / 비-Partial 두 경로 동기 정정 (signed 캐스트) |
| `feedback_process_must_follow` | TDD Stage 0/1 (계획) → Stage 1 RED → Stage 2-3 GREEN → Stage 4-5 sweep → Stage 6 보고서 절차 정합 |
| `feedback_assign_issue_before_work` | Issue #712 컨트리뷰터 self-등록 패턴 (assignee 부재) |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI ALL SUCCESS + 회귀 가드 + 광범위 sweep) + 작업지시자 시각 판정 (선택) |
| `feedback_visual_regression_grows` | 동적 페이지 탐색 영역 — 페이지네이션 변동 영역 견고 영역 의 회귀 가드 영역 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 에서 6 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 신규 issue_712 1 PASS + 직접 페이지 수 측정)
3. (선택) 광범위 sweep + 시각 판정
4. no-ff merge + push + archives 이동 + 5/9 orders 갱신
5. PR #714 close (closes #712 자동 close 정합)

---

작성: 2026-05-09
