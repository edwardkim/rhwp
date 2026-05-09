---
PR: #719
제목: Task #716 — 빈 paragraph fix_overlay push 차단 (page 1 LAYOUT_OVERFLOW_DRAW) (closes #716)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
base / head: devel / pr-task716
mergeStateStatus: BEHIND
mergeable: MERGEABLE — 충돌 0건
CI: ALL SUCCESS
변경 규모: +1378 / -1, 10 files (소스 1 + 통합 테스트 1 + 보고서 6 + plans 2)
검토일: 2026-05-09
---

# PR #719 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #719 |
| 제목 | Task #716 — 빈 paragraph fix_overlay push 차단 (page 1 LAYOUT_OVERFLOW_DRAW) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터 (누적 23 머지) |
| base / head | devel / pr-task716 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — `git merge-tree` 충돌 0건 (auto-merging layout.rs) |
| CI | ALL SUCCESS |
| 변경 규모 | +1378 / -1, 10 files |
| 커밋 수 | 7 (Stage 0~6, TDD 절차 정합) |
| closes | #716 |
| 선행 영역 | Issue #716 — 작업지시자 등록 영역 (PR #644 시각 검증에서 발견) |

## 2. 결함 본질 (Issue #716)

### 2.1 결함 메커니즘

`samples/20250130-hongbo.hwp` page 0 (1쪽) 마지막 텍스트 줄 영역 의 cropping 결함:
```
LAYOUT_OVERFLOW_DRAW: section=0 pi=15 line=2 y=1048.2 col_bottom=1028.0 overflow=20.1px
LAYOUT_OVERFLOW: page=0, col=0, para=15, type=PartialParagraph, y=1059.4, bottom=1028.0, overflow=31.3px
```

### 2.2 본 환경 직접 재현 ✅

```
$ rhwp export-svg samples/20250130-hongbo.hwp -p 0
LAYOUT_OVERFLOW_DRAW: section=0 pi=15 line=2 y=1048.2 col_bottom=1028.0 overflow=20.1px
LAYOUT_OVERFLOW: page=0, col=0, para=15, type=PartialParagraph, y=1059.4, bottom=1028.0, overflow=31.3px
```

→ PR 본문 측정 정합 (overflow +20.1 px 시각 cropping).

### 2.3 Root cause — Task #9 `fix_overlay_active` 의 빈 paragraph push 누락

수행계획서 영역 의 정적 분석 영역 ("음수 line_spacing(ls<0) 미반영") 가설 → Stage 2 instrument 영역 의 실측 결과 영역 line advance 영역 자체 영역 ls 가산 정합 ✅ → drift 영역 의 진원지 재식별 영역.

**진원지**: Task #9 의 `fix_overlay_active` push 영역 의 빈 paragraph (text_len=0) 영역 push 영역.

| pi | 종류 | host ls (HU) | table_bottom (px) | y_in (px) | push 누적 |
|----|------|-------------|-------------------|-----------|-----------|
| 0 | TAC 1x3 | -600 | 149.93 | — | (TAC 자체) |
| **1** | **empty para** | — | — | 141.93 | **+8.00** |
| 2 | TAC 1x4 | -900 | 211.45 | — | (TAC 자체) |
| **3** | **empty para** | — | — | 199.45 | **+12.00** |

→ 누적 **+20.00 px** ≈ RED overflow +20.15 px (99.3% 일치).

## 3. PR 의 정정 — `is_empty_para` 가드 13 라인

### 3.1 본질 정정 (`src/renderer/layout.rs:1562-1582`, +13 LOC)

```rust
// [Task #716] 빈 paragraph (text_len=0 또는 control 문자/object placeholder 만 존재)
// 는 시각적으로 invisible. fix_overlay push 가 적용되어도 보이는 차이가 없는 반면
// y_offset 만 (table_bottom - y_offset) 만큼 누적되어 forward drift 의 누적 원인이 된다
// (page 1 LAYOUT_OVERFLOW 의 99.3%: pi=1 +8 px + pi=3 +12 px). Task #9 의 push 의도
// (텍스트 paragraph 가 TAC 표 위에 침범하지 않도록 보호) 는 그대로 유지하고,
// 빈 paragraph 는 push 대상에서 제외한다. fix_overlay_active 는 유지하여 후속
// 비-empty paragraph 가 push 대상이 될 수 있게 한다.
let is_empty_para = paragraphs.get(item_para)
    .map(|p| p.text.is_empty()
        || p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}'))
    .unwrap_or(false);
if !is_fixed && !is_empty_para {
    let table_bottom = fix_table_start_y + fix_table_visual_h;
    if y_offset < table_bottom {
        y_offset = table_bottom;
    }
}
```

### 3.2 영향 좁힘 (`feedback_hancom_compat_specific_over_general`)

- **Task #9 의 push 의도 유지** — 텍스트 paragraph 가 TAC 표 위 침범 차단 영역 보존
- **`fix_overlay_active` 유지** — 후속 비-empty paragraph 영역 의 push 대상 영역
- **빈 paragraph 만 차단** — `text.is_empty()` 또는 control 문자 / FFFC (object placeholder) 만 영역

## 4. 회귀 가드 (`tests/issue_716.rs`, +91 LOC, 신규)

### 4.1 검증 영역
```rust
fn collect_body_text_line_bboxes(...) {
    // 머리말/꼬리말 (Header/Footer 자식) 영역 제외 — 본문 컬럼 결함만 검증
    if matches!(node.node_type, RenderNodeType::Header | RenderNodeType::Footer) { return; }
    // Body 영역 안의 TextLine bbox 수집
}

#[test]
fn issue_716_page1_last_text_line_within_body() {
    // page 0 body bbox 수집 + 모든 TextLine bbox 수집
    // assert: max_bottom <= body_bottom + 0.5 (sub-pixel rounding 허용)
}
```

→ 본문 영역 영역 의 마지막 TextLine 영역 의 cropping 부재 영역 직접 검증 영역.

## 5. PR 본문 효과

### 5.1 RED → GREEN
| 영역 | Before | After |
|------|--------|-------|
| `max_bottom` | 1048.19 | **1028.19** |
| `overflow` | +20.15 px | **+0.15 px** (허용 0.5 px 이내) |

### 5.2 stderr 출력 비교
| 메시지 | Before | After |
|--------|--------|-------|
| `LAYOUT_OVERFLOW_DRAW` (시각 cropping) | 발생 | **0건** ✓ |
| `LAYOUT_OVERFLOW` (y_offset 누적) | 31.3 px | 11.3 px |

→ 본 결함 (DRAW 영역 의 시각 cropping) 영역 100% 해소 영역. 잔존 11.3 px 영역 의 `LAYOUT_OVERFLOW` 영역 영역 trailing ls 영역 (Task #452 영역) 영역 별도.

### 5.3 광범위 sweep (169 샘플)

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| `LAYOUT_OVERFLOW_DRAW` 총 | 187 | 185 | **−2** (정정) |
| `LAYOUT_OVERFLOW` 총 | 279 | 277 | **−2** (부수 개선) |
| panic | 0 | 0 | 0 |
| 페이지 수 변동 | — | — | **0 샘플** ✅ |

영향 샘플:
- `20250130-hongbo.hwp` / `20250130-hongbo-no.hwp`: DRAW 1→0 (본 결함 정정)
- `table-vpos-01.hwp` / `table-vpos-01.hwpx`: FLOW 1→0 (부수 개선)
- 나머지 165 샘플: 변동 부재

## 6. 영향 범위

### 6.1 무변경 영역
- 텍스트 paragraph 영역 의 push (Task #9 의도 유지)
- Fixed line spacing 영역 (`is_fixed` 가드 영역)
- 비-`fix_overlay_active` 경로 영역
- 다른 165 샘플 영역

### 6.2 변경 영역 (영향 좁힘)
- 빈 paragraph (text_len=0 또는 control/FFFC 만) 영역 의 push 차단
- 본문 영역 의 marrow drift 누적 차단

→ **위험 매우 낮음**. 13 라인 본질 변경 + 광범위 sweep 회귀 0 + 페이지 수 변동 0.

## 7. 충돌 / mergeable

- `mergeStateStatus: BEHIND` (PR base = `215abb52`, devel HEAD = `c3b10ea7`, 51 commits 뒤처짐)
- `git merge-tree --write-tree` 실측: **CONFLICT 0건** (auto-merging layout.rs)

## 8. 처리 옵션

### 옵션 A — 7 commits 단계별 보존 cherry-pick + no-ff merge (추천)

PR 의 TDD 절차 (Stage 0~6) 정합. PR #694/#693/#695/#699/#706/#707/#710/#711/#714/#715/#718 패턴 일관.

```bash
git branch local/task719 c3b10ea7
git checkout local/task719
git cherry-pick aa0889e5^..48ba1921
git checkout local/devel
git merge --no-ff local/task719
```

→ **옵션 A 추천**.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release --test issue_716` — 1 PASS (회귀 가드)
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0
- [ ] **본 환경 직접 측정**: `rhwp export-svg samples/20250130-hongbo.hwp -p 0` 영역 의 LAYOUT_OVERFLOW_DRAW 영역 부재 확증

### 시각 판정 게이트 (선택)
- 본 PR 영역 의 시각 영역 영향 좁음 (hongbo p1 마지막 줄 cropping 해소 영역)
- 결정적 검증 (CI ALL SUCCESS + 회귀 가드 + 광범위 sweep) 통과 영역 → 시각 판정 게이트 면제 가능

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 (누적 23 머지) |
| `feedback_hancom_compat_specific_over_general` | `is_empty_para` 가드 영역 영역 의 영향 좁힘 — Task #9 텍스트 push 의도 유지 |
| `feedback_process_must_follow` | TDD Stage 0/1 → 1 RED → 2 분석 (가설 갱신) → 3 GREEN → 4 회귀 → 5 광범위 → 6 보고서 절차 정합 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI + 회귀 가드 + 광범위 sweep 169 샘플) 통과 영역 → 시각 판정 게이트 면제 정합 |
| `feedback_assign_issue_before_work` | Issue #716 작업지시자 등록 영역 (assignee 부재 영역, 컨트리뷰터 self-take) |

## 11. 처리 순서 (승인 후)

1. `local/devel` 에서 7 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 신규 issue_716 1 PASS + 직접 측정)
3. 광범위 sweep + (선택) 시각 판정
4. no-ff merge + push + archives 이동 + 5/9 orders 갱신
5. PR #719 close (closes #716 자동 close 정합)

---

작성: 2026-05-09
