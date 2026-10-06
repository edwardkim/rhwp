# Task 1282 Stage 11 - 쪽 영역 안으로 제한 렌더 구현

## 목적

- 한컴 도움말의 `쪽 영역 안으로 제한` 의미를 기준으로 새 샘플을 PDF oracle과 동일하게 렌더링한다.
- 대상 샘플:
  - `samples/ta-pic-001-r-쪽영역안제한.hwp`
  - `samples/ta-pic-001-r-쪽영역안제한.hwpx`
  - `samples/ta-pic-001-r-쪽영역안제한no.hwp`
  - `samples/ta-pic-001-r-쪽영역안제한no.hwpx`

## 공식 의미

- 세로 위치 기준이 `문단`일 때 적용된다.
- 개체의 세로 위치가 쪽 영역 밖으로 나가면 개체를 다음 쪽으로 넘긴다.
- 쪽 영역은 편집 가능한 위/아래 끝, 즉 편집 용지의 위/아래 여백과 머리말/꼬리말 공간을 뺀 영역이다.
- HWP5 `CommonObjAttr bit 13`, HWPX `<hp:pos flowWithText>`가 같은 값이다.
- rhwp UI/명령 JSON에서는 한컴 UI 의미에 맞춰 `restrictInPage`로 노출한다.
- 이 값이 켜지면 한컴 UI에서 `서로 겹침 허용`은 비활성/false 취급된다.

## 원인 분석

- HWPX picture serializer가 `flowWithText`/`allowOverlap`을 실제 `CommonObjAttr` 값으로 내보내지 않았다.
- picture 속성 UI와 명령 JSON에 `쪽 영역 안으로 제한`/`서로 겹침 허용` 값이 빠져 있어 편집 후 보존되지 않았다.
- 표 셀 내부 `TopAndBottom` 그림의 높이 계산이 `flow_with_text` 여부와 무관하게 행/셀 높이에 반영되었다.
- `flow_with_text=false`인 표 안 그림은 한컴처럼 셀 높이를 밀지 않고 기존 문단 기준 위치에 떠 있어야 하는데, rhwp는 셀 클립 아래에 넣어 잘랐다.
- 전체 표 렌더 경로에서 TAC 표의 문단 `line_seg.vertical_pos` 보정이 빠져, 제한 해제 샘플의 표 위치가 PDF보다 위로 올라갔다.

## 구현

- HWPX picture serializer에서 `flowWithText`와 `allowOverlap`을 `CommonObjAttr` 값으로 직렬화한다.
- `picture-props-dialog`와 document command JSON에 `restrictInPage`, `allowOverlap`을 추가했다.
- `restrictInPage=true`이면 `allowOverlap=false`로 강제하고 UI에서도 겹침 허용 항목을 비활성화한다.
- 표/height 측정 공통 계산에서 `TopAndBottom` 비-TAC 그림은 `flow_with_text=true`일 때만 세로 오프셋+높이를 행/셀 흐름에 반영한다.
- `flow_with_text=false`, `VertRelTo=Para`, `TopAndBottom` 그림은 셀 클립 대상에서 분리해 표 노드 기준으로 렌더한다.
- TAC 표가 제한 해제 그림 때문에 행 높이를 줄일 때도 문단 `line_seg.vertical_pos`를 표 시작 y에 반영하도록 보정했다.

## PDF 대비 위치 검증

최종 PNG 비흰색 bbox 기준이다.

| 샘플 | PDF bbox | rhwp bbox | 판단 |
|------|----------|-----------|------|
| `쪽영역안제한` | `(56,132)-(691,758)` | `(56,132)-(692,760)` | PDF와 거의 동일 |
| `쪽영역안제한no` | `(56,277)-(691,987)` | `(56,279)-(692,989)` | PDF와 거의 동일 |

생성 자료:

- `mydocs/report/assets/task_m100_1282_stage11/comparison_restrict.png`
- `mydocs/report/assets/task_m100_1282_stage11/comparison_no_restrict.png`
- `mydocs/report/assets/task_m100_1282_stage11/rhwp_restrict/ta-pic-001-r-쪽영역안제한.png`
- `mydocs/report/assets/task_m100_1282_stage11/rhwp_no_restrict/ta-pic-001-r-쪽영역안제한no.png`
- `mydocs/report/assets/task_m100_1282_stage11/tree_restrict/render_tree_001.json`
- `mydocs/report/assets/task_m100_1282_stage11/tree_no_restrict/render_tree_001.json`

## 검증

- `cargo fmt --check`
- `cargo test --test issue_1282_rotated_cell_picture_resize`
- `wasm-pack build --target web --out-dir pkg`
- `npm run build` (`rhwp-studio`)
- `cargo build --release --features native-skia`
- `node e2e/table-picture-resize-1282.test.mjs --mode=headless`

## 상태

- 자동 검증 완료.
- 작업지시자 시각 판단 대기.
