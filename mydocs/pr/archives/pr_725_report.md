---
PR: #725
제목: fix: 표 셀 빈 영역 hitTest 컨텍스트 이탈 정정 (closes #717)
컨트리뷰터: @postmelee — 15+ 사이클 핵심 컨트리뷰터 (rhwp-studio editor 영역)
처리: 옵션 A — 1 commit cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: bbf01424
---

# PR #725 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + no-ff merge `bbf01424`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `bbf01424` (--no-ff merge) |
| closes | #717 |
| 시각 판정 | 면제 합리 (cursor_rect.rs query API + 회귀 가드 통과 + sweep 0 회귀) |
| 자기 검증 | lib + 통합 ALL GREEN + 신규 3 PASS + sweep 170/170 same |

## 2. 정정 본질 — `src/document_core/queries/cursor_rect.rs` (+72/-17)

### 2.1 `cellIndex` 만 매칭 영역 의 결함
HWP 영역 영역 영역 여러 표 (자료 표 + 페이지 번호 표 + 바탕쪽 표) 영역 영역 모두 `cellIndex=0` 영역 영역 낮은 인덱스 반복 영역 → hit-test 결과 영역 영역 다른 표 영역 컨텍스트 영역 으로 오염.

### 2.2 정정 (4 영역)
- **`table_id: Option<u32>` 추가** — `RunInfo` / `CellBboxInfo` 영역 영역 소속 표 RenderNode id 보존
- **`collect_runs()` 영역 `current_table_id` 인자** — Table 노드 진입 시 `Some(node.id)` 영역 자식 전파
- **cell_bboxes meta 보완 영역 격리** — `r.table_id == cb.table_id` + `ctx.innermost().cell_index == cb.cell_index` (중첩 표 정합)
- **clicked_cell 선택 영역** — `.min_by_key(w*h)` 영역 의 가장 작은 bbox 선택 (중첩 표 내부 셀 우선)
- **빈 셀 fallback** — `cb.cell_context` 영역 영역 의 전체 `cellPath` 영역 직렬화 (path entries → JSON 배열)

## 3. 회귀 가드 (3건 신규)

`tests/issue_717_table_cell_hit_test.rs` (+130 LOC):

| 케이스 | 좌표 | 기대 |
|--------|------|------|
| 자료 표 제목 행 빈 영역 | `page=0, x=191.0, y=356.0` | `parentParaIndex=1`, `controlIndex=0` |
| 회색 헤더 내부표 빈 영역 | `page=0, x=100.0, y=350.0` | `cellPath=[(0,0,0),(1,1,0)]` 보존 |
| `<보기>` 표 빈 영역 | `page=0, x=110.0, y=865.0` | `parentParaIndex=6`, `controlIndex=0` |

내부표 케이스 영역 영역 hit-test + path 기반 X 삽입 후 `getTextInCellByPath()` / `getCursorRectByPath()` 정상 동작 까지 확인.

## 4. 본 환경 cherry-pick + 검증

### 4.1 cherry-pick (1 commit)
```
ef67efa1 Task #717: Fix table cell whitespace hit test
```
충돌 0건.

### 4.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (29.75s) |
| `cargo test --release --test issue_717_table_cell_hit_test` | ✅ **3 PASS** |
| `cargo test --release` (전체) | ✅ ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets -- -D warnings` | 48 errors **모두 기존 코드** (devel 시점 52 → PR 영역 영역 4건 정정) — 본 PR 무관 |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |

### 4.3 시각 판정 게이트 면제 합리
- `cursor_rect.rs` 영역 영역 query API 만 변경 — SVG/PNG 시각 출력 영역 무영향
- 회귀 가드 3건 신규 영역 결정적 검증 + 광범위 sweep 0 회귀 영역 보장
- @postmelee PR 본문 영역 영역 before/after 영상 첨부 (시각 입증)
- `feedback_visual_judgment_authority` 정합 — 시각 출력 무변경 영역 영역 면제 합리

## 5. 영향 범위

### 5.1 변경 영역
- rhwp-studio editor 영역 의 표 셀 빈 영역 클릭 영역 hit-test 정합
- 중첩 표 영역 cellPath 보존 (`getCursorRectByPath()` / `getTextInCellByPath()` API 정합)

### 5.2 무변경 영역
- 셀 내부 텍스트 hit-test (이미 정합)
- 표 외부 본문 hit-test
- HWP3/HWP5 변환본 시각 정합 (sweep 170/170 same)
- WASM 빌드 외부 의존성

## 6. 후속 분리 (PR 본문 명시)

회색 헤더 내부표 영역 의 극단적 낮은 빈 셀 (높이 ~5.1px) 영역 영역 텍스트 입력 영역 가시성 문제 영역 별건 — "초저높이 빈 셀 편집 진입/표 선택 UX 정책" 영역 영역 후속 작업 영역 으로 분리.

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @postmelee 15+ 사이클 핵심 컨트리뷰터 (rhwp-studio editor 영역) |
| `feedback_image_renderer_paths_separate` | cursor_rect.rs 영역 영역 격리 — 다른 query/render 경로 영역 무영향 |
| `feedback_process_must_follow` | TDD Stage 1~5 절차 정합 + 후속 분리 (초저높이 빈 셀 UX) |
| `feedback_visual_judgment_authority` | 시각 출력 무변경 + 회귀 가드 통과 영역 영역 시각 판정 면제 합리 |
| `feedback_hancom_compat_specific_over_general` | `table_id` 영역 영역 의 격리 — `cellIndex` 일반화 영역 의 결함 영역 본질 영역 |

## 8. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- 초저높이 빈 셀 UX 영역 영역 별건 (이슈 등록 권장)

---

작성: 2026-05-10
