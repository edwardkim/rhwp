---
PR: #715
제목: Task #713 — 분할 표 orphan sliver(<25px) 행 단위 push (closes #713)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
base / head: devel / pr-task713
mergeStateStatus: BEHIND
mergeable: MERGEABLE — 충돌 0건
CI: ALL SUCCESS
변경 규모: +973 / -0, 9 files (소스 1 + 통합 테스트 1 + 보고서 5 + plans 2)
검토일: 2026-05-09
---

# PR #715 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #715 |
| 제목 | Task #713 — 분할 표 orphan sliver(<25px) 행 단위 push |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터 |
| base / head | devel / pr-task713 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — `git merge-tree` 충돌 0건 (auto-merging typeset.rs) |
| CI | ALL SUCCESS |
| 변경 규모 | +973 / -0, 9 files (소스 1 + 통합 테스트 1 + 보고서 5 + plans 2) |
| 커밋 수 | 5 (Stage 0 계획 + Stage 1 RED + Stage 2 가설 + Stage 3 GREEN + Stage 4-5-6 검증/보고서) |
| closes | #713 |

## 2. 결함 본질 — RowBreak 표 인트라-로우 분할

### 2.1 결함 메커니즘
- `samples/2022년 국립국어원 업무계획.hwp` 12×5 일정 표 영역 (pi=586 ci=0, 쪽나눔=RowBreak)
- row 8 영역 (`한국어교육 내실화` + `ㅇ국외 한국어교육 지원 사업 수요조사...`) 영역 페이지 경계 영역 의 **17.6 px 인트라-로우 분할** 발생
- 한컴은 페이지 끝 영역 의 작은 sliver 영역 부재 영역 → 행 전체 영역 다음 페이지 영역 push

### 2.2 본 환경 직접 재현 ✅
```
$ rhwp dump-pages '...국립국어원...'
page_index=30 (page 31):  PartialTable rows=0..9 split_end=17.6 px  ← orphan sliver
page_index=31 (page 32):  PartialTable rows=8..12 cont=true split_start=17.6
```

→ row 8 영역 의 17.6 px 분할 (PR 본문 정합).

## 3. PR 의 정정 — `MIN_TOP_KEEP_PX = 25.0` 가드

### 3.1 본질 정정 (`src/renderer/typeset.rs:1931+`, +5 LOC + 주석)

```rust
// [Task #713] avail_content_for_r 가 한 줄 정도로 너무 작으면 (orphan)
// 분할 대신 행 전체를 다음 페이지로 push. 한컴은 페이지 끝의 작은
// sliver(예: 17.6 px) 를 두지 않고 행 단위로 이동
// (2022 국립국어원 p31 row 8 케이스). 임계값 25 px 는
// synam-001 의 정합 분할 (27.3 px) 과 본 결함 (17.6 px) 사이.
const MIN_TOP_KEEP_PX: f64 = 25.0;
if avail_content_for_r >= MIN_SPLIT_CONTENT_PX
    && avail_content_for_r >= min_first_line
    && avail_content_for_r >= MIN_TOP_KEEP_PX  // [Task #713] orphan 가드
    && remaining_content >= MIN_SPLIT_CONTENT_PX
{
    end_row = r + 1;
}
```

### 3.2 임계값 25 px 결정 근거

| 케이스 | avail_content_for_r | 가드 적용 후 |
|--------|--------------------|----|
| 본 결함 (row 8 sliver) | **17.6 px** | < 25 → 차단 ✓ |
| synam-001 p23 (정합) | **27.3 px** | ≥ 25 → 변경 없음 ✓ |
| 기타 정합 분할 | 93/437/510 px | ≥ 25 → 변경 없음 ✓ |

→ 17.6 ↔ 27.3 px 사이 영역 의 25 px 영역 임계값 영역. **`feedback_hancom_compat_specific_over_general` 정합** (영향 좁힘).

### 3.3 활성 경로 메모

`src/document_core/queries/rendering.rs:1041-1042` — `RHWP_USE_PAGINATOR=1` 미설정 시 `typeset.rs::typeset_section` 영역 활성, `pagination/engine.rs::paginate_with_measured_opts` 영역 fallback. 본 정정은 활성 경로 (typeset.rs) 영역 만 적용 영역.

→ ⚠️ **`feedback_image_renderer_paths_separate` 정합 부분 영역** — fallback 경로 영역에 동일 가드 부재 영역. fallback 경로 영역 의 회귀 위험 영역 평가 필요.

## 4. 회귀 가드 (`tests/issue_713.rs`, +108 LOC, 신규)

### 4.1 동적 페이지 sweep 영역
```rust
// 모든 페이지에서 pi=586 ci=0 표의 row 8 셀을 수집.
// RowBreak 모드라면 행 8 의 모든 셀이 단일 페이지에 위치하고 clip=false 여야 함.
for pn in 0..page_count {
    let tree = doc.build_page_render_tree(pn).expect(...);
    collect_row_cells(&tree.root, TARGET_PI, TARGET_CI, TARGET_ROW, &mut cells);
    ...
}
```

→ **PR #714 와 동일 패턴** (페이지네이션 변동 영역 견고). PR 본문 영역 의 page 32 ↔ 본 환경 page 32 영역 의 차이 영역 자동 적응.

### 4.2 검증 영역
- row 8 셀 영역 단일 페이지 영역 위치 (split_pages.len() == 1)
- `clip=false` (분할 표시 부재)

## 5. 영향 범위

### 5.1 무변경 영역
- avail_content_for_r ≥ 25 px 영역 의 분할 영역 (synam-001 등 정합 영역)
- 비-RowBreak 표 영역
- 비-Partial 분할 영역

### 5.2 변경 영역 (영향 좁힘)
- avail_content_for_r < 25 px 영역 의 orphan sliver 분할 영역 차단
- 행 전체 영역 다음 페이지 영역 push

### 5.3 잠재 영향
- `MIN_TOP_KEEP_PX = 25.0` 영역 의 임계값 영역 — 17.6 ↔ 27.3 px 영역 사이 영역 의 다른 케이스 영역 (만약 존재 영역) 영역 영향 가능성. PR 본문 영역의 광범위 sweep 181 샘플 / 페이지 수 횡단 비교 영역 diff 0 영역 — 회귀 부재 입증.

→ **위험 매우 낮음**. 5 라인 본질 변경 + 임계값 결정 근거 명확.

## 6. 충돌 / mergeable

- `mergeStateStatus: BEHIND` (PR base = `215abb52`, devel HEAD = `9d9aea48`, 44 commits 뒤처짐)
- `git merge-tree --write-tree` 실측: **CONFLICT 0건** (auto-merging typeset.rs, PR #714 와 다른 line 영역)

## 7. 처리 옵션

### 옵션 A — 5 commits 단계별 보존 cherry-pick + no-ff merge (추천)

PR 의 TDD 절차 (Stage 0~6) 정합. PR #694/#693/#695/#699/#706/#707/#710/#711/#714 패턴 일관.

```bash
git branch local/task715 9d9aea48
git checkout local/task715
git cherry-pick 7136550b^..94ccf8db
git checkout local/devel
git merge --no-ff local/task715
```

→ **옵션 A 추천**.

## 8. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release --test issue_713` — 1 PASS (회귀 가드, 동적 페이지 sweep)
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0
- [ ] **본 환경 직접 측정**: row 8 split_end 17.6 px → 0 px (단일 페이지 정합)

### 시각 판정 게이트 (선택)
- 결정적 검증 (CI ALL SUCCESS + 회귀 가드 + 광범위 sweep) 통과 영역 + 작업지시자 직접 시각 판정 가능 영역
- 점검 영역: page 31 (BEFORE 17.6 px sliver) → page 32 (row 8 전체 영역 push) 정합

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 (누적 23 머지) |
| `feedback_hancom_compat_specific_over_general` | 임계값 25 px 영역 의 영향 좁힘 — 17.6 ↔ 27.3 px 사이 영역 |
| `feedback_image_renderer_paths_separate` | ⚠️ 활성 경로 (typeset.rs) 만 정정 — fallback 경로 (pagination/engine.rs) 영역 부재. 본 PR 영역 위험 매우 낮음 (RHWP_USE_PAGINATOR=1 영역 미설정 영역 의 fallback 영역 영역) |
| `feedback_process_must_follow` | TDD Stage 0 → 1 RED → 2 가설 → 3 GREEN → 4-5-6 검증/보고서 절차 정합 |
| `feedback_visual_judgment_authority` | 결정적 검증 + 광범위 sweep 통과 + 본 환경 직접 측정 입증 |
| `feedback_visual_regression_grows` | 회귀 가드 영역 의 동적 페이지 sweep 영역 (PR #714 패턴 정합) |

## 10. 처리 순서 (승인 후)

1. `local/devel` 에서 5 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 신규 issue_713 1 PASS + 직접 측정)
3. 광범위 sweep + (선택) 시각 판정
4. no-ff merge + push + archives 이동 + 5/9 orders 갱신
5. PR #715 close (closes #713 자동 close 정합)

---

작성: 2026-05-09
