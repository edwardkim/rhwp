# Task M100 #717 최종 결과보고서

## 작업 개요

- 이슈: #717 `rhwp-studio: 표 셀 빈 영역 클릭 시 커서가 다른 위치로 이동`
- 브랜치: `local/task717`
- 대상 파일: `src/document_core/queries/cursor_rect.rs`
- 대상 문서: `samples/exam_social.hwp`

## 문제 원인

`hit_test_native()`가 셀 bbox 메타를 보완할 때 `cell_index`만으로 첫 TextRun을 찾아 `section_index/parent_para_index/control_index`를 덮어썼다.

HWP 문서의 여러 표와 바탕쪽 표는 모두 `cell_index=0` 같은 낮은 셀 인덱스를 반복 사용한다. 따라서 클릭 좌표는 `s0:pi=1 ci=0` 자료 표 내부였지만, 실제 결과는 하단 번호 표 쪽 컨텍스트인 `parentParaIndex=0/controlIndex=1`, `cursorRect.y=1393.7`로 반환됐다.

## 정정 내용

- Table 노드에서 이미 채워진 셀 bbox 메타는 권위값으로 유지한다.
- 메타가 없는 셀 bbox만 TextRun으로 보완한다.
- 보완 시 TextRun bbox가 셀 bbox 내부에 있을 때만 사용한다.
- 클릭 좌표가 포함되는 셀 bbox 후보는 `has_meta`가 있는 셀로 제한한다.
- 여러 bbox가 동시에 hit되면 면적이 가장 작은 셀을 선택한다.
- 후속 정정: 중첩 표 내부 `TableCell` bbox가 전체 `cellPath`를 잃지 않도록 소속 표 `RenderNode id`를 기준으로 TextRun `cell_context`를 복원한다.
- 후속 정정: 내부표 빈 영역 클릭 시 최외곽 셀만 비교하지 않고 같은 표 `RenderNode id`와 최내곽 `cellIndex`를 함께 비교한다.
- 후속 정정: TextRun이 없는 빈 셀 fallback도 복원된 전체 `cellPath`를 반환하고, caret 높이를 최소 `12px`로 보정한다.

## 회귀 테스트

`tests/issue_717_table_cell_hit_test.rs`를 추가했다.

검증 좌표:

- 자료 표 제목 행 빈 영역: `page=0, x=191.0, y=356.0`
- 자료 표 회색 헤더 내부표 빈 영역: `page=0, x=100.0, y=350.0`
- `<보기>` 표 빈 영역: `page=0, x=110.0, y=865.0`

각 테스트는 반환 컨텍스트와 caret y 좌표가 클릭한 표 bbox 내부에 머무는지 확인한다. 회색 헤더 내부표 케이스는 `cellPath=[(0,0,0),(1,1,0)]`를 요구하고, 해당 경로로 실제 텍스트 삽입과 삽입 후 cursor rect 조회가 가능한지도 검증한다.

## 검증 결과

| 명령 | 결과 |
|------|------|
| `cargo test --test issue_717_table_cell_hit_test -- --nocapture` | 통과, 3 passed |
| `cargo test --test issue_595` | 통과, 5 passed |
| `cargo test --test issue_658_text_selection_rects` | 통과, 2 passed |
| `cargo test --lib --release` | 통과, 1165 passed / 2 ignored |
| `docker-compose --env-file .env.docker run --rm wasm` | 통과 |
| `cd rhwp-studio && npm run build` | 통과 |

## 산출물 및 상태

- `pkg/` WASM 산출물 생성 완료.
- `rhwp-studio` production build 완료.
- `mydocs/orders/20260508.md`의 #717 상태를 완료로 갱신.

## 잔여 확인

작업지시자 시각 판정이 남아 있다. 코드 레벨 재현/회귀 테스트와 WASM/웹 빌드 검증은 모두 통과했다. dev server는 `http://localhost:7700/`에서 실행 중이며, 브라우저 새로고침 후 새 WASM 산출물로 직접 확인 가능하다.

## 후속 분리

작업지시자 시각 확인에서 회색 헤더 내부표의 극단적으로 낮은 빈 셀에 텍스트가 입력되지만 보이지 않는 현상이 확인됐다. 해당 셀은 실제 파일 구조상 높이 약 `5.1px`의 장식/간격용 내부표 셀이며, 입력 경로 자체는 올바른 `cellPath`로 들어간다.

따라서 이 문제는 #717의 커서 이탈 결함 범위를 넘는 “초저높이 빈 셀 편집 진입/표 선택 UX 정책” 후속 작업으로 분리한다.
