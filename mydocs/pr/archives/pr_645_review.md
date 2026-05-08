---
PR: #645
제목: fix: hit_test_header_footer 영역이 본문 침범 정정 (closes #595)
컨트리뷰터: @johndoekim
사이클: 첫 번째 PR
base: devel (BEHIND 10 commits)
mergeable: MERGEABLE
CI: ALL SUCCESS
변경: +2141/-24, 15 files
---

# PR #645 1차 검토 보고서

## 1. 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #645 |
| 제목 | fix: hit_test_header_footer 영역이 본문 침범 정정 |
| 컨트리뷰터 | @johndoekim (johndoe, amok0316@gmail.com) — 첫 번째 PR |
| base / head | devel / fix/issue-595-header-bbox-bleed |
| mergeStateStatus | BEHIND (devel 10 commits 앞 — PR #644 머지 등) |
| mergeable | MERGEABLE |
| CI | Build & Test / CodeQL(JS-TS, Python, Rust) / Canvas visual diff — ALL SUCCESS |
| 변경 규모 | +2141 / -24, 15 files |
| 커밋 수 | 5 (Stage 1, 2, 3, 후속 sweep, 후속 framing) |
| closes | #595 |
| 후속 이슈 등록 | #685 / #686 (컨트리뷰터가 자체 분리 등록) |

## 2. Issue #595 본질

`exam_math.hwp` 의 수식 더블클릭 동작:
- **page 0 (1p)**: 정상 (수식 편집기 진입)
- **page 1+ (2p~)**: 머리말 편집 모드로 잘못 진입

PR 본문의 진단:
> `hit_test_header_footer_native` 가 사용하는 Header 노드 bbox 가 자식 (단 구분선 line) 까지 확장되어 본문 영역을 머리말로 잘못 인식 → `onDblClick` 의 머리말 분기가 picture selection 분기보다 먼저 실행되어 수식 편집기 진입 차단.

| 페이지 | 정정 전 Header hit y 범위 | 실제 머리말 영역 |
|--------|--------------------------|------------------|
| page 0 | 60.0 ~ 145.0 | 0 ~ 147 (정상) |
| page 1 | **60.0 ~ 1355.0** | 0 ~ 147 (본문 80% 침범) |
| page 2~ | 60.0 ~ 1355.0 | 0 ~ 147 |

차이 원인: page 0 의 단 구분선은 `ci=5` 로 Body 자식, page 1+ 부터 `ci=2` 로 Header 자식 → page 0 만 정상.

## 3. PR 의 정정

### 본질 정정 영역
`src/document_core/queries/cursor_rect.rs::hit_test_header_footer_native` 단일 함수.

**변경 본질**:
- `build_page_tree(page_num)` + 자식 bbox 순회 → `find_page(page_num)` + `layout.header_area` / `layout.footer_area` 직접 hit 판정
- bbox 확장 (`expand_bbox_to_children`) 의도 보존 — 머리말 표 셀 안 Picture 클리핑 방지 (#42 영역)
- 부수 효과: mousedown 마다 호출되던 트리 빌드 비용 제거

```rust
// 정정 후 (요약)
let (page_content, _, _) = self.find_page(page_num)?;
let layout = &page_content.layout;
let h = &layout.header_area;
if x >= h.x && x <= h.x + h.width && y >= h.y && y <= h.y + h.height { ... }
let f = &layout.footer_area;
if x >= f.x && x <= f.x + f.width && y >= f.y && y <= f.y + f.height { ... }
```

코드 변경 규모: **+37 / -24 LOC** (cursor_rect.rs 단일 파일).

### Stage 별

| Stage | commit | 산출물 | 본질 등급 |
|-------|--------|--------|----------|
| Stage 1 | `24a267d` | 본질 진단 + `tests/issue_595.rs` (5 케이스 RED) + 광범위 sweep | ★★★ |
| Stage 2 | `33fc7ac` | 본질 정정 + 회귀 sweep 도구 + WASM 빌드 + 시각 판정 | ★★★ |
| Stage 3 | `938631f` | 최종 보고서 + 회귀 위험성 점검 (관련 이슈/함수/호출처) | ★★★ |
| 후속 | `d591432` | 보류 2건 e2e 정량 측정 + Issue #685/#686 등록 | ★★★ |
| 후속 | `510ea23` | Issue #685 진단 노트 framing 정정 (한컴 호환 결함) | ★★ |

## 4. 본 환경 cherry-pick simulation

### 4.1 깨끗한 적용
- `local/pr645-sim` 브랜치, 5 commits cherry-pick
- **충돌 0건** (devel 의 PR #644 + Task #634 + PR close 영역과 본 PR 영역 비충돌)

### 4.2 결정적 검증
- `cargo test --release` → ALL PASS, failed 0건
- `tests/issue_595.rs` (신규, 5 케이스):
  - `issue_595_page0_body_coord_not_header` ✅
  - `issue_595_page1_body_coord_not_header_regression_guard` ✅
  - `issue_595_page1_equation_coord_not_header` ✅
  - `issue_595_page1_body_center_not_header` ✅
  - `issue_595_page1_header_area_still_hits` ✅
- `cargo test --lib --release` → 1165 passed (회귀 0)
- `cargo clippy --release` → clean

### 4.3 광범위 회귀 sweep (`examples/inspect_595_regression`)

본 환경 직접 실행 결과 (169 fixture / 1788 페이지):

| 검증 영역 | pass / fail |
|-----------|-------------|
| [1] 머리말 hit:true 보장 | **1334 / 0** ✅ |
| [2] 꼬리말 hit:true 보장 | 1361 / 16 (`hwpctl_Action_Table__v1.1.hwp` 한정 — Issue #686 별도 영역) |
| [3] 본문 hit:false 보장 (#595 정정) | **1788 / 0** ✅ |

→ PR 본문의 정량 (본문 32 fail → 0 fail, 머리말 27 fail → 0 fail) 정확히 재현. 본 환경은 fixture 추가로 1788 페이지 기준 0 fail.

## 5. 검토 관점

### 5.1 본질 정정의 정확성
- `expand_bbox_to_children` 의도 (#42 머리말 표 셀 Picture 클리핑 방지) 보존
- hit 판정만 정확한 영역 (`header_area` / `footer_area`) 으로 분리
- 렌더링 동작 무영향 (build_page_tree 호출만 제거)
- TS 측 호출처 2곳 (`input-handler-mouse.ts:494, 784`) 모두 회귀 위험 0

### 5.2 절차 준수
- TDD Stage 1~3 + 후속 영역 정합
- 회귀 차단 가드 (`tests/issue_595.rs` 5 케이스 + `inspect_595_regression.rs` 영구 보존)
- 수행/구현 계획서 + 단계별 보고서 + 최종 보고서 영구 보존
- 보류 2건 별도 이슈 등록 (#685, #686) — 본 PR 영역에 포함 시도 안 함, 깔끔한 분리

### 5.3 메인테이너 시각 판정 영역
PR 본문 명시:
> 작업지시자 직접 환경 (1365×1018, zoom=1.0) 에서 정정 전 결함 재현 + 정정 후 정상 동작 모두 ★ 통과 확인. 진행 환경: macOS + Chrome.

→ **컨트리뷰터 환경에서 시각 판정 통과 영역**. `feedback_v076_regression_origin` 룰에 따라 본 환경 (Linux + WSL2) 에서 추가 시각 판정 권장.

### 5.4 별도 발견 (#685, #686) 분리의 합리성

컨트리뷰터가 광범위 sweep / e2e 진단에서 발견한 본 영역과 다른 본질 영역:
- **#685**: 그리드 모드 (zoom ≤ 0.5) `pageLeft` 단일 컬럼 가정 — 14곳 분기 일괄 어긋남
- **#686**: `hwpctl_Action_Table__v1.1.hwp` 꼬리말 hit:false (`landscape + marginBottom=0`)

→ 본 PR 영역 외 분리, OPEN 상태. 한컴 호환 결함으로 framing 명확.

## 6. 메모리 룰 관점

### `feedback_visual_judgment_authority`
> 결정적 검증만으로 부족, 메인테이너 시각 판정 + 본질 결함 발견 영역의 권위 사례

→ 본 PR 은 컨트리뷰터 시각 판정만 수행. 본 환경 시각 판정 게이트 필요.

### `feedback_check_open_prs_first` + `feedback_assign_issue_before_work`
→ Issue #595 작업 시 assignee 부재. 첫 번째 PR 영역 컨트리뷰터의 본 작업이 외부 기여자에게 "오픈 타스크" 로 인식된 결과 — assignee 사후 지정 필요.

### `feedback_pr_comment_tone`
→ 첫 번째 PR 컨트리뷰터지만 차분하고 사실 중심 응대. 과도한 인사 자제.

## 7. 결정 옵션

| 옵션 | 내용 | 비고 |
|------|------|------|
| **A** | 5 commits 그대로 squash merge + 본 환경 시각 판정 | 단순, TDD 흔적 압축 |
| **B** | 5 commits 단계별 보존 merge + 본 환경 시각 판정 | TDD 5 단계 흔적 보존, 후속 디버깅 시 commit 단위 추적 가능 — PR #644 와 동일 패턴 |
| **C** | merge 보류 — 본 환경 e2e (`rhwp-studio/e2e/issue-595.test.mjs`) 시각 판정 후 결정 | PR Test plan 미체크 영역 처리 |

## 8. 잠정 결정

**옵션 B + 본 환경 시각 판정** 권장.

이유:
1. 결정적 검증 (1165 lib + 5 issue_595) ALL PASS
2. 광범위 sweep 본 환경 재현: 본문 hit:false 1788/0, 머리말 hit:true 1334/0
3. 본질 정정 단일 함수 (+37/-24 LOC), `expand_bbox_to_children` 의도 보존
4. TDD Stage 1~3 + 후속 영역 절차 정합
5. 첫 번째 PR 영역 컨트리뷰터의 본질 정정 + 별도 영역 분리 (Issue #685/#686) 의 깔끔한 절차

## 9. 작업지시자 결정 요청

1. **옵션 선택**: A / B / C 중?
2. **본 환경 시각 판정 시점**: cherry-pick simulation + WASM 빌드 후 rhwp-studio 에서 `exam_math.hwp` page 1+ 수식 더블클릭 시각 확인?
3. **별도 이슈 #685, #686**: 별도 task 진행 (본 PR 영역 외, 별도 영역 분리 정합)?
4. **컨트리뷰터 응대**: 첫 번째 PR 영역 — 차분한 사실 중심 응대 (인사 자제) 정합?

---

작성: 2026-05-08
