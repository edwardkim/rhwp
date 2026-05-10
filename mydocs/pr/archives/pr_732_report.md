---
PR: #732
제목: Task #724 — hwp3-sample5-hwp5.hwp paragraph 441 wrap zone + HWP3 파서 IR 정합 + exam_science 회귀 정정
컨트리뷰터: @jangster77 (Taesup Jang) — 17번째 사이클 (HWP 3.0 파서 영역 핵심)
처리: 옵션 A — 2 commits cherry-pick (ab910d65 skip) + no-ff merge
처리일: 2026-05-10
머지 commit: 0e419fb8
PR_supersede: PR #723 (Task #722) → PR #732 (Task #724) 영역 (c) 패턴 권위 사례
---

# PR #732 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge `0e419fb8`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `0e419fb8` (--no-ff merge) |
| Cherry-pick commits | `86eee508` (Task #724) + `25299a5b` (PR #732 후속) |
| Skip | `ab910d65` (PR #723 영역 영역 적용 영역) |
| closes | #724 |
| 시각 판정 | ✅ 통과 (작업지시자) — HWP3 sample5 + HWP5 변환본 + exam_science 회귀 정정 입증 |
| 자기 검증 | cargo test ALL GREEN + sweep 영역 영역 회귀 정정 입증 + WASM 4.65 MB |

## 2. PR supersede 체인 (c) 패턴 — 권위 사례

### 2.1 직전 PR
- **PR #723** (Task #722, @jangster77 16번째) — 머지 commit `6ced74b0`, 5/10 처리
- 작업지시자 시각 판정: ⓐ 페이지 8/27/48 PDF 정합 ✅ ⓑ exam_science p1/p2/p8 5번/8번/12번 문항 클립핑 ⚠️ 회귀 판정
- 처리 결정: PR #723 머지 유지 + exam_science 회귀 정정 영역 PR #732 영역 영역 통합 처리

### 2.2 본 PR (Task #724) — 컨트리뷰터 자발 영역 영역 통합
- Task #724 본질 정정 (HWP5 변환본 paragraph 441 wrap zone + HWP3 파서 IR 정합)
- **+ exam_science 회귀 정정** (commit `ab2fa527` → cherry-pick `25299a5b`)
- 컨트리뷰터 영역 영역 (c) 패턴 자발 영역 영역 정합 — 작업지시자 결정 영역 영역 + 컨트리뷰터 후속 영역 영역 영역

## 3. 정정 본질

### 3.1 Task #724 본질 (commit `86eee508`)

**작업지시자 핵심 지적**:
> "HWP3 파서가 잘못해석해서 IR 로 잘못 전달, composer.rs 에서 처리하면 너무 예외 — HWP3 파서 ↔ Document IR 표준 정합 영역 본질"

**`src/renderer/typeset.rs` (+49)**:
- anchor host cs=0 caption-style 매칭 가드 (`anchor_image_match`)
- Task #321 vpos-reset 가드 영역 wrap_around 강제 종료 (anchor cs=0 한정)

**`src/parser/hwp3/mod.rs` (+37/-5)**:
- 빈 paragraph + page break flag 가드 (column_type=Normal + force_vpos_reset)
- wrap_zone 영역 끝 sw=col_area_width 정합화
- wrap_zone 비활성 cs=0/sw=0 정합화

CLAUDE.md "HWP3 파서 규칙" 정합 — composer/layout 미수정.

### 3.2 PR #732 후속 (commit `25299a5b`) — exam_science 회귀 정정

```rust
// [PR #732 후속 — exam_science 회귀 가드] image_mr=0 (margin 부재) 이면
// 본 환경 OLD 동작 보존 — Task #722 host_self register skip.
if !is_caption_style && image_margin_right_hu > 0 {
    st.current_column_wrap_anchors.insert(...);
}
```

- exam_science p.21 (5번) / p.37 (8번) / p.60 (12번) 영역 영역 `image_mr=0` → 가드 차단 → OLD col_area-full-width layout 정합 복원
- hwp3-sample5.hwp 페이지 8/27/48 (Task #722 본질 영역 영역) 영역 영역 `image_mr > 0` → 가드 통과 → 정합 유지

## 4. IR 정합 검증 (HWP3 native vs HWP5 변환본)

| paragraph | 정정 전 | 정정 후 | HWP5 변환본 정합 |
|-----------|---------|---------|------------------|
| 171 column_type | Page | Normal | Normal ✓ |
| 189 ls[3~6] | sw=0 | sw=51024 | sw=51024 ✓ |
| 191 ls[0] | sw=0 | sw=51024 | sw=51024 ✓ |
| 435 ls[0~2] vpos | 221760 (절대) | 1440 (페이지 안) | 1440 ✓ |
| 443 ls[2~6] | sw=0 | sw=51024 | sw=51024 ✓ |

## 5. 본 환경 cherry-pick + 검증

### 5.1 cherry-pick

원본 8 commits 영역 영역:
- **`a8b8effa`** → cherry-pick `86eee508` (Task #724 본질)
- **`ab2fa527`** → cherry-pick `25299a5b` (PR #732 후속, exam_science 회귀 정정)
- `ab910d65` (Task #722) → **skip** — PR #723 머지 commit `6ced74b0` 영역 영역 적용 영역
- 5 merge devel commits → skip

cherry-pick 충돌 **0건** ✅.

### 5.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (32.04s) |
| `cargo test --release` (전체) | ✅ ALL GREEN, failed 0 |
| 광범위 sweep (7 fixture / 170 페이지) | **exam_science 2 diff** (회귀 정정 입증 영역 영역 — pre PR `ce5eaa6c` 영역 영역 PR #723 회귀 적용 영역, post PR `local/task724` 영역 영역 정상 복원 영역) |
| WASM 빌드 (Docker) | ✅ 4.65 MB |

### 5.3 회귀 정정 입증 (text 좌표 시프트 영역 영역 정상 복원)

| 글자 | BEFORE (PR #723 회귀) | AFTER (PR #732 정정) | 결과 |
|------|----------------------|---------------------|------|
| "후" | 585.34 | **583.18** | 좌측 시프트 (정상 복원) |
| "시" | 608.23 | **603.93** | 좌측 시프트 |
| "간" | 620.80 | **616.50** | 좌측 시프트 |
| "이" | 633.38 | **629.07** | 좌측 시프트 |
| 그림 | 655.87 | **649.87** | 좌측 시프트 |

PR #723 영역 영역 우측 시프트 회귀 영역 영역 → PR #732 후속 영역 영역 정상 복원 ✅.

## 6. 작업지시자 시각 판정 ✅ 통과

### 6.1 점검 사항 — 모두 통과
- HWP3 sample5 페이지 8/27/48 (PR #723 정합 보존) ✅
- HWP5 변환본 페이지 16/22 (Task #724 본질) ✅
- exam_science p1/p2 (회귀 정정 입증) ✅

## 7. 영향 범위

### 7.1 변경 영역
- HWP5 변환본 paragraph 441 wrap zone 매칭 (페이지 16/22)
- HWP3 native paragraph 75/191/192/435/441/443 정합 (페이지 4/9/16)
- HWP3 파서 IR 정합화 (sw=0 → sw=51024)
- **exam_science 회귀 정정** (PR #723 영역 영역 host_self register 영역 영역 가드 추가)

### 7.2 무변경 영역 (PR 본문 명시 + 본 환경 영역 영역 입증)
- PR #723 (Task #722) 페이지 8/27/48 정합 보존 (`image_mr > 0` 가드 통과)
- 다른 sample (exam_eng/aift/synam-001/exam_kor/exam_math/2010-01-06) 영역 영역 영향 없음 (sweep 0 회귀)

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **17번째 사이클** (PR #723 16번째 후속) |
| `feedback_pr_supersede_chain` | **(c) 패턴 권위 사례** — PR #723 머지 유지 + 후속 PR #732 영역 영역 회귀 정정 통합. 컨트리뷰터 자발 영역 영역 정합. PR #723 → #732 영역 영역 명확 영역 영역 사례 등록 |
| `feedback_hancom_compat_specific_over_general` | `image_margin_right_hu > 0` 가드 영역 영역 case 가드 좁힘 — 일반화 host_self register 영역 영역 회귀 영역 영역 본질 영역 |
| `feedback_visual_judgment_authority` | 작업지시자 시각 판정 ✅ 통과 — HWP3 + HWP5 변환본 + exam_science 회귀 정정 입증 |
| `feedback_image_renderer_paths_separate` | typeset.rs / parser/hwp3 영역 영역 격리 — 다른 layout/render 경로 무영향 |
| `feedback_process_must_follow` | TDD Stage 1~7 절차 정합 + IR 정합화 본질 영역 영역 도달 (CLAUDE.md HWP3 파서 규칙 정합) |

## 9. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- **Issue #762** 영역 영역 별건 — exam_math `inf`/`sup`/`lim` lookup 우선 회귀 (PR #729 영역 영역 영역 별 영역, 후속 PR 영역 영역 통합 처리 권장)
- PR #753 (Task #741, @jangster77 18번째) — hwp3-sample10.hwp HWP3 정합 (5/10 사이클 영역 영역 다음 PR)

---

작성: 2026-05-10
