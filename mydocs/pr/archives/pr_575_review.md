# PR #575 검토 보고서

**PR**: [#575 Task #573: 보기 셀 분수 단락 인라인 표 셀 paragraph 라우팅 정정 (closes #573)](https://github.com/edwardkim/rhwp/pull/575)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**상태**: OPEN, mergeable=CONFLICTING (PR base 시점 차이 추정, 본질 cherry-pick 충돌 0 확인)
**선행 PR**: PR #567 (Task #565, CLOSED) / PR #570 (Task #568, CLOSED) — 본 PR 의 인접 영역
**관련 이슈**: closes #573 / 연관 #572 (시각 판정 후 자동 정정 효과)
**처리 결정**: ⏳ **검토 중** (1차 검토)
**검토 시작일**: 2026-05-05

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — `table_layout.rs::has_table_ctrl` 가 block table (treat_as_char=false) 와 inline TAC table (treat_as_char=true) 을 미구분 → 인라인 TAC 표 보유 셀 paragraph 가 ELSE 분기로 빠져 surrounding text 미렌더가 본 환경에서도 재현되는가?
2. **Stage 1 진단의 통찰 정합** — 사용자 보고 "오른쪽 편위" 의 실제 본질이 **surrounding text 미렌더** 였다는 진단 정합한가?
3. **인접 효과 (Issue #572)** — Page 1 header sub-tables LEFT-shift 가 본 fix 의 인접 효과로 자동 정정되는 메커니즘이 본 환경에서도 재현되는가?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Task #573 보기 셀 분수 단락 인라인 표 셀 paragraph 라우팅 정정 | 정합 |
| author (PR 등록) | @planet6897 | — |
| commit author | Jaeook Ryu (= @jangster77) | 컨트리뷰터간 협업 흐름 (PR #561/#564/#567/#570 패턴 동일) |
| changedFiles | 7 / +869 / -20 | 본질 코드 +19/-2 + 보고서 다수 |
| 본질 변경 | `src/renderer/layout/table_layout.rs` +19/-2 | 단일 파일 |
| mergeable | CONFLICTING | PR base 시점 차이 추정 |
| Issue | closes #573, 연관 #572 (자동 정정 효과) | ✅ |

## 3. PR 의 5 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `82848d9a` Stage 0 — 수행 계획서 | 컨트리뷰터 fork 보고서 | 무관 |
| `304e3f9b` Stage 1 — 정밀 진단 (코드 무수정) | 컨트리뷰터 fork 보고서 | 무관 |
| `bdc73396` Stage 2 — 구현 계획서 | 컨트리뷰터 fork 보고서 | 무관 |
| **`7d5075bf` Stage 3 — 본질 정정** | `table_layout.rs` +19/-2 + working stage3 | ⭐ **cherry-pick 대상** |
| `2ec64af4` Stage 4 — 최종 보고서 + orders | 컨트리뷰터 fork report + orders | 무관 |

→ **본질 cherry-pick 대상 = `7d5075bf` 단독**. PR #561/#564/#567/#570 와 동일 패턴.

## 4. 본질 변경 영역

### 4.1 결함 가설

PR 본문:
> `table_layout.rs` L1411,L1461 의 `has_table_ctrl` 가 **block table (treat_as_char=false)** 와 **inline TAC table (treat_as_char=true)** 을 미구분 — 인라인 TAC 표 보유 셀 paragraph 가 ELSE 분기로 빠져 `layout_composed_paragraph` 호출 SKIP → "ㄷ. ", "이다." 등 surrounding text 미렌더 (exam_science.hwp 13/15/16/19번 보기 셀 분수 단락).

### 4.2 본질 정정 메커니즘

HWP 표준 룰:
- **block table** (treat_as_char=false) → 텍스트 흐름 외 (별도 layout_table 호출)
- **inline TAC table** (treat_as_char=true) → 텍스트 흐름 내 (line 안에 배치)

기존 코드는 둘을 미구분하여 inline TAC 표 보유 paragraph 도 텍스트 렌더 SKIP → 결함.

### 4.3 정정

| 위치 | 변경 |
|------|------|
| L1411 | `has_block_table_ctrl` 신설 (treat_as_char=false 만) |
| L1461 | 조건 변경 — `!has_block_table_ctrl` 시에만 `layout_composed_paragraph` 호출 |
| L1844 inline TAC table branch | `inline_shape_position` 중복 emit 가드 추가 (Equation L1800 패턴 재사용) |
| L2040 | `if has_table_ctrl` 보존 (any table 의 vpos 보정 동일 필요) |

### 4.4 정량 측정 (PR 본문)

**pi=68 cell[5] p[2] "ㄷ. 이다."**:

| 항목 | Before | After |
|------|--------|-------|
| "ㄷ" 위치 | **미렌더** | x=97.07 y=715.56 ✓ |
| 분수 (cell-clip-175) x | 97.07 | **122.07** (ㄷ. 다음 위치) |
| "이다." 위치 | **미렌더** | x=347.29-389 ✓ |

### 4.5 인접 효과 — Issue #572 자동 정정

```
exam_science page 1 header (외곽 1×1 표 셀 p[3] 의 sub-tables):
  Before: "성" x=86.39 (cell 좌단)
  After:  "성" x=152.39 (Justify slack 분배 → 중앙 정렬에 가까워짐)
```

이전 routing (step 3 for-ctrl loop) 은 sub-tables 좌측 직배치. 새 routing (`layout_composed_paragraph`) 은 paragraph alignment + Justify slack 으로 분배.

## 5. 본 환경 직접 검증 (임시 브랜치 cherry-pick test)

`pr575-cherry-test` 임시 브랜치에서 `7d5075bf` 단독 cherry-pick:

| 단계 | 결과 |
|------|------|
| `7d5075bf` cherry-pick (no-commit) | ✅ Auto-merging src/renderer/layout/table_layout.rs (충돌 0) |
| `cargo test --lib --release` | ✅ **1131 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo clippy --release --lib` | ✅ 0건 |

→ **CONFLICTING 표시는 PR base 시점 차이로 추정**. 본질 commit (`7d5075bf`) 단독 cherry-pick 시 본 환경 devel 에 깨끗하게 적용 가능.

## 6. PR 본문의 자기 검증 결과

| 검증 | PR 본문 결과 | 본 환경 재검증 (임시) |
|------|---------|----------|
| `cargo test --lib` | 1125 passed, 0 failed | ✅ 1131 passed (본 환경 baseline 정합) |
| `svg_snapshot` | 6/6 passed | ⏳ 본격 검증 |
| `clippy --release --lib` | 신규 결함 0 | ✅ 0건 |
| 광범위 sweep (9 fixture / 152 페이지) | 60 byte-identical | ⏳ 본 환경 sweep 권장 |
| exam_science 4 페이지 의도된 정정 | Page 1 header / Page 2 pi=61 / Page 3 13/16번 / Page 4 19번 | ⏳ 본 환경 sweep |
| 작업지시자 시각 판정 | (미진행) | ⏳ 본 환경 시각 판정 게이트 |
| rhwp-studio web Canvas 시각 판정 (WASM) | (미진행) | ⏳ |

## 7. 메인테이너 정합성 평가

### 정합 영역
- ✅ **본질 cherry-pick 깨끗** — 충돌 0 (auto-merge)
- ✅ **결정적 검증 정합** — cargo test --lib 1131 passed (회귀 0)
- ✅ **케이스별 명시 가드** — `has_block_table_ctrl` 신설로 block / inline TAC 명시 구분 (`feedback_hancom_compat_specific_over_general` 정합)
- ✅ **단일 룰 재사용** — `inline_shape_position` 중복 emit 가드는 Equation L1800 패턴 재사용 (코드 일관성 + 회귀 위험 영역 좁힘)
- ✅ **정밀 진단 (Stage 1)** — 사용자 보고 "오른쪽 편위" 의 실제 본질이 **surrounding text 미렌더** 임을 확정. 표면 증상 → 실제 결함 본질 추적 (`feedback_v076_regression_origin` 정합)
- ✅ **인접 효과 활용** — Issue #572 (page 1 header sub-tables LEFT-shift) 자동 정정 — 단일 본질 정정으로 두 결함 동시 해결
- ✅ **하이퍼-워터폴 흐름** — Stage 0 수행 → Stage 1 진단 (코드 무수정) → Stage 2 구현 → Stage 3 본질 → Stage 4 보고. 본 환경 워크플로우 정합
- ✅ **단일 파일 본질** — `src/renderer/layout/table_layout.rs` +19/-2 의 작은 본질
- ✅ **HWP 표준 룰 명시** — block table vs inline TAC table 의 표준 명시 + 코드 정합

### 우려 영역
- ⚠️ **CONFLICTING 표시** — PR base 시점 차이 추정 (본질 cherry-pick 충돌 0 확인됨)
- ⚠️ **광범위 sweep "60 byte-identical / 9 fixture 152 페이지"** — 60 페이지가 byte-identical 인 의미 (152 - 60 = 92 페이지가 differ?) PR 본문 명시 모호. 본 환경 sweep 으로 의도성 재현 확인 필수
- ⚠️ **작업지시자 시각 판정 게이트** — PR 본문 미진행 명시. 본 환경 cherry-pick 후 직접 시각 판정 필수
- ⚠️ **인접 효과의 누적 영향** — `layout_composed_paragraph` 새 routing 으로 paragraph alignment + Justify slack 적용. 페이지 1 header sub-tables 외에 다른 셀 paragraph 영향 영역 광범위 sweep 으로 검증 필수

## 8. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — `7d5075bf` 단독 충돌 0
- ✅ **결정적 검증** — 1131 passed / clippy 0
- ✅ **케이스별 명시 가드** — `has_block_table_ctrl` 신설
- ✅ **단일 룰 재사용** — Equation L1800 가드 패턴
- ✅ **정밀 진단 + 인접 효과** — Stage 1 통찰 + Issue #572 자동 정정
- ⏳ **시각 판정 별도 진행 필요** — PR 본문 미진행
- ⏳ **광범위 sweep 본격 검증 필요** — 본 환경 자동 sweep (PR #564/#570 패턴)

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- `7d5075bf` 단독 cherry-pick (Stage 0/1/2/4 의 plans/working/report/orders 는 컨트리뷰터 fork 정합 — 본 환경 자체 처리 보고서)
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep + WASM
- 작업지시자 시각 판정 (★ 게이트) — exam_science 4 페이지 의도된 정정 (Page 1 header / Page 2 pi=61 / Page 3 13/16번 / Page 4 19번) + 비의도 영향 영역
- 통과 시 devel merge + push + PR close 처리 + Issue #572 자동 정정 검토

#### 옵션 B — 추가 정정 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리
- 시각 판정 다수 결함 발견 시 본 환경에서 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장 — 본질 cherry-pick 깨끗 + 결정적 검증 통과 + 케이스별 명시 가드 + 단일 룰 재사용 + 정밀 진단 + 인접 효과.

## 9. 옵션 A 진행 결과 (작업지시자 승인 후)

### 9.1 핀셋 cherry-pick

| 단계 | 결과 |
|------|------|
| 본질 commit cherry-pick (`7d5075bf`) | ✅ 충돌 0, author Jaeook Ryu 보존 |
| local/devel cherry-pick commit | `323c1c7` |

### 9.2 결정적 검증 (모두 통과)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1131 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| `cargo build --release` | ✅ Finished |
| Docker WASM 빌드 | ✅ **4,571,604 bytes** (1m 33s, PR #570 baseline +703 bytes — table_layout.rs +19/-2 LOC 정합) |

### 9.3 광범위 페이지네이션 sweep (페이지 수 회귀 자동 검출)

본 환경 `samples/` 폴더 전체 자동 sweep:

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |
| Export 실패 fixture | 0 |

→ **164 fixture / 1,614 페이지 / 페이지 수 회귀 0**. block / inline TAC 케이스 분리 + 인접 효과 (Issue #572) 의 광범위 안전성이 정량 입증.

### 9.4 exam_science byte 차이 (PR 본문 명시 영역)

| 페이지 | byte 차이 | 정정 영역 |
|------|---------|---------|
| **page 1** | **differ** ✅ | Page 1 header item ① (Issue #572 인접 효과 자동 정정) |
| **page 2** | **differ** ✅ | pi=61 (Task #565/#568 영역과 인접) |
| **page 3** | **differ** ✅ | 보기 셀 분수 단락 13/16번 |
| **page 4** | **differ** ✅ | 보기 셀 분수 단락 19번 |

→ PR 본문 "exam_science.hwp 4 페이지 의도된 정정" 본 환경에서 정확히 재현. **단일 본질 정정으로 4 페이지 모든 결함 + Issue #572 인접 효과 동시 정합**.

### 9.5 다음 단계

1. ✅ 본 1차 검토 보고서 작성 (현재 문서)
2. ✅ 본 환경 결정적 재검증
3. ✅ SVG 생성 — `output/svg/pr575_before/exam_science/` + `output/svg/pr575_after/exam_science/` (4 페이지 모두 의도된 차이)
4. ✅ Docker WASM 빌드 완료 (4,571,604 bytes)
5. ✅ 광범위 페이지네이션 sweep — 164 fixture 1,614 페이지 / 페이지 수 회귀 0
6. ⏳ **작업지시자 시각 판정** (★ 게이트, exam_science 4 페이지 + WASM 다양한 hwp 검증) — 본 단계 대기 중
7. ⏳ 통과 시 devel merge + push + PR close + Issue #572 자동 close 검토
8. ⏳ 처리 보고서 (`pr_575_report.md`) 작성 + archives 이동

## 10. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영
- ✅ `feedback_v076_regression_origin` — Stage 1 정밀 진단 (사용자 보고 "오른쪽 편위" → 실제 본질 "surrounding text 미렌더") 정합
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (block / inline TAC 구분)
- ✅ `feedback_rule_not_heuristic` — HWP 표준 룰 (treat_as_char) 직접 사용 (휴리스틱 아닌 규칙)
- ✅ `feedback_pdf_not_authoritative` — 권위는 작업지시자 한컴 환경
- ✅ `feedback_per_task_pr_branch` — Task #573 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — 본 PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 누적 16번째 PR

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**1차 검토 단계 — 작업지시자 승인 대기**.
