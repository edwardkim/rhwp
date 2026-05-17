---
PR: #964
제목: fix — 글상자 내부 inline equation duplicate emit 차단 (시험지 page 2 <보기> textbox content scramble 해소, closes #962)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 5번째 — 마지막)
처리: 옵션 A — 본질 commit cherry-pick + orders 충돌 수동 해결 + 자기 검증 + WASM 재빌드 + no-ff merge
처리일: 2026-05-18
머지 commit: 808f419e
---

# PR #964 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A. **원 Issue #952 의 5 분리 결함 완결 + @jangster77 연속 5 PR 완결**.

| 항목 | 값 |
|------|-----|
| 머지 commit | `808f419e` (--no-ff merge) |
| Cherry-pick commit | `371fd2be` (본질 `682875fe` 만, orders 1건 충돌 수동 해결) |
| closes | #962 (Issue #952 영역 영역 **Issue 5** — PR #963 page 2 검증 중 발견) |
| 시각 판정 | ✅ 작업지시자 시각 검증 통과 |
| 자기 검증 | cargo test 1288 passed + clippy + sweep **169/169 same** + WASM 4.4 MB |
| 연속 5 PR | #956 ✅ → #958 ✅ → #961 ✅ → #963 ✅ → **#964 (5번째 마지막) ✅** |

## 2. 본질 (Issue #962)

시험지 (3-11월) page 2 문14 <보기> textbox (InFrontOfText TAC 사각형 + 내부 글상자)
inline 수식 각각 2번 emit → ㄱㄴㄷ prefix + 본문 + 수식 시각 overlap.

### Root cause (SVG 분석)
보기 textbox 영역 (y 440-540, x 400-760) 영역 영역 equation **12개** (6 × 2 duplicates):
- **Set 1** (정상, gap 위치): paragraph_layout inline TAC (paragraph_layout.rs:2078+)
- **Set 2** (duplicate, textbox 좌측 edge x=406): shape_layout 두번째 loop legacy fallback (shape_layout.rs:1609)

shape_layout 두번째 loop 영역 영역 paragraph_layout 미지원 이전 legacy fallback.
현재 paragraph_layout 가 textbox 내부 inline TAC 정상 처리 → 중복 emit.

## 3. 정정 본질 — `src/renderer/layout/shape_layout.rs:1620`

```rust
let equiv_cell_ctx = CellContext {
    parent_para_index: para_index,
    path: { parent_cell_path + CellPathEntry { control_index, cell_para=pi } },
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
- paragraph_layout 등록 시 `inline_x += eq_w` 만 (duplicate 차단)
- 미등록 시 legacy fallback 유지

## 4. 영역 좁힘

| 영역 | 영향 |
|------|------|
| textbox 내부 inline Equation (paragraph_layout 등록) | duplicate 제거 (회귀 fix) |
| textbox 내부 inline Equation (legacy fallback) | 영향 없음 (else 분기 유지) |
| textbox 내부 Shape/Picture/Table | 본 fix 미대상 |
| textbox 외부 standalone equation | 영향 없음 |

→ paragraph_layout 등록 case 한정 — `feedback_hancom_compat_specific_over_general` 정합.

## 5. 본 환경 충돌 수동 해결

| 파일 | 충돌 | 정합 |
|------|------|------|
| `mydocs/orders/20260517.md` | changed in both | `git checkout --ours` (본 환경 PR 처리 표 보존) + Task #962 작업 일지 갱신 |
| `src/renderer/layout/shape_layout.rs` | auto-merge | devel 변경 부재 |
| `task_m100_962*` 8 | added in remote | 신규 추가 |
| 시험지 fixture/PDF | 미포함 | PR 본문 명시 — 이전 PR 영역 영역 추가, 중복 방지 |

devel merge commit (`93dfe0a8`/`3d1cdf31`) 영역 영역 cherry-pick 제외 — 본질 `682875fe` 만.

## 6. 본 환경 검증 — PR #956~#964 5 정정 양립

| PR | 파일:라인 | 정정 |
|----|----------|------|
| #956 | `layout.rs:770` | paper_based = true |
| #958 | `layout.rs:3491` | caption_is_empty |
| #961 | `layout.rs:3537` | saved_y_offset |
| #963 | `paragraph_layout.rs:1730` | allow_end_tac |
| **#964** | **`shape_layout.rs:1620`** | **equiv_cell_ctx duplicate 차단** |

→ **5 정정 모두 다른 파일/영역, 양립 확인**.

| 검증 | 결과 |
|------|------|
| `cherry-pick` 본질 commit + orders 수동 해결 | ✅ |
| `cargo test --release --lib` | ✅ **1288 passed, 0 failed** (PR 본문 정합) |
| `cargo clippy --release --lib -- -D warnings` | ✅ 통과 |
| **광범위 sweep 7 fixture / 169 페이지** | ✅ **169 same / 0 diff** (회귀 부재) |
| WASM 재빌드 | ✅ 4.4 MB |
| 작업지시자 시각 판정 | ✅ **통과** |

sweep fixture 영역 영역 시험지 미포함 → 작업지시자 시각 검증 영역 영역 핵심 게이트.

## 7. 작업지시자 시각 판정 ✅ 통과

- 시험지 (3-11월) page 2 문14 <보기> textbox — equations 12→6, ㄱ. h(1)=3 / ㄴ. 함수 h(x)는... / ㄷ. 함수 g(x)가... 정합 (한컴 PDF `pdf/3-11월_실전_통합_2022.pdf` 권위, scramble 해소)
- 시험지 4종 page 2 / exam_kor/math/eng / sample10~14 회귀 부재
- PR #956/#958/#961/#963 회귀 부재 (5 정정 양립)

## 8. Issue #952 / 분리 결함 최종 완결

| Issue | PR | 머지 |
|-------|-----|------|
| #952 Issue 1 (외곽선) | #956 | `b31e38ff` |
| #952 Issue 2 (sample16 p18) | #958 (#957) | `0b630773` |
| #952 Issue 3 (시험지 p1 문9) | #961 (#959) | `586e3cc0` |
| #960 (시험지 p2 cases) | #963 | `415b9d8d` |
| **#962 (보기 textbox duplicate)** | **#964** | **`808f419e`** |

→ 원 Issue #952 의 **5 분리 결함 모두 완결**. 진단 방법론 — 1 통합 issue →
5 분리 결함 → 부분 해결 + 명확한 분리 (archive/task936 "9회 시도 + 5회 revert"
대조 교훈) 정합. 잔존 결함 부재.

## 9. CI 통과

✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff (전 항목)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 5번째 — 마지막) |
| `feedback_image_renderer_paths_separate` 권위 사례 강화 | shape_layout.rs (legacy fallback) vs paragraph_layout.rs (정상) 두 경로 동시 emit duplicate 정확 진단 |
| `feedback_hancom_compat_specific_over_general` | paragraph_layout 등록 case 한정 가드 — 케이스별 명시 (legacy fallback else 유지) |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | SVG 분석 영역 영역 Set 1 + Set 2 12개 (6×2) duplicate 정확 진단 |
| `feedback_pr_supersede_chain` 권위 사례 강화 | Issue #952 (5 분리 결함) → #956/#958/#961 + #960 (#963) + #962 (#964) — 검증 중 결함 발견 5-연쇄 완결 |
| `feedback_visual_judgment_authority` | sweep fixture 미포함 영역 영역 작업지시자 시각 판정 영역 영역 핵심 게이트 (PR #963 exam_math p18 선례) |
| `reference_authoritative_hancom` | 시험지 한컴 PDF (`pdf/3-11월`) 권위 page 2 보기 textbox ㄱ/ㄴ/ㄷ 기준 |

## 11. 연속 5 PR (@jangster77) 완결 총평

| PR | Issue | 본질 | 진단 도구 | 머지 |
|----|-------|------|----------|------|
| #956 | #952 Issue 1 | page border paper/body | bisect (`4bb11289`) | `b31e38ff` |
| #958 | #957 (Issue 2) | sample16 p18 빈 caption phantom | RHWP_DEBUG_TAC_CURSOR | `0b630773` |
| #961 | #959 (Issue 3) | 시험지 p1 문9 column picture advance | RHWP_DEBUG_TAC_CURSOR | `586e3cc0` |
| #963 | #960 (Issue 4) | 시험지 p2 cases off-by-one | RHWP_DEBUG_PARA_TAC | `415b9d8d` |
| #964 | #962 (Issue 5) | 시험지 p2 보기 textbox duplicate | SVG 분석 | `808f419e` |

- 환경변수 진단 도구 (RHWP_DEBUG_*) 영구화 — 차후 재회귀 추적 인프라
- 5 정정 모두 다른 파일/영역 — 정확한 결함 분리 + 양립
- 매 PR 작업지시자 시각 판정 + cargo test 1288 + sweep — 결정적 검증 일관
- archive/task936 "9회 시도 + 5회 revert" 대조 — 부분 해결 + 명확한 분리 방법론 입증

## 12. 잔존 후속

- 원 Issue #952 의 5 분리 결함 모두 완결 — 잔존 결함 부재
- Issue #962 close 완료
- @jangster77 연속 5 PR 완결

---

작성: 2026-05-18
