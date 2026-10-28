# Task M100 #1633 Stage 7 작업 문서

## 목표

`셀 테두리/배경 - 하나의 셀처럼 적용`에서 대각선과 중심선이 동시에 적용되는 rhwp 동작을 한컴처럼 배타 적용으로 보정한다.

## 관찰

- 사용자가 rhwp에서 대각선/중심선을 함께 설정한 뒤 HWP로 저장해 한컴에서 열었을 때 중심선이 표시되지 않음을 확인했다.
- HWPX는 `borderFill@centerLine`에 `VERTICAL/HORIZONTAL/CROSS` 값을 저장하지만, HWP5 바이너리 스펙은 BorderFill attr `bit 13 = 중심선 유무`만 정의한다.
- 현재 구현은 HWPX 중심선 방향을 `CenterLine` enum으로 보존하고, HWP 저장 시 `center_line.hwp_attr_bits()`를 BorderFill attr에 OR 한다.
- HWP 저장 후 재파싱 회귀 테스트는 cellzone 대각선만 확인하고 중심선 보존 여부는 아직 확인하지 않는다.
- 사용자 저장본 `/Users/tsjang/Downloads/test.hwp` 확인 결과:
  - 전체 cellzone은 `zone row=0..7 col=0..9 bf=4`이고 BF#4는 `attr=0x0048`로 X 대각선이다.
  - 중심선/대각선을 추가 적용한 셀은 `cell#01 row=0 col=1 bf=5`이다.
  - BF#5는 `attr=0x20fc`, slash=7, backslash=7, `center_line=Cross`, diagonal type=1, width=0이다.
  - 즉 rhwp가 저장한 HWP에는 중심선 bit 13이 존재하지만 한컴 표시에서는 중심선처럼 보이지 않는다.
- Stage 7 최초 수정 후 사용자 재검증 저장본은 BF#5가 `attr=0x2548`이었다.
  - bit8/bit10은 HWP5에서 중심선 방향이 아니라 대각선 꺾은선 비트로도 쓰이므로, 한컴 표시가 더 깨졌다.
  - 따라서 HWP 바이너리 `CROSS` 중심선에 bit8/bit10을 추가하는 가설은 폐기한다.
- 사용자가 한컴에서는 대각선과 중심선을 동시에 적용할 수 없음을 확인했다.
  - 대각선이 선택된 상태에서는 중심선 버튼이 비활성화되고, 중심선이 선택된 상태에서는 대각선 버튼이 비활성화된다.
  - `samples/대각선샘플3.hwp` 비교군도 대각선 BorderFill과 중심선 BorderFill을 분리해 저장한다.
  - `samples/대각선샘플3.hwpx`에서도 중심선 BorderFill은 `centerLine`만 사용하고 slash/backSlash 방향은 `NONE`이다.
- 사용자가 `samples/대각선샘플4.{hwp,hwpx}` 대각선-only 비교군을 추가했다.
  - HWPX는 모든 BorderFill이 `centerLine="NONE"`이고 대각선 BorderFill만 slash/backSlash `CENTER`를 가진다.
  - HWP 바이너리도 대각선 BorderFill attr이 `0x0048`로 bit13 중심선 없이 slash/backSlash만 가진다.
- Stage 7 추가 수정 전 사용자 저장본 `/Users/tsjang/Downloads/test.hwp`와 `samples/대각선샘플4.hwp`는 모두 전체 표 cellzone 대각선과 첫 셀 개별 대각선 BorderFill을 함께 가진다.
  - 한컴은 전체 cellzone 대각선만 표시하고 첫 셀의 개별 대각선 조각은 표시하지 않는다.
  - rhwp는 cellzone 대각선을 zone bbox에 한 번 그린 뒤 셀 고유 대각선을 다시 그려 좌상단에 짧은 대각선 조각이 중복 표시됐다.
- 사용자가 WASM 갱신 후 기존 전체 X cellzone 위에서 선택 셀 중심선을 적용하면 중심선이 화면에 보이지 않음을 확인했다.
  - 원인은 중복 대각선 방지 로직이 cellzone 범위 안의 셀 고유 중심선까지 함께 숨긴 것이다.
  - 셀 고유 대각선은 숨기되, 셀 고유 중심선은 cellzone 대각선 위에 표시해야 한다.
- 사용자가 전체 X cellzone이 있는 표에서 중심선을 설정하면 표가 찌그러지고 큰 X가 표 밖으로 렌더되는 현상을 다시 확인했다.
  - 원인은 전체 cellzone X가 유지된 상태에서 모든 셀에 개별 중심선을 적용해도 렌더러가 기존 cellzone 대각선을 계속 그린 것이다.
  - cellzone 범위의 모든 그리드 셀이 개별 대각선/중심선 BorderFill로 덮였으면 기존 cellzone 대각선 렌더를 생략해야 한다.
- 사용자 저장본 `/Users/tsjang/Downloads/test.hwp`와 `samples/대각선샘플3.hwp` 비교 결과, 한컴에서 찌그러져 보이는 직접 원인은 표 객체 높이 저장값 차이다.
  - `test.hwp`: 8행×10열, `table.common.height=2256`, `cell.height=282`, 전체 cellzone BF#4, 첫 셀 BF#5.
  - `samples/대각선샘플3.hwp`: 10행×10열, `table.common.height=12820`, `cell.height=282`, 전체 cellzone BF#4, 첫 셀 BF#5.
  - 샘플3에서 크기 필드 없이 `setCellProperties`로 중심선만 적용해도 `table.common.height`가 `12820 -> 2820`으로 줄어드는 것을 확인했다.
  - 원인은 `set_cell_properties_native()`가 테두리/대각선/중심선만 바꾸는 경우에도 `table.update_ctrl_dimensions()`를 호출하고, 이 함수가 `cell.height` 합계로 `common.height`를 덮어쓰기 때문이다.
  - 한컴 HWP 실물은 `cell.height=282`와 더 큰 `table.common.height`가 공존하므로, 일반 속성 변경 중 무조건 재계산하면 표가 납작해진다.
- 한컴 UI는 대각선/역대각선/중심선 각 줄의 첫 버튼을 해제 버튼으로 제공한다.
  - rhwp도 같은 위치에 빈 사각형 해제 버튼을 노출해야 한다.
- 한컴에서는 `셀 테두리/배경 - 하나의 셀처럼 적용` 경로에서 중심선 자체가 비활성화된다.
  - rhwp도 as-one/cellzone 적용에서는 중심선을 UI와 WASM API 양쪽에서 비활성/무시해야 한다.

## 작업 범위

1. 대각선 탭 UI에서 대각선 그룹과 중심선 그룹을 상호 배타로 만든다.
2. `create_border_fill_from_json()`에서 JSON으로 모순 조합이 들어와도 한쪽만 저장되도록 정규화한다.
3. HWP/HWPX 직렬화에서 기존 모순 BorderFill도 저장 시 배타 조합으로 내보내도록 방어한다.
4. 조회와 export/reparse 회귀 테스트로 한컴식 배타 동작을 고정한다.
5. cellzone 대각선이 적용된 셀에서는 렌더러가 셀 고유 대각선을 중복 표시하지 않도록 보정한다.
6. cellzone 대각선 위에서도 선택 셀에 적용한 중심선은 렌더되도록 보정한다.
7. 대각선/역대각선/중심선 각 그룹에 한컴식 해제 버튼을 추가한다.
8. `하나의 셀처럼 적용`에서는 중심선을 선택/저장할 수 없도록 막는다.

## 검증 계획

- `cargo test --test issue_1623_cellzone_diagonal`
- `git diff --check`
- UI 확인은 사용자가 `wasm-pack build --target web --out-dir pkg` 후 7700에서 수행한다.

## 구현 결과

- `CenterLine`에 HWP 바이너리 저장 전용 `hwp_binary_attr_bits()`를 추가했다.
  - HWPX/IR용 `hwp_attr_bits()`는 기존처럼 유지한다.
  - HWP 바이너리 저장에서 `CROSS`는 bit13만 사용한다.
  - `VERTICAL/HORIZONTAL`은 기존 HWP/HWPX 호환 관찰대로 각각 bit8/bit10 보조 비트를 유지한다.
- `셀 테두리/배경` 대각선 탭은 대각선이 선택되면 중심선 버튼을, 중심선이 선택되면 대각선 버튼을 비활성화한다.
- `create_border_fill_from_json()`은 `centerLine != NONE`이면 slash/backSlash 방향을 0으로 정규화한다.
- `build_border_fill_json_by_id()`는 기존 모순 BorderFill도 UI 조회 시 중심선 우선으로 배타 표시한다.
- HWP/HWPX 직렬화는 중심선이 있는 BorderFill에서 대각선 방향 비트를 제거하고 저장한다.
- HWP serializer 단위 테스트를 추가해 CROSS 중심선이 기존 대각선 방향 비트를 함께 저장하지 않음을 고정했다.
- #1633 HWP export/reparse 테스트에 centerLine CROSS와 slash/backSlash 배타 검증을 추가했다.
- `대각선샘플3/4` HWP/HWPX 회귀 테스트를 추가해 분리 저장과 대각선-only 파일이 유지되는지 검증한다.
- 표 렌더러는 대각선/중심선이 있는 cellzone 범위를 표시용 커버 맵으로 기록하고, 해당 범위 안의 셀 고유 대각선은 생략한다.
  - 표 배경, 셀 배경, 일반 테두리, 속성 조회, 저장 모델은 변경하지 않는다.
  - `대각선샘플4` 렌더 회귀 테스트를 추가해 전체 cellzone X는 유지하면서 한 셀 크기의 짧은 대각선 선분이 나오지 않도록 고정했다.
- 선택 셀에 중심선이 직접 적용된 경우에는 cellzone 대각선 범위 안이어도 셀 고유 중심선을 렌더한다.
  - 기존 전체 X cellzone 위에 선택 셀 CROSS 중심선을 적용하는 회귀 테스트를 추가했다.
  - 같은 범위의 cellzone을 중심선으로 대체하는 회귀 테스트도 추가했다.
- 기존 cellzone X 위에서 모든 셀에 중심선을 직접 적용한 경우에는 이전 cellzone 대각선을 렌더하지 않는다.
  - 전체 cellzone 범위가 셀 고유 대각선/중심선으로 완전히 덮였는지 확인한 뒤, 완전히 덮인 경우 cellzone 대각선 SVG를 생략한다.
  - 표 높이가 찌그러지고 큰 X가 표 밖으로 뻗는 회귀 케이스를 테스트로 고정했다.
- `set_cell_properties_native()`는 실제 셀 폭/높이가 바뀐 경우에만 `table.update_ctrl_dimensions()`를 호출한다.
  - 크기 변경 없는 테두리/배경/대각선/중심선 편집에서는 한컴 저장본의 `table.common.height`와 `raw_ctrl_data` 높이를 보존한다.
  - `samples/대각선샘플3.hwp`에서 중심선만 적용한 뒤 HWP로 다시 저장해도 표 객체 높이가 `cell.height` 합계로 줄지 않도록 회귀 테스트를 추가했다.
- 대각선 탭의 `\ 대각선`, `/ 대각선`, `+ 중심선` 각 버튼 그룹 첫 칸에 빈 사각형 해제 버튼을 추가했다.
  - 해제 버튼은 해당 그룹 값만 `0` 또는 `NONE`으로 돌린다.
  - 대각선과 중심선의 상호 비활성화 규칙은 유지한다.
- `applyMode === "asOne"`이면 대각선 탭 중심선 그룹 전체를 비활성화하고, 내부 상태도 항상 `centerLine=NONE`으로 정규화한다.
- `set_cell_zone_properties_native()`는 직접 호출로 `centerLine`이 들어와도 `NONE`으로 덮어쓴 뒤 BorderFill을 생성한다.
  - as-one/cellzone 중심선 입력은 저장되지 않고, 선택 셀 중심선(`set_cell_properties`)은 계속 허용한다.
- `rhwp convert /Users/tsjang/Downloads/test.hwp ...` 단순 재저장은 원본 `/DocInfo` raw_stream 보존 때문에 BF#5가 그대로 `0x20fc`였다.
  - 실제 UI 편집 경로는 `create_border_fill_from_json()`에서 `doc_info.raw_stream_dirty=true`를 설정하므로 DocInfo가 재직렬화되어 이번 변경이 적용된다.

## 검증 결과

- PASS: `cargo test --test issue_1623_cellzone_diagonal`
  - 15개 테스트 통과. `대각선샘플3` 분리 저장, `대각선샘플4` 대각선-only, cellzone/셀 대각선 중복 방지, 선택 셀 중심선 렌더, as-one 중심선 비활성화, 전체 cellzone X 위 전체 셀 중심선 적용 시 기존 X 억제, 크기 변경 없는 셀 테두리 편집의 표 객체 높이 보존 검증 포함.
- PASS: `cargo test --test issue_1623_cellzone_diagonal issue_1633_all_cells_centerline_suppresses_old_cellzone_diagonal_render -- --nocapture`
  - 전체 cellzone X 위에서 모든 셀 중심선을 적용하는 찌그러짐 재현 케이스가 통과했다.
- PASS: `cargo test --test issue_1623_cellzone_diagonal issue_1633_cell_border_edit_preserves_table_object_height -- --nocapture`
  - `samples/대각선샘플3.hwp`에서 중심선만 적용해도 표 객체 높이가 보존됨을 확인했다.
- PASS: `cargo test --lib center_bit_only`
- PASS: `npm --prefix rhwp-studio run build`
  - Vite의 CanvasKit externalize/chunk size 경고는 기존 번들 구조 경고이며 TypeScript 빌드는 통과했다.
- PASS: `cargo fmt --check`
- PASS: `git diff --check`
- PASS: `wasm-pack build --target web --out-dir pkg`
  - 빌드는 완료됐다. 현재 Codex 실행 환경에서는 `wasm-bindgen` prebuilt target 인식 경고 후 cargo install fallback이 발생한다.
- PASS: `cargo run --quiet --bin rhwp -- export-svg samples/대각선샘플4.hwp -o /tmp/rhwp_diag_stage7_after -p 0`
  - 수정 전 SVG에는 첫 셀 크기의 짧은 대각선 2개가 있었으나, 수정 후에는 전체 cellzone X 2개와 표 격자만 남는다.
- PASS: `cargo run --quiet --bin rhwp -- export-svg /Users/tsjang/Downloads/test.hwp -o /tmp/rhwp_diag_stage7_test_hwp_after -p 0`
  - 사용자 저장본도 전체 cellzone X와 표 격자만 렌더되고 좌상단 짧은 대각선 조각은 나오지 않는다.
- PASS: `curl -I --max-time 5 http://127.0.0.1:7700/`
  - Vite dev server 응답 200 OK 확인.
- 제한: 인앱 브라우저 검증
  - Browser runtime의 `agent.browsers.list()`가 빈 배열을 반환해 인앱 브라우저 스크린샷 검증은 수행하지 못했다.
