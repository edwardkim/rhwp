# Task #595 최종 결과 보고서

**Issue**: [#595](https://github.com/edwardkim/rhwp/issues/595) — exam_math.hwp 2페이지부터 수식 더블클릭 hitTest 오동작 — 1페이지만 정상
**Milestone**: M100 (v1.0.0)
**브랜치**: `local/task595` → `local/devel` 머지 영역
**완료일**: 2026-05-07

---

## 1. 본질 요약

`samples/exam_math.hwp` 의 수식 객체를 더블클릭했을 때 1페이지만 정상 동작, **2페이지부터 수식 편집기가 안 나타남 + 캐럿이 머리말 영역으로 이동**. 사용자 보고 단서 "hover 시 손바닥 표시는 뜨는데 클릭 반응 없음" 으로 본질 영역 확정.

**본질 결함 위치**: `src/renderer/layout.rs::build_header` ([line 928](../../src/renderer/layout.rs#L928)) 의 `expand_bbox_to_children` 호출이 머리말 자식 (단 구분선 line `paraIdx=0 ci=2`, h≈1227px) 의 bbox 까지 Header 영역으로 확장 → `hit_test_header_footer_native` 가 본문 좌표를 머리말 hit 으로 잘못 인식 → `onDblClick` 의 머리말 분기가 picture selection 분기보다 먼저 실행되어 수식 편집기 진입 차단.

**page 0 vs page 1+ 차이**: page 0 의 단 구분선은 ci=5 로 Body 자식, page 1+ 부터 ci=2 로 Header 자식 → page 0 만 정상.

## 2. 정정 영역 (옵션 A)

**파일**: `src/document_core/queries/cursor_rect.rs`
**함수**: `hit_test_header_footer_native`
**LOC**: +20 / -13 (단일 함수)

`build_page_tree` 의 Header/Footer 노드 bbox (자식 노드 까지 확장됨) 대신 **`layout.header_area` / `layout.footer_area`** (PageDef margin 으로 계산된 정확한 영역) 로 hit 판정. `expand_bbox_to_children` 은 무수정 — 머리말 표 셀 내 Shape 클리핑 방지 의도 보존.

**부수 효과**: `build_page_tree` 호출 제거 → mousedown 마다 호출되던 비싼 트리 빌드 비용 제거.

**정합 영역**: HWP IR 표준 직접 사용 (`feedback_rule_not_heuristic` 정합), 회귀 위험 영역 좁힘 (단일 함수, 렌더링 무영향), 본질 정정 (우회 패치 아님).

## 3. 검증 결과 (정량)

| 검증 항목 | 정정 전 | 정정 후 |
|-----------|---------|---------|
| `tests/issue_595.rs` (5 케이스) | 3 fail / 2 pass | **5 pass** |
| 본문 hit:false sweep (164 fixture / 1684p) | 1652p / 32 fail | **1684 / 0 fail** (+32 본질 정정) |
| 머리말 hit (margin_header > 0, 1356p) | 1329 / 27 fail | **1356 / 0 fail** (+27 부수 개선) |
| 꼬리말 hit (margin_footer > 0) | 1383 / 16 fail | 1383 / 16 fail (회귀 0, 별도 영역) |
| `cargo test --lib --release` | (baseline) | **1140 pass** (회귀 0) |
| `cargo clippy --release` (lib) | (baseline) | clean |
| `cargo build --release` | (baseline) | clean |
| `cargo test --release --test issue_516/530/546/554/595` | (baseline) | 30 pass (회귀 0) |
| WASM 빌드 (Docker) | (baseline) | **4,531,883 bytes** clean |
| 작업지시자 시각 판정 ★ | 결함 재현 | **정상 동작** ★ |

## 4. 회귀 위험성 점검

**관련 이슈** (CLOSED):

| 이슈 | 영역 | 회귀 위험 |
|------|------|----------|
| #236 | `PageAreas::from_page_def` 머리말/본문 영역 공식 | **0** — 본 정정이 #236 정정의 영역을 일관 활용 (정확성 강화) |
| #42 | 머리말/꼬리말 내 Picture 렌더링 | **0** — 렌더링 무영향 |
| #36 | 머리말 표 셀 안 이미지 미렌더링 | **0** — 렌더링 무영향 |
| #340 | exam_math 13페이지 머리말 누출 | **0** — typeset 무영향 |

**관련 함수**:

| 함수 | 정정 영향 | 회귀 위험 |
|------|----------|----------|
| `hit_test_in_header_footer_native` (편집 모드 텍스트 hit) | 무수정 | **0** — 본 정정 영역 안에서 정상 동작 |
| `get_active_hf_info` / `find_section_for_page` | 무수정 (본 정정에서 호출) | **0** |
| `build_page_tree` | 호출 제거 | 0 — 다른 호출처 무영향 |

**TS 측 호출처**: `hitTestHeaderFooter` 호출은 `input-handler-mouse.ts` 단 2곳만:
- L494 onMouseDown 머리말 모드 탈출 — **개선** (hit:false 정확)
- L784 onDblClick 본질 영역 — **정정**

## 5. 단계별 진행

| Stage | 산출물 | commit |
|-------|--------|--------|
| Stage 1 | 본질 진단 + 재현 단위 테스트 (5 케이스) + 광범위 sweep + 정정 영역 후보 분석 | [`54c0af2`](#) |
| Stage 2 | hit_test_header_footer 영역 정정 + 회귀 sweep 도구 + 정정 효과 광범위 검증 | [`0d20917`](#) |
| Stage 3 | 최종 보고서 + 회귀 위험성 점검 + 별도 task 등록 | (본 commit) |

## 6. 산출물 목록

**소스 코드**:
- `src/document_core/queries/cursor_rect.rs` — 본질 정정 (+20 / -13 LOC)

**단위 테스트** (영구 보존, 회귀 차단 가드):
- `tests/issue_595.rs` — 5 케이스 (정정 전 3 fail / 2 pass → 정정 후 5 pass)

**검증 도구** (영구 보존, 향후 재사용):
- `examples/inspect_595.rs` — Header bbox / 머리말 hit y 범위 sweep
- `examples/inspect_595_regression.rs` — 머리말/꼬리말/본문 영역 광범위 회귀 sweep

**문서**:
- `mydocs/plans/task_m100_595.md` — 수행 계획서
- `mydocs/plans/task_m100_595_impl.md` — 구현 계획서
- `mydocs/working/task_m100_595_stage1.md` — Stage 1 보고서
- `mydocs/working/task_m100_595_stage2.md` — Stage 2 보고서
- `mydocs/report/task_m100_595_report.md` — 본 최종 보고서

**e2e 진단** (비-회귀, 향후 다른 이슈에서 패턴 재사용):
- `rhwp-studio/e2e/issue-595.test.mjs` — 1365×1018 사용자 환경 모사 + zoom 변동 시나리오

**임시 파일 정리**:
- `rhwp-studio/public/samples/exam_math.hwp` — Stage 3 정리 시 제거 완료

## 7. 보조 메모 — 본 task 분리 영역 (참고만)

광범위 sweep / e2e 진단 중 발견된 본 이슈 #595 와 본질이 다른 영역. 본 task 정정과 무관 (회귀 0 확인됨), 사용자 시각 검증 안 됨, 한컴 호환 진단 필요 → **별도 이슈 등록 보류**. 본 보고서에는 참고 기록만 보존하고 본 task 정리 영역에서 분리.

| 영역 | 본질 (추정) | 정정 전후 | 사용자 검증 |
|------|-------------|-----------|-------------|
| 그리드 모드 (zoom ≤ 0.5) hit 좌표 | TS 측 `pageLeft` 단일 컬럼 가정 vs 그리드 `pageLefts[i]` 불일치 | 동일 | 안 함 |
| hwpctl_Action_Table 꼬리말 hit:false | landscape + `marginBottom=0` 양식의 `PageAreas::from_page_def` `footer_area.height = 0` | 동일 | 안 함 (정정 전후 동일 확인) |

향후 사용자 시각 검증 또는 한컴 호환 비교로 결함 확정 시 별도 task 진입.

## 8. 본 사이클 정합

- **하이퍼-워터폴 절차 정합** — 이슈 → 브랜치 → 수행 계획서 → 구현 계획서 → 단계별 진행 → 단계별 보고서 → 최종 보고서 모두 정상 진행. 작업지시자 단계별 승인 + 시각 판정 통과.
- **광범위 회귀 sweep 패턴** (`feedback_wide_regression_sweep` 정합) — 164 fixture / 1684p 광범위 측정으로 정정 안전성 입증.
- **본질 정정 영역 좁힘** (`feedback_root_cause_only` 정합) — 단일 함수 정정, 렌더링 무영향, expand 의도 보존.
- **이슈 본문 정오표 갱신** (`feedback_full_disclosure` 정합) — 이슈 작성자의 초기 진단 (a)/(b)/(c) 가 본질 영역 밖이었음을 코멘트로 명시.
- **회귀 위험성 영역 점검** — 관련 이슈 (#236, #42, #36, #340) + 관련 함수 + 호출처 모두 점검.
- **단위 테스트 영구 가드** — `tests/issue_595.rs` 5 케이스 회귀 차단 영구 보존.

## 9. 본 task 종결

본 task 는 **Issue #595 완전 해결** + 회귀 0 + 부수 개선 +27p + 별도 task 영역 식별로 완료. `local/devel` merge → `devel` push → main release 영역은 작업지시자 결정.

**Issue #595 close 영역**: 본 보고서 + 정정 코드 + 검증 데이터를 코멘트로 등록 후 close.
