# PR #642 검토 보고서

**PR**: [#642 Task #598: 본문 각주 마커 이동 및 삭제 (closes #598)](https://github.com/edwardkim/rhwp/pull/642)
**작성자**: @postmelee (Taegyu Lee, meleeisdeveloping@gmail.com) — **활발한 컨트리뷰터** (13 PR, MERGED 5 + CLOSED 7 + OPEN 본 PR)
**상태**: OPEN, **mergeable=CONFLICTING / mergeStateStatus=DIRTY** (PR base 28 commits 뒤 — 본 사이클 #578/#629/#611/#620 처리분 누적, mydocs/orders/20260506.md add/add 충돌)
**관련**: closes #598 (작업지시자 직접 등록 권위 영역, **크롬 확장 사용자 별점 리뷰 요청**)
**처리 결정**: ⏳ **옵션 A 진행 중 — web 환경 시각 판정 게이트 대기** (작업지시자 승인 후 합본 cherry-pick 적용 + 결정적 재검증 통과 + WASM 빌드 진행 중)
**검토 시작일**: 2026-05-07

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — Issue #598 의 5 영역 ((1) 본문 각주 마커 hit test (2) 커서 이동 단위에 각주 마커 포함 (3) Delete/Backspace 양방향 + 동일 확인창 (4) 번호 재계산 (5) Undo/Redo) 모두 본 PR 에서 정정되는가?
2. **회귀 위험** — 큰 영역 (32 파일 +2,751/-34) 이지만 본질 영역은 src 9 파일 + rhwp-studio TS 4 파일 + tests 1 파일. 텍스트 입력 / 페이지네이션 등 다른 영역 회귀 보존 정합?
3. **PR base CONFLICTING — 본 환경 cherry-pick 충돌 영역** — 본 환경 직접 점검 결과 mydocs/orders/20260506.md add/add 충돌만 발생, src 영역 충돌 0 (auto-merge 깨끗 통과)
4. **활발한 컨트리뷰터 영역** — Issue assignee 정합 + `feedback_pr_comment_tone` (차분/사실 중심) 정합. postmelee 는 4/16 PR #167 부터 본 사이클 누적 13 PR (Firefox/Chrome 확장, Safari 영역, Native bridge API, 브라우저 확장 보강 등 다양한 영역 기여)

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Task #598: 본문 각주 마커 이동 및 삭제 | 정합 (한글) |
| author | @postmelee (Taegyu Lee, meleeisdeveloping@gmail.com) — **활발한 컨트리뷰터 (13 PR, MERGED 5 + CLOSED 7 + OPEN 본 PR, 4/16~5/6 누적)** | Issue #598 assignee 정합 (작업지시자 직접 지정) |
| changedFiles | **32** / +2,751 / -34 | 매우 큰 영역 (계획서/단계별 보고서 다수 + 본질 +830 LOC) |
| 본질 변경 | src 9 파일 (+497) + rhwp-studio TS 4 파일 (+135/-2) + tests 1 파일 (+194) | 본질 +826 / -34 |
| 컨트리뷰터 fork plans/working/orders | 11 파일 +1,924 (본 환경 미도입) | hyper-waterfall 절차 충실 (계획서 / Stage 보고서 다수) |
| 본질 commits | 5 commits (`85dd982` + `025220d` + `90d0bae` + `738680a` + `7e5b763`) — 단일 task 영역의 후속 보강 패턴 | squash cherry-pick 또는 합본 patch 정합 |
| **mergeable** | CONFLICTING (UI), 본 환경 직접 점검 — orders add/add 만 충돌, src 영역 충돌 0 | 본 환경 patch 적용 깨끗 통과 |
| Issue | closes #598 (작업지시자 등록 + 컨트리뷰터 assignee) | ✅ |
| Issue assignee | @postmelee (작업지시자 직접 지정) | ✅ `feedback_assign_issue_before_work` 정합 |
| CI | 모두 SUCCESS (Build & Test / CodeQL × 3 / Canvas visual diff / WASM Build skipped) | ✅ |

## 3. PR 의 5 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `85dd982` 본문 각주 마커 이동 및 삭제 (본질) | 본질 정정 + 계획서/Stage 1~3_4 보고서 | ⭐ 합본 patch 적용 |
| `025220d` 각주 삭제 확인창과 Undo 검증 보강 | rhwp-studio e2e + Stage 4_1/4_2 보고서 | ⭐ 합본 patch 적용 |
| `90d0bae` 각주 앞 Backspace anchor 복원 보정 | helpers.rs / paragraph.rs / Stage 4_3 보고서 | ⭐ 합본 patch 적용 |
| `738680a` CI 저장 테스트 회귀 보정 | composer.rs / paragraph_layout.rs / Stage 4_4 보고서 | ⭐ 합본 patch 적용 |
| `7e5b763` PR open 전 e2e 검증 보강 | rhwp-studio e2e 추가 + Stage 4_5/4_6 보고서 | ⭐ 합본 patch 적용 |

→ **단일 task 영역의 후속 보강 패턴** — squash cherry-pick 또는 합본 patch 적용이 정합. 본 환경 검토 시 **합본 patch 방식** 사용 (orders 충돌 우회, src 영역 깨끗 적용).

## 4. 본질 변경 영역

### 4.1 결함 가설 (Issue #598 인용)

> 크롬 확장 사용자들의 별점 리뷰에서 **각주 삭제 기능** 요청이 다수 접수되었습니다. (...) 한컴 정합 UX 5 영역: (1) 본문 각주 마커 hit test → 각주 영역 커서 이동 (2) 커서 이동 단위에 각주 마커 포함 (3) Delete/Backspace 양방향 + 동일 확인창 (4) 번호 재계산 (5) Undo/Redo

→ rhwp-studio 의 본문 각주 마커 hit test 미구현 + 커서 이동 시 마커 건너뜀 + Delete/Backspace 분기 부재 + 삭제 API 부재 + 확인창 부재.

### 4.2 정정 — WASM API 신규 (`src/wasm_api.rs` +48 LOC)

```rust
// 신규 WASM bridge 메서드
pub fn hitTestBodyFootnoteMarker(...) -> Option<FootnoteHitResult> {...}
pub fn getFootnoteAtCursor(...) -> Option<FootnoteInfo> {...}
pub fn deleteFootnote(...) -> Result<(), JsValue> {...}
```

native 영역 동일 함수 (`hit_test_body_footnote_marker_native` / `get_footnote_at_cursor_native` / `delete_footnote_native`) 도 추가.

### 4.3 정정 — `footnote_ops.rs` 신규 (+176 LOC)

```rust
// 핵심 로직
pub fn delete_footnote(doc, sec, para, ctrl_idx) -> Result<DocumentEvent, ...> {
    // 1. 본문 footnote control 제거
    // 2. ctrl_data_records 동기화 (Document 의 footnote 본문 영역)
    // 3. char_offsets / char_count 보정
    // 4. 후속 각주 번호 재계산 (section 내 문서 순서대로)
    // 5. reflow / pagination / cache 무효화
}
```

### 4.4 정정 — `cursor_rect.rs` (+145/-4)

본문 각주 마커 hit test + 커서 이동 단위 영역:
- `FootnoteMarkerNode.control_index` 를 실제 `para.controls` 인덱스로 보정 (PR 본문 명시)
- `marker_pos` 왼쪽 caret + `marker_pos + 1` 오른쪽 caret 두 위치 모두 지원 (인라인 cursor unit)

### 4.5 정정 — `paragraph.rs` (+37/-11)

- `delete_text_at()` 의 UTF-16 삭제 길이 계산 보정 — Backspace 시 marker anchor 가 줄 끝으로 밀리지 않도록
- `insert_text_at()` 의 inline control 앞 삽입 조건 정정 — Undo 성격 삽입이 각주 마커 뒤로 들어가지 않도록

### 4.6 정정 — rhwp-studio TypeScript (+135/-2, 4 파일)

- `core/wasm-bridge.ts` (+20/-1): 본문 마커 hit test / cursor 위치 footnote 조회 / 각주 삭제 bridge 메서드 신규
- `core/types.ts` (+32): `FootnoteHitResult` / `FootnoteInfo` 타입 신규
- `engine/input-handler-mouse.ts` (+28): 본문 각주 마커 클릭 시 각주 편집 모드 진입
- `engine/input-handler-text.ts` (+55/-1): Delete/Fn+Delete 마커 앞 분기 + Backspace 마커 뒤 분기 + 동일 `showConfirm("각주 삭제", "각주를 삭제하시겠습니까?")` 호출 + `SnapshotCommand` 경로로 Undo 통합

→ **확인 다이얼로그 단일 컴포넌트** (`showConfirm()`) — Issue #598 명세 정합.

### 4.7 정정 — `composer.rs` / `paragraph_layout.rs` / `helpers.rs` 등 보조

- `composer.rs` (+5/-5): 각주 마커 영역 정합 보강
- `paragraph_layout.rs` (+4/-8): 각주 인라인 영역 정합 (-4 LOC)
- `helpers.rs` (+21/-1): 각주 영역 헬퍼 보강
- `model/event.rs` (+3): footnote delete event 신규
- `object_ops.rs` (+2/-2): footnote 영역 정합

### 4.8 회귀 차단 — `tests/issue_598_footnote_marker_nav.rs` 신규 (+194 LOC)

```rust
test issue_598_body_footnote_marker_can_be_found_and_deleted_from_cursor
test issue_598_body_footnote_marker_has_hit_and_cursor_unit
test issue_598_second_body_footnote_marker_has_same_cursor_unit
test issue_598_backspace_before_marker_keeps_marker_anchor_and_undo_restores_it
```

→ **4 신규 통합 테스트** 회귀 차단 가드.

### 4.9 회귀 차단 — `rhwp-studio/e2e/footnote-delete-confirm.test.mjs` 신규 (+186 LOC)

Puppeteer 기반 e2e 테스트 (Delete/Backspace 양쪽 확인창 + Ctrl+Z 복원 검증).

## 5. 본 환경 직접 검증 (임시 브랜치 `pr642-cherry-test`)

### 5.1 cherry-pick 충돌 영역

5 commits squash cherry-pick 시도 결과 **mydocs/orders/20260506.md add/add 충돌만 발생** + src/* 영역 충돌 0 (auto-merge 깨끗 통과). 본 환경 검토 단계에서는 **합본 patch 방식** (`git diff <merge-base> FETCH_HEAD -- 'src/*' 'rhwp-studio/*' 'tests/*'`) 사용으로 src 영역만 적용 — 컨트리뷰터 fork plans/working/orders 영역 제외 정합.

### 5.2 결정적 검증 (Rust)

| 단계 | 결과 |
|------|------|
| 합본 patch (`/tmp/pr642_essential.patch`, 16 파일) | ✅ `git apply --check` 통과 |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1141 passed** / 0 failed (PR 본문 1135 + 본 환경 baseline 정합) |
| **`cargo test --release --test issue_598_footnote_marker_nav`** | ✅ **4 passed** (PR 본문 명시 4 영역 모두 통과) |
| `cargo test --release --test svg_snapshot --test issue_546 --test issue_554` | ✅ 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |

### 5.3 결정적 검증 (rhwp-studio TypeScript)

| 단계 | 결과 |
|------|------|
| `npm run build` (`tsc && vite build`) | ✅ TypeScript 타입 체크 통과 + dist 빌드 (`dist/index-CKsYNtwg.js` 691,386 bytes / `rhwp_bg-DN7QfwxB.wasm` 4,587,318 bytes) |

→ **본 환경 base skew 28 commits 영향 0** — 합본 patch 16 파일 src 영역 깨끗 적용 + 결정적 검증 모두 통과.

### 5.4 Docker WASM 빌드

| 단계 | 결과 |
|------|------|
| Docker WASM 빌드 | ✅ **4,587,318 bytes** (1m 28s, PR #620 baseline 4,590,537 **-3,219 bytes** — composer.rs / paragraph_layout.rs 의 각주 영역 정합 보강 -8 LOC + 본질 신규 영역의 LLVM 최적화 합산 정합) |

## 6. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **167** (161 hwp + 6 hwpx) |
| 총 페이지 (BEFORE PR #620 baseline) | **1,687** |
| 총 페이지 (AFTER PR #642) | **1,687** |
| **fixture 별 페이지 수 차이** | **0** |

→ 본 PR 의 변경 영역이 페이지네이션 (Rust 영역) 에 영향 없음 (회귀 0).

## 7. PR 본문의 자기 검증 결과 (본 환경 재검증)

| 검증 | PR 본문 결과 | 본 환경 재검증 |
|------|---------|----------|
| `cargo test --test issue_598_footnote_marker_nav` | 4 passed | ✅ 4 passed |
| `cargo test wasm_api::tests::test_save_text_only` | 통과 | ✅ 1141 passed 안에 포함 |
| `cargo test --lib` | 1135 passed | ✅ **1141 passed** (본 환경 baseline 정합) |
| `cargo test navigable_text_len_counts_trailing_footnote_marker` | 통과 | ✅ 1141 passed 안에 포함 |
| `cargo build` | 통과 | ✅ Finished |
| `cd rhwp-studio && npm run build` | 통과 | ✅ tsc + vite build 정합 |
| `docker-compose --env-file .env.docker run --rm wasm` | 통과 | ✅ **4,587,318 bytes** |
| 새 WASM 반영 후 `cd rhwp-studio && npm run build` | 통과 | ✅ `rhwp_bg-DN7QfwxB.wasm` 4,587,318 정합 |
| Puppeteer e2e (`footnote-delete-confirm.test.mjs`) | 통과 | (본 환경 e2e 미실행, 작업지시자 web 시각 판정 게이트로 대체) |
| 작업지시자 web 시각 판정 | (미진행) | ⏳ 본 단계 (작업지시자 직접 각주 동작 테스트 권고) |

## 8. 메인테이너 정합성 평가

### 정합 영역 — 우수
- ✅ **Issue #598 5 영역 모두 본질 정합** — hit test / 커서 이동 단위 / Delete/Backspace 양방향 동일 확인창 / 번호 재계산 / Undo
- ✅ **WASM API 신규 정합** — `hitTestBodyFootnoteMarker` / `getFootnoteAtCursor` / `deleteFootnote` (+ native 영역 동일 함수) 명료한 영역 분리
- ✅ **HWP IR 표준 직접 정합** — `delete_footnote` 가 본문 footnote control + ctrl_data_records 동기화 + char_offsets/char_count 보정 + 번호 재계산 모두 정합 처리
- ✅ **확인 다이얼로그 단일 컴포넌트** — `showConfirm("각주 삭제", "각주를 삭제하시겠습니까?")` Delete/Backspace 양쪽 동일 호출 (Issue #598 명세 정합)
- ✅ **`SnapshotCommand` 경로 Undo 통합** — Ctrl+Z 복원 정합
- ✅ **회귀 차단 가드** — 4 신규 통합 테스트 (`issue_598_footnote_marker_nav.rs`) + Puppeteer e2e (`footnote-delete-confirm.test.mjs`)
- ✅ **결정적 검증 정합** — 1141 passed / clippy 0 / TypeScript build 정합 / WASM 빌드 정합 / 광범위 sweep 회귀 0
- ✅ **PR 본문 정합** — 작업 범위 충족도 표 + 주요 리뷰 포인트 + 검증 체크리스트 + e2e 검증 항목 + 컨트리뷰터 안내 대응 + 수동 검증 자료 (3 영상) 모두 명시
- ✅ **Issue assignee 정합** — 작업지시자 직접 지정 (`feedback_assign_issue_before_work` 정합)
- ✅ **활발한 컨트리뷰터 (13 PR 누적) + 한컴 권위 영역 인지** — "한컴 2010/2022 직접 실행 환경은 없어 로컬 웹서버와 e2e로 검증" + "PR에는 메인테이너가 한컴 환경에서 확인하기 쉬운 수동 판정 포인트를 남겼습니다" 명시 (`reference_authoritative_hancom` 정합)
- ✅ **컨트리뷰터 fork hyper-waterfall 절차 충실** — 계획서 (`task_m100_598.md` + `_impl.md` + `_delete_impl.md`) + Stage 1/3_1~3_4/4_1~4_6 보고서 다수 작성
- ✅ **범위 한정 명시** — "셀/글상자 내부 각주는 이번 이슈의 본문 각주 삭제 범위에 포함하지 않고 기존 동작을 유지" — 명료한 영역 분리

### 우려 영역
- ⚠️ **PR base CONFLICTING (mydocs/orders 만)** — 본 환경 직접 검증 결과 src 영역 충돌 0, mydocs/orders/20260506.md add/add 만 발생. 본 환경 처리 시 합본 patch 방식 또는 ours 보존으로 해결 가능 (저위험 영역)
- ⚠️ **작업지시자 web 시각 판정 게이트** — 본 PR 은 web editor 의 각주 삭제 UX 영역으로 SVG byte 비교 무관, **작업지시자 직접 web 환경에서 각주 동작 테스트 필수** (작업지시자 안내: "이번 PR 은 wasm 빌드후 메인테이너가 직접 각주 동작 테스트를 해야 합니다.")
- ⚠️ **셀/글상자 내부 각주 영역** — PR 본문 명시 범위 외, 별도 후속 task 후보

## 9. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — 5 commits (orders add/add 만 충돌, src 영역 충돌 0). 합본 patch 방식 또는 commit 별 cherry-pick + orders ours 보존 모두 정합
- ✅ **결정적 검증** — 1141 passed / clippy 0 / TypeScript build 정합 / 광범위 sweep 회귀 0 / WASM 4,587,318 bytes
- ✅ **Issue #598 5 영역 모두 본질 정정**
- ✅ **회귀 차단 가드** — 4 통합 테스트 + Puppeteer e2e
- ⏳ **작업지시자 web 시각 판정 별도 진행 필요** — 각주 동작 테스트 (vite dev server 또는 vite preview)

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 합본 cherry-pick + web 환경 시각 판정 (권장)
- 5 commits squash cherry-pick (또는 본질 src/rhwp-studio/tests 영역만 합본 patch 적용)
- author postmelee 보존 (5 commits 모두 동일 author)
- mydocs/orders 충돌은 ours 보존 (본 환경 5/6 orders 유지)
- 컨트리뷰터 fork plans/working 본 환경 미도입 (`pr/` 폴더 정책)
- 본 환경 결정적 재검증 (1141 passed / clippy 0 / TypeScript build / WASM 빌드)
- **작업지시자 web 환경 시각 판정** — vite dev server + 브라우저에서 (1) 본문 각주 마커 클릭 → 각주 영역 이동 (2) 좌우 화살표로 마커 통과 (3) Delete 마커 앞 → 확인창 → 삭제 (4) Backspace 마커 뒤 → 동일 확인창 → 삭제 (5) Ctrl+Z 복원 (6) 번호 재계산 (7) 텍스트 입력/이동 회귀 0 검증
- 통과 시 devel merge + push + PR close (한글 댓글)

#### 옵션 B — 추가 영역 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장.

## 10. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 페이지네이션 sweep (167 fixture / 1,687 페이지) + 1141 passed 회귀 0 + 4 통합 테스트 + Puppeteer e2e
- ✅ `feedback_hancom_compat_specific_over_general` — 본문 각주 영역만 정정, 셀/글상자 내부 각주 보존 (case-specific)
- ✅ `feedback_rule_not_heuristic` — HWP IR 표준 (footnote control + ctrl_data_records + char_offsets) 직접 사용, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 작업지시자 web 시각 판정 게이트 정합 운영
- ✅ `feedback_pdf_not_authoritative` — PDF 미사용 (web editor 영역)
- ✅ `feedback_per_task_pr_branch` — Task #598 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 활발한 컨트리뷰터 영역, 차분/사실 중심 톤 (반복 컨트리뷰터에 매번 같은 인사 부적절 영역 정합)
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 처리분 점검 후 진행
- ✅ **`feedback_assign_issue_before_work` — 본 PR 의 권위 케이스**: Issue #598 의 작업지시자 직접 assignee 지정 (@postmelee) 으로 외부 컨트리뷰터의 일차 방어선 정합. 메모리 권위 영역 강화.
- ✅ `feedback_small_batch_release_strategy` — v0.7.10 후 처리분 영역
- ✅ `reference_authoritative_hancom` — 한컴 정합 UX 가 권위 기준 명시 + 작업지시자 직접 시각 판정 영역 정합
- ✅ **활발한 컨트리뷰터 (13 PR 누적) 영역** — 한컴 권위 영역 인지 + 차분/정중한 톤 + 본질 정합 우수 (이전 PR 들에서 누적된 절차 / 톤 정합 영역 유지)

## 9.5 옵션 A 진행 결과 (작업지시자 승인 후)

### 9.5.1 합본 cherry-pick

| 단계 | 결과 |
|------|------|
| 5 commits 합본 patch (`/tmp/pr642_essential.patch`, src/* + rhwp-studio/* + tests/* 만 추출) | ✅ `git apply --check` 통과 |
| 16 파일 +959/-34 적용 | ✅ 충돌 0 (orders 영역 제외, plans/working 영역 제외) |
| local/devel commit | `17434e9` (**author postmelee 보존**, committer edward) |

→ **컨트리뷰터 fork plans/working/orders 영역 본 환경 미도입** (`pr/` 폴더 정책 정합) + **mydocs/orders/20260506.md ours 보존** (본 환경 5/6 orders 유지).

### 9.5.2 결정적 재검증 (local/devel cherry-pick 후)

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1141 passed** / 0 failed (회귀 0) |
| **`cargo test --release --test issue_598_footnote_marker_nav`** | ✅ **4 passed** (PR 본문 명시 4 영역 정합) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| **rhwp-studio `npm run build`** (`tsc && vite build`) | ✅ TypeScript 타입 체크 통과 + dist 빌드 (`index-CKsYNtwg.js` 691,386 bytes / `rhwp_bg-DN7QfwxB.wasm` 4,587,318 bytes) |
| **Docker WASM 빌드** | ✅ **4,587,318 bytes** (1m 28s, PR #620 baseline 4,590,537 **-3,219 bytes** — composer.rs / paragraph_layout.rs 영역 정합 보강 -8 LOC + 본질 신규 영역의 LLVM 최적화 합산 정합) |

### 9.5.3 광범위 페이지네이션 sweep (1차 검토 시 측정 완료)

| 통계 | 결과 |
|---|---|
| 총 fixture | **167** (161 hwp + 6 hwpx) |
| 총 페이지 (BEFORE PR #620 baseline) | **1,687** |
| 총 페이지 (AFTER PR #642) | **1,687** |
| **fixture 별 페이지 수 차이** | **0** |

→ 본 PR 의 변경 영역이 페이지네이션 (Rust 영역) 에 영향 없음 (회귀 0).

### 9.5.4 시각 판정 자료 (작업지시자 검증용 — web 환경)

본 PR 은 web editor 의 각주 삭제 UX 영역으로 SVG byte 비교 자료 무관. **메인테이너 직접 web 환경에서 각주 동작 테스트 필수** (작업지시자 안내 정합).

**시각 판정 권위 영역** (vite dev server / preview 실행 후 브라우저에서 검증):

1. **본문 각주 마커 클릭 → 각주 영역 이동** — 각주가 삽입된 본문에서 마커 클릭 시 페이지 하단 각주 영역의 해당 번호 위치로 커서 이동 ✓
2. **좌우 화살표로 마커 통과 (인라인 cursor unit)** — 본문 커서 이동 시 각주 마커가 한 칸 단위로 처리 (현재 건너뛰지 않음) ✓
3. **Delete/Fn+Delete 마커 앞 → 확인창 → 각주 삭제** — 각주 마커 바로 앞에 커서가 있을 때 Delete 누르면 "각주를 삭제하시겠습니까?" 확인창 표시 ✓
4. **Backspace 마커 뒤 → 동일 확인창 → 각주 삭제** — 각주 마커 바로 뒤에서 Backspace 누르면 **3 과 동일** 확인창 표시 (단일 컴포넌트 호출) ✓
5. **확인창 취소 → 각주 마커/본문/번호 유지** — 양방향 모두 정합 ✓
6. **확인창 확인 → 각주 마커 + 각주 영역 본문 모두 삭제** — 후속 각주 번호 자동 재정렬 ✓
7. **Ctrl+Z → 각주 복원** — 마커/본문/번호 모두 복원 (`SnapshotCommand` 경로) ✓
8. **텍스트 입력 / 개체 이동 / 표 리사이즈 Undo 회귀 0** — 본 PR 은 각주 영역만 정정, 다른 영역 보존 검증 ✓

**실행 명령** (메인테이너 환경):
```bash
cd rhwp-studio
npx vite preview --host 0.0.0.0 --port 7700
# 또는
npx vite --host 0.0.0.0 --port 7700
# 브라우저로 http://localhost:7700 접속 후 각주 삽입 → 위 8 영역 검증
```

**WASM 산출물**: `pkg/rhwp_bg.wasm` 4,587,318 bytes (Docker WASM 빌드 1m 28s, PR #620 baseline -3,219 bytes). `rhwp-studio/dist/assets/rhwp_bg-DN7QfwxB.wasm` 4,587,318 bytes (vite plugin copy, dist 산출물 정합).

### 9.5.5 다음 단계

5. ⏳ **작업지시자 web 환경 시각 판정** (★ 게이트, 위 8 영역 권위 검증) — 본 단계 대기 중
6. ⏳ 통과 시 devel merge + push + PR close (한글 댓글) + Issue #598 close (closes #598 자동 처리 미발동 시 수동)
7. ⏳ 처리 보고서 (`pr_642_report.md`) 작성 + archives 이동 + 5/7 orders 갱신

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**옵션 A 진행 — web 환경 시각 판정 게이트 대기**.
