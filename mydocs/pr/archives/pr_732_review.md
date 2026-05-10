---
PR: #732
제목: Task #724 — hwp3-sample5-hwp5.hwp paragraph 441 wrap zone + HWP3 파서 IR 정합 + exam_science 회귀 정정
컨트리뷰터: @jangster77 (Taesup Jang) — 17번째 사이클 (HWP 3.0 파서 영역 핵심)
base / head: devel / local/task724
mergeStateStatus: DIRTY
mergeable: CONFLICTING — `git merge-tree` 충돌 1건 (`src/renderer/typeset.rs`)
CI: 미실행 (PR base 영역 영역 PR #723 영역 영역 분기 영역 영역)
변경 규모: +2408 / -13, 26 files (소스 4 + 보고서 22)
검토일: 2026-05-10
PR_supersede: PR #723 (Task #722) → PR #732 (Task #724) 영역 (c) 패턴 정합
---

# PR #732 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #732 |
| 제목 | Task #724 — hwp3-sample5-hwp5.hwp paragraph 441 wrap zone + HWP3 파서 IR 정합 + exam_science 회귀 정정 |
| 컨트리뷰터 | @jangster77 — **17번째 사이클** (HWP 3.0 파서 영역 핵심, 직전 PR #723 16번째 영역 영역 후속) |
| base / head | devel / local/task724 |
| mergeStateStatus | DIRTY, mergeable: **CONFLICTING** — `src/renderer/typeset.rs` 충돌 |
| CI | 결과 부재 영역 영역 (PR base 영역 영역 PR #723 영역 영역 분기 영역 영역) |
| 변경 규모 | +2408 / -13, 26 files (소스 4 + 보고서 22) |
| 커밋 수 | 8 (Task #722 1건 + Task #724 본질 1건 + 5 merge devel + PR #732 후속 1건) |
| closes | #724 |

## 2. PR supersede 체인 — (c) 패턴 정합

### 2.1 직전 PR
- **PR #723** (Task #722, @jangster77 16번째) — 머지 commit `6ced74b0`, 5/10 처리
- 작업지시자 시각 판정: ⓐ 페이지 8/27/48 PDF 정합 ✅ ⓑ exam_science p1/p2/p8 5번/8번/12번 문항 클립핑 ⚠️ 회귀 판정
- 처리 결정 (`feedback_pr_supersede_chain` (c) 패턴): PR #723 머지 유지 + exam_science 회귀 정정 영역 영역 PR #732 영역 영역 통합 처리

### 2.2 본 PR (Task #724)
- Task #724 본질 정정 (HWP5 변환본 paragraph 441 wrap zone + HWP3 파서 IR 정합)
- **+ exam_science 회귀 정정** (commit `ab2fa527`) — (c) 패턴 정합 영역 영역 컨트리뷰터 영역 영역 자발 영역 영역 통합

## 3. 결함 본질

### 3.1 Task #724 (Issue #724)
HWP5 변환본 (한컴 내부 변환, `samples/hwp3-sample5-hwp5.hwp`) 페이지 16 paragraph 441 ("하드디스크는 하나 이상의 둥그런 플래터(platter)로 구성되고...") 영역 image 우측 wrap zone 영역 영역 layout 부재 영역 — 좁은 폭 분할 (한컴 PDF 권위 자료 위반).

**본질 진단** (작업지시자 핵심 지적):
> "HWP3 파서가 잘못해석해서 IR 로 잘못 전달, composer.rs 에서 처리하면 너무 예외 — HWP3 파서 ↔ Document IR 표준 정합 영역 본질"

본 환경 HWP3 파서 영역 영역 wrap_zone 영역 끝 / 페이지 break 후 paragraph 영역 영역 LINE_SEG.sw 영역 영역 **0** 영역 영역 인코딩. 한컴 HWP5 변환본 IR 정합 영역 영역 **sw=51024** (col_area 전체 폭). 본 환경 composer/layout 영역 영역 sw=0 fallback 결함 영역 영역 좁은 폭 분산 layout 발생.

### 3.2 PR #723 영역 영역 회귀 (commit `ab2fa527`)
PR #723 (Task #722) 영역 영역 의 `host_self register` 영역 영역 — `image_voff=0` + `image_mr=0` 영역 영역 의 paragraph 영역 영역 fire → exam_science p.21 (5번) / p.37 (8번) / p.60 (12번) 영역 영역 의 Square wrap picture 영역 영역 의도 외 영향 → text justification 영향 → 글자 위치 ±2~13px 시프트 + 좌측 경계 클립핑.

## 4. PR 의 정정 — 4 영역

### 4.1 `src/renderer/typeset.rs` (+108/-5)

**Task #724 본질 (commit `a8b8effa`)**:
1. **anchor host cs=0 caption-style 매칭 가드** (`anchor_image_match`)
   - `expected_cs = (image_x_offset + width + 2*margin) - body_left + tolerance 200 HU + para_cs+sw <= body_w`
2. **Task #321 vpos-reset 가드 영역 wrap_around 강제 종료** (anchor cs=0 한정):
   ```rust
   if st.wrap_around_cs == 0 {
       st.wrap_around_cs = -1;
       st.wrap_around_sw = -1;
       st.wrap_around_any_seg = false;
   }
   if st.wrap_around_cs < 0 { st.advance_column_or_new_page(); }
   ```
   - 일반 wrap_around (anchor cs>0) 영역 영역 기존 동작 (Task #362 vpos-reset 무시) 유지

**PR #723 회귀 정정 (commit `ab2fa527`)**:
- `host_self register` 영역 영역 `image_margin_right_hu > 0` 가드 추가
- exam_science 영역 영역 `image_mr=0` 영역 영역 → 가드 차단 → OLD 의 col_area-full-width layout 정합 복원
- hwp3-sample5.hwp 영역 영역 page 8/27/48 영역 영역 `image_mr > 0` → 가드 통과 → 정합 유지

```rust
if !is_caption_style && image_margin_right_hu > 0 {
    st.current_column_wrap_anchors.insert(...);
}
```

### 4.2 `src/parser/hwp3/mod.rs` (+37/-5)

**HWP3 파서 IR 정합화** (CLAUDE.md "HWP3 파서 규칙" 정합):
1. **빈 paragraph + page break flag 가드 2 곳** — `column_type=Normal + force_vpos_reset` (HWP5 변환본 IR 정합 paragraph 171 column_type=Normal)
2. **wrap_zone 영역 끝 sw=col_area_width** (본 환경 sw=0 인코딩 결함 정정)
3. **wrap_zone 비활성 cs=0/sw=0 → sw=col_area_width** (페이지 break 후 paragraph 정합)

### 4.3 `src/renderer/pagination.rs` (+6) + `src/renderer/layout/paragraph_layout.rs` (+10/-3)

PR #723 영역 영역 (Task #722) 영역 영역 영역 commits 영역 영역 (`ab910d65`) 영역 영역 — 본 환경 영역 영역 이미 적용 영역 영역 (PR #723 머지 영역 영역).

## 5. IR 정합 검증 (HWP3 native vs HWP5 변환본) — PR 본문 명시

| paragraph | 정정 전 | 정정 후 | HWP5 변환본 정합 |
|-----------|---------|---------|------------------|
| 171 column_type | Page | Normal | Normal ✓ |
| 189 ls[3~6] | sw=0 | sw=51024 | sw=51024 ✓ |
| 191 ls[0] | sw=0 | sw=51024 | sw=51024 ✓ |
| 435 ls[0~2] vpos | 221760 (절대) | 1440 (페이지 안) | 1440 ✓ |
| 443 ls[2~6] | sw=0 | sw=51024 | sw=51024 ✓ |

## 6. 본 환경 점검

### 6.1 충돌 영역 영역 분석

`git merge-tree --write-tree local/devel pr732-head` → **CONFLICT 1건** (`src/renderer/typeset.rs`).

**충돌 본질**: PR #732 영역 영역 영역 PR #723 commit (`ab910d65`) 영역 영역 분기 영역 영역 — 본 환경 영역 영역 PR #723 영역 영역 머지된 commit (`6ced74b0`) 영역 영역 의 `ab910d65` 영역 영역 적용 영역 영역. 그러나 PR #732 영역 영역 영역 동일 commit `ab910d65` + Task #724 본질 (`a8b8effa`) + 후속 (`ab2fa527`) 영역 영역 누적 변경 영역 영역 → typeset.rs 영역 영역 충돌.

### 6.2 cherry-pick 전략

`ab910d65` 영역 영역 영역 본 환경 영역 영역 이미 적용 (PR #723 머지) 영역 영역 영역 **skip 영역**. cherry-pick 영역 영역 commits:
- **`a8b8effa`** — Task #724 본질
- **`ab2fa527`** — PR #732 후속 (exam_science 회귀 정정)

PR #729/#730 영역 영역 영역 동일 패턴 영역 영역 — 개별 cherry-pick 충돌 발생 영역 영역 영역 PR HEAD squash cherry-pick 영역 영역 단일 commit 적용 영역.

### 6.3 의존성 점검

| 의존성 | 상태 |
|--------|------|
| PR #723 (Task #722) 머지 | ✅ 본 환경 영역 영역 머지 commit `6ced74b0` |
| `samples/hwp3-sample5-hwp5.hwp` 존재 | 점검 필요 |
| `samples/hwp3-sample5.hwp` 존재 (PR #723 영역 영역 보존) | ✅ |
| PDF 권위본 (`pdf/hwp3-sample5-2022.pdf`) | ✅ |

## 7. 영향 범위

### 7.1 변경 영역
- HWP5 변환본 영역 영역 paragraph 441 wrap zone 매칭 (페이지 16/22)
- HWP3 native 영역 영역 paragraph 75/191/192/435/441/443 정합 (페이지 4/9/16)
- HWP3 파서 IR 정합화 (본 환경 sw=0 인코딩 결함 영역 영역)
- **exam_science 회귀 정정** (PR #723 영역 영역 host_self register 영역 영역)

### 7.2 무변경 영역 (PR 본문 명시)
- PR #723 (Task #722) 영역 영역 페이지 8/27/48 정합 보존 (`image_mr > 0` 가드 통과)
- 다른 sample (exam_eng/aift/synam-001) 영역 영역 영향 없음

### 7.3 위험 영역
- **typeset.rs 영역 영역 누적 변경** — Task #722 + Task #724 + PR #732 후속 영역 영역 동일 함수 영역 영역 변경 — 회귀 가드 영역 영역 점검 필요
- HWP5 변환본 영역 영역 영역 fixture 영역 영역 한정 — 광범위 영향 영역 영역 점검 필요

## 8. 작업지시자 시각 판정 (PR 본문 명시)

PR 본문 영역 영역 ★ 통과 명시:
- HWP3 native 페이지 4/8/9/16/22 ✓
- HWP5 변환본 페이지 16/22 ✓
- PR #723 영역 영역 (페이지 8/27/48) 정합 보존 ✓

본 환경 영역 영역 영역 작업지시자 시각 판정 게이트 영역 영역 점검 필요:
- HWP3 sample5 영역 영역 페이지 8/27/48 (PR #723 정합 보존)
- HWP5 변환본 영역 영역 페이지 16/22 (Task #724 본질)
- **exam_science p1/p2/p8** (PR #732 후속 영역 영역 회귀 정정 입증)
- **광범위 sweep** — 7 fixture / 170 페이지 / 회귀 0 영역 영역 점검

## 9. 처리 옵션

### 옵션 A — Task #724 본질 + PR #732 후속 영역 영역 cherry-pick (추천)

```bash
git checkout -b local/task724 ce5eaa6c
git cherry-pick a8b8effa ab2fa527
# 충돌 발생 시 squash cherry-pick 채택
git checkout local/devel
git merge --no-ff local/task724
```

### 옵션 B — PR HEAD 영역 영역 영역 영역 squash cherry-pick (개별 충돌 발생 시)

```bash
git checkout -b local/task724 ce5eaa6c
# ab910d65 영역 영역 본 환경 적용 영역 영역 → squash 영역 영역 영역 동일 변경 영역 영역 무시 가능
git cherry-pick --no-commit a8b8effa^..ab2fa527
# 충돌 영역 영역 수동 해결 (typeset.rs 영역 영역 PR #723 + Task #724 + PR #732 후속 영역 영역 통합)
```

→ **옵션 A 영역 영역 시도 후 충돌 발생 시 옵션 B** 권장.

## 10. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (특히 **exam_science** 영역 영역 회귀 정정 입증)
- [ ] PR #723 영역 영역 (hwp3-sample5 페이지 8/27/48) 정합 보존

### 시각 판정 게이트 — **작업지시자 권장**

본 PR 영역 영역 의 본질 영역 영역 **시각 정합** (HWP5 변환본 영역 영역 한컴 PDF 권위 자료 정합):
- HWP3 sample5 페이지 8/27/48 (PR #723 정합 보존)
- HWP5 변환본 페이지 16/22 (Task #724 본질)
- **exam_science p1/p2/p8** (회귀 정정 입증)

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **17번째 사이클** (PR #723 16번째 후속) |
| `feedback_pr_supersede_chain` | **(c) 패턴 권위 사례 강화** — PR #723 머지 유지 + 후속 PR #732 영역 영역 회귀 정정 통합 영역 영역 컨트리뷰터 자발 영역 영역 정합 |
| `feedback_hancom_compat_specific_over_general` | `image_margin_right_hu > 0` 가드 영역 영역 case 가드 좁힘 — 일반화 host_self register 영역 영역 회귀 영역 영역 본질 영역 |
| `feedback_visual_judgment_authority` | 작업지시자 시각 판정 권위 — PR 본문 ★ 통과 + 본 환경 영역 영역 재검증 권장 |
| `feedback_image_renderer_paths_separate` | typeset.rs / parser/hwp3 영역 영역 격리 — 다른 layout/render 경로 무영향 |
| `feedback_process_must_follow` | TDD Stage 1~9 절차 정합 + IR 정합화 본질 영역 영역 도달 (CLAUDE.md HWP3 파서 규칙 정합) |

## 12. 처리 순서 (승인 후)

1. `local/task724` 영역 영역 cherry-pick (Task #724 본질 `a8b8effa` + 후속 `ab2fa527`)
2. 충돌 발생 시 squash cherry-pick 채택 (PR #729/#730 동일 패턴)
3. 자기 검증 (cargo test/build/clippy + 광범위 sweep + exam_science 회귀 정정 입증)
4. WASM 빌드 + 작업지시자 시각 판정 (HWP3 sample5 + HWP5 변환본 + exam_science 회귀)
5. 시각 판정 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
6. PR #732 close (closes #724) + Issue #762 close (PR #729 영역 영역 의 (c) 패턴 영역 영역 후속 통합 — 별 영역 영역 의 영역 영역)

---

작성: 2026-05-10
