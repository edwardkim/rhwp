# PR #642 처리 보고서 — 합본 cherry-pick 머지 + 시각 판정 ★ 4 권위 영역 통과

**PR**: [#642 Task #598: 본문 각주 마커 이동 및 삭제 (closes #598)](https://github.com/edwardkim/rhwp/pull/642)
**작성자**: @postmelee (Taegyu Lee, meleeisdeveloping@gmail.com) — **활발한 컨트리뷰터** (13 PR 누적, MERGED 5 + CLOSED 7 + 본 PR, 4/16~5/6 약 3주 사이클)
**관련**: closes #598 (작업지시자 직접 등록 권위 영역, **크롬 확장 사용자 별점 리뷰 요청**)
**처리 결정**: ✅ **합본 cherry-pick 머지 + push + PR/Issue close**
**처리일**: 2026-05-07

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 합본 cherry-pick (5 commits squash, src 9 + rhwp-studio TS 4 + tests 1 + e2e 1 + 보조 1 = 16 파일 +959/-34) + devel merge + push + PR/Issue close |
| 시각 판정 | ★ **4 권위 영역 모두 통과** (작업지시자 web 환경 + `samples/footnote-01.hwp` 권위 샘플) |
| Devel merge commit | `30afaae` |
| Cherry-pick commit (local/devel) | `17434e9` (5 commits squash) |
| Cherry-pick 충돌 | mydocs/orders/20260506.md add/add 만 (ours 보존), src 영역 충돌 0 |
| Author 보존 | ✅ postmelee (meleeisdeveloping@gmail.com) 보존 |
| PR #642 close | ✅ 한글 댓글 등록 + close |
| Issue #598 close | ✅ 수동 close (closes #598 키워드는 cherry-pick merge 로 자동 처리 안 됨, 안내 댓글 등록) |
| 광범위 페이지네이션 sweep | 167 fixture / 1,687 페이지 / 회귀 0 |
| 회귀 차단 가드 | 4 통합 테스트 (`issue_598_footnote_marker_nav.rs`) + Puppeteer e2e (`footnote-delete-confirm.test.mjs`) |

## 2. 본질 결함 (Issue #598 권위 영역)

> 크롬 확장 사용자들의 별점 리뷰에서 **각주 삭제 기능** 요청이 다수 접수되었습니다. (...) 한컴 정합 UX 5 영역: (1) 본문 각주 마커 hit test → 각주 영역 커서 이동 (2) 커서 이동 단위에 각주 마커 포함 (3) Delete/Backspace 양방향 + 동일 확인창 (4) 번호 재계산 (5) Undo/Redo

→ rhwp-studio 의 본문 각주 마커 hit test 미구현 + 커서 이동 시 마커 건너뜀 + Delete/Backspace 분기 부재 + 삭제 API 부재 + 확인창 부재.

## 3. 본질 정정

### 3.1 WASM API 신규 (`src/wasm_api.rs` +48 LOC)

```rust
pub fn hitTestBodyFootnoteMarker(...) -> Option<FootnoteHitResult>
pub fn getFootnoteAtCursor(...) -> Option<FootnoteInfo>
pub fn deleteFootnote(...) -> Result<(), JsValue>
```

native 영역 동일 함수 (`hit_test_body_footnote_marker_native` / `get_footnote_at_cursor_native` / `delete_footnote_native`) 도 추가.

### 3.2 footnote_ops.rs 신규 (+176 LOC)

```rust
pub fn delete_footnote(doc, sec, para, ctrl_idx) -> Result<DocumentEvent, ...> {
    // 1. 본문 footnote control 제거
    // 2. ctrl_data_records 동기화
    // 3. char_offsets / char_count 보정
    // 4. 후속 각주 번호 재계산 (section 내 문서 순서대로)
    // 5. reflow / pagination / cache 무효화
}
```

### 3.3 cursor_rect.rs (+145/-4)

본문 각주 마커 hit test + 커서 이동 단위:
- `FootnoteMarkerNode.control_index` 를 실제 `para.controls` 인덱스로 보정
- `marker_pos` 왼쪽 caret + `marker_pos + 1` 오른쪽 caret 두 위치 모두 지원 (인라인 cursor unit)

### 3.4 paragraph.rs (+37/-11)

- `delete_text_at()` UTF-16 삭제 길이 계산 보정 (Backspace 시 marker anchor 유지)
- `insert_text_at()` inline control 앞 삽입 조건 정정 (Undo 삽입이 마커 뒤로 들어가지 않도록)

### 3.5 rhwp-studio TypeScript (+135/-2, 4 파일)

- `core/wasm-bridge.ts` (+20/-1): 본문 마커 hit test / cursor 위치 footnote 조회 / 각주 삭제 bridge 메서드
- `core/types.ts` (+32): `FootnoteHitResult` / `FootnoteInfo` 타입 신규
- `engine/input-handler-mouse.ts` (+28): 본문 각주 마커 클릭 시 각주 편집 모드 진입
- `engine/input-handler-text.ts` (+55/-1): Delete/Fn+Delete 마커 앞 + Backspace 마커 뒤 분기 + 동일 `showConfirm("각주 삭제", "각주를 삭제하시겠습니까?")` + `SnapshotCommand` Undo

### 3.6 보조 변경 — composer.rs / paragraph_layout.rs / helpers.rs / event.rs / object_ops.rs

각주 영역 정합 보강 (-8 LOC composer.rs / paragraph_layout.rs + 신규 footnote delete event + helpers 보강).

### 3.7 회귀 차단 가드

**통합 테스트** (`tests/issue_598_footnote_marker_nav.rs` 신규 +194 LOC):
```rust
test issue_598_body_footnote_marker_can_be_found_and_deleted_from_cursor
test issue_598_body_footnote_marker_has_hit_and_cursor_unit
test issue_598_second_body_footnote_marker_has_same_cursor_unit
test issue_598_backspace_before_marker_keeps_marker_anchor_and_undo_restores_it
```

**Puppeteer e2e** (`rhwp-studio/e2e/footnote-delete-confirm.test.mjs` 신규 +186 LOC):
Delete/Backspace 양쪽 확인창 + Ctrl+Z 복원 검증.

## 4. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1141 passed** / 0 failed (회귀 0, PR 본문 1135 + 본 환경 baseline 정합) |
| `cargo test --release --test issue_598_footnote_marker_nav` | ✅ **4/4 passed** (PR 본문 명시 4 영역 정합) |
| `cargo test --release --test svg_snapshot --test issue_546 --test issue_554` | ✅ 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |
| `rhwp-studio npm run build` (`tsc && vite build`) | ✅ TypeScript 타입 체크 통과 + dist 빌드 (`index-CKsYNtwg.js` 691,386 bytes / `rhwp_bg-DN7QfwxB.wasm` 4,587,318 bytes) |
| Docker WASM 빌드 | ✅ **4,587,318 bytes** (1m 28s, PR #620 baseline 4,590,537 **-3,219 bytes**) |

## 5. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|------|------|
| 총 fixture | **167** (161 hwp + 6 hwpx) |
| 총 페이지 (BEFORE) | **1,687** |
| 총 페이지 (AFTER) | **1,687** |
| **fixture 별 페이지 수 차이** | **0** |

→ 본 PR 의 변경 영역이 페이지네이션 (Rust 영역) 에 영향 없음.

## 6. WASM 정량 정합

| Baseline | bytes |
|---|---|
| PR #578 (5/6 첫 처리) | 4,583,156 |
| PR #629 (5/6 두 번째) | 4,590,307 |
| PR #611 (5/6 세 번째) | 4,590,307 (동일, rhwp-studio 영역 변경 0) |
| PR #620 (5/7 첫 처리) | 4,590,537 |
| PR #642 (5/7 두 번째, 본 PR) | **4,587,318** ← **-3,219 bytes** |

→ 각주 영역 정합 보강 + composer.rs / paragraph_layout.rs -8 LOC + LLVM 최적화 합산 정합.

## 7. 시각 판정 (★ 게이트 — web 환경, `samples/footnote-01.hwp` 권위 샘플)

작업지시자 시각 검증 결과:
> 웹에디터에서 samples/footnote-01.hwp 파일을 대상으로 메인테이너가
> - 각주 hitTest 판정 성공
> - 각주 캐럿 이동 판정 성공
> - 각주 앞에서 del 키 판정 성공
> - 각주 뒤에서 backspace 키 판정 성공

→ ★ **4 권위 영역 모두 통과**.

권위 샘플: `samples/footnote-01.hwp` (32,768 bytes, 본 환경 git tracked 영역, 시각 판정 자료 영역으로 영구 보존).

## 8. PR / Issue close 처리

### 8.1 PR #642 close
- 댓글 등록 (한글, cherry-pick 결과 + 결정적 재검증 + 광범위 sweep + PR 본문 100% 재현 + 시각 판정 ★ 4 권위 영역 통과 + 본질 평가 8가지 + 후속 영역 안내)
- close 처리

### 8.2 Issue #598
- closes #598 키워드는 cherry-pick merge 로 자동 처리 안 됨 (PR #570/#629/#611/#620 등 동일 패턴) → 수동 close + 안내 댓글
- 후속 영역 (셀/글상자 내부 각주) 별도 후속 task 후보 명시

## 9. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 sweep (167 fixture / 1,687 페이지) + 1141 passed 회귀 0 + 4 통합 테스트 + Puppeteer e2e
- ✅ `feedback_hancom_compat_specific_over_general` — 본문 각주 영역만 정정, 셀/글상자 내부 각주 보존 (case-specific)
- ✅ `feedback_rule_not_heuristic` — HWP IR 표준 (footnote control + ctrl_data_records + char_offsets) 직접 사용, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 작업지시자 web 시각 판정 게이트 정합 운영 (★ 4 권위 영역 통과)
- ✅ `feedback_pdf_not_authoritative` — PDF 미사용 (web editor 영역)
- ✅ `feedback_per_task_pr_branch` — Task #598 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 활발한 컨트리뷰터 영역, 차분/사실 중심 톤 (반복 컨트리뷰터에 매번 같은 인사 부적절 영역 정합)
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 처리분 점검 후 진행
- ✅ **`feedback_assign_issue_before_work` — 본 PR 의 권위 케이스**: Issue #598 의 작업지시자 직접 assignee 지정 (@postmelee) 으로 외부 컨트리뷰터의 일차 방어선 정합. **메모리 권위 영역 강화**.
- ✅ `feedback_small_batch_release_strategy` — v0.7.10 후 처리분 영역 (본 사이클 5/7 두 번째 PR)
- ✅ `reference_authoritative_hancom` — 한컴 정합 UX 가 권위 기준 명시 + 작업지시자 직접 시각 판정 영역 정합

## 10. 본 PR 의 본질 — v0.7.10 후 다섯 번째 처리 PR

본 PR 의 처리 본질에서 가장 우수한 점:

1. **Issue #598 5 영역 모두 본질 정정** — hit test / 커서 이동 단위 / Delete/Backspace 양방향 동일 확인창 / 번호 재계산 / Undo
2. **WASM API 신규 정합** — `hitTestBodyFootnoteMarker` / `getFootnoteAtCursor` / `deleteFootnote` (+ native 영역 동일 함수) 명료한 영역 분리
3. **HWP IR 표준 직접 정합** — `delete_footnote` 가 본문 footnote control + ctrl_data_records 동기화 + char_offsets/char_count 보정 + 번호 재계산 모두 정합 처리
4. **확인 다이얼로그 단일 컴포넌트** — `showConfirm()` Delete/Backspace 양쪽 동일 호출 (Issue #598 명세 정합)
5. **`SnapshotCommand` 경로 Undo 통합** — Ctrl+Z 복원 정합
6. **회귀 차단 가드 영구 보존** — 4 신규 통합 테스트 + Puppeteer e2e + `samples/footnote-01.hwp` 권위 샘플 영구 보존
7. **한컴 권위 영역 인지** — "한컴 2010/2022 직접 실행 환경은 없어 로컬 웹서버와 e2e로 검증" + "메인테이너가 한컴 환경에서 확인하기 쉬운 수동 판정 포인트" 명시 (`reference_authoritative_hancom` 정합)
8. **활발한 컨트리뷰터 13 PR 누적 + 단일 task 영역 후속 보강 5 commits** — 첫 PR 등록 후 4 commits 후속 보강 (CI 회귀 정정 + Backspace anchor 보정 + e2e 검증 보강) 통합 정합

**v1.0.0 진입 직전 핵심 사용성 영역 (크롬 확장 사용자 별점 리뷰 요청) 의 본질 정합으로 처리** — DTP 엔진 (`project_dtp_identity`) 의 web editor 영역 정합 강화.

## 11. 본 사이클 사후 처리

- [x] PR #642 close (cherry-pick 머지 + push + 한글 댓글)
- [x] Issue #598 close (수동 close + 안내 댓글)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_642_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_642_review.md` → `mydocs/pr/archives/pr_642_review.md`)
- [ ] 5/7 orders 갱신 (PR #642 항목 추가)

## 12. 후속 영역 (별도 task 후보)

PR 본문 명시 — 셀/글상자 내부 각주는 본 PR 범위 외, 별도 후속 task 영역으로 분리 검토 예정.
