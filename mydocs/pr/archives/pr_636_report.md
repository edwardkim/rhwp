# PR #636 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #636 — Task #630 aift p4 목차 `·` 포함 라인 `(페이지 표기)` 8.67px 좌측 이탈 정정 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 9번째 사이클 PR |
| 연결 이슈 | #630 (closed) + **#635 흡수** (closed, Stage 6 영역) |
| 처리 옵션 | 옵션 A — 8 commits 단계별 cherry-pick + `b534dc9` skip |
| devel commits | `1a31c5a` Stage 1 + `e7d80fd` Stage 2 + `393d5f7` Stage 3 + `069de4e` Stage 4 + `27e7dd9` Stage 5 + `e75c974` 최종 보고서 + `cb78efc` Stage 6 + `1aa48fc` Stage 6 보강 |
| 처리 일자 | 2026-05-07 |

## 2. cherry-pick 결과

8 commits 단계별 보존 (author Jaeook Ryu + Co-Authored-By Claude Opus 4.7):

| Stage | hash | 변경 |
|-------|------|------|
| Stage 1 | `1a31c5a` | 베이스라인 측정 (164 fixture / 1614 페이지) + 수행/구현 계획서 |
| Stage 2 | `e7d80fd` | 단위 테스트 RED 확인 (test_630 신규) |
| Stage 3 | `393d5f7` | 정정 1 (`·` 측정 통일) — `is_halfwidth_punct` 에서 U+00B7 제거 |
| Stage 4 | `069de4e` | 정정 2 시도 → 회귀 발견 → 철회 (HWP5 ext[0] LEFT fallback 영역의 인코딩 의도 정합) |
| Stage 5 | `27e7dd9` | 광범위 회귀 검증 + 골든 SVG 갱신 |
| 최종 | `e75c974` | 결과 보고서 |
| Stage 6 | `cb78efc` | inline_tabs RIGHT + leader 본문 우측 끝 클램프 (Issue #635 흡수) |
| Stage 6 보강 | `1aa48fc` | WasmTextMeasurer 정합 (web canvas 영역) |

### 제외 / 충돌 처리
- `b534dc9` (orders only commit) skip — src 변경 부재
- `mydocs/orders/20260506.md` add/add 충돌 → `git checkout --ours` 본 환경 영역 보존 정합 (PR #622/#627/#632 패턴)

## 3. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1157 passed** (1156 + test_630 신규 정합) |
| `cargo test --lib test_630_middle_dot_full_width_in_registered_font` | ✅ 1/1 |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 (issue_147 + issue_267 골든 갱신) |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 |
| `cargo test --test issue_418 --release` | ✅ 1/1 |
| `cargo test --test issue_501 --release` | ✅ 1/1 |
| `cargo test --test issue_630 --release` | ✅ **1/1** (신규 회귀 차단 가드) |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,572,439 bytes** (PR #632 baseline 4,577,370 -4,931 bytes) |
| `rhwp-studio/public/{rhwp_bg.wasm, rhwp.js}` | ✅ 갱신 (vite dev server web 영역) |

## 4. 권위 영역 직접 측정 (PR 본문 100% 일치)

`samples/aift.hwp` page 4 목차 영역:

| 항목 | PR 본문 | 본 환경 측정 |
|------|---------|--------------|
| `(페이지 표기)` 모든 라인 paren_x | 605.45 (spread 0.00) | **`x="605.4533333333335"` 모든 라인** ✓ |
| `·` 포함 6 라인 영역 | 정확히 +8.67 px 우측 이동 | ✓ |
| `·` 미포함 라인 영향 | 없음 | ✓ |

PR 본문 명세와 정확 일치.

## 5. 본질 정정 영역

### 5.1 정정 1 (Stage 3, 단일 룰)
`src/renderer/layout/text_measurement.rs:859-862`:
```diff
 let is_halfwidth_punct = matches!(c,
-    '\u{2018}'..='\u{2027}' | // 구두점/기호
-    '\u{00B7}'                 // · MIDDLE DOT
+    '\u{2018}'..='\u{2027}' // 구두점/기호
 );
```

스마트 따옴표 등 (`'\u{2018}'..='\u{2027}'`) 영역 보존 — 케이스별 명시.

### 5.2 정정 2 시도 → 회귀 발견 → 철회 (Stage 4)
당초 가설: native `tab_type = ext[2]` raw u16 → LEFT fallback 영역 본질 영역.
Stage 4 결과: aift p4 23/24 라인 영역이 113 px 좌측 이탈 (≈seg_w 이중 차감).
원인: HWP5 의 `tab_extended[0]` 영역이 이미 right-tab 결과 위치 (= 우측 끝 - 한컴_seg_w) 로 저장 → LEFT fallback 영역이 인코딩 의도와 정합.

→ 정정 2 영역 철회 + 코멘트 갱신 (재발 방지).

### 5.3 정정 3 (Stage 6, Issue #635 흡수)
inline_tabs `(high_byte=2 && fill_low ≠ 0)` 케이스 신설 (3 곳):
```rust
(2, _) if fill_low != 0 => {
    let seg_w = measure_segment_from(...);
    x = (body_right - seg_w).max(x);  // 본문 우측 끝 - our_seg_w
}
```

3 곳 (Stage 6 + Stage 6 보강):
- `EmbeddedTextMeasurer::compute_char_positions` (native, SVG)
- `WasmTextMeasurer::compute_char_positions` (web canvas)
- `estimate_text_width` (공통)

케이스별 명시 가드 — LEFT raw 0/1, CENTER raw 2, RIGHT + fill=0 영역 영향 없음.

### 5.4 정량 측정 (PR 본문)

| 메트릭 | Before | Stage 5 | Stage 6 |
|--------|--------|---------|---------|
| 정렬 그룹 수 | 4 | 1 (≈600~601) | **1 (모두 605.45, spread 0.00)** |
| 8.67 px 이탈 라인 | 6 | 0 | 0 |
| `)` 끝 | 713.66 px | 713.66 px | **718.12 px (본문 우측 끝)** |
| SVG 우측 마진 | — | 1.16 mm | **−0.01 mm (PDF 0.11 mm 정합)** |

## 6. 메인테이너 시각 판정 ★ 통과

작업지시자 평가: "시각 판정 통과했습니다."

권위 영역:
- aift p4 목차 영역의 `·` 포함 6 라인 + 미포함 라인 모두 본문 우측 끝까지 정렬
- KTX p1 leader 도트 영역 — Stage 6 영역의 leader 도트 끝점 15 px 단축 영역 회귀 부재

## 7. devel 머지 + push

### 진행
1. `git cherry-pick 4cd3a98 b39c6b2 4a243b6 c49c429 f022e9c 2773d58 ef72ef0 fd51030` (8 commits)
2. `b534dc9` (orders only commit) skip — `git checkout --ours mydocs/orders/20260506.md` + `git cherry-pick --skip`
3. devel ← local/devel ff merge
4. push: `565f9b1..1aa48fc`

### 분기 처리
- 본 cherry-pick 시점 origin/devel 분기 0 — `feedback_release_sync_check` 정합

## 8. PR / Issue close

- PR #636: 한글 댓글 등록 + close (`gh pr close 636`)
- Issue #630: 한글 댓글 등록 + close (`gh issue close 630`)
- **Issue #635 흡수 close**: 한글 댓글 등록 + close (`gh issue close 635`) — Stage 6 영역에서 흡수 정정 영역

## 9. 본질 정정의 가치

### TDD 5 단계 + Stage 6 + Stage 6 보강 영역의 권위 패턴
- Stage 1: 베이스라인 측정 (164 fixture / 1614 페이지)
- Stage 2: 단위 테스트 RED 우선 (test_630 신규)
- Stage 3: 정정 1 (`·` 측정 통일) — 단일 룰
- Stage 4: 정정 2 시도 → 회귀 발견 → 철회 (학습 영역의 권위 사례)
- Stage 5: 광범위 회귀 검증 + 골든 SVG 갱신
- Stage 6: Issue #635 흡수 (RIGHT + leader 본문 우측 끝 클램프)
- Stage 6 보강: WasmTextMeasurer 정합 (작업지시자 "wasm 에서만 나타나는 현상" 피드백 반영)

### 학습 영역의 권위 사례 (Stage 4)
- 당초 가설 (정정 2): native `tab_type = ext[2]` raw u16 → LEFT fallback 본질 영역
- Stage 4 검증 결과: HWP5 `tab_extended[0]` 영역이 이미 right-tab 결과 위치로 저장 → LEFT fallback 영역이 인코딩 의도와 정합
- 결과: 정정 2 영역 철회 + 코멘트 갱신 (재발 방지)

→ **합성 데이터 단위 테스트 함정 + 통합 테스트 분리 가치 영역의 권위 사례**.

## 10. 메모리 룰 적용

- **`feedback_hancom_compat_specific_over_general` 권위 사례 강화 누적** — `is_halfwidth_punct` 에서 `U+00B7` 만 제외 (단일 룰, 케이스별 명시) + RIGHT + leader 케이스 신설 (구조적 가드, 측정 의존 없음). 본 사이클의 권위 사례 누적: PR #621 + PR #622 + PR #632 + **본 PR**
- **`feedback_image_renderer_paths_separate` 정합** — Stage 6 보강 영역의 권위 사례. Embedded (native, SVG) + Wasm (web canvas) 양쪽 경로 영역 정정
- **`feedback_essential_fix_regression_risk`** — TDD 5 단계 + 단위 테스트 RED 우선 + 통합 테스트 분리 + 광범위 sweep 패턴
- **`feedback_pdf_not_authoritative` (5/7 갱신)** — 한글 2022 PDF 비교로 잔여 1 mm 마진 발견 → Issue #635 등록 → Stage 6 흡수
- `reference_authoritative_hancom` — `pdf/aift-2022.pdf` (PR #670 영구 보존 영역) 권위 정답지 영역 비교
- `feedback_close_issue_verify_merged` — Issue #630/#635 close 시 본 PR 머지 검증 + 수동 close
- `feedback_assign_issue_before_work` — Issue #630/#635 assignee 미지정 영역
- PR #627/#668 패턴 정합 — Co-Authored-By Claude 영역 명시 영역의 본 사이클 정착 영역

## 11. 본 사이클 (5/7) PR 처리 누적 — **14건**

| # | PR | Task / Issue | 결과 |
|---|-----|--------------|------|
| 1 | PR #620 | Task #618 (Picture flip/rotation, Task #519 누락) | 시각 판정 ★ + close |
| 2 | PR #642 | Task #598 (각주 마커) | 시각 판정 ★ + close |
| 3 | PR #601 | Task #594 (복수 제목행) | 옵션 A-2 + close + Issue #652 신규 |
| 4 | PR #659 | Task #653 (ir-diff 표 속성) | 시각 판정 ★ + close |
| 5 | PR #602 | Issue #449 (rhwpDev) | close + Issue #449 reopen |
| 6 | PR #668 | Task #660 (Neumann ingest) | 첫 PR + 시각 판정 ★ + close |
| 7 | PR #609 | Task #604 (Document IR) | 11 commits 단계별 + 시각 판정 ★ + close |
| 8 | PR #670 | (이슈 미연결) 한글 2022 PDF 199 | 메모리 룰 갱신 + close |
| 9 | PR #621 | Task #617 (표 셀 padding) | 옵션 B + 시각 판정 ★ + close |
| 10 | PR #622 | Task #619 (다단 vpos-reset) | 옵션 A + web editor 시각 판정 ★ + close |
| 11 | PR #626 | (Follow-up to #599) 수식 replay | 옵션 A + PNG 시각 판정 ★ + close |
| 12 | PR #627 | Task #624 (Task #520 누락 회귀) | 옵션 A + TDD RED→GREEN + web editor 시각 판정 ★ + close |
| 13 | PR #632 | Task #631 (vpos-reset 인접 line 보존) | 옵션 B + 결정적 검증 통과 + 시각 판정 스킵 + close |
| 14 | **PR #636** | **Task #630 (`·` 8.67 px 이탈) + Issue #635 흡수** | **옵션 A + TDD 5 단계 + Stage 6 + Stage 6 보강 + 시각 판정 ★ + 2 Issue close** |

### 본 사이클의 권위 사례 누적 영역
- **`feedback_hancom_compat_specific_over_general`**: PR #621 + PR #622 + PR #632 + **PR #636** (구조적 가드 + 단일 룰 + 케이스별 명시)
- **`feedback_close_issue_verify_merged`**: PR #620 + PR #627 (PR cherry-pick base diff 점검 누락 패턴)
- **`feedback_image_renderer_paths_separate`**: **PR #636 Stage 6 보강** (Embedded + Wasm 양쪽 경로 정정 영역의 권위 사례)
- **Co-Authored-By Claude 패턴**: PR #627 + PR #636 + PR #668

본 PR 의 **TDD 5 단계 + Stage 4 회귀 발견 → 철회 + Stage 6 Issue #635 흡수 + Stage 6 보강 WASM 정합 + 본 환경 명명 규약 (m100) 정합 + 권위 영역 100% 일치 + 메인테이너 시각 판정 ★ 통과 + 2 Issue close + Co-Authored-By Claude 패턴 정합 영역 모두 정합**.
