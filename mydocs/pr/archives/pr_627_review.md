# PR #627 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #627 |
| 제목 | fix: Task #520 partial revert restore — exam_science p2 7번 글상자 ㉠ 사각형 y 회귀 정정 (closes #624) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 7번째 사이클 PR (PR #629/#620/#578/#670/#621/#622 직후) |
| base / head | `devel` ← `planet6897:pr-task624` |
| state / mergeable | OPEN / **CONFLICTING** / **DIRTY** (PR base 83 commits 뒤) |
| 변경 | 10 files, +965 / -3 (src 1곳 +9/-2 + 통합 테스트 +58 + 거버넌스 8) |
| commits | 5 (Stage 0 / 1 / 2 분석 / 2 정정 / 3) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **closes #624** (PR 본문 명시) |
| 작성일 / 갱신 | 2026-05-06 01:11 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED

### Co-Authored-By 영역
- `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>` 명시 — AI 페어프로그래밍 영역 명시 (PR #668 패턴 정합)

### 댓글 영역
- 댓글 없음

---

## 2. Issue #624 권위 영역

### 결함
`samples/exam_science.hwp` page 2 의 7번 문제 글상자 (pi=33 ci=0, 1x1 Table) 안 p[1] 단락의 ㉠ 사각형 (`Control::Shape`, `treat_as_char=true`, ls[1] 위치) 이 Line 1 영역 (y≈213.95 px) 에 잘못 그려져 본문 텍스트 "분자당 구성 …" 위에 겹침.

### 회귀 출처 (Issue 본문 + PR 본문 명시)

| 시점 | 커밋 | 상태 |
|------|------|------|
| Task #520 | `313e65d` | 정정 적용 — "exam_science p3 7번 박스: ㉠ 사각형이 [탐구 과정 및 결과] 라인을 침범하던 증상 해소" |
| Task #544 v2 Stage 3 (원저자 @planet6897) | `9dc40dd` | Task #520 fix 유지 + Task #548 추가 |
| **Task #548 cherry-pick (PR #561)** | **`3de0505`** | **부분 revert** — Task #520 fix 일부 누락 (3 라인) |

→ PR #561 cherry-pick 메인테이너 정리 단계에서 conflict resolution 영역 누락 추정.

### 회귀 발생 4 조건 분석 (PR 본문)
| 조건 | exam_science p[1] | 다른 5 multi-line + tac rect 케이스 |
|------|-------------------|------------------------------------|
| (a) cell 안 paragraph | ✓ | ✓ |
| (b) multi-line | ✓ (2) | ✓ (2~10) |
| (c) target_line > 0 (rect on ls[1]+) | ✓ | varies |
| (d) **first_vpos > 0 (paragraph[i>0])** | **✓ (1610)** | **✗ (모두 0)** |
| 회귀 가시화 | **YES** | **NO** |

→ exam_science p[1] 만이 (c) AND (d) 동시 성립 — 광범위 sweep 결과 (1/1496) 와 정확히 부합.

### Issue #624 assignee 영역
- **assignee 미지정** — 컨트리뷰터 (@planet6897) 자기 등록 영역. `feedback_assign_issue_before_work` 일차 방어선 부재 사례.

---

## 3. 본 환경 정합 상태 점검

### 본 환경 devel 의 직전 영역 (회귀 출처 영역 정합)
```
84bced9 Merge local/devel: PR #561 cherry-pick (Task #548 셀 inline TAC Shape margin + indent — @planet6897 / Jaeook Ryu 2 commits)
309cfbf Task #548 fixup: test_548 의 y 범위를 본 devel 측정값 기준으로 조정
bee0c77 Task #548: 셀 내부 paragraph 첫줄 inline TAC Shape margin_left + indent
```

본 환경 devel 영역에 PR #561 (`84bced9`) 영역 cherry-pick 머지 영역 정합 — **Task #520 정정 누락 영역 (3 라인) 회귀 영역 잔존** 영역 본 환경 직접 확인 영역.

### 본 환경 src/renderer/layout/table_layout.rs 직접 확인 영역

| 영역 | 본 환경 line | 영역 | 회귀 영역 |
|------|-------------|------|---------|
| Picture 분기 `tac_img_y` 산식 | 1625 | `seg.vertical_pos` (회귀) | ✗ `- first_vpos` 누락 |
| Shape 분기 `tac_img_y` 산식 | 1732-1733 | `seg.vertical_pos - first_vpos` (정상) | — |
| Shape `shape_area.y` | 1830 | `para_y_before_compose` (회귀) | ✗ `tac_img_y` 사용 안 함 |
| Shape `layout_cell_shape para_y` | 1834 | `para_y_before_compose` (회귀) | ✗ |

→ **PR #627 영역 진단 영역 100% 정합** — 본 환경 영역에 회귀 3건 영역 잔존.

### 본 사이클 (5/7) 의 인접 task 영역 정합
- PR #620 (Task #618) — Picture flip/rotation 회귀 (Task #519 영역 누락) — `feedback_close_issue_verify_merged` 권위 영역 강화
- **PR #627 (Task #624) — Task #520 영역 누락** — 본 PR 영역의 회귀 출처 영역 정합

→ 본 사이클의 권위 사례 영역 누적 — `feedback_close_issue_verify_merged` 권위 영역 두 번째 강화 영역 (PR #561 cherry-pick base diff 점검 누락 영역).

---

## 4. PR 의 본질 정정 영역

### 4.1 본질 정정 (3 라인, +9/-2)

`src/renderer/layout/table_layout.rs`:

1. **Picture 분기 `tac_img_y` 산식**:
   - `seg.vertical_pos` → `seg.vertical_pos - first_vpos`
   - 이중 합산 회귀 해소 (first_vpos=0 케이스 동일 동작 보장)
2. **Shape 분기 `shape_area.y`**:
   - `para_y_before_compose` → `tac_img_y`
   - target_line 기반 위치 사용 (target_line=0 케이스 동일 동작 보장)
3. **Shape 분기 `layout_cell_shape` para_y 인자**:
   - `para_y_before_compose` → `tac_img_y`

### 4.2 안전성 영역 (PR 본문 edge cases 분석)

| 케이스 | 정정 후 동작 | 사유 |
|--------|-------------|------|
| `first_vpos = 0` (cell 첫 paragraph) | 동일 동작 보장 | `seg.vpos - 0 = seg.vpos`, 산식 결과 동일 |
| `target_line = 0` (rect on ls[0]) | 동일 동작 보장 | `target_line > current_tac_line` 가드로 if 블록 미진입 |
| `line_segs.len() = 1` (single-line) | 동일 동작 보장 | `target_line = 0` 으로 위와 동일 |
| `wrap = Square / InFrontOfText / TopAndBottom` | 영향 없음 | `treat_as_char=true` 만 처리 |

### 4.3 회귀 차단 가드 영구 보존
- `src/renderer/layout/integration_tests.rs` (+58 LOC)
- `test_624_textbox_inline_shape_y_on_line2_p2_q7` 신규 — y=213.95 회귀 재현 (RED → GREEN)
- y 좌표 [230, 240] 범위 검증 영역

### 4.4 TDD 흐름 영역 (PR 본문 명시)

| Stage | commit | 영역 |
|-------|--------|------|
| 0 | `b3408fe` | 수행 계획서 + 구현 계획서 |
| 1 | `e5811c9` | TDD RED 통합 테스트 (test_624) — y=213.95 회귀 재현 |
| 2 (분석) | `c30e886` | 영향 범위 분석 + edge case 검증 + 후속 권고 |
| 2 (정정) | `242955b` | Task #520 부분 회귀 정정 (3 line) |
| 3 | `9cf9fc5` | 광범위 sweep + 최종 보고서 + orders 갱신 |

---

## 5. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr627_test`) 에서 진행:

### cherry-pick
- 5 commits 영역 — 4 commits 충돌 0 (auto-merge `table_layout.rs` 통과) + 1 commit (`9cf9fc5` Stage 3) 영역에서 `mydocs/orders/20260506.md` add/add 충돌 영역
- 충돌 처리: `git checkout --ours` 영역 본 환경 영역 보존 정합 (PR #622 영역과 동일 패턴)

### 결정적 검증 결과 (모두 통과)

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1156 passed** (1155 + test_624 신규 정합, 회귀 0) |
| `cargo test --lib --release test_624` | ✅ 1/1 (RED → GREEN) |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 (issue_617 신규 포함) |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 |
| `cargo test --test issue_418 --release` | ✅ 1/1 |
| `cargo test --test issue_501 --release` | ✅ 1/1 |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |

### 권위 영역 직접 측정 (exam_science.hwp page 2)

| 항목 | PR 본문 | 본 환경 측정 |
|------|---------|--------------|
| ㉠ rect | `y="235.413"` | `y="235.41333..."` ✓ |
| ㉠ text | `y="251.453"` | `y="251.45333..."` ✓ |

PR 본문 명세와 100% 일치 — Δ +21.47 px = ls[1].vpos − ls[0].vpos / 75 정확 일치.

### 광범위 fixture sweep (PR 본문)
- 158 fixture / 1,496 페이지
- 의도된 정정 1 페이지 (0.067%)
- **회귀 0** (1,495 페이지 byte-identical)

---

## 6. 옵션 분류

본 환경 cherry-pick simulation 결과 + 거버넌스 영역 본 환경 명명 규약 (m100) 정합 영역 + Co-Authored-By Claude 명시 영역 (PR #668 패턴 정합) 기반:

### 옵션 A — 전체 cherry-pick (5 commits 단계별 보존)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick b3408fe e5811c9 c30e886 242955b 9cf9fc5
# 5번째 commit 영역의 mydocs/orders/20260506.md add/add 충돌 영역 → ours 보존 정합
```

**장점**:
- TDD 흐름 영역 (Stage 0 → 1 → 2 분석 → 2 정정 → 3) 모두 보존
- author Jaeook Ryu (jaeook.ryu@gmail.com) 5 commits 모두 보존
- 거버넌스 산출물 영역 (`task_m100_624*`) 본 환경 명명 규약 정합 영역 — 그대로 cherry-pick
- 회귀 차단 가드 (`test_624_textbox_inline_shape_y_on_line2_p2_q7`) 영구 보존
- TDD 흐름 영역의 미세 영역 보존 (Stage 1 RED → Stage 2 정정 → GREEN 영역의 bisect 영역 정합)

**잠재 위험**:
- `mydocs/orders/20260506.md` add/add 충돌 영역 영역 — `git checkout --ours` 영역 처리 영역 정합 (PR #622 영역 패턴 정합)

### 옵션 A-2 — squash 머지 (1 단일 commit)
**진행 영역**:
```bash
git checkout local/devel
git merge --squash local/pr627
git commit --author="Jaeook Ryu <jaeook.ryu@gmail.com>" -m "fix: Task #520 partial revert restore (closes #624)"
```

**장점**:
- 단일 commit 영역 정리

**잠재 위험**:
- TDD 흐름 영역 (Stage 1 RED → Stage 2 정정 → GREEN) 영역 손실 — bisect 영역의 미세 영역 손실
- Co-Authored-By Claude 영역 (5 commits 모두 명시) 영역 — squash 시 단일 commit 영역으로 통합 영역

### 권장 영역 — 옵션 A (5 commits 단계별 보존)

**사유**:
1. **본 환경 결정적 검증 모두 통과** — cargo test 1156 (test_624 신규 정합) / svg_snapshot 7/7 / issue_546/554/418/501 통과 / clippy 0
2. **권위 영역 100% 일치** — PR 본문 측정과 본 환경 측정 정확 일치
3. **TDD 흐름 영역의 본질 영역** — Stage 1 RED → Stage 2 정정 → GREEN 영역의 권위 패턴 영역. 향후 동일 결함 영역 발생 시 bisect 가능 영역
4. **`feedback_close_issue_verify_merged` 권위 영역 강화 누적** — PR #620 (Task #519 누락) + 본 PR (Task #520 누락) = 본 사이클의 권위 사례 누적 영역
5. **거버넌스 영역 본 환경 명명 규약 정합** — `task_m100_624*` 영역 그대로 cherry-pick 영역 정합 (PR #622 영역과 동일 영역)

### 옵션 영역 요약 표

| 옵션 | 진행 가능 | Issue #624 정정 | 결정적 검증 | 거버넌스 영역 | 권장 |
|------|----------|----------------|------------|--------------|------|
| **A** (전체 5 commits) | ✅ 충돌 1 (orders ours) | ✅ TDD RED→GREEN 정합 | ✅ 1156/7/12 | task_m100_624 정합 | ⭐ |
| **A-2** (squash) | ✅ | ✅ | ✅ 동일 | 단일 commit | ❌ TDD 흐름 손실 |

---

## 7. 잠정 결정

### 권장 결정
- **옵션 A 진행** — 5 commits 단계별 cherry-pick (TDD 흐름 영역 보존 + Co-Authored-By Claude 영역 5 commits 모두 보존)
- `mydocs/orders/20260506.md` add/add 충돌 영역 → `git checkout --ours` 영역 본 환경 보존 정합 (PR #622 영역 패턴)
- 본 환경 결정적 검증 + WASM 빌드 + rhwp-studio public 갱신 + 시각 판정 ★

### 검증 영역 (옵션 A 진행 시 본 환경 직접 점검)
1. cherry-pick (5 commits) — simulation 영역 통과 영역 정합
2. `cargo test --lib --release` 1156 passed (test_624 신규 정합)
3. `cargo test --test svg_snapshot --release` 7/7
4. `cargo test --test issue_546 / issue_554 / issue_418 / issue_501` 통과
5. `cargo clippy --lib -- -D warnings` 0
6. Docker WASM 빌드 + byte 측정
7. `rhwp-studio/public/{rhwp_bg.wasm,rhwp.js}` 영역 갱신 (vite dev server web 영역)
8. **시각 판정 ★** — `samples/exam_science.hwp` page 2 의 7번 글상자 ㉠ 사각형 영역 작업지시자 직접 시각 판정 (한컴 2010/2022 편집기 또는 `pdf/exam_science-2022.pdf` 권위 영역)

---

## 8. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- `feedback_close_issue_verify_merged` — **권위 사례 강화 누적**. PR #620 (Task #519 누락) + 본 PR (Task #520 누락) = 본 사이클의 권위 사례 누적 영역. PR #561 cherry-pick base diff 점검 누락 영역의 권위 영역 사례
- `feedback_hancom_compat_specific_over_general` — 본 PR 의 정정 영역은 산식 영역 정정 (구조적 판단 영역 외) 이지만 회귀 발생 4 조건 분석 영역의 명시 영역 정합 (first_vpos > 0 AND target_line > 0 동시 성립 영역만 가시화)
- `reference_authoritative_hancom` — 한컴 2010/2022 편집기 권위 정답지 영역 비교 영역 (시각 판정 영역)
- `feedback_pdf_not_authoritative` (5/7 갱신) — 한글 2020/2022 PDF 정답지 영역 정합. `pdf/exam_science-2022.pdf` (PR #670 영구 보존 영역) 영역 정합
- `feedback_assign_issue_before_work` — Issue #624 assignee 미지정 영역
- PR #668 패턴 정합 — Co-Authored-By Claude 영역 명시 영역의 본 사이클 정착 영역

---

## 9. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_627_review.md` 작성 → 승인 요청
3. (필요 시) `pr_627_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정 ★) + 판단 → `pr_627_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (5 commits 단계별, 권장) / 옵션 A-2 (squash)
2. **시각 판정 권위 영역** — `samples/exam_science.hwp` page 2 7번 글상자 ㉠ 사각형 영역 작업지시자 직접 시각 판정 진행 가/부
   - 자료 후보: SVG 영역 (`output/svg/pr627_after/`) + web editor 영역 (vite dev server) + `pdf/exam_science-2022.pdf` 권위 영역 비교
3. **WASM 빌드 + rhwp-studio public 갱신** 가/부 (PR #621/#622 영역 패턴 정합)

결정 후 본 환경 cherry-pick + 결정적 검증 + WASM 빌드 + 시각 판정 ★ + `pr_627_report.md` 작성.
