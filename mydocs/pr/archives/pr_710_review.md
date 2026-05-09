---
PR: #710
제목: Task #702 — shortcut.hwp 다단 정의 후속 갱신 누락 정정 (closes #702)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
base / head: devel / pr-task702
mergeStateStatus: BEHIND
mergeable: MERGEABLE — 충돌 0건 (auto-merge typeset.rs)
CI: ALL SUCCESS
변경 규모: +799 / -6, 7 files (소스 1 + 신규 통합 테스트 1 + 보고서 5)
검토일: 2026-05-09
---

# PR #710 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #710 |
| 제목 | Task #702 — shortcut.hwp 다단 정의 후속 갱신 누락 정정 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터 |
| base / head | devel / pr-task702 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — `git merge-tree` 충돌 0건 (auto-merging typeset.rs) |
| CI | ALL SUCCESS |
| 변경 규모 | +799 / -6, 7 files (소스 1 + 신규 통합 테스트 1 + 보고서 5) |
| 커밋 수 | 2 (본질 + 거버넌스 산출물 분리) |
| closes | #702 |
| 분리된 후속 | Issue #708 (1쪽 시프트 잔존), Issue #709 (부수 시각 결함 4건) |

## 2. 결함 본질 — 두 영역 (Issue #702)

### 2.1 본질 1A — Distribute 다단의 짧은 컬럼 vpos-reset 임계값

`src/renderer/typeset.rs:430-446` 영역 의 inter-paragraph vpos-reset 검출 임계값 `pv > 5000` 이 짧은 Distribute (배분) 컬럼 (예: 지우기 3+3 분배, 마지막 paragraph vpos=3000) 에서 미달 → column-advance 미발동 → 6항목 1단 적층.

### 2.2 본질 1B — Page/Column break + 새 ColumnDef 미적용

shortcut.hwp p2 의 파일/미리보기/편집 sections 패턴:
- `[쪽나누기] + 단정의:1단 + 표(header)`
- `[단나누기] + 단정의:2단 배분`

기존 코드는 `MultiColumn` break 만 ColumnDef 적용 → Page/Column break 동반 ColumnDef 무시 → `col_count` 가 이전 zone 값 유지 → 페이지 분기 폭주.

## 3. PR 의 정정

### 3.1 ColumnType 추적 (`TypesetState.current_zone_column_type` 필드)

```rust
// 새 필드 추가
current_zone_column_type: ColumnType,

// new() 시그니처 + column_type 인자 추가
TypesetState::new(layout, col_count, section_index, ..., column_type)
```

`process_multicolumn_break` 내부 ColumnDef 매칭 시 `current_zone_column_type` 갱신.

### 3.2 vpos-reset trigger Distribute 한정 완화

```rust
let is_distribute = st.col_count > 1
    && matches!(st.current_zone_column_type, ColumnType::Distribute);
let trigger = if st.col_count > 1 {
    if is_distribute {
        cv < pv && pv > 0          // [Task #702] Distribute 한정 완화
    } else {
        cv < pv && pv > 5000        // Normal (NEWSPAPER) 다단 — 기존 유지
    }
} else {
    cv == 0 && pv > 5000             // 단일 단 — 기존 유지
};
```

**영향 좁힘** (`feedback_hancom_compat_specific_over_general` 정합):
- Normal (NEWSPAPER) 다단 → 기존 `pv > 5000` 유지 → Task #321/#418/#470 회귀 차단
- 단일 단 → 기존 `cv == 0 && pv > 5000` 유지

### 3.3 Page/Column break + 새 ColumnDef 검출 + zone 재정의

```rust
let new_col_def_opt: Option<ColumnDef> = para.controls.iter().find_map(|c| {
    if let Control::ColumnDef(cd) = c { Some(cd.clone()) } else { None }
});
let has_diff_col_def = new_col_def_opt.as_ref().map(|cd| {
    cd.column_count.max(1) != st.col_count
        || cd.column_type != st.current_zone_column_type
}).unwrap_or(false);

// Column + has_diff_col_def: process_multicolumn_break 호출 (zone 재정의)
// Column + 동일 ColumnDef: 기존 advance_column_or_new_page
// Page/Section + has_diff_col_def: force_new_page 후 ColumnDef 적용
```

## 4. 회귀 가드 (`tests/issue_702.rs`, +102 LOC, 신규)

| 테스트 | 검증 |
|--------|------|
| `shortcut_distribute_short_column_split` | 페이지 수 ≤ 8 (기존 10 → 8 또는 7 정합 목표) |
| `shortcut_page2_has_three_sections` | 페이지 2 SVG 에 "파일" / "편집" 두 섹션 헤더 모두 존재 — SVG `<text>` 글자 좌표 클러스터링으로 라인 복원 후 substring 검사 |

## 5. PR 본문 효과

| 항목 | 수정 전 | 수정 후 |
|------|--------|--------|
| 페이지 수 | 10 | **8** (PDF 7 + 1쪽 잔존) |
| `LAYOUT_OVERFLOW` | 다수 (40~60px) | 1건 (페이지 8 마지막) |
| 페이지 1 지우기 | 1단 6항목 적층 | **2단 3+3 분할** ✓ |
| 페이지 2 섹션 | 파일 header 만 | **파일+미리보기+편집 통합** ✓ |

## 6. 본 환경 fixture 직접 측정 (BEFORE)

| 파일 | rhwp BEFORE | PDF 권위 | PR 적용 후 기대 |
|------|-------------|---------|-----------------|
| `samples/basic/shortcut.hwp` | **10 페이지** | 7 페이지 | **8 페이지** (PDF +1, Issue #708 잔존) |

→ PR 본문 정합 영역 본 환경에서 직접 입증 가능.

## 7. 분리된 후속 — Issue #708 / #709

PR 본문 명시:
- **Issue #708** (1쪽 시프트 잔존) — pi=94 bare `[단나누기]` at last col. fix 시도 시 다른 회귀 발견 → 별도 task 분리
- **Issue #709** (부수 시각 결함 4건) — PUA 글자 / 탭 leader / 바탕쪽 자동번호 / 우측 정렬

→ scope 정확 분리 정합 (`feedback_process_must_follow` 정합 — scope 확장 충동 억제).

## 8. 충돌 / mergeable + 다른 PR 영향

- `mergeStateStatus: BEHIND` (PR base = `215abb52`, devel HEAD = `27027cac`, 27 commits 뒤처짐)
- `git merge-tree --write-tree` 실측: **CONFLICT 0건** (Auto-merging `src/renderer/typeset.rs`)

PR 본문 명시:
- PR #644 (Task #643): L563-571 vpos_end + L881 `LAYOUT_DRIFT_SAFETY_PX` (다른 메커니즘) — devel 미반영
- PR #679 (Task #676): L629/L894-898/L1005-1029 typeset_paragraph (`col_count == 1` 한정) — **devel 머지 완료** (`bd3b63dd`)
- PR #707 (Task #703): L1368-1382 BehindText/InFrontOfText 표 분기 — **devel 머지 완료** (`e3484101`, 직전 사이클)

→ 각각 다른 line 영역, 다른 함수 분기. auto-merge 정합 확증.

## 9. 처리 옵션

### 옵션 A — 2 commits 단계별 보존 cherry-pick + no-ff merge (추천)

PR 의 commit 분리 (본질 + 거버넌스 산출물) 정합. PR #694/#693/#695/#699/#706/#707 패턴 일관.

```bash
git branch local/task710 27027cac
git checkout local/task710
git cherry-pick 42b1a8f4^..2259c4b6
git checkout local/devel
git merge --no-ff local/task710
```

→ **옵션 A 추천**.

## 10. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release --test issue_702` — 2 PASS (회귀 가드)
- [ ] `cargo test --release --test exam_eng_multicolumn` — 14 PASS (Normal 다단 회귀 차단)
- [ ] `cargo test --release --test issue_418` — 1 PASS
- [ ] `cargo test --release --test svg_snapshot` — 8 PASS (form-002 갱신 후)
- [ ] `cargo test --release` — 전체 ALL GREEN
- [ ] `cargo clippy --release --all-targets` clean
- [ ] **직접 페이지 수 측정**: shortcut.hwp 10 → 8 정합 회복 확증

### 시각 판정 게이트
- 본 PR 은 다단 분배 영역의 본질 정정 — 페이지 분할 영향. 작업지시자 시각 판정 권장:
  - `samples/basic/shortcut.hwp` p1 지우기 2단 3+3 분배 (한컴 PDF 정합)
  - `samples/basic/shortcut.hwp` p2 파일/미리보기/편집 통합 (한컴 PDF 정합)
  - 기존 다단 영역 (exam_eng 등) 회귀 부재
- `feedback_visual_judgment_authority` 정합 — 결정적 검증 + 작업지시자 시각 판정 영역의 안전 게이트

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 정확 표현 |
| `feedback_hancom_compat_specific_over_general` | Distribute 한정 임계값 완화 (영향 좁힘) — Normal/단단 영역 보존 |
| `feedback_process_must_follow` | 후속 Issue #708/#709 분리 (scope 확장 회피) — fix 시도 회귀 발견 시 rollback + 별도 task 분리 정합 |
| `feedback_assign_issue_before_work` | Issue #702/#708/#709 컨트리뷰터 self-등록 패턴 (assignee 부재) |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI ALL SUCCESS + 회귀 가드 + 광범위 sweep) + 작업지시자 시각 판정 (권장) |
| `feedback_pr_supersede_chain` | PR #710 단독 PR — supersede 부재 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 에서 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 신규 issue_702 2 PASS + 회귀 가드 + 직접 페이지 수 측정)
3. (선택) WASM 빌드 + 시각 판정 (rhwp-studio 또는 export-svg)
4. no-ff merge + push + archives 이동 + 5/9 orders 갱신
5. PR #710 close (closes #702 자동 close 정합)
6. Issue #708 / #709 OPEN 유지 (분리된 후속)

---

작성: 2026-05-09
