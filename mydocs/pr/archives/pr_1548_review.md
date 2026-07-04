# PR #1548 처리 보고서 — visible-host co-anchored float 표 exclusion 침범 회귀 수정

- PR: https://github.com/edwardkim/rhwp/pull/1548
- 제목: `Task #1535: visible-host co-anchored float 표가 선행 float 점유영역을 침범하던 회귀 수정`
- 작성자: kkyu8925 (**rhwp 첫 기여**)
- 연결: Closes #1535 (bug/layout/regression/rendering), PR #1518 후속
- base ← head: `devel` ← `kkyu8925:fix/1535-coanchored-float-exclusion`
- 처리일: 2026-06-27

## 1. 처리 결정

**admin merge.** 같은 visible host 문단에 co-anchored 된 para-relative TopAndBottom float 표가
여러 개일 때, 후행 float 표가 선행 float 표의 점유 세로 영역(`visible_float_exclusions`)을 무시하고
겹쳐 그려지던 회귀(#1518 후속)를 수정한다. fork PR 이라 GitHub CI 가 자동 실행되지 않아
로컬 전체 회귀로 대체 검증했고, 전부 통과 + 충돌 0건.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `src/renderer/layout.rs` | visible-host float 표 배치에서 활성 `visible_float_exclusions` consult (~24줄) |
| `tests/issue_1535.rs` | 회귀 가드 (red→green) |
| `samples/hwpx/issue1535_coanchored_float_exclusion.hwpx` | 합성 fixture (양수 offset 표 2개) |
| `mydocs/plans·report/task_m100_1535*` | 계획/보고서 |

## 3. 코드 검토

- 표 자연 상단(`para_y + outer_margin + max(v_offset, 0)`)이 활성 exclusion 영역 안에서
  시작하면 그 영역 하단으로 `table_y_start` 를 끌어올린다. `compute_table_y_position` 의
  `raw_y.max(y_start)` 클램프로 시작점 상향만으로 표가 영역 아래로 밀린다.
- 문단 텍스트 jump 로직과 동형(같은 함수 내 exclusion 소비부와 일관).
- 변경은 layout.rs 단일 지점. HWP3 전용 분기 아님. 단일 양수 offset(#1510)은 자연 상단이
  형제 zone 밖이라 무영향.

## 4. 검증 (로컬 — CI 대체)

| 항목 | 결과 |
|---|---|
| GitHub CI | no checks (fork PR 자동 미실행) → 로컬 전체 회귀로 대체 |
| 충돌 시뮬레이션 (`merge-tree`) | 0건 |
| 신규 `issue_1535` | **red→green 확정** (수정 revert 시 FAILED, 적용 시 pass) |
| 회귀 `issue_1510` | 4/4 (`page_count==1` 유지) |
| **golden SVG 불변** (`svg_snapshot`) | 8/8 (기존 렌더 무변경) |
| `hwpx_roundtrip_baseline` | 4/4 |
| **전체 `cargo test --tests`** | **FAILED 0건** (lib 1937 passed, 통합 전부 통과) |
| `cargo fmt --check` / `cargo clippy --lib` | clean |

## 5. 의의

- 페이지네이션 영향 없음(golden 불변 + #1510 page_count==1 유지). "같은 visible host 문단에
  양수 offset float 표 2개 이상" 조합에만 작용.
- 작업일지류 양식에서 헤더 셀 텍스트 겹침 해소.

## 6. 첫 기여 메모

kkyu8925 의 rhwp 첫 기여. CONTRIBUTING 가이드(페이지네이션 영향 명시, 검증 환경 명시,
golden 불변, red→green fixture)를 충실히 따른 고품질 PR. 환영 코멘트와 함께 merge.

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1548_review.md`
