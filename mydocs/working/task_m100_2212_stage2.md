# Task M100 #2212 — 2·3단계 완료 보고: 인라인 TAC 중첩 표 cell_context 미전파

- 이슈: #2212 / 브랜치: `local/task2212` / 작성일: 2026-07-12

## 2단계 — 이분 판정: 가설 A 확정

렌더 트리 실덤프: ppi=5 컨텍스트가 외곽 셀 1단 경로 11종뿐 — 내부 18×9 표
TextRun에 2단 경로 부재. 귀속: 셀 안 **인라인 TAC 표**는 run_tacs 경로
([paragraph_layout.rs:4907](../../src/renderer/layout/paragraph_layout.rs#L4907))
로 렌더되는데, 이 호출이 `cell_context=None, depth=0` — table_layout의 중첩
분기(:3475)는 올바르게 경로를 확장하지만 이 경로만 누락. 부수: 조회 API의
Err→JsValue 변환이 네이티브에서 abort함을 확인(진단 시 유의점으로 기록).

## 3단계 — 정정 + 게이트

run_tacs 인라인 TAC 표 렌더에 외곽 셀 경로를 확장한 2단 cell_context 전달
(+depth 1) — table_layout 중첩 분기와 동일 규칙. 내부 entry의 cell/cp는
layout_table 셀 루프가 채운다.

| 검증 | 결과 |
|------|------|
| 2단 경로 기록 | 11종 → **101종** (내부 셀 전체) |
| `get_table_cell_bboxes_by_path` (실패 경로 그대로) | Err → **Ok** (내부 셀 48개 bbox) |
| 렌더 픽셀 불변 | 주보 4페이지 SVG **바이트 동일** (patch 왕복 실측 — 조회 전용 정정) |
| 표적 테스트 신설 `tests/issue_2212_nested_cell_path_bbox.rs` | 수정 전 FAILED 실증 |
| fmt/clippy/전수 `--no-fail-fast` | 통과/0/**3,050/0** |
| OVR 3샘플 | 회귀 0건 |

## 4단계 판정 기준

렌더 불변이므로 시각 판정 대신 **studio 실사용**: WASM 빌드 후 주보 p1 좌측 단
표 영역 드래그/키 이동 시 `updateCellSelection` 예외 소멸 + 셀 선택 UI 동작.
