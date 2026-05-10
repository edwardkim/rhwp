---
PR: #725
제목: fix: 표 셀 빈 영역 hitTest 컨텍스트 이탈 정정 (closes #717)
컨트리뷰터: @postmelee — 15+ 사이클 핵심 컨트리뷰터 (rhwp-studio editor / TAC / 셀 hit-test 영역)
base / head: devel / fix/issue-717-table-cell-hit-test
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS (Build & Test / CodeQL / Analyze rust+js+py / WASM SKIPPED)
변경 규모: +822 / -17, 11 files (소스 1 + 테스트 1 + 보고서 9)
검토일: 2026-05-10
---

# PR #725 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #725 |
| 제목 | fix: 표 셀 빈 영역 hitTest 컨텍스트 이탈 정정 (closes #717) |
| 컨트리뷰터 | @postmelee — 15+ 사이클 핵심 컨트리뷰터 (rhwp-studio editor 영역, PR #169/#209/#214/#224/#243/#339/#385/#437/#510/#531/#642/#663/#664/#718/#725 등) |
| base / head | devel / fix/issue-717-table-cell-hit-test |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS |
| 변경 규모 | +822 / -17, 11 files |
| 커밋 수 | 1 (`e8d46ee2`) |
| closes | #717 |

## 2. 결함 본질 (Issue #717)

### 2.1 결함 영역
`samples/exam_social.hwp` 1/4쪽 영역 의 자료 표 영역 의 **제목 행 빈 영역 클릭** 영역 영역 → 커서 영역 영역 페이지 하단 번호 표(`32`) 영역 영역 이동.

### 2.2 결함 본질
`src/document_core/queries/cursor_rect.rs::hit_test_native()` 영역 의 `TableCell` bbox 메타 보완 영역 영역 `cellIndex` 만 영역 기준 영역 영역 TextRun 매칭. HWP 영역 영역 영역 여러 표 (자료 표 + 페이지 번호 표 + 바탕쪽 표) 영역 영역 모두 `cellIndex=0` 영역 영역 낮은 인덱스 반복 영역 → 클릭 좌표 영역 영역 자료 표 내부 (`s0:pi=1 ci=0`) 영역 영역, hit-test 결과 영역 영역 페이지 번호 표 영역 컨텍스트 영역 으로 오염.

### 2.3 보조 결함 (회색 헤더 내부표)
중첩 표 영역 의 `TableCell` bbox 영역 영역 전체 `cellPath` 영역 잃고 최외곽 표 영역 의 셀 영역 처럼 반환. `[(0,0,0),(1,1,0)]` 영역 영역 의 경로 영역 영역 내부표 `cellIndex=1` 영역 영역 최외곽 1x1 표 영역 의 `cellIndex=1` 영역 처럼 처리.

## 3. PR 의 정정 — 4 영역

### 3.1 RunInfo / CellBboxInfo 영역 `table_id` 추가 (+~30 LOC)

```rust
// RunInfo
table_id: Option<u32>,  // 소속 표 RenderNode id

// CellBboxInfo
table_id: Option<u32>,
text_direction: u8,
cell_context: Option<CellContext>,
```

`collect_runs()` 영역 영역 `current_table_id` 인자 영역 추가 — Table 노드 진입 시 `Some(node.id)` 영역 영역 자식 영역 영역 영역 전파.

### 3.2 cell_bboxes meta 보완 영역 격리 (+~25 LOC)

```rust
let same_cell_run = runs.iter().find(|r| {
    r.table_id == cb.table_id  // ← 같은 표 안에서만 매칭
        && r.cell_context.as_ref().map(|ctx| {
            ctx.innermost().cell_index == cb.cell_index  // ← 최내곽 (중첩 표 정합)
        }).unwrap_or(false)
});
let template_run = same_cell_run.or_else(|| {
    runs.iter().find(|r| r.table_id == cb.table_id && r.cell_context.is_some())
});
// template_run 의 cell_context 클론 → 최내곽 cell_index/cell_para_index/text_direction 갱신
```

→ 중첩 표 영역 의 `cellPath` 영역 보존 영역 정합.

### 3.3 clicked_cell 선택 영역 영역 가장 작은 bbox (+~3 LOC)

```rust
let clicked_cell: Option<&CellBboxInfo> = cell_bboxes.iter()
    .filter(|cb| cb.has_meta)
    .filter(|cb| x >= cb.x && x <= cb.x + cb.w && y >= cb.y && y <= cb.y + cb.h)
    .min_by_key(|cb| ((cb.w.max(0.0) * cb.h.max(0.0)) * 1000.0) as i64);
```

→ 여러 셀 bbox 영역 동시 포함 영역 영역 메타 영역 영역 후보 영역 중 가장 작은 bbox 영역 선택 (중첩 표 영역 영역 내부 셀 영역 우선).

### 3.4 셀 내부 TextRun 검색 영역 영역 `table_id` + 최내곽 `cellIndex` (+~5 LOC)

```rust
ctx.parent_para_index == cb.parent_para_index
    && r.table_id == cb.table_id  // ← 추가
    && ctx.innermost().cell_index == cb.cell_index  // ← 최내곽
```

### 3.5 빈 셀 fallback 영역 영역 전체 cellPath 반환 (+~25 LOC)

`cb.cell_context` 영역 영역 존재 시 영역 영역 path entries 직렬화 (`controlIndex`/`cellIndex`/`cellParaIndex`) → JSON `cellPath` 배열 영역 반환. 미존재 시 (이전 fallback) 영역 영역 단일 entry 영역.

## 4. 회귀 가드 (3건 신규)

`tests/issue_717_table_cell_hit_test.rs` (+130 LOC):

| 케이스 | 좌표 | 기대 |
|--------|------|------|
| 자료 표 제목 행 빈 영역 | `page=0, x=191.0, y=356.0` | `parentParaIndex=1`, `controlIndex=0`, caret 자료 표 안 |
| 자료 표 회색 헤더 내부표 빈 영역 | `page=0, x=100.0, y=350.0` | `cellPath=[(0,0,0),(1,1,0)]`, path 기반 삽입/조회 가능 |
| `<보기>` 표 빈 영역 | `page=0, x=110.0, y=865.0` | `parentParaIndex=6`, `controlIndex=0`, caret `<보기>` 표 안 |

내부표 케이스 영역 영역 hit-test 결과 영역 영역 만 점검 영역 영역, 반환된 `cellPath` 영역 영역 `X` 삽입 후 `getTextInCellByPath()` / `getCursorRectByPath()` 정상 동작 영역 까지 확인.

## 5. 본 환경 점검

- fixture: `samples/exam_social.hwp` 존재 ✓
- merge-tree 충돌: 0건 ✓
- merge-base = `215abb52` (5/9 PR #691 후속 시점) — devel HEAD 영역 가까움
- `CellContext::innermost()` API (src/renderer/layout.rs:71) 존재 ✓ — PR 영역 영역 의존 영역
- 코드 변경 영역 영역 격리: cursor_rect.rs (1 file) — 다른 layout/render 경로 영역 무관

## 6. 영향 범위

### 6.1 변경 영역
- rhwp-studio editor 영역 의 표 셀 빈 영역 클릭 영역 의 hit-test 정합
- 중첩 표 영역 cellPath 보존 (`getCursorRectByPath()` / `getTextInCellByPath()` API 정합)

### 6.2 무변경 영역
- 셀 내부 텍스트 hit-test (이미 정합)
- 표 외부 본문 hit-test
- HWP3/HWP5 변환본 시각 정합
- WASM 빌드 영역 의 외부 의존성

### 6.3 위험 영역
- 변경 영역 영역 `cursor_rect.rs::hit_test_native()` 영역 영역 격리 — rhwp-studio 영역 의 native bridge API 영역 만 영향
- 회귀 가드 3건 영역 영역 신규 영역 — 정합 입증

## 7. 후속 분리 (PR 본문 명시)

회색 헤더 내부표 영역 의 극단적 낮은 빈 셀 (높이 ~5.1px) 영역 영역 텍스트 입력 영역 시 가시성 문제 영역 영역 별건 — "초저높이 빈 셀 편집 진입/표 선택 UX 정책" 영역 영역 후속 작업 영역 으로 분리. 본 PR 영역 영역 입력 경로 영역 정정 (cellPath) 영역 영역 본질 영역.

## 8. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — 5/10 사이클 영역 의 PR #720/#723 머지 영역 영역 devel 영역 영역 진전 영역, 본 PR 영역 영역 cursor_rect.rs 단일 변경 영역 영역 충돌 부재

## 9. 처리 옵션

### 옵션 A — 1 commit cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task717 1c783a89
git cherry-pick e8d46ee2
git checkout local/devel
git merge --no-ff local/task717
```

→ **옵션 A 추천**.

## 10. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo test --release --test issue_717_table_cell_hit_test` — 신규 3 PASS
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0
- [ ] form-002 / test_634 / issue_712/713/716 회귀 가드 영역 보존

### 시각 판정 게이트 — **면제 합리**

본 PR 영역 영역 의 본질 영역 영역 **rhwp-studio editor 영역 의 hit-test API 영역**:
- 결정적 검증 영역 영역 명시 (회귀 가드 3건 신규)
- 시각 출력 (SVG/PNG) 영역 영역 변경 부재 — `cursor_rect.rs` 영역 영역 query API 만 영향
- 광범위 sweep 회귀 0 영역 보장 영역 → 시각 본질 영역 보존 영역
- @postmelee 영역 영역 의 PR 본문 영역 영역 video before/after 영상 (변경 전/후) 영역 첨부

→ `feedback_visual_judgment_authority` 정합 — 결정적 검증 + 회귀 가드 통과 영역 → 시각 판정 면제 합리.

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @postmelee 15+ 사이클 핵심 컨트리뷰터 (rhwp-studio editor 영역) |
| `feedback_image_renderer_paths_separate` | cursor_rect.rs 영역 영역 격리 — 다른 query/render 경로 영역 무영향 |
| `feedback_process_must_follow` | TDD Stage 1~5 절차 정합 + 후속 분리 (초저높이 빈 셀 UX) |
| `feedback_visual_judgment_authority` | 결정적 검증 + 회귀 가드 통과 영역 → 시각 판정 면제 합리 |
| `feedback_hancom_compat_specific_over_general` | `table_id` 영역 영역 의 격리 영역 영역 case 가드 영역 — `cellIndex` 일반화 영역 의 결함 영역 본질 영역 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 1 commit cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 광범위 sweep + 신규 3 PASS)
3. 시각 판정 면제 합리 — 결정적 검증 통과 영역 영역 즉시 머지
4. no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #725 close (closes #717 자동 정합)

---

작성: 2026-05-10
