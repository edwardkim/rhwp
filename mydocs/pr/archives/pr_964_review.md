---
PR: #964
제목: fix — 글상자 내부 inline equation duplicate emit 차단 (시험지 page 2 <보기> textbox content scramble 해소, closes #962)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 5번째 — 마지막)
base / head: devel / local/task962
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +696 / -29, 9 files (코드 1 / 문서 8)
커밋: 3 (본질 1 + devel merge 2)
검토일: 2026-05-18
---

# PR #964 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #964 |
| 제목 | fix: 글상자 내부 inline equation duplicate emit 차단 (시험지 page 2 <보기> textbox scramble) |
| 컨트리뷰터 | @jangster77 — **24+ 사이클** (연속 5 PR **5번째 — 마지막**) |
| base / head | devel / local/task962 |
| mergeable | MERGEABLE (BEHIND — base 갱신만) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +696 / -29, 9 files (코드 1 / 문서 8) |
| 커밋 수 | 3 (본질 `682875fe` + devel merge `93dfe0a8`/`3d1cdf31`) |
| closes | #962 (Issue #952 영역 영역 **Issue 5** — PR #963 page 2 검증 중 발견) |
| 연속 5 PR | #956 ✅ → #958 ✅ → #961 ✅ → #963 ✅ → **#964 (5번째 마지막)** |

## 2. 본질 (Issue #962)

시험지 (3-11월) page 2 문14 <보기> textbox (InFrontOfText TAC 사각형 + 내부 글상자)
의 inline 수식이 **각각 2번 emit** → ㄱㄴㄷ prefix + 본문 + 수식 시각 overlap.

### Root cause (SVG 분석)
보기 textbox 영역 (y 440-540, x 400-760) 영역 영역 equation **12 개** (예상 6 × 2 duplicates):
- **Set 1** (정상, gap 위치): paragraph_layout inline TAC 처리 (paragraph_layout.rs:2078+)
- **Set 2** (duplicate, textbox 좌측 edge x=406): shape_layout 두번째 loop Equation branch (shape_layout.rs:1609)

shape_layout 두번째 loop 영역 영역 paragraph_layout 미지원 이전의 **legacy fallback**.
현재 paragraph_layout 가 textbox 내부 inline TAC 정상 처리 → 중복 emit.

## 3. 정정 본질 — `src/renderer/layout/shape_layout.rs:1610`

```rust
let equiv_cell_ctx = CellContext {
    parent_para_index: para_index,
    path: { /* parent_cell_path + textbox entry (control_index, cell_para=pi) */ },
};
if tree.get_inline_shape_position(
    section_index, pi, ctrl_idx_in_para, Some(&equiv_cell_ctx)
).is_some() {
    inline_x += eq_w;  // paragraph_layout 가 이미 emit — duplicate 차단
} else {
    // legacy fallback (기존 emit 분기 유지)
}
```

- emit 전 `get_inline_shape_position` 확인 (equiv_cell_ctx — textbox entry 포함 경로)
- paragraph_layout 등록 시 `inline_x += eq_w` 만 (duplicate emit 차단)
- 미등록 시 legacy fallback 유지

## 4. 영역 좁힘 (PR 본문 명시)

| 영역 | 영향 |
|------|------|
| textbox 내부 inline Equation (paragraph_layout 등록) | duplicate 제거 (회귀 fix) |
| textbox 내부 inline Equation (legacy fallback) | 영향 없음 (else 분기 유지) |
| textbox 내부 Shape/Picture/Table | 본 fix 미대상 |
| textbox 외부 standalone equation | 영향 없음 |

→ paragraph_layout 등록 case 한정 — `feedback_hancom_compat_specific_over_general` 정합.

## 5. 본 환경 충돌 분석

| 파일 | 충돌 | 정합 |
|------|------|------|
| `mydocs/orders/20260517.md` | changed in both | `git checkout --ours` (본 환경 PR 처리 표 보존) + Task #962 작업 일지 갱신 |
| `src/renderer/layout/shape_layout.rs` | 충돌 없음 | devel 변경 부재. PR #956/#958/#961 (layout.rs) + #963 (paragraph_layout.rs) + 본 PR (shape_layout.rs) — **5 정정 모두 다른 파일, 양립** |
| `task_m100_962*` 8 | added in remote | 신규 추가 (충돌 없음) |
| 시험지 fixture/PDF | 미포함 | PR 본문 명시 — 이전 PR 영역 영역 추가, 중복 방지 |

devel merge commit (`93dfe0a8`/`3d1cdf31`) 영역 영역 cherry-pick 제외 — 본질 `682875fe` 만.

## 6. 본 환경 점검

### 6.1 PR #956/#958/#961/#963/#964 5 정정 양립
- PR #956: `layout.rs:770` paper_based
- PR #958: `layout.rs:3491` caption_is_empty
- PR #961: `layout.rs:3537` saved_y_offset
- PR #963: `paragraph_layout.rs:1730` allow_end_tac
- PR #964: `shape_layout.rs:1610` equiv_cell_ctx duplicate 차단
→ 5 정정 모두 다른 파일/영역, 양립.

### 6.2 CI 통과
- ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff (전 항목)

### 6.3 검증 (PR 본문)
- cargo test --release --lib: 1288 passed, 0 failed
- 시험지 page 2 보기 textbox equations: 12 → 6 ✓
- 시각: ㄱ. h(1)=3 / ㄴ. 함수 h(x)는... / ㄷ. 함수 g(x)가... ✓ 한컴 PDF 정합
- LAYOUT_OVERFLOW count: 325 → 325 (회귀 0)
- exam_kor/math/eng, sample14, 시험지 4종: 시각 회귀 0

## 7. Issue #952 / 분리 결함 최종 추적

원 Issue #952 영역 영역 **5 분리 결함** — 본 PR 머지 영역 영역 5 issue 모두 해결 (Issue #952 영역 영역
PR #961 머지 영역 영역 이미 close).

| Issue | PR | 상태 |
|-------|-----|------|
| #952 Issue 1 (외곽선) | #956 | ✅ merged |
| #952 Issue 2 (sample16 p18) | #958 (#957) | ✅ merged |
| #952 Issue 3 (시험지 p1 문9) | #961 (#959) | ✅ merged (Issue #952 closed) |
| #960 (시험지 p2 cases) | #963 | ✅ merged |
| **#962 (보기 textbox duplicate)** | **#964 (본 PR)** | 처리 중 → 5 issue 완결 |

## 8. 처리 옵션

### 옵션 A (권장) — 본질 commit cherry-pick + orders 충돌 수동 해결 + 자기 검증 + WASM 재빌드

```bash
git checkout local/devel
git cherry-pick 682875fe   # 본질만 (devel merge commit 2개 제외)
# orders/20260517.md 충돌 수동 해결 (--ours + Task #962 작업 일지 갱신)
# cargo test + 광범위 sweep (shape_layout equation emit 변경 → sweep 필수)
# WASM 재빌드
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash 3 commits (devel merge 포함, 비권장)

## 9. 검증 게이트

### 9.1 자기 검증
- [ ] cherry-pick `682875fe` (본질만) + orders 충돌 수동 해결
- [ ] PR #956~#964 5 정정 양립 확인 (layout.rs 3 + paragraph_layout.rs 1 + shape_layout.rs 1)
- [ ] cargo test --release --lib ALL GREEN (PR 본문 1288 passed)
- [ ] cargo clippy --release -- -D warnings
- [ ] **광범위 sweep 7 fixture / 169 페이지** — shape_layout equation emit 변경 영역 영역 회귀 점검 필수
- [ ] LAYOUT_OVERFLOW count 회귀 0
- [ ] WASM 재빌드 (shape_layout.rs 변경)

### 9.2 시각 판정 게이트 — **작업지시자 시각 검증 권장**
- 시험지 (3-11월) page 2 문14 <보기> textbox — equations 12→6, ㄱ/ㄴ/ㄷ 정합 (한컴 PDF `pdf/3-11월_실전_통합_2022.pdf` 권위, scramble 해소)
- 시험지 4종 page 2 정상
- exam_kor/math/eng, sample10~14 회귀 부재
- PR #956/#958/#961/#963 회귀 부재 (5 정정 양립)
- sweep diff 발생 시 작업지시자 한컴 정합 판정 (PR #963 exam_math p18 패턴)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 5번째 — 마지막) |
| `feedback_image_renderer_paths_separate` 권위 사례 강화 | shape_layout.rs (legacy fallback) vs paragraph_layout.rs (정상) duplicate emit — 두 경로 동시 emit 본질 정확 진단 |
| `feedback_hancom_compat_specific_over_general` | paragraph_layout 등록 case 한정 가드 — 케이스별 명시 (legacy fallback else 유지) |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | SVG 분석 영역 영역 Set 1 (paragraph_layout) + Set 2 (shape_layout legacy) 12개 (6×2) duplicate 정확 진단 |
| `feedback_pr_supersede_chain` 권위 사례 강화 | Issue #952 (5 분리 결함) → #956/#958/#961 + #960 (#963) + #962 (본 PR) — 검증 중 결함 발견 5-연쇄 완결 |
| `feedback_visual_judgment_authority` | sweep diff 시 작업지시자 시각 판정 권위 (PR #963 exam_math p18 선례) |
| `reference_authoritative_hancom` | 시험지 한컴 PDF (`pdf/3-11월`) 권위 page 2 보기 textbox ㄱ/ㄴ/ㄷ 기준 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick `682875fe` (본질만) + orders 충돌 수동 해결
2. 자기 검증 — PR #956~#964 5 정정 양립 + cargo test + clippy + 광범위 sweep + LAYOUT_OVERFLOW 회귀 0 + WASM 재빌드
3. 작업지시자 시각 검증 (시험지 page 2 보기 textbox 12→6 한컴 PDF 정합 + 회귀 부재)
4. 검증 통과 → no-ff merge + push + archives + 5/17 orders
5. Issue #962 close (원 #952 의 5 issue 모두 완결)
6. PR #964 close — **연속 5 PR (@jangster77) 완결**

---

작성: 2026-05-18
