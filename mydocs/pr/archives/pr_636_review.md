# PR #636 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #636 |
| 제목 | Task #630: aift p4 목차 `·` 포함 라인 `(페이지 표기)` 8.67px 좌측 이탈 정정 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 9번째 사이클 PR |
| base / head | `devel` ← `planet6897:pr-task630` |
| state / mergeable | OPEN / **CONFLICTING** / **DIRTY** (PR base 95 commits 뒤) |
| 변경 | 14 files, +1,760 / -262 |
| commits | 9 (Stage 1~6 + 5 단계별 + 5/6 orders + Stage 6 보강) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **closes #630** + **Issue #635 흡수** (PR Stage 6) |
| 작성일 / 갱신 | 2026-05-06 04:29 / 05:25 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED

### Co-Authored-By 영역
- Co-Authored-By: Claude Opus 4.7 (1M context) 명시 (9 commits 모두) — PR #627/#668 패턴 정합

### 댓글 영역
- 5/6 04:46 (컨트리뷰터): Stage 6 추가 정정 + Issue #635 흡수 (작업지시자 "왼쪽으로 밀림" 피드백 반영)
- 5/6 05:25 (컨트리뷰터): Stage 6 보강 — WasmTextMeasurer 정합 (작업지시자 "wasm 에서만 나타나는 현상" 피드백 반영)

---

## 2. Issue #630 권위 영역

### 결함
`samples/aift.hwp` 4페이지 목차 영역의 `·` (U+00B7 MIDDLE DOT) 포함 라인의 `(페이지 표기)` 가 정확히 **8.67 px** (반각 1자) 좌측 이탈.

### 본질 결함
`is_halfwidth_punct` (`text_measurement.rs:859-862`) 가 `U+00B7` 영역을 강제 반각 (em/2) 처리. 한컴 저장 시점의 측정값은 전각 (em_size) 기반이므로 `tab_extended[0]` 영역이 전각 기준 산출 → 본 환경의 반각 측정과 8.67 px 차이 → right-tab 정렬 시 `·` 포함 라인이 8.67 px 좌측 이탈.

### Issue #635 (Stage 6 흡수 영역)
PR #636 Stage 6 영역에서 추가 발견 + 흡수:
- 정정 1 후에도 모든 라인이 PDF (한컴 2022) 대비 약 **1.05 mm 좌측** 위치
- 원인: in-run RIGHT 탭 + leader (fill_type ≠ 0) 영역의 본문 우측 끝 클램프 누락
- Stage 6 정정: `inline_tabs RIGHT + leader` 케이스 신설 — `x = (body_right - seg_w).max(x)`

### Issue assignee 영역
- assignee 미지정 — 컨트리뷰터 자기 등록 영역

---

## 3. 본 환경 정합 상태 점검

### 본 환경 text_measurement.rs 영역 (직접 확인)
- line 859-862: `is_halfwidth_punct` 영역 잔존 — `U+00B7` 포함 (PR 정정 영역의 본질 영역)
- line 244, 358, 617: `inline_tabs` 영역 (3 곳 — `estimate_text_width` + `EmbeddedTextMeasurer::compute_char_positions` + `WasmTextMeasurer::compute_char_positions`)
- line 1466: 기존 테스트 `compute_char_positions("가\u{00B7}나", ...)` 영역 잔존

### 본 환경 영역의 회귀 영역
PR 의 정정 영역 (4 곳, 총 +117/-17):
1. `is_halfwidth_punct` 에서 `U+00B7` 영역 제거 (단일 룰)
2. Stage 4 검증 결과 코멘트 갱신 (재발 방지)
3. `EmbeddedTextMeasurer::compute_char_positions` inline_tabs RIGHT + leader 케이스 신설 (Stage 6)
4. `WasmTextMeasurer::compute_char_positions` 동일 케이스 신설 (Stage 6 보강) — `feedback_image_renderer_paths_separate` 정합

### 본 사이클 영역의 Co-Authored-By Claude 패턴 정합
- PR #627 (Task #624): Co-Authored-By Claude Opus 4.7 영역 명시 (5 commits)
- PR #636 (Task #630): Co-Authored-By Claude Opus 4.7 영역 명시 (9 commits 모두)
- PR #668 (Task #660): Co-Authored-By Claude Opus 4.7 영역 명시
→ 본 사이클의 Co-Authored-By Claude 패턴 정합 누적 영역

---

## 4. PR 의 본질 정정 영역

### 4.1 정정 1 (Stage 3, 단일 룰)
`src/renderer/layout/text_measurement.rs:859-862`:
```diff
 let is_halfwidth_punct = matches!(c,
-    '\u{2018}'..='\u{2027}' | // 구두점/기호
-    '\u{00B7}'                 // · MIDDLE DOT
+    '\u{2018}'..='\u{2027}' // 구두점/기호
 );
```

스마트 따옴표 등 (`'\u{2018}'..='\u{2027}'`) 영역은 보존 — 케이스별 명시.

### 4.2 정정 2 시도 → 회귀 발견 → 철회 (Stage 4)
당초 가설: native `tab_type = ext[2]` raw u16 → LEFT fallback 영역 본질 영역.
**Stage 4 결과**: aift p4 23/24 라인 영역이 113 px 좌측 이탈 (≈seg_w 이중 차감).
**원인**: HWP5 의 `tab_extended[0]` 가 이미 right-tab 결과 위치 (= 우측 끝 - 한컴_seg_w) 로 저장 → LEFT fallback 영역이 인코딩 의도와 정합.

→ 정정 2 영역 철회 + 코멘트 갱신 (재발 방지).

### 4.3 정정 3 (Stage 6, Issue #635 흡수)
inline_tabs `(high_byte=2 && fill_low ≠ 0)` 케이스 신설 (3 곳):
- `EmbeddedTextMeasurer::compute_char_positions` (native)
- `WasmTextMeasurer::compute_char_positions` (WASM)
- `estimate_text_width` (공통)

```rust
(2, _) if fill_low != 0 => {
    let seg_w = measure_segment_from(...);
    x = (body_right - seg_w).max(x);  // 본문 우측 끝 - our_seg_w
}
```

케이스별 명시 가드 — LEFT raw 0/1, CENTER raw 2, RIGHT + fill=0 영역 영향 없음.

### 4.4 정량 측정 (PR 본문)

| 메트릭 | Before | After |
|--------|--------|-------|
| 정렬 그룹 수 | 4 (592 / 600 / 601 / 293) | 1 (≈600~601) → **모두 605.45 (Stage 6)** |
| 8.67 px 이탈 라인 | 6 | **0** |
| spread (본 라인) | 9.08 px | **1.13 px → 0.00 (Stage 6)** |
| `)` 끝 | 713.66 px | **718.12 px (본문 우측 끝)** |
| SVG 우측 마진 | 1.16 mm | **−0.01 mm (PDF 0.11 mm 정합)** |

---

## 5. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr636_test`) 에서 진행:

### cherry-pick
- 9 commits 영역 — 8 commits 충돌 0 + 1 commit (`b534dc9` orders 갱신) 영역에서 충돌 + skip
- `mydocs/orders/20260506.md` add/add 충돌 → `git checkout --ours` 본 환경 영역 보존 정합 (PR #622/#627/#632 패턴)
- `b534dc9` 영역은 src 변경 부재 (orders only) → skip 정합

### 결정적 검증 결과 (모두 통과)

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1157 passed** (1156 + test_630 신규 정합) |
| `cargo test --lib test_630_middle_dot_full_width_in_registered_font` | ✅ 1/1 |
| `cargo test --lib test_624` (PR #627 영역) | ✅ 1/1 (회귀 0) |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 (issue_147 골든 갱신 + issue_267 KTX 갱신 포함) |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 |
| `cargo test --test issue_418 --release` | ✅ 1/1 |
| `cargo test --test issue_501 --release` | ✅ 1/1 |
| `cargo test --test issue_630 --release` | ✅ **1/1** (신규 회귀 차단 가드) |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |

### 골든 SVG 영역 갱신
- `tests/golden_svg/issue-147/aift-page3.svg` 갱신 (+48 bytes 영역, Stage 5 영역)
- `tests/golden_svg/issue-267/ktx-toc-page.svg` 갱신 (Stage 6 영역) — KTX leader 도트 끝점 15 px 단축

---

## 6. 옵션 분류

본 환경 cherry-pick simulation 결과 + 본 PR 영역의 본질 정합 영역 + Co-Authored-By Claude 영역 정합 영역 기반:

### 옵션 A — 전체 cherry-pick (8 commits 단계별 + skip 1)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick 4cd3a98 b39c6b2 4a243b6 c49c429 f022e9c 2773d58 ef72ef0 fd51030
# b534dc9 (orders only commit) skip
# 첫 cherry-pick 영역의 orders 충돌 → ours 보존
```

**장점**:
- TDD 5 단계 + Stage 6 + Stage 6 보강 모두 보존
- author Jaeook Ryu + Co-Authored-By Claude 8 commits 모두 보존
- 거버넌스 산출물 영역 (`task_m100_630*`) 본 환경 명명 규약 정합 영역
- WASM 영역 (Stage 6 보강) 영역의 본 사이클 패턴 정합 (PR #621/#622/#627 영역의 web editor 영역 영역)

**잠재 위험**:
- `b534dc9` orders only commit 영역 skip 영역 정합

### 옵션 A-2 — squash 머지 (1 단일 commit)
**진행 영역**:
```bash
git checkout local/devel
git merge --squash local/pr636
git commit --author="Jaeook Ryu <jaeook.ryu@gmail.com>" -m "Task #630: ..."
```

**잠재 위험**:
- TDD 흐름 영역 (Stage 1~6) 손실 — Stage 4 영역의 회귀 발견 → 철회 영역의 학습 영역 손실
- Stage 6 의 Issue #635 흡수 영역의 본질 영역 손실

### 권장 영역 — 옵션 A (8 commits 단계별 + skip 1)

**사유**:
1. **본 환경 결정적 검증 모두 통과** — cargo test 1157 (test_630 신규 정합) / svg_snapshot 7/7 / issue_546/554/418/501/630 모두 통과 / clippy 0
2. **TDD 5 단계 + Stage 6 + Stage 6 보강 영역의 권위 패턴 정합** — Stage 4 영역의 회귀 발견 → 철회 영역 + Stage 6 Issue #635 흡수 영역 + Stage 6 보강 WASM 영역 정합
3. **`feedback_image_renderer_paths_separate` 정합** — Embedded (native, SVG) + Wasm (web canvas) 양쪽 경로 영역 정정 영역 (Stage 6 보강 영역)
4. **`feedback_hancom_compat_specific_over_general` 권위 사례 강화 누적** — `is_halfwidth_punct` 에서 `U+00B7` 만 제외 (단일 룰, 케이스별 명시) + RIGHT + leader 케이스 신설 (구조적 가드)
5. **Co-Authored-By Claude 패턴 정합** — 본 사이클의 PR #627/#668 영역과 정합 패턴 누적
6. **거버넌스 영역 본 환경 명명 규약 정합** — `task_m100_630*` 영역 그대로 cherry-pick

### 옵션 영역 요약 표

| 옵션 | 진행 가능 | Issue #630 정정 | 결정적 검증 | TDD 흐름 | 권장 |
|------|----------|----------------|------------|----------|------|
| **A** (8 commits + skip 1) | ✅ 충돌 1 (orders ours) | ✅ test_630 RED→GREEN | ✅ 1157/7/12 | Stage 1~6 보존 | ⭐ |
| **A-2** (squash) | ✅ | ✅ 동일 | ✅ 동일 | 단일 commit | ❌ TDD 흐름 손실 |

---

## 7. 잠정 결정

### 권장 결정
- **옵션 A 진행** — 8 commits 단계별 cherry-pick + `b534dc9` orders only commit skip
- 본 환경 결정적 검증 + WASM 빌드 + rhwp-studio public 갱신 + 시각 판정 ★

### 검증 영역 (옵션 A 진행 시 본 환경 직접 점검)
1. cherry-pick (8 commits) — orders 충돌 ours 보존 + b534dc9 skip
2. `cargo test --lib --release` 1157 passed (test_630 신규 정합)
3. `cargo test --test svg_snapshot --release` 7/7
4. `cargo test --test issue_546 / issue_554 / issue_418 / issue_501 / issue_630` 통과
5. `cargo clippy --lib -- -D warnings` 0
6. Docker WASM 빌드 + byte 측정
7. `rhwp-studio/public/{rhwp_bg.wasm, rhwp.js}` 영역 갱신 (vite dev server web 영역)
8. **시각 판정 ★** — `samples/aift.hwp` page 4 목차 영역의 `·` 포함 라인 (1-1 / 3-1 / 3-4 / 4-1 / 7-2 / 8-1) `(페이지 표기)` 위치 영역 + KTX p1 leader 도트 영역 영역 시각 판정 (한컴 2022 PDF 권위 정답지 비교 — `pdf/aift-2022.pdf` PR #670 영구 보존 영역)

---

## 8. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- **`feedback_hancom_compat_specific_over_general` 권위 사례 강화 누적** — `is_halfwidth_punct` 에서 `U+00B7` 만 제외 (단일 룰, 케이스별 명시) + RIGHT + leader 케이스 신설 (구조적 가드, 측정 의존 없음)
- **`feedback_image_renderer_paths_separate` 정합** — Embedded (native) + Wasm (web canvas) 양쪽 경로 영역 정정. Stage 6 보강 영역의 권위 사례
- **`feedback_essential_fix_regression_risk`** — TDD 5 단계 + 단위 테스트 RED 우선 + 통합 테스트 분리 + 광범위 sweep 패턴
- **`feedback_pdf_not_authoritative` (5/7 갱신)** — PDF 보조 ref. 한글 2022 PDF 비교로 잔여 1 mm 마진 발견 → Issue #635 등록 → Stage 6 흡수
- `reference_authoritative_hancom` — `pdf/aift-2022.pdf` (PR #670 영구 보존 영역) 권위 정답지 영역 비교 영역
- `feedback_close_issue_verify_merged` — Issue #630 close 시 본 PR 머지 검증 + 수동 close
- `feedback_assign_issue_before_work` — Issue #630/#635 assignee 미지정 영역
- PR #627/#668 패턴 정합 — Co-Authored-By Claude 영역 명시 영역

---

## 9. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_636_review.md` 작성 → 승인 요청
3. (필요 시) `pr_636_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정 ★) + 판단 → `pr_636_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (8 commits + skip 1, 권장) / 옵션 A-2 (squash)
2. **시각 판정 권위 영역** — `samples/aift.hwp` page 4 목차 영역의 `·` 포함 라인 6개 영역 + KTX p1 leader 도트 영역 작업지시자 직접 시각 판정 진행 가/부
3. **WASM 빌드 + rhwp-studio public 갱신** 가/부 (Stage 6 보강 WASM 영역 정합 영역으로 web editor 영역 시각 판정 영역 정합 영역)
4. **Issue #635 영역 close** 가/부 (PR Stage 6 영역에서 흡수 영역으로 자동 close 영역)

결정 후 본 환경 cherry-pick + 결정적 검증 + WASM 빌드 + 시각 판정 ★ + `pr_636_report.md` 작성.
