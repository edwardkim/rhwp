# PR #578 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ 통과

**PR**: [#578 Task #576: 수식 토크나이저 times/sim 키워드 prefix-split 정정 (closes #576)](https://github.com/edwardkim/rhwp/pull/578)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**관련**: closes #576, **PR #579 (@oksure) — CLOSED 영역** (5/4 협업 PR 권유 후 self-close)
**처리 결정**: ✅ **핀셋 cherry-pick 머지 + push + PR/Issue close**
**처리일**: 2026-05-06

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (`6f3d94aa` 단독) + devel merge + push + PR close |
| 시각 판정 | ★ **통과** (작업지시자 웹 시각 판정) |
| Devel merge commit | `72cd9fd` |
| Cherry-pick 충돌 | 0 건 |
| Author 보존 | ✅ Jaeook Ryu (@jangster77) 보존 |
| Issue #576 | CLOSED (closes #576 자동 처리) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,614 페이지 / 회귀 0 |

## 2. 본질 결함 (PR 진단)

`src/renderer/equation/tokenizer.rs::read_command` (L104) 의 prefix-split 키워드 list 가 폰트 스타일 모디파이어 3 개 (`bold/it/rm`) 만 처리. `times` / `sim` 등 연산자 키워드가 변수와 인접 시 단일 식별자로 토큰화:
- `"a timesm"` → 단일 `"timesm"` italic 식별자
- `"X simZ"` → 단일 `"simZ"` italic 식별자

## 3. 본질 정정 (4 키워드 추가)

```rust
// list 확장: 'bold/it/rm' → 'bold/it/rm/times/sim/TIMES/SIM'
```

**광범위 sweep (Stage 1, 158 fixture / 563 unique 수식 scripts)** 결과 발현 4 키워드만 추가 → 회귀 위험 0.

### 3.1 정량 측정

| Script | Before | After |
|--------|--------|-------|
| `{b} over {a timesm}` 분모 | "a timesm" italic 식별자 | **"a × m"** (× = U+00D7) ✓ |
| `rm X simZ` | "X simZ" italic 식별자 | **"X ∼ Z"** (∼ = U+223C) ✓ |
| `1TIMES10^-14` | "1TIMES10" italic 식별자 | **"1 × 10⁻¹⁴"** ✓ |
| `rmA SIMC` | "rmA SIMC" italic 식별자 | **"A ∼ C"** ✓ |

### 3.2 회귀 차단 (6 신규 unit tests)

```rust
test_task576_times_lowercase_prefix_split    "a timesm" → ["a", "times", "m"]
test_task576_sim_lowercase_prefix_split      "rm X simZ" → ["rm", "X", "sim", "Z"]
test_task576_times_uppercase_prefix_split    "1TIMES10" → ["1", "TIMES", "10"]
test_task576_sim_uppercase_prefix_split      "rmA SIMC" → ["rm", "A", "SIM", "C"]
test_task576_alpha_no_split                  "alpha"/"alphabet" 분리 안 됨 (회귀 차단)
test_task576_times_followed_by_space         "a times b" 공백 구분 보존
```

## 4. PR #579 영역 (5/4 협업 PR 권유 시도, 본 사이클 첫 사례)

5/4 작업지시자 협업 PR 권유 댓글 후 PR #579 (@oksure) **self-close**:
- 작업지시자 의도: "양쪽 통합" (PR #579 인프라 + PR #578 sweep 결과)
- 실제 결과: @oksure 가 PR #579 close (PR #578 단독으로 영역 진행)
- @planet6897 답신 (5/4 22:23): "issue 처리시에 조금 더 주의해서 진행하겠습니다." (협업 영역 인지)

→ **PR #578 단독 cherry-pick 으로 영역 정합** + 협업 PR 권유 시도는 본 사이클 학습 영역.

## 5. PR 의 5 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `fca11482` Stage 0 — 수행 계획서 | 컨트리뷰터 fork plans | 무관 |
| `48642e15` Stage 1 — 정밀 진단 + 광범위 sweep | 컨트리뷰터 fork working | 무관 |
| `79a5f16b` Stage 2 — 구현 계획서 | 컨트리뷰터 fork plans | 무관 |
| **`6f3d94aa` Stage 3 — 본질 정정** | `tokenizer.rs` +53 LOC + working stage3 | ⭐ cherry-pick |
| `40e147c4` Stage 4 — 최종 보고서 | 컨트리뷰터 fork report | 무관 |

→ 본질 1 commit 만 cherry-pick.

## 6. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| **task576 unit tests** | ✅ **6/6 passed** (alpha 회귀 차단 가드 포함) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,583,156 bytes** (1m 26s, v0.7.10 baseline +1,691 bytes — tokenizer.rs +53 LOC + 6 unit tests 정합) |

## 7. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

→ 수식 토크나이저 변경이 페이지네이션에 영향 없음.

## 8. SVG byte 차이 (PR 본문 100% 재현)

| 페이지 | byte 차이 | 정정 영역 |
|------|---------|---------|
| page 1 | identical | ✅ 회귀 0 |
| page 2 | identical | ✅ 회귀 0 |
| **page 3** | **differ** | 15번 본문/보기 (`rm W simY/Z`, `rmA SIMC` 등) |
| **page 4** | **differ** | 20번 본문/응답 (`{b} over {a timesm}`, `1TIMES10^-14`, `rm X simZ`) |

## 9. 시각 판정 (★ 게이트)

작업지시자 시각 검증 결과:
> 웹 시각판정 통과입니다.

→ ★ **통과**.

## 10. PR / Issue close 처리

### 10.1 PR #578 close
- 댓글 등록 (한글, cherry-pick 결과 + 결정적 검증 + 광범위 sweep + PR 본문 100% 재현 + PR #579 협업 영역 학습 + 컨트리뷰터 협업 인정)
- close 처리

### 10.2 Issue #576
- closes #576 키워드 자동 처리 — CLOSED 확인

## 11. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 sweep (158 fixture / 563 scripts) + 6 신규 tokenizer tests + 164/1,614 페이지 회귀 0
- ✅ `feedback_hancom_compat_specific_over_general` — 명시 4 키워드 list (case-specific) + alpha/over/sqrt 회귀 차단
- ✅ `feedback_rule_not_heuristic` — 명시적 키워드 list, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ 통과)
- ✅ `feedback_pdf_not_authoritative` — PR 본문 명시 (Unicode 코드포인트 검증, PDF 미사용)
- ✅ `feedback_per_task_pr_branch` — Task #576 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 한글 답변 (PR #599 학습 적용)
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + PR #579 close 영역 점검 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/6 v0.7.10 후) 후속 patch 영역 정합

## 12. 본 PR 의 본질 — v0.7.10 후 첫 처리 PR

본 PR 의 처리 본질에서 가장 우수한 점:

1. **광범위 sweep + 명시 list 결합** — 158 fixture / 563 scripts 의 정밀한 회귀 영역 측정 + 4 키워드 명시 list (case-specific 가드)
2. **6 신규 unit tests** — 회귀 차단 가드 (alpha 그리스 문자 prefix 충돌 + 공백 구분 보존)
3. **PR #579 협업 영역 학습** — 5/4 협업 PR 권유 시도 후 컨트리뷰터의 학습 의지 영역 인지
4. **v0.7.10 후 첫 처리** — 본 사이클 (5/6) 의 첫 PR 처리 패턴 정합 영역 (한글 답변 / commit 단위 cherry-pick / 광범위 sweep)

## 13. 본 사이클 사후 처리

- [x] PR #578 close (cherry-pick 머지 + push + 한글 댓글)
- [x] Issue #576 close (closes #576 자동 처리)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_578_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_578_review.md` → `mydocs/pr/archives/pr_578_review.md`)
- [ ] 5/6 orders 신규 작성 (PR #578 항목 첫 추가)
