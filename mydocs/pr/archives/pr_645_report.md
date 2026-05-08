---
PR: #645
제목: fix: hit_test_header_footer 영역이 본문 침범 정정 (closes #595)
컨트리뷰터: @johndoekim (첫 번째 PR)
처리: MERGE (옵션 B — 5 commits 단계별 보존)
처리일: 2026-05-08
---

# PR #645 최종 보고서

## 1. 결정

**옵션 B (5 commits 단계별 보존 merge)** + WASM 빌드 + 작업지시자 시각 판정 ★ 통과.

merge commit: `11f3bf13`

## 2. 본 환경 검증 결과

### 2.1 cherry-pick simulation
- `local/pr645-sim` 브랜치, 5 commits cherry-pick
- **충돌 0건** (devel 의 PR #644 머지 + Task #634 + PR #638/#641 close 영역과 본 PR 영역 비충돌)

### 2.2 결정적 검증
- `cargo test --release` → ALL PASS, failed 0건
- `tests/issue_595.rs` (신규, 5 케이스):
  - `issue_595_page0_body_coord_not_header` ✅
  - `issue_595_page1_body_coord_not_header_regression_guard` ✅
  - `issue_595_page1_equation_coord_not_header` ✅
  - `issue_595_page1_body_center_not_header` ✅
  - `issue_595_page1_header_area_still_hits` ✅
- `cargo test --lib --release` → 1165 passed (회귀 0)
- `cargo clippy --release` → clean

### 2.3 광범위 회귀 sweep (`examples/inspect_595_regression`)

본 환경 직접 실행 (169 fixture / 1788 페이지):

| 검증 영역 | pass / fail |
|-----------|-------------|
| [1] 머리말 hit:true 보장 | **1334 / 0** ✅ |
| [2] 꼬리말 hit:true 보장 | 1361 / 16 (`hwpctl_Action_Table__v1.1.hwp` 한정 — Issue #686 별도 영역) |
| [3] 본문 hit:false 보장 (#595 정정) | **1788 / 0** ✅ |

PR 본문 정량 (본문 32 fail → 0 fail, 머리말 27 fail → 0 fail) 정확 재현.

### 2.4 시각 검증
- WASM 빌드 완료 (`pkg/rhwp_bg.wasm` 4,573,671 bytes)
- 작업지시자 시각 판정: **★ 통과**
  - 작업지시자 인용: "웹 에디터에서 2페이지 이후 수식 더블클릭 시 수식편집창 동작 확인했습니다. 컨트리뷰터의 정확한 개선에 감사드립니다."
  - exam_math.hwp page 1+ 수식 더블클릭 → 수식 편집기 정상 진입 확인

## 3. 본질 정정의 정확성

### 정정 영역
`src/document_core/queries/cursor_rect.rs::hit_test_header_footer_native` 단일 함수 (+37/-24 LOC).

### 본질
- 정정 전: `build_page_tree(page_num)` + 자식 bbox 순회 (`expand_bbox_to_children` 으로 단 구분선 line 의 bbox 까지 확장 → 본문 80% 침범)
- 정정 후: `find_page(page_num)` + `layout.header_area` / `layout.footer_area` 직접 hit 판정 (정확한 영역)

### 의도 보존
- `expand_bbox_to_children` 무수정 — 머리말 표 셀 안 Picture 클리핑 방지 (#42 영역) 보존
- `build_page_tree` 호출 제거 → mousedown 마다 호출되던 트리 빌드 비용 부수 효과 제거
- TS 측 호출처 2곳 (`input-handler-mouse.ts:494, 784`) 모두 회귀 위험 0

## 4. 컨트리뷰터 절차 정합

@johndoekim 첫 번째 PR. TDD Stage 1~3 + 후속 영역 절차 정합:

| Stage | 산출물 |
|-------|--------|
| Stage 1 | 본질 진단 + `tests/issue_595.rs` (5 케이스 RED) + 광범위 sweep |
| Stage 2 | 본질 정정 + 회귀 sweep 도구 (`inspect_595_regression.rs`) + WASM 빌드 + 시각 판정 |
| Stage 3 | 최종 보고서 + 회귀 위험성 점검 (관련 이슈/함수/호출처) |
| 후속 | 보류 2건 e2e 정량 측정 + Issue #685/#686 분리 등록 |
| 후속 | Issue #685 진단 노트 framing 정정 (한컴 호환 결함) |

회귀 차단 가드 영구 보존:
- `tests/issue_595.rs` (5 케이스)
- `examples/inspect_595_regression.rs` (169 fixture / 1788 페이지 sweep)
- `rhwp-studio/e2e/issue-595.test.mjs` (1365×1018 사용자 환경 모사)

## 5. 별도 이슈 분리 등록

컨트리뷰터가 광범위 sweep / e2e 진단에서 발견한 본 영역 외 결함을 본 PR 영역에 포함하지 않고 별도 이슈로 분리 등록:

| Issue | 본질 | 상태 |
|-------|------|------|
| #685 | rhwp-studio: zoom ≤ 0.5 그리드 모드 click 좌표 단일 컬럼 가정 — 14곳 분기 일괄 어긋남 | OPEN (한컴 호환 결함 framing) |
| #686 | rhwp-studio: master page 글상자 더블클릭 시 첫 페이지로 점프 — 한컴 mismatch (`hwpctl_Action_Table__v1.1.hwp` 16p) | OPEN |

→ 본 PR 영역 외 분리 정합. 추후 별도 task 진행.

## 6. 메모리 룰 적용 결과

### `feedback_visual_judgment_authority`
> 결정적 검증만으로 부족, 메인테이너 시각 판정 + 본질 결함 발견 영역의 권위 사례

→ 본 PR 은 작업지시자 시각 판정 (★ 통과) 게이트 정합. 결정적 검증 (1165 lib + 5 issue_595 + 광범위 sweep 1788/0) 외에 시각 판정 영역도 통과.

### `feedback_v076_regression_origin`
> 외부 PR 컨트리뷰터들이 자기 환경 PDF 를 정답지로 사용 → 작업지시자 환경에서 회귀

→ 본 PR 은 컨트리뷰터 환경 (macOS + Chrome, 1365×1018) + 작업지시자 환경 (Linux + WSL2) 두 영역 모두 시각 판정 통과 — 환경 차이로 인한 회귀 가능성 차단.

### `feedback_pr_comment_tone`
→ 첫 번째 PR 영역 컨트리뷰터지만 차분한 사실 중심 응대 유지 (감사 표현은 작업지시자 인용 그대로 전달).

### `feedback_check_open_prs_first` + `feedback_assign_issue_before_work`
→ Issue #595 작업 시 assignee 부재가 외부 기여자에게 "오픈 타스크" 로 인식된 영역. 향후 내부 작업 결정 시 즉시 assignee 지정 필요 — 본 PR 의 첫 번째 컨트리뷰터 케이스가 룰의 권위 사례 강화.

## 7. 산출물

| 산출물 | 경로 |
|--------|------|
| 검토 보고서 | `mydocs/pr/archives/pr_645_review.md` |
| 최종 보고서 | `mydocs/pr/archives/pr_645_report.md` (본 문서) |
| merge commit | `11f3bf13` |
| 별도 이슈 (컨트리뷰터 자체 등록) | #685, #686 |

## 8. 컨트리뷰터 응대

@johndoekim (첫 번째 PR) 안내:
- 본질 정정 (`hit_test_header_footer_native` 단일 함수 정정) 정확
- 본 환경 결정적 검증 통과 + 광범위 sweep 1788/0 fail 정합
- 작업지시자 시각 판정 ★ 통과 — exam_math.hwp page 1+ 수식 더블클릭 정상 동작 확인
- TDD Stage 1~3 + 후속 + 별도 이슈 분리 절차 정합
- merge 결정

작성: 2026-05-08
