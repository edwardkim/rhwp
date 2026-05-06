# PR #620 검토 보고서

**PR**: [#620 fix: Picture flip/rotation 누락 회귀 정정 — Task #519 정정 재적용 (closes #618)](https://github.com/edwardkim/rhwp/pull/620)
**작성자**: @planet6897 (Jaeuk Ryu) / Jaeook Ryu (commit author, jaeook.ryu@gmail.com)
**상태**: OPEN, **mergeable=MERGEABLE**, **mergeStateStatus=BEHIND** (PR base 22 commits 뒤 — 본 사이클 #578/#629/#611 처리분 누적)
**관련**: closes #618, **회귀 origin = Issue #519 (CLOSED)** — PR #527 (CLOSED) 묶음 머지 영역의 누락
**처리 결정**: ⏳ **옵션 A 진행 중 — 시각 판정 게이트 대기** (작업지시자 승인 후 cherry-pick 적용 + 결정적 재검증 통과)
**검토 시작일**: 2026-05-07

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — Task #519 의 fix (4 파일 6 ImageNode 생성 지점에 `transform: extract_shape_transform(&pic.shape_attr)` 채우기) 가 본 환경 devel 에 부재한지, 본 PR 의 fix 가 정확히 재적용되는지?
2. **회귀 origin 진단** — PR #527 (CLOSED, "Layout 리팩터링 Phase 0~2 + 옛한글 PUA + Square wrap fixes") 의 묶음 머지 `a7e43f9` 가 본 환경 devel 에 부재 → Task #519 + 다른 task (#517/#518/#520/#523) 도 동일 누락?
3. **회귀 위험** — 4 파일 +10/-4 의 작은 영역, `extract_shape_transform` 헬퍼는 `utils.rs:109` 에 그대로 살아있어 신규 단위 변환 코드 0줄 → 회귀 위험 좁음
4. **PR base skew (5/6 등록 → 본 사이클 #578/#629/#611 처리분 22 commits 뒤)** — 본 환경 cherry-pick 충돌 0?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | fix: Picture flip/rotation 누락 회귀 정정 — Task #519 정정 재적용 (closes #618) | 정합 |
| author | @planet6897 / Jaeook Ryu (jaeook.ryu@gmail.com) | 본 사이클 다수 PR 동일 컨트리뷰터 |
| changedFiles | **4** / +10 / -4 | 매우 작은 영역 |
| 본질 변경 | `layout.rs` (+1) + `paragraph_layout.rs` (+4/-1) + `picture_footnote.rs` (+3/-1) + `table_cell_content.rs` (+2/-2) | 4 파일 6 ImageNode 생성 지점 |
| **mergeable** | MERGEABLE (UI), BEHIND (PR base 22 commits 뒤) | 본 환경 cherry-pick 충돌 0 (auto-merge layout.rs / paragraph_layout.rs 깨끗 통과) |
| Issue | closes #618 (회귀 origin = #519, CLOSED) | ✅ |
| CI | 모두 SUCCESS (Build & Test / CodeQL × 3 / Canvas visual diff) | ✅ |

## 3. PR 의 1 commit 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| **`7ac87770`** fix: Picture flip/rotation 누락 회귀 정정 — Task #519 정정 재적용 (closes #618) | 4 파일 +10/-4 | ⭐ cherry-pick |

→ **단일 본질 commit**. PR #561~#629 와 동일 패턴.

## 4. 본질 변경 영역

### 4.1 결함 가설 (Issue #618 + 본 환경 직접 검증)

**본 환경 (edwardkim/rhwp devel) 에서 직접 검증**:

```bash
$ git merge-base --is-ancestor 7ead89d devel
# ❌ devel 에 부재 (Task #519 fix commit)

$ git merge-base --is-ancestor a7e43f9 devel
# ❌ devel 에 부재 (묶음 머지)

$ git branch -a --contains 7ead89d
# pr-551-review (본 환경의 PR 검토 임시 브랜치)
```

**`a7e43f9` 의 출처 분석**:
- 컨트리뷰터 (Jaeook Ryu) 의 fork 에서 만든 묶음 머지 (Task #517/#518/#519/#520/#521/#523/#528)
- **PR #527 (CLOSED 상태)** 으로 본 환경에 올라온 영역 — close 처리됨 (즉, 묶음 머지가 본 환경 devel 에 도달 못 함)
- 이후 개별 task 들 중 **PR #564 (Task #521)** + **PR #592 (Task #528)** 만 cherry-pick 으로 본 환경 devel 에 적용됨
- **Task #517 / #518 / #519 / #520 / #523 은 PR #527 close 후 본 환경 미머지 상태**

→ 본 PR 의 진단 정합성 100% 확정.

### 4.2 정정 (4 파일 6 ImageNode 생성 지점에 `transform:` 재적용)

| 파일 | 라인 | 변경 |
|------|------|------|
| `src/renderer/layout.rs` | 2851 | `transform:` 추가 (TAC Picture / col_node) |
| `src/renderer/layout/picture_footnote.rs` | 117, 328 | `transform:` 추가 + import |
| `src/renderer/layout/paragraph_layout.rs` | 1834, 2108, 2218 | `transform:` 추가 + import (TAC 인라인 3 사이트) |
| `src/renderer/layout/table_cell_content.rs` | 644 | **명시적 `ShapeTransform::default()` → `extract_shape_transform(&pic.shape_attr)`** + import |

**`extract_shape_transform` 헬퍼**: `src/renderer/layout/utils.rs:109` 에 그대로 살아있음 (현재 devel 에서 `shape_layout.rs:530` 1곳에서만 사용 중) → **신규 단위 변환 코드 0줄**.

### 4.3 정량 측정 (본 환경 BEFORE/AFTER 직접 재현)

**`samples/exam_eng.hwp` SVG export 8 페이지**:

| 페이지 | BEFORE `<g transform=>` | AFTER `<g transform=>` | byte 차이 |
|---|---|---|---|
| page 1 | 0 | 0 | identical |
| page 2 | 0 | 0 | identical |
| page 3 | 0 | 0 | identical |
| **page 4** | **0** | **1** | **differ (2,519,257 → 2,519,364, +107 bytes)** |
| page 5 | 0 | 0 | identical |
| page 6 | 0 | 0 | identical |
| page 7 | 0 | 0 | identical |
| page 8 | 0 | 0 | identical |

**page 4 transform 영역** (PR 본문 명시값 100% 일치):
```
<g transform="translate(1602.426666666667,0) scale(-1,1) translate(0,2217.373333333333) scale(1,-1)">
```

→ **PR 본문 명시값 정확 일치** (Q28 박스 종이-말림 그림 horz_flip+vert_flip 정상 적용).

## 5. 본 환경 직접 검증 (임시 브랜치 `pr620-cherry-test`)

| 단계 | 결과 |
|------|------|
| `7ac87770` cherry-pick | ✅ 4 파일 충돌 0 (auto-merge 깨끗 통과) |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --release --test issue_546 --test issue_554` | ✅ issue_546 1 + issue_554 12 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,590,537 bytes** (1m 30s, PR #611 baseline 4,590,307 +230 — 4 파일 transform: extract_shape_transform 호출 추가 정합) |

→ **본 환경 base skew 22 commits 영향 0** — 4 파일 충돌 0 + 결정적 검증 모두 통과.

## 6. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **167** (161 hwp + 6 hwpx, PR #611 의 3개 권위 샘플 추가 포함) |
| 총 페이지 (BEFORE) | **1,687** (PR #611 baseline) |
| 총 페이지 (AFTER) | **1,687** |
| **fixture 별 페이지 수 차이** | **0** |

→ Picture transform 추가가 페이지네이션에 영향 없음 (회귀 0).

## 7. PR 본문의 자기 검증 결과 (본 환경 재검증)

| 검증 | PR 본문 결과 | 본 환경 재검증 |
|------|---------|----------|
| `cargo test --lib` | 1134 passed | ✅ **1140 passed** (본 환경 baseline 정합, +6 = task576 신규 등) |
| `cargo test --test svg_snapshot` | 6/6 passed | ✅ 6/6 passed |
| `cargo clippy --lib -- -D warnings` | clean | ✅ 0건 |
| exam_eng 8페이지 광역 비교 (page 4 만 정정) | 명시 영역 | ✅ 본 환경 재현 정합 — page 4 만 +107 bytes, 7 페이지 byte-identical |
| export-svg p4 transform 래퍼 카운트 0 → 1 | 명시 영역 | ✅ 본 환경 정확 재현 |
| transform 값 (`translate(1602.42... scale(-1,1) ... scale(1,-1)`) | 명시 영역 | ✅ 본 환경 100% 일치 |
| 작업지시자 시각 판정 | (미진행) | ⏳ 본 환경 시각 판정 게이트 (Q28 박스 좌상단 curl) |

## 8. 메인테이너 정합성 평가

### 정합 영역 — 우수
- ✅ **본질 진단 100% 정확** — Task #519 fix 누락 + 묶음 머지 `a7e43f9` 미적용 본 환경 직접 확인 (PR 본문의 `git merge-base` 결과 정합)
- ✅ **회귀 origin 정확 식별** — PR #527 (CLOSED) 의 묶음 머지 영역이 본 환경에 도달 못 함을 정확히 분석
- ✅ **`extract_shape_transform` 헬퍼 재사용** — 신규 단위 변환 코드 0줄, `feedback_rule_not_heuristic` 정합
- ✅ **결정적 검증 정합** — 1140 passed / clippy 0 / svg_snapshot 6/6 / 광범위 sweep 회귀 0
- ✅ **정량 측정 정합** — BEFORE 0 → AFTER 1 transform 래퍼, transform 값 PR 본문 명시 100% 일치
- ✅ **byte 차이 정합** — page 4 만 +107 bytes (transform 래퍼 1쌍 추가 정합), 다른 7 페이지 byte-identical
- ✅ **작은 영역 본질 정정** — 4 파일 +10/-4, 회귀 위험 영역 좁음
- ✅ **회귀 방지 후속 권고** — PR 본문에 묶음 머지 `a7e43f9` 의 다른 task 누락 가능성 점검 권고 명시 (메모리 정합 영역)

### 우려 영역
- ⚠️ **묶음 머지 a7e43f9 의 다른 task 누락 영역** — Task #517 / #518 / #519 / #520 / #523 5개 task 모두 본 환경 devel 에 부재 가능성. 본 PR 은 **Task #519 만 재적용**, 다른 4개는 별도 후속 task 영역.
- ⚠️ **PR base BEHIND 22 commits** — UI MERGEABLE 표시지만 본 환경 cherry-pick 충돌 0 확인 (저위험 영역)
- ⚠️ **작업지시자 시각 판정 게이트** — exam_eng page 4 의 Q28 박스 좌상단 curl 그림 정상 위치 (PDF 정합) 시각 검증 필수

## 9. 회귀 방지 후속 영역 — 묶음 머지 a7e43f9 의 다른 task 누락 가능성

본 환경 직접 점검 결과:

| Task | Issue state | 본 환경 PR | 상태 | 누락 영역? |
|------|-------------|----------|------|---------|
| #517 | (등록 영역 불명) | PR #527 | CLOSED | **누락 가능성** |
| #518 | (등록 영역 불명) | PR #527 | CLOSED | **누락 가능성** |
| **#519** | CLOSED | PR #527 (CLOSED) → **본 PR #620 재적용** | OPEN | ⭐ 본 PR 영역 |
| #520 | CLOSED | PR #527 + PR #627 (revert restore) | OPEN | **PR #627 영역** (별도 후속) |
| #521 | CLOSED | **PR #564 cherry-pick 적용 완료** | CLOSED (devel 정합) | ✅ 정합 |
| #523 | CLOSED | PR #527 (CLOSED) | (no follow-up PR) | **누락 가능성** |
| #528 | CLOSED | **PR #592 cherry-pick 적용 완료** | CLOSED (devel 정합) | ✅ 정합 |

→ **별도 후속 영역**: Task #517 / #518 / #523 누락 가능성 + Task #520 (PR #627 의 별도 검토 영역, 본 PR 검토 후 진행 예정).

## 10. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — `7ac87770` 4 파일 충돌 0
- ✅ **결정적 검증** — 1140 passed / clippy 0 / svg_snapshot 6/6 / 광범위 sweep 회귀 0
- ✅ **정량 측정 정합** — BEFORE 0 → AFTER 1 transform 래퍼, PR 본문 명시 100% 일치
- ✅ **본질 진단 정확** — Task #519 fix 누락 본 환경 직접 확인
- ⏳ **시각 판정 별도 진행 필요** (Q28 박스 좌상단 curl 그림 정상 위치)

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- `7ac87770` 단독 cherry-pick (author Jaeook Ryu 보존)
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep 통과 확인 완료
- WASM 빌드 산출물 검증 완료
- SVG 생성 + 작업지시자 시각 판정 (★ 게이트, exam_eng page 4 Q28 박스 좌상단 curl 그림)
- 통과 시 devel merge + push + PR close (한글 댓글)

#### 옵션 B — Task #517 / #518 / #523 후속 영역 묶음 처리 권유
- 본 PR 처리 후 컨트리뷰터에게 별도 후속 task 권유 댓글 등록 — Task #517 / #518 / #523 누락 영역 재적용 PR

#### 옵션 C — close + 본 환경 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장 + 옵션 B 후속 권유 결합.

## 11. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 페이지네이션 sweep (167 fixture / 1,687 페이지) + 1140 passed 회귀 0
- ✅ `feedback_hancom_compat_specific_over_general` — 4 파일 6 ImageNode 생성 지점만 명시 정정, 다른 영역 무영향 (case-specific)
- ✅ `feedback_rule_not_heuristic` — `extract_shape_transform` 헬퍼 재사용, 신규 단위 변환 코드 0줄, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (exam_eng page 4 Q28 박스)
- ✅ `feedback_pdf_not_authoritative` — PDF 는 권위 기준 미입증이지만, 본 PR 은 IR 표준 (HWP 의 horz_flip/vert_flip 속성) 직접 검증 영역, 권위 영역 정합
- ✅ `feedback_per_task_pr_branch` — Task #618 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 처리분 + PR #527 close 영역 점검 후 진행
- ✅ `feedback_close_issue_verify_merged` — **본 PR 의 회귀 origin 자체가 이 메모리 영역의 권위 케이스**: Task #519 close 시 정정 commit 의 devel 머지 검증 미수행 → 동일 결함 재발 (Issue #618). 본 PR 은 이 패턴을 정확히 진단하고 재적용. 메모리 권위 영역 강화.
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/6 v0.7.10 후 처리) 영역 정합

## 9.5 옵션 A 진행 결과 (작업지시자 승인 후)

### 9.5.1 핀셋 cherry-pick

| 단계 | 결과 |
|------|------|
| `7ac87770` cherry-pick | ✅ 4 파일 충돌 0 (auto-merge layout.rs / paragraph_layout.rs 깨끗 통과) |
| local/devel commit | `95b228e` (**author Jaeook Ryu 보존**, committer edward) |

### 9.5.2 결정적 재검증 (local/devel cherry-pick 후)

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| `cargo clippy --release --lib` | ✅ 0건 |
| **Docker WASM 빌드** | ✅ **4,590,537 bytes** (1m 30s, PR #611 baseline 4,590,307 **+230 bytes** — 4 파일 transform: extract_shape_transform 호출 추가 정합) |

### 9.5.3 광범위 페이지네이션 sweep (1차 검토 시 측정 완료)

| 통계 | 결과 |
|---|---|
| 총 fixture | **167** (161 hwp + 6 hwpx) |
| 총 페이지 (BEFORE) | **1,687** |
| 총 페이지 (AFTER) | **1,687** |
| **fixture 별 페이지 수 차이** | **0** |

→ Picture transform 추가가 페이지네이션에 영향 없음 (회귀 0).

### 9.5.4 SVG byte 차이 + 정량 측정 (PR 본문 100% 재현, 1차 검토 시 측정 완료)

| 페이지 | BEFORE `<g transform=>` | AFTER `<g transform=>` | byte 차이 |
|---|---|---|---|
| page 1 | 0 | 0 | identical |
| page 2 | 0 | 0 | identical |
| page 3 | 0 | 0 | identical |
| **page 4** | **0** | **1** | **differ (2,519,257 → 2,519,364, +107)** |
| page 5 | 0 | 0 | identical |
| page 6 | 0 | 0 | identical |
| page 7 | 0 | 0 | identical |
| page 8 | 0 | 0 | identical |

**page 4 transform 영역** (PR 본문 명시값 100% 일치):
```
<g transform="translate(1602.426666666667,0) scale(-1,1) translate(0,2217.373333333333) scale(1,-1)">
```

### 9.5.5 시각 판정 자료 (작업지시자 검증용)

| 자료 | 위치 | 비고 |
|---|---|---|
| **Before** (devel HEAD `7f89147`, fix 미적용) | `output/svg/pr620_before/exam_eng/exam_eng_00{1..8}.svg` | 8 페이지, page 4 = 0 transform |
| **After** (cherry-pick `95b228e` 적용) | `output/svg/pr620_after/exam_eng/exam_eng_00{1..8}.svg` | 8 페이지, page 4 = **1 transform** |
| **차이 페이지** | page 4 단독 | 다른 7 페이지 byte-identical |

**시각 판정 권위 영역**:
- **page 4 — Q28 박스 종이-말림(curl) 데코레이션 그림** — `bin_id=2`, `flip=(h=true,v=true)`, `rot=0` 의 horz/vert flip 정상 적용으로 박스 좌상단 정상 위치 출력 (PDF 정합).
- 다른 페이지 / 영역은 회귀 없음 (광범위 sweep 167 fixture / 1,687 페이지 차이 0).

**WASM 산출물**: `pkg/rhwp_bg.wasm` 4,590,537 bytes (Docker WASM 빌드 1m 30s, PR #611 baseline +230 bytes — 4 파일 transform: extract_shape_transform 호출 추가 정합).

### 9.5.6 다음 단계

5. ⏳ **작업지시자 시각 판정** (★ 게이트, exam_eng page 4 Q28 박스 좌상단 curl 그림 정상 위치) — 본 단계 대기 중
6. ⏳ 통과 시 devel merge + push + PR close (한글 댓글) + Issue #618 close (closes #618 자동 처리 미발동 시 수동)
7. ⏳ 처리 보고서 (`pr_620_report.md`) 작성 + archives 이동 + 5/7 orders 신규 작성
8. ⏳ **옵션 B 후속 권유 댓글** — Task #517 / #518 / #523 누락 영역 재적용 권유 (별도 후속)

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**옵션 A 진행 — WASM 빌드 + 시각 판정 게이트 대기**.
