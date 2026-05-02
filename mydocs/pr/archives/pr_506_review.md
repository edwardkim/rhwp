# PR #506 검토 문서

**제목**: Task #460: HWP 3.0 파서 + Square wrap 그림 어울림 렌더링
**작성자**: jangster77 (Taesup Jang)
**Base/Head**: `devel ← devel` (작성자 본인 fork 의 devel 분기)
**상태**: OPEN, MERGEABLE, BLOCKED (admin 머지 영역 점검 필요)
**규모**: +13,018 / -52, **51 commits**
**연결 이슈**: Task #417, Task #460

## 핵심 정합

### PR 본질

**HWP 3.0 바이너리 포맷 파서 신규 구현** + Square wrap 어울림 렌더링 정정.

- HWP 3.0 = 한컴이 1990년대~2000년대 초반 사용한 레거시 포맷 (현재의 HWP 5.0 = OLE2 / HWPX = ZIP+XML 이전 포맷)
- Johab (조합형) 인코딩 지원, OLE 컨테이너, 페이지 경계, 표/그림/하이퍼링크
- 기존 렌더러 (HWP5/HWPX) 와 공통 IR (`Document`) 경유 — 렌더러 분기 없음 (정합 영역)

### 영역 분류

| 영역 | 본질 | 회귀 위험 |
|------|------|----------|
| **HWP 3.0 파서 (10 파일, 신규)** | `src/parser/hwp3/` 본질 신규 영역 — mod / records / paragraph / encoding / johab / johab_map / special_char / drawing / ole | **신규 영역, 회귀 0** |
| 파서 라우팅 | `src/parser/mod.rs` (+46 lines) — `FileFormat::Hwp3` → `parse_hwp3()` | 신규 분기 추가 |
| **렌더러 layout 영역** ★ | `src/renderer/layout.rs` (+154 lines) — Square wrap `wrap_pic_bottom_y` + 앵커 Shape y_offset | 본 사이클 #480/#476 영역과 충돌 가능 |
| **typeset 영역** ★ | `src/renderer/typeset.rs` (+66 lines) — wrap zone 종료 시 current_height 보정 | 본 사이클 #501 (cell.padding 무관) + #479 (미흡수) 영역 영향 점검 |
| pagination 영역 | `src/renderer/pagination/engine.rs` (+26 lines) | 회귀 점검 필요 |
| paragraph_layout | `src/renderer/layout/paragraph_layout.rs` (+6 lines, 작은 변경) | 본 사이클 #488/#490/#489 영역 — 작은 변경이라 충돌 가능성 낮음 |
| picture_footnote | `src/renderer/layout/picture_footnote.rs` (+5 lines) | 작은 변경 |
| document_core | `document.rs` + `hwpx_to_hwp.rs` | HWP3 IR → HWP5 저장 영역 (Stage 8/9) |
| **table_layout 영역** | 변경 0 | 본 사이클 #501 (cell.padding) 정정과 **충돌 없음** ★ |
| 신규 샘플 | `samples/hwp3-sample.hwp` + `hwp3-sample4.hwp` + `hwp3-sample5.hwp` | HWP 3.0 검증 자료 |

## 51 commits 본질 분류

| Stage | Commit | 영역 |
|-------|--------|------|
| **Task #417 Stage 1** | `675354e` | HWP 3.0 파서 초기 구현 + 코드 정리 |
| Task #417 Stage 2 | `a2bbdc0` | HWP3 그림 겹침 + 캡션 렌더링 + AutoNumber |
| Task #417 Stage 3 | `4f44a75` | HWP3 하이퍼링크 URL 추출 |
| Task #417 Stage 4 | `ba1e4dd` | 최종 보고서 |
| Task #417 머지 | `4ac8e57` | local/task417 머지 |
| **Task #460 Stage 1~3** | `e806ac6` ~ `3643d1d` | HWP3 AutoNumber + LINE_SEG 높이 보정 + 검증 |
| Task #460 Stage 4 | (border_fill_id + body clip) | 후속 정정 |
| Task #460 Stage 5 | (LINE_SEG line_spacing=0) | 표 텍스트 겹침 |
| Task #460 Stage 6 | 페이지 경계 대형 그림 anchor | typeset + 파서 |
| Task #460 Stage 7 | 비-TAC 그림 LAYOUT_OVERFLOW | 파서 |
| Task #460 Stage 8 | HWP3 → HWP5 저장 | document_core |
| **Task #460 Stage 9** | `da90f68` | HWP3 → HWP5 재열기 그림 위치 복원 (common.attr 비트필드) |
| 보완4~5 | `9d9b4ed`, `82e41ba` | Square wrap 텍스트 y 위치 보정 |
| Merge upstream/devel | `da90f68` | devel 머지 (충돌 해결: .gitignore + integration_tests.rs) |

## 본 사이클 정합 충돌 점검

### 본 사이클 정정 영역 (v0.7.9)
- Task #501 cell.padding 한컴 방어 로직 (table_layout.rs)
- PR #478 (8 Task / 11 commits) — paragraph_layout / table_layout / layout / 수식 영역
- PR #498 — JS E2E + CI workflow (Rust 영향 0)

### 충돌 점검

| 영역 | 본 사이클 | PR #506 |
|------|----------|---------|
| `table_layout.rs` | Task #501 cell.padding | 변경 0 — **충돌 없음** ★ |
| `paragraph_layout.rs` | #488/#490/#489/#495 | +6 lines (작은 변경) — 점검 필요 |
| `layout.rs` | #480/#476 | +154 lines — **충돌 가능 영역** ★ |
| `typeset.rs` | (없음, #479 미흡수) | +66 lines — wrap zone current_height 보정 |
| `pagination/engine.rs` | (없음) | +26 lines |

→ **layout.rs + typeset.rs 영역 충돌 점검 필수**. 다른 영역은 신규 또는 작은 변경.

## 검증 정합 (작성자 보고)

- `cargo test`: 1016 passed (작성자 시점) — 본 사이클 1102 보다 작음 (작성자 분기가 본 사이클 정정 미흡수 정합)
- `cargo clippy`: clean
- `rhwp-sample.hwp` export-svg: 20페이지 SVG 생성 성공
- HWP5/HWPX 기존 파일 SVG: **미체크 (체크리스트 미체크)** — 회귀 점검 영역 ★

## 처리 옵션

| 옵션 | 진행 |
|------|------|
| A. **단계 분리 cherry-pick** | Task #417 (HWP 3.0 파서, Stage 1~4) → Task #460 (Stage 1~9 + 보완) 분리 |
| B. **HWP 3.0 파서만 흡수** | src/parser/hwp3/ 신규 영역 (회귀 0) + 라우팅만. 렌더러 영역 (#460) 별도 사이클 |
| C. **머지 후 한 번에 시각 검증** | 13,018 lines 누적 영향 영역 — 위험 |
| D. 정정 요청 | 본 사이클 정정 영역 정합 + 회귀 검증 추가 의뢰 |

## 권장 — B (HWP 3.0 파서만 흡수)

**근거**:
1. **HWP 3.0 파서 (Task #417)** = 신규 포맷 지원, 본 사이클 회귀 영역과 무관 — 안전
2. **Task #460 (Square wrap + 렌더러 정정)** = layout.rs (+154) / typeset.rs (+66) 변경 — **본 사이클 PR #478 후속 정정 영역과 충돌 위험**
3. 작업지시자 통찰 정합 — `feedback_small_batch_release_strategy` 메모리 룰: 큰 묶음 회피 + 작은 단위 회전 운영
4. HWP 3.0 파서는 본 사이클 안정화 후 별도 PR / Task #460 (렌더러 영역) 도 별도 사이클

### B 옵션 진행 정합

- Task #417 (HWP 3.0 파서) 영역 commits 만 cherry-pick → 1차 머지
- 시각 검증 + 검증 게이트 통과 → close
- Task #460 (렌더러 영역) 은 **별도 task** 로 분리 — 본 사이클 PR #478 안정화 + Task #501 회귀 점검 후 재처리

## 권장 — A (단계 분리 cherry-pick)

**근거**:
1. 작성자가 단계별 commit 분리 + 광범위 검증
2. Task #417 + Task #460 모두 본질 정합
3. 단계별 시각 판정 게이트 적용 — 본 사이클 PR #478 (5 단계 머지) 정합

### A 옵션 진행 정합

| 단계 | Task | 영역 |
|------|------|------|
| 1 | **Task #417 Stage 1~4** | HWP 3.0 파서 본질 (10 파일 신규 + 라우팅) |
| 2 | **Task #460 Stage 1~3** | HWP3 AutoNumber + LINE_SEG 높이 보정 |
| 3 | Task #460 Stage 4~5 | border_fill_id + LINE_SEG line_spacing |
| 4 | Task #460 Stage 6~7 | 대형 그림 anchor + 비-TAC 그림 |
| 5 | **Task #460 Stage 8~9 + 보완** | HWP3 → HWP5 저장 + Square wrap y 보정 |

각 단계 시각 판정 게이트 + 본 사이클 회귀 검증 (cargo test + svg_snapshot + issue_418 + issue_501 + clippy).

## 위험 영역

- **41 파일 변경 + 51 commits** — 광범위 영역
- **layout.rs + typeset.rs 본 사이클 정정 영역 충돌 가능** ★
- **HWP 3.0 파서 검증 자료 광범위** 필요 — 단순 hwp3-sample.hwp 외 다양한 케이스 검증 필요
- **Task #460 의 보완4~5 + Stage 9** 가 본질 정정 영역 — 작성자도 다단계 정정 누적 — 안정화 영역
- 작성자 보고 cargo test 1016 passed (본 사이클 1102 미반영) — devel 머지 충돌 시 재검증 필수

## 작업지시자 결정 게이트

- 옵션 결정 (A 단계 분리 / B HWP 3.0 파서만 / C 한 번에 / D 정정 요청)
- HWP 3.0 파서 본질 자체 가치 + 본 사이클 회귀 위험 영역 분리 결정

## 다음 단계

1. 작업지시자 옵션 결정
2. 옵션 A/B 의 경우 — Stage 1 (Task #417 HWP 3.0 파서) cherry-pick
3. 검증 게이트 + 시각 판정
4. 후속 단계 cherry-pick + 시각 판정 (옵션 A) 또는 close (옵션 B)
5. 결과 보고서 + PR close
