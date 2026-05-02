# PR #478 검토 문서

**제목**: Task #476 + #479 + #480 + #483 + #488 + #489 + #490 + #492: layout 정합 + 수식 정정 합본
**작성자**: planet6897 (Jaeuk Ryu)
**Base/Head**: `devel ← devel` (본인의 devel 분기)
**상태**: OPEN, **CONFLICTING**
**규모**: +6,715 / -683, **97 commits**

## 핵심 정합

PR #472 와 같은 패턴 — **본인 devel 분기에서 누적된 PR**, 다수의 이미 cherry-pick 머지된 commits 포함. **본질 신규 9 Task** 만 추출 후 cherry-pick.

## 본질 신규 commits (9 Task)

| Task | 본질 Commit | 영역 | 영향 |
|------|------------|------|------|
| **#476** | `4414876` | PartialParagraph 인라인 Shape 페이지 라우팅 | layout |
| **#479** ★ | `fa73785` + `ef1ff58` (v2) + `73c6a95` (보고서) | **paragraph trailing line_spacing 정합 (HWP vpos)** | **typeset core** |
| **#480** | `9e8d498` | wrap=Square 표 paragraph margin 반영 | layout |
| **#483** | `b6142e3` + `1b4eaa8` (follow-up) | 각주 multi-paragraph line_spacing | footnote |
| **#488** ★ | `6d8d0e1` + `1af552d` + `d245791` | **수식 토크나이저 + 렌더러 italic** | equation |
| **#489** ★ | `1991af5` | Picture+Square wrap 호스트 텍스트 LINE_SEG | paragraph_layout |
| **#490** ★ | `cf80c25` | 빈 텍스트 + TAC 수식 셀 alignment | paragraph_layout |
| **#492** | `462db34` | (자연 해소, orders 갱신만) | docs |
| **#495** | `7bd933a` | 셀 paragraph 인라인 Shape 분기 가드 | layout |

★ 표시 = 메인테이너 핵심 검증 영역 (작성자 보고)

## 영역 충돌 점검

| 영역 | 본 사이클 정정 | PR #478 |
|------|---------------|---------|
| `typeset.rs` | Task #474 (RowBreak), #470 (cv != 0) | **#479 paragraph trailing ls 정합** — 핵심 영역 |
| `paragraph_layout.rs` | Task #477 (셀 폭 클램프) | **#489, #490** — paragraph 영역 |
| `layout.rs` | Task #471 (stroke_sig) | **#476, #480, #495** |
| `equation/` | (영향 없음) | **#488** 신규 영역 |
| `paragraph_layout.rs::layout_footnote_paragraph_with_number` | (영향 없음) | **#483** 신규 영역 |

→ **본 사이클 정정 영역과 일부 충돌** — typeset.rs / paragraph_layout.rs 영역. cherry-pick 시 충돌 해결 필요.

## 위험 영역 (★ Task #479)

**Task #479 가 가장 큰 영역** — paragraph 누적에서 trailing line_spacing 제외 (HWP vpos 정합).

작성자 본문 인용:
> `samples/aift.hwp` p3 모든 paragraph **diff = +9.5 → +0.0** (HWP vpos 완전 정합).

→ **본 사이클의 Task #431 (compute_cell_line_ranges 단위 mismatch) 와 다른 영역의 본질 정정**. typeset.rs:802 의 paragraph total_height 가 마지막 line 의 line_spacing 누적 결함. **paragraph 17개 누적 ≈ 200px drift**.

본 정정의 작성자 검증:
- 21_언어_기출 p12 23번 박스 y: 1166.0 → 1040.2 (-125.8)
- aift.hwp p3 paragraph diff +9.5 → +0.0

이 영역은 **typeset 의 본질 영역 정정** — 본 사이클 진행 중 메인테이너의 #241 (vertical_offset) 작업 시도가 layout 누적 drift 본질 미해결로 포기됐던 영역. **#479 가 본 영역 정정**.

## 검증 정합 (작성자 보고)

| 영역 | 검증 |
|------|------|
| TYPESET_DRIFT (#479) | aift.hwp p3 모든 paragraph diff = +0.0 |
| #488 raw prefix | 8 핵심 샘플 59 페이지 0 잔존 |
| #489 byte 비교 | 9 종 263 페이지 261 동일 / 2 정정 |
| #490 byte 비교 | 9 종 263 페이지 257 동일 / 6 정정 |
| 작성자 cargo test | 1078+ passed |

## 처리 옵션

| 옵션 | 진행 |
|------|------|
| A. **본질 신규 9 Task cherry-pick (분리)** | PR #472 와 동일 패턴 — Task 별 commit 분리 cherry-pick |
| B. **단계 분리** | 9 Task 를 우선순위로 분류 → ★ 표시 우선 (#479 #488 #489 #490) → 부분 머지 → 시각 검증 → 잔여 머지 |
| C. **순차 머지** | 충돌 해결 + 순차 cherry-pick (시간 소모) |
| D. **머지 후 한 번에 시각 검증** | 위험 — 9 Task 누적 영향 영역 큼 |

## 권장 — B (단계 분리)

본 사이클의 본질 정정 (5건) 후 PR #478 의 9 Task 누적은 **단일 머지로 처리하기 큰 영역**. 단계별 분리:

### Stage 1: 핵심 ★ Task 4건 (#479 #488 #489 #490)
- typeset / equation / paragraph_layout 의 핵심 본질 정정
- 작성자가 광범위 검증 완료 (★ 표시)
- 시각 검증 1차

### Stage 2: 보조 Task 5건 (#476 #480 #483 #492 #495)
- 영역별 정정 — 단계 1 후 회귀 점검
- 시각 검증 2차

### Stage 3: 머지 + 작업지시자 시각 검증 + close

## Task #479 의 본 사이클 영향 영역

본 사이클의 **Task #241 (issue #241 vertical_offset)** 시도가 **layout 누적 drift 본질** 으로 포기됐던 영역 — Task #479 가 그 본질을 정정하는 영역. 본 PR cherry-pick 후 issue #241 재시도 가능 영역.

## 다음 단계

1. 작업지시자 옵션 결정 (B 권장)
2. Stage 1 ★ Task 4건 우선 cherry-pick
3. 검증 게이트 + 시각 검증
4. Stage 2 보조 Task cherry-pick
5. 결과 보고서 + PR close
