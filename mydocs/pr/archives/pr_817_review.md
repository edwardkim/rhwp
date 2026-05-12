---
PR: #817
제목: fix — 1×1 래퍼 표 shortcut 다수 중첩 표 누락 수정 (closes #726)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 14번째 시도 (PR #815 close 후 다른 본질)
처리: 옵션 C — devel 영역 영역 이미 해결 (Task #688) + byte-identical SVG + Issue #726 진짜 본질 별 PR 후속
처리일: 2026-05-12
---

# PR #817 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #817 |
| 제목 | fix: 1×1 래퍼 표 shortcut 다수 중첩 표 누락 수정 (#726) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 (5/11 사이클 14번째 시도, PR #815 close 후) |
| base / head | devel / contrib/fix-nested-table-wrapper |
| mergeable | CONFLICTING (DIRTY) |
| CI | 결과 부재 |
| 변경 규모 | +6 / -1, 1 file |
| 커밋 수 | 1 |
| closes 표기 | #726 (그러나 실제 본질 영역 영역 무관) |

## 2. 본 PR 본질

PR 본문 영역 영역 명시:
- table-vpos-01.hwpx 5쪽 nested 11×3 그리드 SVG 누락
- 결함 추정: 셀[0] paras=2 영역 영역 `find_map` 영역 영역 첫 nested 만 반환 → 두 번째 (11×3 그리드) 영역 영역 스킵
- 정정: `nested_table_count == 1` 가드 영역 영역 다수 table 시 일반 경로

## 3. 정정 본질 — `table_layout.rs` +6/-1

```rust
if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
    let cell = &table.cells[0];
    let has_visible_text = cell.paragraphs.iter()
        .any(|p| p.text.chars().any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n'));
    let nested_table_count = cell.paragraphs.iter()
        .flat_map(|p| p.controls.iter())
        .filter(|c| matches!(c, Control::Table(_)))
        .count();
    if !has_visible_text && nested_table_count == 1 {
        // shortcut 적용
    }
}
```

## 4. ⚠️ devel HEAD 영역 영역 이미 해결됨 (Task #688)

devel HEAD 영역 영역 `src/renderer/layout/table_layout.rs::layout_table` — Task #688 (PR #694, commit `40ecbe26`/`917447dd`) 영역 영역 다른 가드 영역 영역 동일 본질 해결:

```rust
// (Task #688) 셀 paragraphs 가 2개 이상이면 첫 nested 표만 unwrap 시 나머지
// paragraph 의 nested 표가 누락되므로 paragraphs.len() == 1 가드를 둔다.
if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
    let cell = &table.cells[0];
    if cell.paragraphs.len() == 1 {  // ← Task #688 가드 (paragraphs 수)
        let p = &cell.paragraphs[0];
        // shortcut + 외곽 박스 border 렌더링 보존
    }
}
```

table-vpos-01.hwpx p.5 영역 영역 셀[0] `paras=2` → Task #688 가드 영역 영역 false → shortcut 우회 → 일반 경로 영역 영역 모든 nested table 렌더링.

### 두 가드 비교

| 가드 | 본질 | 외곽 박스 border |
|------|------|-----------------|
| **Task #688** (devel) | `cell.paragraphs.len() == 1` — paragraphs 수 가드 | ✅ 추가 정합 (exam_social.hwp pi=15 자료 박스 정정) |
| **PR #817** | `nested_table_count == 1` — nested table 수 가드 | ❌ 부재 |

## 5. 본 환경 시각 비교 — byte-identical

작업지시자 요청 영역 영역 PR cherry-pick + 5쪽 SVG 내보내기 비교:

```
=== before (devel HEAD, Task #688 가드) ===
text=343  polygon=0  image=0  path=2  lines=474  size=131K

=== after (PR #817 nested_table_count == 1 가드) ===
text=343  polygon=0  image=0  path=2  lines=474  size=131K

diff exit code: 0 (byte-identical)
```

→ 두 SVG 영역 영역 **완전 동일**. PR 정정 영역 영역 본 환경 영역 영역 시각적/구조적 효과 부재.

## 6. PR base 시점 분석

| 항목 | 값 |
|------|-----|
| PR #817 base | `30351cdf` (5/9 시점) — Task #688 머지 전 |
| Task #688 머지 | `40ecbe26`/`917447dd` (5/9~) — PR #817 base 이후 |
| 현재 devel HEAD | `6a1c9795` (5/11, 13 PR 머지 후) |

→ PR #817 영역 영역 작성된 후 Task #688 영역 영역 같은 본질 영역 영역 다른 방식 영역 영역 먼저 머지. 본 PR 영역 영역 **이미 해결된 본질 영역 영역 중복 정정** + **외곽 박스 border 회귀 위험**.

## 7. Issue #726 영역 영역 진짜 본질 — 화살표 도형 미출력

Issue #726 본문 명시:
> samples/table-vpos-01.hwpx 5쪽 nested 11×3 그리드의 **4대 그룹 사이 구분 도형 2개**가 SVG 출력에서 누락.

본 환경 점검 (devel HEAD + PR #817 적용 후 동일):
- `<polygon>` **0개** — 화살표 도형 완전 미출력
- `<image>` **0개**
- IR 영역 영역 `셀[18] ctrl[0] 다각형: tac=true, wrap=TopAndBottom` 영역 영역 1개 존재

→ **셀 안 다각형/도형 컨트롤 SVG 렌더링 경로 누락** — 본 PR 영역 영역 무관, 별 본질.

### 두 결함 후보 (Issue #726 본문 명시)

| 결함 | 영역 | 본질 |
|------|------|------|
| (a) SVG renderer 다각형 미출력 | `src/renderer/` 도형 분기 (svg.rs / web_canvas.rs / paint/json.rs) | 셀 안 다각형 컨트롤 (tac=true, wrap=TopAndBottom) SVG 출력 경로 누락 또는 cell-clip 외부 위치 |
| (b) HWPX 파서 다각형 1개 누락 | `src/parser/hwpx/` table cell 안 GenShape 파싱 | IR 영역 영역 1개 (셀[18])만, PDF 권위본 영역 영역 3개 — 셀[6]/셀[13] 영역 영역 `ctrls=0` |

## 8. 처리 — 옵션 C (close + 분리 PR 가이드)

본 PR 영역 영역:
1. **본 PR 정정 본질** (nested 11×3 그리드 누락) → Task #688 영역 영역 이미 해결 + 외곽 박스 border 추가 정합
2. **Issue #726 영역 영역 진짜 본질** (4대 그룹 사이 화살표 도형 2개 미출력) → 본 PR 영역 영역 해결 부재
3. **`closes #726` 영역 영역 잘못된 연결** — 본 PR 영역 영역 머지해도 Issue #726 본질 영역 영역 잔존

### 컨트리뷰터 안내 본질 (정중 톤, [#817#issuecomment-4425741327](https://github.com/edwardkim/rhwp/pull/817#issuecomment-4425741327)):
- byte-identical SVG 결과 명시
- Task #688 영역 영역 이미 해결됨 안내 + 외곽 박스 border 보존 영역 영역 정교한 정합
- Issue #726 영역 영역 진짜 본질 (화살표 도형 SVG 미출력) 영역 영역 분리 PR 가이드
- 두 결함 후보 (a/b) 명시 — `feedback_image_renderer_paths_separate` 영역 영역 점검 사례

## 9. 본 환경 reset

cherry-pick 후 시각 비교 → byte-identical 확인 → `git reset --hard origin/devel` 영역 영역 PR commits 제거 → devel 무영향.

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 14번째 시도, PR #815 close 후 다른 본질) |
| `feedback_image_renderer_paths_separate` 권위 사례 강화 후보 | Issue #726 영역 영역 진짜 본질 — 셀 안 다각형 SVG/Canvas/paint json 4 backend 영역 영역 동기 정정 필요 영역 영역 별 PR 후속 |
| `feedback_process_must_follow` | PR base 영역 영역 5/9 시점 영역 영역 작성 → 그 후 Task #688 영역 영역 먼저 머지 → 본 PR 영역 영역 중복 정정. base 갱신 영역 영역 영역 점검 필요 |
| `feedback_hancom_compat_specific_over_general` | Task #688 영역 영역 외곽 박스 border 영역 영역 추가 정합 (exam_social.hwp pi=15 정정 포함) — 본 PR 영역 영역 단순 가드만 |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | **본 PR 본질** (nested 11×3 그리드 누락, Task #688 이미 해결) vs **Issue #726 진짜 본질** (셀 안 화살표 도형 SVG 미출력) 영역 영역 두 본질 분리 진단 — PR 영역 영역 잘못 연결 (`closes #726`) |
| `feedback_visual_judgment_authority` | 작업지시자 영역 영역 5쪽 SVG 시각 비교 요청 → byte-identical 결과 입증 — 결정적 검증 영역 영역 본 PR 영역 영역 효과 없음 명확화 |
| `feedback_pr_supersede_chain` | PR #694 (Task #688, nested table 가드 + 외곽 박스 border 도입) → Issue #726 (잔존 본질 — 화살표 도형 미출력) → **PR #817** (close, 중복 정정) → 분리 PR (Issue #726 진짜 본질 후속) (a) 패턴 |

## 11. 잔존 후속

- Issue #726 OPEN 유지 — 진짜 본질 (셀 안 화살표 도형 SVG 미출력) 영역 영역 분리 PR 후속
- 분리 PR 본질 영역 영역:
  - 후보 (a): `src/renderer/` 도형 분기 (svg.rs / web_canvas.rs / paint/json.rs) — 셀 컨텍스트 영역 영역 다각형 렌더링 경로
  - 후보 (b): `src/parser/hwpx/` table cell 안 GenShape 파싱 영역 영역 셀[6]/셀[13] 도형 누락
- @oksure 후속 PR 영역 영역 대기 또는 작업지시자 영역 영역 별 본질 진단 후 후속

---

작성: 2026-05-12
