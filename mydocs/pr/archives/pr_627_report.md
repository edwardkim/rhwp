# PR #627 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #627 — fix: Task #520 partial revert restore — exam_science p2 7번 글상자 ㉠ 사각형 y 회귀 정정 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 7번째 사이클 PR (PR #629/#620/#578/#670/#621/#622 직후) |
| 연결 이슈 | #624 (closed) |
| 처리 옵션 | 옵션 A — 5 commits 단계별 cherry-pick |
| devel commits | `fc8675c` (Stage 0) + `dc94a55` (Stage 1 RED) + `e042a15` (Stage 2 분석) + `f8085c8` (Stage 2 정정) + `c55ee3c` (Stage 3) |
| 처리 일자 | 2026-05-07 |

## 2. cherry-pick 결과

5 commits 단계별 보존 (author Jaeook Ryu, Co-Authored-By Claude Opus 4.7, committer edward):

| Stage | hash | 변경 |
|-------|------|------|
| Stage 0 | `fc8675c` | 수행 계획서 + 구현 계획서 |
| Stage 1 (TDD RED) | `dc94a55` | `test_624_textbox_inline_shape_y_on_line2_p2_q7` 신규 — y=213.95 회귀 재현 |
| Stage 2 분석 | `e042a15` | 영향 범위 + edge case + 후속 진단 분석 |
| Stage 2 정정 | `f8085c8` | 본질 정정 (3 line, +9/-2) |
| Stage 3 | `c55ee3c` | 광범위 sweep + 최종 보고서 + orders 갱신 |

### 충돌 처리
- `mydocs/orders/20260506.md` add/add 충돌 영역 발생 (Stage 3 commit) — 본 환경 영역 보존 (`git checkout --ours`) 정합 (PR #622 영역 패턴 정합)
- `src/renderer/layout/table_layout.rs` 영역 auto-merge 통과 (Stage 2 정정 영역)

## 3. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1156 passed** (1155 + test_624 신규 정합, 회귀 0) |
| `cargo test --lib --release test_624` | ✅ 1/1 (**RED → GREEN**) |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 |
| `cargo test --test issue_418 --release` | ✅ 1/1 |
| `cargo test --test issue_501 --release` | ✅ 1/1 |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,578,751 bytes** (PR #626 baseline 4,578,641 +110 bytes) |
| `rhwp-studio/public/{rhwp_bg.wasm, rhwp.js}` | ✅ 갱신 (vite dev server web 영역) |

## 4. 권위 영역 직접 측정 (PR 본문 100% 일치)

| 항목 | PR 본문 | 본 환경 측정 |
|------|---------|--------------|
| ㉠ rect | `y="235.413"` | `y="235.41333..."` ✓ |
| ㉠ text | `y="251.453"` | `y="251.45333..."` ✓ |
| Δ | +21.47 px | +21.47 px ✓ (ls[1].vpos − ls[0].vpos / 75) |

## 5. 본질 정정 영역

### 5.1 회귀 출처 영역
PR #561 cherry-pick (`3de0505`) 영역에서 Task #520 정정 누락 (3 라인) — 메인테이너 conflict resolution 영역의 누락 영역.

### 5.2 본질 정정 (3 라인, +9/-2)

`src/renderer/layout/table_layout.rs`:

1. **Picture 분기 `tac_img_y` 산식** (line 1625):
   - 회귀: `seg.vertical_pos`
   - 정정: `seg.vertical_pos - first_vpos`
   - 효과: 이중 합산 회귀 해소
2. **Shape 분기 `shape_area.y`** (line 1830):
   - 회귀: `para_y_before_compose`
   - 정정: `tac_img_y`
   - 효과: target_line 기반 위치 사용
3. **Shape 분기 `layout_cell_shape para_y` 인자** (line 1834):
   - 회귀: `para_y_before_compose`
   - 정정: `tac_img_y`
   - 효과: 동일

### 5.3 안전성 영역 (PR 본문 edge cases 분석)

| 케이스 | 정정 후 동작 | 사유 |
|--------|-------------|------|
| `first_vpos = 0` (cell 첫 paragraph) | 동일 동작 보장 | `seg.vpos - 0 = seg.vpos` |
| `target_line = 0` (rect on ls[0]) | 동일 동작 보장 | `target_line > current_tac_line` 가드로 if 블록 미진입 |
| `line_segs.len() = 1` (single-line) | 동일 동작 보장 | `target_line = 0` 으로 동일 |
| `wrap = Square / InFrontOfText / TopAndBottom` | 영향 없음 | `treat_as_char=true` 만 처리 |

### 5.4 회귀 발생 4 조건 분석 (PR 본문)
| 조건 | exam_science p[1] | 다른 5 multi-line + tac rect 케이스 |
|------|-------------------|------------------------------------|
| (a) cell 안 paragraph | ✓ | ✓ |
| (b) multi-line | ✓ (2) | ✓ (2~10) |
| (c) target_line > 0 | ✓ | varies |
| (d) **first_vpos > 0** | **✓ (1610)** | **✗ (모두 0)** |
| 회귀 가시화 | **YES** | **NO** |

→ exam_science p[1] 만 (c) AND (d) 동시 성립 — 광범위 sweep (1/1496) 와 정확 부합.

## 6. 메인테이너 web editor 시각 판정 ★ 통과

권위 영역 — `samples/exam_science.hwp` page 2 7번 문제 글상자:
- ㉠ 사각형 — Line 2 (y=235.41) 정상 위치 (이전: Line 1 y=213.95 회귀)
- ㉠ 텍스트 — Line 2 (y=251.45) 정상 위치
- 본문 "분자당 구성 …" 위 겹침 부재

작업지시자 평가: "웹 에디터에서 7번 문제 시각 판정 통과입니다."

## 7. 회귀 차단 가드 영구 보존

`src/renderer/layout/integration_tests.rs::test_624_textbox_inline_shape_y_on_line2_p2_q7` (+58 LOC):
- y 좌표 [230, 240] 범위 검증 영역
- 향후 동일 결함 영역 발생 시 즉시 검출 영역
- TDD RED → GREEN 권위 패턴 정합

## 8. 광범위 fixture sweep (PR 본문)

| 카테고리 | 페이지 수 | 비율 |
|----------|-----------|------|
| 의도된 정정 | 1 | 0.067% |
| **회귀** | **0** | **0%** |
| byte-identical | 1,495 | 99.933% |

## 9. devel 머지 + push

### 진행
1. `git cherry-pick b3408fe e5811c9 c30e886 242955b 9cf9fc5` (5 commits)
2. Stage 3 commit 영역에서 `mydocs/orders/20260506.md` add/add 충돌 → `git checkout --ours` 본 환경 보존
3. devel ← local/devel ff merge
4. push: `868e6df..c55ee3c`

### 분기 처리
- 본 cherry-pick 시점 origin/devel 분기 0 — `feedback_release_sync_check` 정합

## 10. PR / Issue close

- PR #627: 한글 댓글 등록 + close (`gh pr close 627`)
- Issue #624: 한글 댓글 등록 + close (`gh issue close 624`)

> `closes #624` 키워드 영역은 cherry-pick merge 영역의 hash 재생성으로 자동 처리 안 됨 — 수동 close 진행. `feedback_close_issue_verify_merged` 정합.

## 11. 메모리 룰 적용

- **`feedback_close_issue_verify_merged` 권위 사례 강화 누적** — 본 사이클의 권위 사례 누적 영역:
  - PR #620 (Task #519 누락) — 첫 번째 사례
  - **본 PR #627 (Task #520 누락)** — 두 번째 사례
  - 모두 PR #561 cherry-pick base diff 점검 누락 영역의 권위 영역 사례
- `reference_authoritative_hancom` — 한컴 2010/2022 편집기 권위 정답지 영역 비교 영역 (web editor 시각 판정 영역)
- `feedback_pdf_not_authoritative` (5/7 갱신) — `pdf/exam_science-2022.pdf` (PR #670 영구 보존 영역) 영역 정합
- `feedback_assign_issue_before_work` — Issue #624 assignee 미지정 영역 (외부 컨트리뷰터 자기 등록 사례)
- PR #668 패턴 정합 — Co-Authored-By Claude 영역 명시 영역의 본 사이클 정착 영역
- 거버넌스 영역 본 환경 명명 규약 정합 영역 (`task_m100_624*`) — PR #622 패턴 정합

## 12. TDD 흐름 영역의 권위 패턴

본 PR 의 TDD 흐름 영역은 본 사이클의 권위 패턴 영역:

```
Stage 1 (TDD RED) — 회귀 재현 통합 테스트 (y=213.95)
   ↓
Stage 2 분석 — 영향 범위 + edge case + 후속 진단 (작업지시자 "분석만 계속 진행" 영역 정합)
   ↓
Stage 2 정정 — 본질 정정 (3 line)
   ↓
Stage 3 (TDD GREEN) — test_624 GREEN + 광범위 sweep 회귀 0
```

→ TDD 흐름 영역의 단계별 보존 영역이 본 사이클의 권위 패턴 영역 정합 (Stage 2 분석 + Stage 2 정정 분리 영역의 본질 영역).

## 13. 본 사이클 (5/7) PR 처리 누적 — **12건**

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
| 12 | **PR #627** | **Task #624 (Task #520 누락 회귀)** | **옵션 A + TDD RED→GREEN + web editor 시각 판정 ★ + close** |

### 본 사이클의 권위 사례 누적 영역
- **`feedback_close_issue_verify_merged`** — PR #620 + **PR #627** = 누적 (PR #561 cherry-pick base diff 점검 누락 패턴)
- **`feedback_hancom_compat_specific_over_general`** — PR #621 + PR #622 = 누적 (구조적 가드)

본 PR 의 **TDD 흐름 영역의 권위 패턴 + 회귀 차단 가드 영구 보존 + 본 환경 명명 규약 (m100) 정합 + 권위 영역 100% 일치 + 메인테이너 web editor 시각 판정 ★ 통과 + `feedback_close_issue_verify_merged` 권위 사례 강화 누적 + Co-Authored-By Claude 영역 명시 패턴 모두 정합**.
