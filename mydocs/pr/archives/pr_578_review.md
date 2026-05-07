# PR #578 검토 보고서

**PR**: [#578 Task #576: 수식 토크나이저 times/sim 키워드 prefix-split 정정 (closes #576)](https://github.com/edwardkim/rhwp/pull/578)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**상태**: OPEN, **mergeable=MERGEABLE**
**관련**: closes #576, **PR #579 (@oksure) — CLOSED 영역** (5/4 협업 PR 권유 후 self-close)
**처리 결정**: ✅ **핀셋 cherry-pick + devel merge + push + PR close 완료** (옵션 A 진행 — ★ 시각 판정 통과)
**검토 시작일**: 2026-05-06
**처리 완료일**: 2026-05-06

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — `tokenizer.rs::read_command` (L104) 의 prefix-split 키워드 list 가 `bold/it/rm` 만 처리해서 `times/sim/TIMES/SIM` 결합 식별자가 단일 토큰화되는 결함이 본 환경에서도 재현되는가?
2. **PR #579 영역과의 정합** — 5/4 협업 PR 권유 후 PR #579 self-close 영역 — PR #578 단독 처리 정합?
3. **PR base skew (5/4 등록 → 5/6 v0.7.10 release 후 영역)** — 본 사이클 처리분과 충돌 없이 cherry-pick 가능?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Task #576 수식 토크나이저 times/sim 키워드 prefix-split 정정 | 정합 |
| author | @planet6897 (PR 등록) / Jaeook Ryu (commit author, 협업 흐름) | PR #561~#600 동일 패턴 |
| changedFiles | 6 / +774 / -1 | 본질 코드 +53 LOC + 보고서 다수 |
| 본질 변경 | `src/renderer/equation/tokenizer.rs` (+53 LOC, list 확장 + 6 신규 unit tests) | 단일 파일 |
| **mergeable** | MERGEABLE (UI) | base skew 없음 확인 필요 |
| Issue | closes #576 | ✅ |

## 3. PR 의 5 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `fca11482` Stage 0 — 수행 계획서 | 컨트리뷰터 fork plans | 무관 |
| `48642e15` Stage 1 — 정밀 진단 + 광범위 영향 sweep | 컨트리뷰터 fork working | 무관 |
| `79a5f16b` Stage 2 — 구현 계획서 | 컨트리뷰터 fork plans | 무관 |
| **`6f3d94aa` Stage 3 — 본질 정정** | `tokenizer.rs` +53 LOC + working stage3 | ⭐ cherry-pick |
| `40e147c4` Stage 4 — 최종 보고서 | 컨트리뷰터 fork report + orders | 무관 |

→ **본질 cherry-pick = `6f3d94aa` 단독**. PR #561~#600 와 동일 패턴.

## 4. 본질 변경 영역

### 4.1 결함 가설

`src/renderer/equation/tokenizer.rs::read_command` (L104) 의 prefix-split 키워드 list 가 폰트 스타일 모디파이어 3개 (`bold/it/rm`) 만 처리. `times` / `sim` 등 연산자 키워드가 변수와 인접 시 단일 식별자로 토큰화:
- `"a timesm"` → 단일 `"timesm"` 토큰 (italic 식별자)
- `"X simZ"` → 단일 `"simZ"` 토큰

### 4.2 정정 (4 키워드 추가)

```rust
// list 확장: 'bold/it/rm' → 'bold/it/rm/times/sim/TIMES/SIM'
```

**광범위 sweep 결과 (Stage 1, 158 fixture / 563 unique 수식 scripts)**: 결함 발현 4 키워드 (`times/sim/TIMES/SIM`) 만 추가. 다른 keyword (alpha/over/sqrt 등) 는 항상 공백 구분되어 prefix-split 불필요.

### 4.3 회귀 위험 회피 — alpha/over/sqrt 미포함

PR #579 (@oksure) 의 일반 인프라 (`is_known_command()` + `longest_keyword_prefix_len()`) 가 가졌던 잠재 회귀:
- `alphabet → alpha + bet` (그리스 문자 prefix 충돌)
- `sqrtest → sqrt + est`

→ 본 PR 의 sweep 결과로 **이 영역 발현 안 됨** + 명시 4 키워드 list 만 추가 → 회귀 위험 0.

### 4.4 정량 측정

| Script | Before | After |
|--------|--------|-------|
| `{b} over {a timesm}` 분모 | "a timesm" italic 식별자 | **"a × m"** (× = U+00D7) ✓ |
| `rm X simZ` | "X simZ" italic 식별자 | **"X ∼ Z"** (∼ = U+223C) ✓ |
| `1TIMES10^-14` | "1TIMES10" italic 식별자 | **"1 × 10⁻¹⁴"** ✓ |
| `rmA SIMC` | "rmA SIMC" italic 식별자 | **"A ∼ C"** ✓ |

## 5. 본 환경 직접 검증 (임시 브랜치 cherry-pick test)

`pr578-cherry-test` 임시 브랜치에서 `6f3d94aa` 단독 cherry-pick:

| 단계 | 결과 |
|------|------|
| `6f3d94aa` cherry-pick | ✅ 충돌 0 |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (PR 본문 1131 + 본 환경 baseline) |
| **task576 unit tests** (6 신규) | ✅ **6/6 passed** (alpha 회귀 차단 가드 포함) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546/554` | ✅ 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |

→ **본 환경 base skew 0** (5/4 등록 PR 이지만 본질 영역이 본 사이클 v0.7.10 처리분과 0 중첩) → cherry-pick 가능.

## 6. PR #579 영역 정합

5/4 작업지시자 협업 PR 권유 댓글 후 PR #579 (@oksure) **self-close**:
- 작업지시자 의도: "양쪽 통합" (PR #579 인프라 + PR #578 sweep 결과)
- 실제 결과: @oksure 가 PR #579 close (PR #578 단독으로 영역 진행)
- @planet6897 답신 (5/4 22:23): "issue 처리시에 조금 더 주의해서 진행하겠습니다." (협업 영역 인지)

→ **PR #578 단독 cherry-pick 으로 영역 정합** + 협업 PR 권유 시도는 본 사이클 학습 영역.

## 7. PR 본문의 자기 검증 결과

| 검증 | PR 본문 결과 | 본 환경 재검증 (임시) |
|------|---------|----------|
| `cargo test --lib` | 1131 passed (+6 신규 tokenizer tests) | ✅ 1140 passed (본 환경 baseline 정합) |
| `cargo test --test svg_snapshot` | 6/6 passed | ✅ 6/6 passed |
| `cargo clippy --release --lib` | 신규 경고 0 | ✅ 0건 |
| 광범위 sweep (8 fixture 60+ 페이지) | exam_science 외 byte-identical | ⏳ 본 환경 sweep 권장 |
| exam_science page 3/4 의도된 정정 | 명시 영역 | ⏳ 본 환경 시각 판정 |
| 작업지시자 시각 판정 | (미진행) | ⏳ 본 환경 시각 판정 게이트 |

## 8. 메인테이너 정합성 평가

### 정합 영역
- ✅ **본질 cherry-pick 깨끗** — 충돌 0, base skew 0
- ✅ **결정적 검증 정합** — 1140 passed / clippy 0 / svg_snapshot 6/6
- ✅ **task576 unit tests 6 신규** — 회귀 차단 가드 (alpha/공백 분리 보존)
- ✅ **광범위 sweep + case-specific** — `feedback_essential_fix_regression_risk` 정합 (158 fixture / 563 scripts) + `feedback_hancom_compat_specific_over_general` 정합 (4 키워드 명시 list)
- ✅ **명시적 룰** — `feedback_rule_not_heuristic` 정합 (휴리스틱 미도입, 키워드 list)
- ✅ **PR #579 영역 학습** — 협업 PR 권유 시도 후 단독 영역 정합

### 우려 영역
- ⚠️ **base skew 영역 (5/4 등록 → 5/6 v0.7.10 후)** — UI MERGEABLE + 본 환경 cherry-pick test 충돌 0 확인 (저위험 영역)
- ⚠️ **작업지시자 시각 판정 게이트** — PR 본문 미진행. 본 환경 cherry-pick 후 직접 시각 판정 필수 (exam_science page 3/4 정정 영역)

## 9. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — `6f3d94aa` 단독 충돌 0
- ✅ **결정적 검증** — 1140 passed / 6 신규 tests / clippy 0
- ✅ **광범위 sweep + 명시 list** — 회귀 위험 0
- ✅ **PR #579 협업 시도 후 단독 처리 정합**
- ⏳ **시각 판정 별도 진행 필요**

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- `6f3d94aa` 단독 cherry-pick
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep
- SVG 생성 + 작업지시자 시각 판정 (★ 게이트, exam_science page 3/4 + 회귀 sweep)
- 통과 시 devel merge + push + PR close (한글 댓글)

#### 옵션 B — 추가 정정 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장.

## 9.5 옵션 A 진행 결과 (작업지시자 승인 후)

### 9.5.1 핀셋 cherry-pick

| 단계 | 결과 |
|------|------|
| `6f3d94aa` cherry-pick | ✅ 충돌 0, author Jaeook Ryu 보존 |
| local/devel commit | `fda98d2` |

### 9.5.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1140 passed** (회귀 0) |
| **task576 unit tests** | ✅ **6/6 passed** (alpha 회귀 차단 가드 포함) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| `cargo build --release` | ✅ 통과 |
| **Docker WASM 빌드** | ✅ **4,583,156 bytes** (1m 26s, v0.7.10 baseline +1,691 bytes — tokenizer.rs +53 LOC + 6 unit tests 정합) |

### 9.5.3 광범위 페이지네이션 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

### 9.5.4 SVG byte 차이 (PR 본문 100% 재현)

| Fixture | 페이지 수 | byte 차이 | 평가 |
|---|---|---|---|
| **exam_science** | 4 | **2 (page 3, 4)** | ✅ PR 본문 명시 정정 영역 |

→ page 1/2 byte-identical (회귀 0) + page 3/4 의도된 정정 (수식 토크나이저).

### 9.5.5 시각 판정 자료 (작업지시자 검증용)

| 자료 | 위치 | 비고 |
|---|---|---|
| **Before** (devel HEAD, fix 미적용) | `output/svg/pr578_before/exam_science/exam_science_00{1..4}.svg` | 4 페이지 |
| **After** (cherry-pick `fda98d2` 적용) | `output/svg/pr578_after/exam_science/exam_science_00{1..4}.svg` | 4 페이지 |
| **차이 페이지** | page 3 / page 4 | page 1/2 byte-identical |

**본질 정정 시각 판정 권위 영역**:
- **page 3** — 15번 본문/보기: `rm W simY/Z`, `rmA SIMC` 의 `~` (∼ U+223C) 출력
- **page 4** — 20번 본문/응답: `{b} over {a timesm}` 분모 `a × m` (× U+00D7), `1TIMES10^-14` → `1 × 10⁻¹⁴`, `rm X simZ` → `X ∼ Z`

**WASM 산출물**: `pkg/rhwp_bg.wasm` 4,583,156 bytes (Docker WASM 빌드 1m 26s, v0.7.10 baseline +1,691 bytes — tokenizer.rs +53 LOC + 6 unit tests 정합).

### 9.5.6 작업지시자 시각 판정 결과 — ★ 통과

작업지시자 평가:
> 웹 시각판정 통과입니다.

→ ★ **통과**.

### 9.5.7 후속 처리 완료

| 항목 | 결과 |
|---|---|
| devel merge | ✅ `72cd9fd` |
| push origin devel | ✅ |
| PR #578 close | ✅ 한글 댓글 등록 |
| Issue #576 close | ✅ closes #576 자동 처리 |
| 처리 보고서 | ✅ `mydocs/pr/archives/pr_578_report.md` |
| 검토 보고서 archives 이동 | ✅ 본 문서 (`mydocs/pr/archives/pr_578_review.md`) |
| 5/6 orders 신규 | ✅ `mydocs/orders/20260506.md` (PR #578 첫 항목) |

## 10. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 sweep (158 fixture / 563 scripts) + 6 신규 tokenizer tests
- ✅ `feedback_hancom_compat_specific_over_general` — 명시 4 키워드 list (case-specific) + alpha/over/sqrt 회귀 차단
- ✅ `feedback_rule_not_heuristic` — 명시적 키워드 list, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영
- ✅ `feedback_pdf_not_authoritative` — PR 본문 명시 (Unicode 코드포인트 검증, PDF 미사용)
- ✅ `feedback_per_task_pr_branch` — Task #576 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 + PR #579 close 영역 점검 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/4 등록, 5/6 v0.7.10 후 처리) 영역 정합

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**처리 완료** — 옵션 A 진행 + 시각 판정 ★ 통과 + devel merge `72cd9fd` + push + PR/Issue close.
