# PR #575 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ 통과 + 인접 효과 (Issue #572 자동 정정)

**PR**: [#575 Task #573: 보기 셀 분수 단락 인라인 표 셀 paragraph 라우팅 정정 (closes #573)](https://github.com/edwardkim/rhwp/pull/575)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**처리 결정**: ✅ **핀셋 cherry-pick 머지 + push + PR close + Issue #572/#573 close**
**처리일**: 2026-05-05

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (`7d5075bf` 단독) + devel merge + push + PR close |
| 시각 판정 | ★ **통과** (메인테이너 시각 판정) |
| Devel merge commit | `cb8d0a8` |
| Cherry-pick 충돌 | 0 건 (PR mergeable=CONFLICTING 표시는 PR base 시점 차이) |
| Author 보존 | ✅ Jaeook Ryu (@jangster77) 보존 |
| Issue #573 | CLOSED (closes #573, 수동 close + 안내 댓글) |
| Issue #572 | **CLOSED (인접 효과 자동 정정)** + 안내 댓글 |
| 광범위 페이지네이션 sweep | 164 fixture / 1,614 페이지 / 페이지 수 회귀 0 |

## 2. 본질 결함 (PR 진단)

### 2.1 결함 가설

`table_layout.rs::has_table_ctrl` 가 block table (treat_as_char=false) 와 inline TAC table (treat_as_char=true) 을 미구분 — 인라인 TAC 표 보유 셀 paragraph 가 ELSE 분기로 빠져 `layout_composed_paragraph` 호출 SKIP → "ㄷ. ", "이다." 등 surrounding text 미렌더 (exam_science.hwp 13/15/16/19번 보기 셀 분수 단락).

### 2.2 Stage 1 정밀 진단의 통찰

사용자 보고 "오른쪽 편위" 의 실제 본질이 **surrounding text 미렌더** 임을 확정 (분수가 cell 좌단에 단독 노출되어 우측 편위로 인지). 표면 증상 → 실제 결함 본질 추적 (`feedback_v076_regression_origin` 정합).

### 2.3 본질 정정 메커니즘

HWP 표준 룰:
- **block table** (treat_as_char=false) → 텍스트 흐름 외 (별도 layout_table 호출)
- **inline TAC table** (treat_as_char=true) → 텍스트 흐름 내 (line 안에 배치)

기존 코드는 둘을 미구분하여 inline TAC 표 보유 paragraph 도 텍스트 렌더 SKIP → 결함.

본 fix 후:
- `!has_block_table_ctrl` → `layout_composed_paragraph` 호출 → surrounding text + inline 수식 + inline TAC 표 모두 `run_tacs` 경로에서 정상 배치
- 인라인 TAC 표는 `paragraph_layout.rs` L1888-1903 에서 layout_table + `set_inline_shape_position` 등록
- `table_layout` step 3 의 inline TAC table branch 는 `inline_shape_position` 확인 후 중복 emit 방지

## 3. 인접 효과 — Issue #572 자동 정정

PR #575 의 본질 정정이 Issue #572 (Page 1 header sub-tables LEFT-shift) 도 자동 해결:

```
exam_science page 1 header (외곽 1×1 표 셀 p[3] 의 sub-tables):
  Before: "성" x=86.39 (cell 좌단)
  After:  "성" x=152.39 (Justify slack 분배 → 중앙 정렬에 가까워짐)
```

이전 routing (step 3 for-ctrl loop) 은 sub-tables 좌측 직배치. 새 routing (`layout_composed_paragraph`) 은 paragraph alignment + Justify slack 으로 분배.

→ **단일 본질 정정으로 Task #573 + Issue #572 두 결함 동시 해결**.

## 4. 정량 측정 (PR 본문)

**pi=68 cell[5] p[2] "ㄷ. 이다."**:

| 항목 | Before | After |
|------|--------|-------|
| "ㄷ" 위치 | **미렌더** | x=97.07 y=715.56 ✓ |
| 분수 (cell-clip-175) x | 97.07 | **122.07** (ㄷ. 다음 위치) |
| "이다." 위치 | **미렌더** | x=347.29-389 ✓ |

## 5. PR 의 5 commits 분석 (cherry-pick 대상 식별)

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `82848d9a` Stage 0 — 수행 계획서 | 컨트리뷰터 fork 보고서 | 무관 |
| `304e3f9b` Stage 1 — 정밀 진단 (코드 무수정) | 컨트리뷰터 fork 보고서 | 무관 |
| `bdc73396` Stage 2 — 구현 계획서 | 컨트리뷰터 fork 보고서 | 무관 |
| **`7d5075bf` Stage 3 — 본질 정정** | `table_layout.rs` +19/-2 + working stage3 | ⭐ cherry-pick |
| `2ec64af4` Stage 4 — 최종 보고서 + orders | 컨트리뷰터 fork report + orders | 무관 |

→ 본질 1 commit 만 cherry-pick. PR #561/#564/#567/#570 와 동일 패턴.

## 6. cherry-pick 진행

### 6.1 대상 commit (1개, 충돌 0)

```
323c1c7 Task #573 Stage 3: table_layout.rs 인라인 TAC 표 셀 paragraph 라우팅 정정
```

`Jaeook Ryu <jaeook.ryu@gmail.com>` author 보존.

### 6.2 변경 영역

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | +19 / -2 (has_block_table_ctrl 신설 + L1461 조건 정정 + L1844 inline_shape_position 중복 가드 + L2040 보존) |
| `mydocs/working/task_m100_573_stage3.md` | +176 (Stage 3 작업 보고서) |

### 6.3 핵심 변경

| 위치 | 변경 |
|------|------|
| L1411 | `has_block_table_ctrl` 신설 (treat_as_char=false 만) |
| L1461 | 조건 변경 — `!has_block_table_ctrl` 시에만 `layout_composed_paragraph` 호출 |
| L1844 inline TAC table branch | `inline_shape_position` 중복 emit 가드 추가 (Equation L1800 패턴 재사용) |
| L2040 | `if has_table_ctrl` 보존 (any table 의 vpos 보정 동일 필요) |

## 7. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1131 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,571,604 bytes** (1m 33s, PR #570 baseline +703 bytes — table_layout.rs +19/-2 LOC 정합) |

## 8. 광범위 페이지네이션 회귀 sweep

본 환경 `samples/` 폴더 전체 자동 sweep:

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |
| Export 실패 fixture | 0 |
| 측정 도구 | `./target/release/rhwp export-svg` (60s timeout / fixture) |

→ **164 fixture / 1,614 페이지 / 페이지 수 회귀 0**. block / inline TAC 케이스 분리 + 인접 효과 (Issue #572) 의 광범위 안전성 정량 입증.

## 9. exam_science byte 차이 (PR 본문 100% 재현)

| 페이지 | byte 차이 | 정정 영역 |
|------|---------|---------|
| **page 1** | **differ** ✅ | Page 1 header (Issue #572 인접 효과 자동 정정) |
| **page 2** | **differ** ✅ | pi=61 (Task #565/#568 인접 영역) |
| **page 3** | **differ** ✅ | 보기 셀 분수 단락 13/16번 (Task #573 권위 영역) |
| **page 4** | **differ** ✅ | 보기 셀 분수 단락 19번 (Task #573 권위 영역) |

→ 단일 본질 정정으로 4 페이지 모든 결함 + Issue #572 동시 정합 회복.

## 10. 시각 판정 (★ 게이트)

### 10.1 SVG 자료 + WASM 환경

- `output/svg/pr575_before/exam_science/` (devel 기준, 4 페이지)
- `output/svg/pr575_after/exam_science/` (cherry-pick 후, 4 페이지)
- WASM: `pkg/rhwp_bg.wasm` 4,571,604 bytes (다양한 hwp 직접 검증용)

### 10.2 작업지시자 시각 판정 결과

> 메인테이너의 시각 판정 통과입니다.

→ ★ **통과**.

## 11. PR / Issue close 처리

### 11.1 PR #575 close
- 댓글 등록 (cherry-pick 결과 + 결정적 검증 + 광범위 sweep + 시각 판정 + Stage 1 정밀 진단 + 인접 효과 본질 + 선행 PR 연계 + 컨트리뷰터 협업 인정)
- close 처리

### 11.2 Issue #573 수동 close
- closes #573 키워드는 PR merge 가 아닌 close 로 자동 처리 안 됨 (PR #564/#570 와 동일 패턴)
- 수동 close + 안내 댓글

### 11.3 Issue #572 수동 close (인접 효과 자동 정정)
- PR 본문 §"인접 효과" 명시 영역 + 메인테이너 시각 판정 통과로 정정 확인
- 수동 close + 인접 효과 본질 안내 댓글 ("AsimD 의" x=86.39 → 152.39)

## 12. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ 통과)
- ✅ `feedback_v076_regression_origin` — Stage 1 정밀 진단 (사용자 보고 "오른쪽 편위" → 실제 본질 "surrounding text 미렌더") 정합. 본 사이클 최고 수준 진단 사례.
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (block / inline TAC 구분, has_block_table_ctrl 신설)
- ✅ `feedback_rule_not_heuristic` — HWP 표준 룰 (treat_as_char) 직접 사용 + Equation L1800 가드 패턴 재사용
- ✅ `feedback_pdf_not_authoritative` — 권위는 작업지시자 한컴 환경
- ✅ `feedback_per_task_pr_branch` — Task #573 단일 본질 PR 정합 (선행 #565/#568 와 인접하나 별개 분기 경로)
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터 협업 인정
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 활발한 외부 기여의 빠른 회전 (16번째 PR 처리)

## 13. 본 사이클 최고 수준 PR 사례 — 진단의 통찰

본 PR 의 처리 본질에서 가장 우수한 점:

1. **Stage 1 정밀 진단으로 표면 증상과 실제 본질의 차이 식별** — 사용자가 보고한 "오른쪽 편위" 가 사실 "surrounding text 미렌더" 였음을 확정
2. **단일 본질 정정으로 두 결함 동시 해결** — Task #573 (보기 셀 분수 단락) + Issue #572 (Page 1 header sub-tables) 자동 정정
3. **HWP 표준 룰 직접 사용** — `treat_as_char` 분기로 block / inline 명시 구분 (휴리스틱 아닌 규칙)
4. **단일 룰 재사용** — Equation L1800 의 `inline_shape_position` 중복 가드 패턴을 inline TAC table branch 에 재사용

`feedback_hancom_compat_specific_over_general` (일반화보다 케이스별 명시 가드) + `feedback_rule_not_heuristic` (휴리스틱 아닌 규칙) + `feedback_v076_regression_origin` (표면 → 본질) 의 메모리 룰 운영이 가장 일관 적용된 PR.

## 14. 본 사이클 사후 처리

- [x] PR #575 close (cherry-pick 머지 + push)
- [x] Issue #573 수동 close (안내 댓글)
- [x] Issue #572 수동 close (인접 효과 자동 정정 안내 댓글)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_575_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_575_review.md` → `mydocs/pr/archives/pr_575_review.md`)
- [ ] 5/5 orders 갱신 (PR #575 항목 추가)
