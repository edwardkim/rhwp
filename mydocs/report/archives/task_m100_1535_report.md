# Task #1535 결과보고서 — co-anchored float 표 겹침 (선행 float 점유영역 미반영)

## 이슈

- GitHub #1535 (라벨 bug, #1518 후속): visible host 문단에 co-anchored float 표 다수 시 인접 셀 텍스트 겹침.

## 근본 원인 (계측 확정)

`src/renderer/layout.rs` 의 visible host 문단 float 표 배치(`is_current_visible_para_float` 경로,
`table_y_start` 계산)가 활성 `visible_float_exclusions`(선행 float 표가 점유한 세로 영역)를 consult하지
않았다. 문단 텍스트는 같은 함수에서 exclusion 아래로 밀려나지만(현 ~3961-3970), float 표는 exclusion 을
push(현 ~5698)만 하고 consult 하지 않아, 뒤에 배치되는 float 표가 앞 float 표 위에 겹쳐 그려졌다.

- 표 렌더 치수는 #1518 전후 동일 → 행 높이 압축은 원인 아님.
- #1510 단일 문단·단일 양수 offset 표는 자연 상단이 형제 zone 밖이라 영향 없음(기존 테스트 통과 유지).
- 트리거: 같은 visible host 문단에 양수 vertical_offset co-anchored TopAndBottom float 표가 2개 이상.

## 수정

`layout.rs` visible-host float 표 배치에서, 표의 자연 상단(`para_y + outer_margin + max(v_offset, 0)`)이
활성 exclusion 영역 안에서 시작하면 그 영역 하단으로 `table_y_start` 를 끌어올린다.
`compute_table_y_position` 이 `raw_y.max(y_start)` 로 클램프하므로 시작점 상향만으로 표가 영역 아래로
밀린다(문단 텍스트의 jump 로직과 동일 의미). 약 20줄, 코어 레이아웃 외 다른 모듈 변경 없음.

## 산출물

- 수정: `src/renderer/layout.rs`
- 회귀 fixture: `samples/hwpx/issue1535_coanchored_float_exclusion.hwpx` — issue1510 HWPX 기반(Clone and
  Narrow). 같은 host 문단에 양수 offset 표 A(16996)·B(18000), B 선언 위치가 A 점유영역 안.
- 회귀 테스트: `tests/issue_1535.rs` — B 표 상단이 A 표 하단 이상(겹침 금지)임을 렌더트리에서 단언.
  fix 제거 시 실패(b_top≈376 ∈ A[362,437]), fix 적용 시 통과(b_top≈437).
- CHANGELOG.md / CHANGELOG_EN.md `[Unreleased]` 항목.

## 검증

- 전체 `cargo test`: 실패 0 (`test result: ok` 148 그룹).
- `tests/visual_roundtrip_baseline.rs`(시각 회귀): 통과.
- `tests/issue_1510.rs`: 4/4 통과(회귀 가드).
- `tests/issue_1535.rs`: red→green.
- `cargo fmt --all -- --check`: clean. `cargo clippy -- -D warnings`: clean.
- 작업지시자 제공 실파일(비공개) 5종: 실제 cross-cell 겹침 0 + 1페이지 압축 유지(로컬 수동 확인,
  비공개라 fixture 미커밋).

## 비고

진단 과정에서 1차로 `layout.rs` 양수 분기, 2차로 export 경로를 의심했으나, 동일 문서의 내부 좌표 표현이
여럿(render-tree bbox 단위 ≠ SVG/PDF px)이라 생긴 단위 혼동이었다. 최종은 pre/devel 동일 좌표 계측 +
exclusion 활성 여부 계측으로 확정. 상세 과정은 `mydocs/plans/task_m100_1535.md`.
