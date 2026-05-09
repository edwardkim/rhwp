---
PR: #719
제목: Task #716 — 빈 paragraph fix_overlay push 차단 (page 1 LAYOUT_OVERFLOW_DRAW)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
처리: 옵션 A — 7 commits 단계별 보존 cherry-pick + no-ff merge
처리일: 2026-05-09
머지 commit: b8defa26
---

# PR #719 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (7 commits 단계별 보존 cherry-pick + no-ff merge `b8defa26`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `b8defa26` (--no-ff merge) |
| Issue #716 | close 자동 정합 (closes #716) |
| 시각 판정 | 게이트 면제 (결정적 검증 + 광범위 sweep + 직접 측정 통과) |
| 자기 검증 | lib **1173** + 통합 ALL GREEN + issue_716 1/1 + clippy clean |
| 광범위 sweep | 7 fixture / **170 페이지 / 회귀 0** |

## 2. 정정 본질

### 2.1 결함 메커니즘
Task #9 의 `fix_overlay_active` push 영역 의 빈 paragraph (text_len=0) push → drift +20 px 누적 → `LAYOUT_OVERFLOW_DRAW: overflow=20.1px` (시각 cropping).

### 2.2 본 환경 직접 재현 + 정정 입증

| 영역 | BEFORE | AFTER |
|------|--------|-------|
| `LAYOUT_OVERFLOW_DRAW` | overflow=20.1px (시각 cropping) | **부재** ★ |
| `LAYOUT_OVERFLOW` | overflow=31.3px | overflow=**11.3px** (trailing ls 잔존, Task #452 영역) |

### 2.3 정정 (`src/renderer/layout.rs:1562-1582`, +13 LOC)

```rust
let is_empty_para = paragraphs.get(item_para)
    .map(|p| p.text.is_empty()
        || p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}'))
    .unwrap_or(false);
if !is_fixed && !is_empty_para {  // [Task #716] 빈 paragraph 가드
    let table_bottom = fix_table_start_y + fix_table_visual_h;
    if y_offset < table_bottom {
        y_offset = table_bottom;
    }
}
```

### 2.4 영향 좁힘 (`feedback_hancom_compat_specific_over_general`)
- 텍스트 paragraph 영역 무영향 (Task #9 push 의도 유지)
- Fixed line spacing 영역 무영향 (is_fixed 가드)
- 비-fix_overlay_active 경로 무영향
- 빈 paragraph 만 차단

## 3. 본 환경 cherry-pick + 검증

### 3.1 cherry-pick (7 commits)
```
3609d73e Task #716 Stage 0: 수행 + 구현 계획서
5e7c5ea5 Task #716 Stage 1 (RED): 회귀 테스트 + FAIL 확인
ab600061 Task #716 Stage 2 (분석): drift 진원지 = Task #9 fix_overlay push
30e21c2d Task #716 Stage 3 (GREEN): fix_overlay 빈 paragraph skip
4ec84044 Task #716 Stage 4 (회귀): cargo test --release 0 failed + 골든 SVG 회귀 0
8a0a5f8e Task #716 Stage 5 (광범위): 169 샘플 회귀 0 + 페이지 수 변동 0
2361666a Task #716 Stage 6 (최종): 결과 보고서 + closes #716
```
충돌 0건 (auto-merging layout.rs).

### 3.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (29.38s) |
| `cargo test --release --test issue_716` | ✅ **1/1 PASS** (회귀 가드) |
| `cargo test --release --test svg_snapshot` | ✅ 8/8 (form-002 PR #706 영역 보존) |
| `cargo test --release --test issue_712` | ✅ PASS (PR #714 영역 보존) |
| `cargo test --release --test issue_713` | ✅ PASS (PR #715 영역 보존) |
| `cargo test --release` | ✅ lib **1173** + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets` | ✅ 신규 경고 0 |
| 광범위 sweep | 7 fixture / **170 페이지 / 회귀 0** ✅ |

### 3.3 본 환경 직접 측정 — `LAYOUT_OVERFLOW_DRAW` 부재 ★

```
$ rhwp export-svg samples/20250130-hongbo.hwp -p 0
# AFTER (PR #719 적용)
LAYOUT_OVERFLOW: page=0, col=0, para=15, type=PartialParagraph, y=1039.4, bottom=1028.0, overflow=11.3px
# (LAYOUT_OVERFLOW_DRAW 메시지 부재)
```

→ 본 PR 본질 (시각 cropping) 100% 해소. 잔존 11.3 px 영역 trailing ls 영역 별도.

### 3.4 머지 commit
`b8defa26` — `git merge --no-ff local/task719` 단일 머지 commit. PR #694/#693/#695/#699/#706/#707/#710/#711/#714/#715/#718 패턴 일관.

### 3.5 시각 판정 게이트 면제
- 결정적 검증 (CI ALL SUCCESS + 회귀 가드 + 광범위 sweep + 본 환경 직접 측정) 모두 통과
- 시각 영역 영향 좁음 (hongbo p1 마지막 줄 cropping 해소)
- `feedback_visual_judgment_authority` 정합 — 결정적 검증 + sweep 통과 영역의 면제 합리

## 4. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 (누적 23 머지) |
| `feedback_hancom_compat_specific_over_general` | is_empty_para 가드 영역 영향 좁힘 — Task #9 텍스트 push 의도 유지 |
| `feedback_process_must_follow` | TDD Stage 0 → 1 RED → 2 분석 (가설 갱신) → 3 GREEN → 4 회귀 → 5 광범위 → 6 보고서 절차 정합 |
| `feedback_visual_judgment_authority` | 결정적 검증 + sweep + 직접 측정 통과 → 시각 판정 면제 정합 |
| `feedback_assign_issue_before_work` | Issue #716 작업지시자 등록 영역 (PR #644 시각 검증에서 발견 영역) |

## 5. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- `LAYOUT_OVERFLOW` 11.3 px 잔존 — trailing ls (Task #452 영역) 별도 task 영역. 본 PR 범위 외.

---

작성: 2026-05-09
