---
PR: #745
제목: Task #634 — 첫 NewNumber Page 발화 전 쪽번호 미표시 (한컴 호환)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 14번째 PR)
base / head: devel / contrib/page-number-newnumber-gate
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS
변경 규모: +42 / -4, 3 files
검토일: 2026-05-10
---

# PR #745 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #745 |
| 제목 | Task #634 — 첫 NewNumber Page 발화 전 쪽번호 미표시 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 14번째 PR) |
| base / head | devel / contrib/page-number-newnumber-gate |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS |
| 변경 규모 | +42 / -4, 3 files |
| 커밋 수 | 2 (Task + Copilot 리뷰) |
| closes | #634 |

## 2. 결함 본질 (Issue #634)

### 2.1 결함 영역
한컴오피스 PDF 영역 영역 첫 `새번호 (NewNumber Page)` 컨트롤 등장 페이지부터만 쪽번호 표시. rhwp 영역 영역 `PageNumberPos` 등록 시점부터 모든 페이지 쪽번호 표시 → 표지/요약문/목차 영역 영역 한컴 시각 차이 발생.

### 2.2 검증 사례 (Issue 본문)
- `samples/aift.hwp` 영역 영역 표지/요약문/목차 (페이지 1~6) — 한컴 PDF 미표시 vs rhwp "- 1 -" ~ "- 3 -"
- `samples/aift.hwp` 페이지 7 (NewNumber Page=1 발화) — 한컴 PDF "- 1 -" 시작
- `samples/2022년 국립국어원 업무계획.hwp` 영역 영역 동일 패턴

## 3. PR 의 정정 — 3 files, +42/-4

### 3.1 `src/renderer/page_number.rs` (+36/-2)

**`PageNumberAssigner` 영역 영역 `numbering_started: bool` 추가**:
```rust
pub(crate) struct PageNumberAssigner<'a> {
    new_page_numbers: &'a [(usize, u16)],
    consumed: HashSet<usize>,
    counter: u32,
    /// NewNumber 컨트롤이 1건 이상 소비되었는지 여부.
    numbering_started: bool,
}
```

**`assign()` 영역 영역 첫 NewNumber 소비 시점 영역 영역 `true` 전환**:
```rust
if Self::para_first_appears(page, nn_pi) {
    self.counter = nn_num as u32;
    self.consumed.insert(idx);
    self.numbering_started = true;  // 신규
}
```

**`should_hide_page_number()` 메서드**:
```rust
pub fn should_hide_page_number(&self) -> bool {
    !self.new_page_numbers.is_empty() && !self.numbering_started
}
```

→ NewNumber 부재 영역 영역 항상 `false` (기존 동작 100% 보존).

**신규 unit tests 2건**:
- `should_hide_before_first_new_number` — NewNumber 발화 전/후 상태 검증
- `should_not_hide_when_no_new_numbers` — NewNumber 부재 시 항상 표시

### 3.2 `src/renderer/pagination/engine.rs` (+3/-1)

```rust
if !assigner.should_hide_page_number() {
    page.page_number_pos = page_number_pos.clone();
}
```

`finalize_pages()` 영역 영역 `page_number_pos` 할당 영역 영역 가드.

### 3.3 `src/renderer/typeset.rs` (+3/-1)

동일 가드 — TypesetEngine 경로 (Copilot 리뷰 반영, commit `9a9a4272`).

→ **두 경로 모두 적용** — pagination/engine + typeset 영역 영역 동기 정합 (`feedback_image_renderer_paths_separate` 정합).

## 4. 영향 범위

### 4.1 변경 영역
- 한컴 호환 영역 영역 첫 NewNumber Page 발화 전 페이지 영역 영역 쪽번호 미표시
- 표지/요약문/목차 영역 영역 시각 정합

### 4.2 무변경 영역 (opt-in 정합)
- NewNumber 부재 문서 (대부분 문서) — `new_page_numbers` 빈 벡터 영역 영역 `should_hide_page_number()` 항상 `false` → 기존 동작 100% 보존
- 다른 layout/render 경로

### 4.3 위험 영역
- **opt-in 정합** — 회귀 위험 좁힘
- 두 경로 (pagination/engine + typeset) 동기 정합 — `feedback_image_renderer_paths_separate` 영역 영역 권위 사례 강화

## 5. 본 환경 점검

- merge-base: `30351cdf` (5/9 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 격리: page_number.rs + pagination/engine.rs + typeset.rs — 다른 layout/render 경로 무관

## 6. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 5/10 사이클 진전, 본 PR 격리 변경으로 충돌 부재

## 7. 처리 옵션

### 옵션 A — 2 commits cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task634 7acd9f39
git cherry-pick 1d7c1795 9a9a4272
git checkout local/devel
git merge --no-ff local/task634
```

→ **옵션 A 추천**.

## 8. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release --lib renderer::page_number` — 신규 2건 PASS + 기존 보존
- [ ] `cargo test --release` ALL GREEN
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] **광범위 sweep — 7 fixture / 170 페이지** — `aift.hwp` (NewNumber 사용) 영역 영역 페이지 1~6 쪽번호 미표시 정합 + 페이지 7 부터 표시 (의도된 변경) + 다른 fixture (NewNumber 부재) 회귀 0

### 시각 판정 게이트 — **작업지시자 권장**

본 PR 본질은 **시각 정합** (한컴 PDF 정합):
- `samples/aift.hwp` 영역 영역 페이지 1~6 쪽번호 미표시 정합 (한컴 PDF 정합)
- `samples/aift.hwp` 페이지 7 부터 "- 1 -" 정상 출력
- `samples/2022년 국립국어원 업무계획.hwp` 동일 패턴 점검

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 14번째 PR) |
| `feedback_image_renderer_paths_separate` | **권위 사례 강화** — pagination/engine + typeset 두 경로 동기 정합 (Copilot 리뷰 반영) |
| `feedback_hancom_compat_specific_over_general` | NewNumber 부재 영역 영역 기존 동작 보존 (opt-in case 가드) — 회귀 위험 좁힘 |
| `feedback_visual_judgment_authority` | 한컴 PDF 정합 영역 영역 권위 — 작업지시자 시각 판정 권장 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 광범위 sweep — aift 의도된 변경 확인 + 다른 fixture 회귀 0)
3. 작업지시자 시각 판정 (`samples/aift.hwp` 영역 영역 한컴 PDF 정합)
4. 시각 판정 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #745 close (closes #634 자동 정합)

---

작성: 2026-05-10
