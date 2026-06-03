# Task M100 #717 Stage 5 완료보고서

## 단계 목표

작업지시자 직접 테스트 중 확인된 후속 증상, 즉 `samples/exam_social.hwp` 1쪽 왼쪽 자료 표의 회색 헤더 내부표 빈 영역 클릭 시 커서가 입력 불가능한 위치로 이동하거나 입력이 보이지 않는 문제를 정정한다.

## 확인한 구조

해당 영역은 단순 표가 아니라 다음 중첩 구조다.

- 본문 문단 `0.1`
- 최외곽 `1x1` 표, `controlIndex=0`, `cellIndex=0`
- 최외곽 셀 첫 문단 내부의 `2x2` 내부표, `controlIndex=1`

따라서 내부표 셀을 클릭할 때는 `cellPath=[(0,0,0),(1,*,0)]` 형태의 전체 경로가 반환되어야 한다.

## 원인

Stage 1~4 정정 후에도 셀 bbox 기반 hit-test의 후속 분기에서 중첩 표의 `TableCell` bbox를 처리할 때 전체 `cellPath`를 보존하지 못했다.

특히 회색 헤더 내부표의 빈 영역 좌표 `(page=0, x=100, y=350)`은 내부표 `cellIndex=1`에 해당하지만, 기존 반환값은 이를 최외곽 표의 `cellIndex=1`처럼 반환했다. 최외곽 표는 `1x1`이라 `cellIndex=1`이 존재하지 않으므로 Studio의 입력 라우팅이 실패하거나 입력 결과가 보이지 않았다.

## 정정 내용

- `hit_test_native()`의 `RunInfo`와 `CellBboxInfo`에 소속 표 `RenderNode id`를 기록했다.
- 중첩 표의 `TableCell` bbox는 같은 표 `RenderNode id` 안의 TextRun `cell_context`를 템플릿으로 사용해 전체 `cellPath`를 복원한다.
- 셀 bbox 내부에서 가장 가까운 TextRun을 고를 때 최외곽 셀만 비교하지 않고 같은 표 `RenderNode id`와 최내곽 `cellIndex`를 함께 비교한다.
- TextRun이 없는 빈 셀 fallback도 복원된 전체 `cellPath`를 반환한다.
- 빈 셀 fallback caret 높이는 최소 `12px`로 보정해 극단적으로 낮은 행에서도 커서가 1px 수준으로 보이지 않는 문제를 완화했다.

## 회귀 테스트

`tests/issue_717_table_cell_hit_test.rs`에 다음 케이스를 추가했다.

- `issue_717_exam_social_nested_header_empty_area_returns_editable_path`
  - 좌표: `page=0, x=100.0, y=350.0`
  - 기대 경로: `[(0,0,0), (1,1,0)]`
  - 검증: 해당 경로로 `X` 삽입 후 `getTextInCellByPath()`에서 `X` 확인
  - 검증: 삽입 후 `getCursorRectByPath()` y 좌표가 회색 헤더 셀 범위 안에 유지

## 검증 결과

| 명령 | 결과 |
|------|------|
| `cargo test --test issue_717_table_cell_hit_test -- --nocapture` | 통과, 3 passed |
| `cargo test --test issue_595` | 통과, 5 passed |
| `cargo test --test issue_658_text_selection_rects` | 통과, 2 passed |
| `docker-compose --env-file .env.docker run --rm wasm` | 통과 |
| `cd rhwp-studio && npm run build` | 통과 |
| `cargo test --lib --release` | 통과, 1165 passed / 2 ignored |

## 상태

Stage 5 구현과 결정적 검증은 완료됐다. 작업지시자가 dev server에서 직접 새로고침 후 시각 확인할 수 있도록 WASM 산출물과 Studio build 산출물도 갱신했다.

## 후속 분리

작업지시자 시각 확인 결과, 회색 헤더 내부표의 극단적으로 낮은 빈 셀에 텍스트가 입력되지만 보이지 않는 현상이 남아 있음을 확인했다.

이 영역은 실제 파일 구조상 높이 약 `5.1px`의 장식/간격용 내부표 셀이고, 입력 자체는 현재 올바른 `cellPath`로 들어간다. 따라서 #717의 “클릭한 표가 아닌 다른 표/셀로 커서가 튀는 결함” 범위를 넘는 UX 정책 문제로 판단해 현재 작업에서는 다루지 않고 후속 작업으로 분리한다.
