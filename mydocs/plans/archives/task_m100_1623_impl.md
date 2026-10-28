# Task M100 #1623 구현 계획서

## 목표

#1623은 rhwp-studio UI 정합 문제로 등록되었지만, 작업지시자가 제공한 `대각선샘플` 기준으로 렌더러의 cellzone 대각선/중심선 처리 누락까지 포함한다.

구현은 렌더링 데이터 경로를 먼저 안정화한 뒤, 그 속성을 편집 UI에서 설정할 수 있도록 연결한다.

## Stage 1. 샘플 기반 회귀 가드

1. `samples/대각선샘플.hwpx`와 `samples/대각선샘플.hwp`의 표 구조를 확인하는 focused 테스트를 추가한다.
2. HWPX 파서가 다음 값을 보존하는지 확인한다.
   - `centerLine="VERTICAL"`
   - `slash`, `backSlash` 방향 코드
   - `Crooked`, `isCounter`
   - `cellzoneList`의 `borderFillIDRef`
3. 렌더 트리 또는 SVG 문자열에서 다음 선이 생성되는지 확인하는 테스트를 추가한다.
   - zone 단위 `centerLine`
   - zone 단위 굵은 X
   - 개별 셀 대각선

## Stage 2. cellzone 대각선/중심선 렌더링

1. `table_layout.rs`에 cellzone의 실제 bounding box를 계산하는 작은 헬퍼를 분리한다.
2. 기존 cellzone 배경 렌더링에서 계산하던 좌표를 재사용하거나 같은 기준으로 통일한다.
3. cellzone의 `borderFillIDRef`가 대각선 또는 중심선을 포함하면 `render_cell_diagonal`을 zone bounding box에 대해 호출한다.
4. zone 렌더링 순서는 배경 이후, 셀 콘텐츠/개별 셀 대각선과 충돌하지 않도록 Hancom 2024 샘플 기준으로 조정한다.

## Stage 3. 대각선 shape 코드 보강

1. `render_cell_diagonal`이 `Crooked`와 `isCounter` 비트를 무시하는 현재 동작을 보강한다.
2. HWP5 스펙의 bit 2..12 의미와 HWPX `slash/backSlash` 속성 이름을 기준으로, 기존 case를 유지하면서 Hancom 2024 샘플에서 과도한 fan line이 나오지 않도록 한다.
3. `THICK_SLIM`처럼 굵은 대각선 표현에 필요한 선 종류가 현재 단순 실선으로 떨어지는지 확인하고, 필요하면 `create_border_line_nodes`와 같은 분해 규칙을 대각선에도 적용한다.

## Stage 4. WASM CellProperties 확장

1. `getCellProperties` JSON에 대각선/중심선 필드를 추가한다.
   - `diagonalLine`
   - `diagonalSlash`
   - `diagonalBackSlash`
   - `diagonalWidth`
   - `diagonalColor`
   - `centerLine`
2. `setCellProperties`와 `create_border_fill_from_json`이 위 필드를 받아 `BorderFill.diagonal`, `BorderFill.center_line`, `attr` 비트를 구성하도록 한다.
3. 기존 테두리/배경만 보내는 호출에서는 대각선/중심선을 유지할지 초기화할지 정책을 명시한다. 기본은 기존 BorderFill을 기반으로 수정하여 사용자가 의도하지 않은 대각선 손실을 막는다.

## Stage 5. rhwp-studio 모달 UI

1. `CellBorderBgDialog`의 모달 외곽 크기를 고정하고 탭 내용은 내부 scroll/고정 레이아웃으로 처리한다.
2. 대각선 탭을 한컴 2024 구조에 맞춰 재배치한다.
   - 선 종류, 굵기, 색
   - `\ 대각선` 아이콘 그룹
   - `/ 대각선` 아이콘 그룹
   - `+ 중심선` 아이콘 그룹
   - 미리 보기
   - 선택된 셀/모든 셀 적용 범위
3. 아이콘 버튼은 텍스트만 있는 버튼보다 시각적 mini SVG를 사용한다.
4. 선택 상태 변경 시 미리보기를 즉시 갱신한다.
5. 확인 시 `setCellProperties`로 대각선/중심선 필드를 전달한다.

## Stage 6. 검증과 시각 판단 준비

1. Rust focused test를 먼저 통과시킨다.
2. SVG/PNG 산출물을 `output/poc/issue1623_diagonal_sample/` 아래에 정리한다.
3. WASM 빌드 후 rhwp-studio에서 모달 크기와 대각선 탭을 확인할 수 있게 한다.
4. 작업지시자에게 Hancom 2024 대비 시각 판단을 요청한다.

## 진행 결과 메모

- `cellzone`의 `borderFillIDRef` 대각선/중심선을 zone bbox 기준으로 렌더링하도록 보강했다.
- HWPX/HWP `대각선샘플`의 `CENTER_BELOW`/`ALL` 조합은 렌더 시 단일 slash/backSlash로 정규화해 한컴 2024 샘플과 맞췄다.
- 대각선 `THICK_SLIM` 등 이중선 계열은 단일 굵은 선이 아니라 평행선으로 렌더링한다.
- `getCellProperties`/`setCellProperties` JSON에 대각선/중심선 필드를 추가하고 기존 BorderFill 기반으로 수정해 테두리/배경 손실을 막았다.
- rhwp-studio `셀 테두리/배경` 모달은 고정 크기와 대각선 탭 아이콘 그룹, 미리보기를 추가했다.
- `cargo clippy --all-targets -- -D warnings` 통과를 위해 기존 테스트의 `manual_contains` 경고를 최소 보정했다.
- 남은 시각 판단 포인트: `Crooked=2` 계열의 꺾인 대각선 표현은 현재 저장 비트 보존과 기본 렌더 정합까지만 처리했다.

## 현재 검증

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `git diff --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --doc`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `wasm-pack build --target web --out-dir pkg`
- `cargo test --test svg_snapshot`

## 예상 수정 파일

- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/border_rendering.rs`
- `src/document_core/commands/table_ops.rs`
- `src/document_core/html_table_import.rs`
- `src/model/style.rs` 또는 관련 테스트
- `src/wasm_api/tests.rs` 또는 focused integration test
- `rhwp-studio/src/ui/cell-border-bg-dialog.ts`
- `rhwp-studio/src/styles/table-cell-props.css`
- `rhwp-studio/src/core/types.ts`

## 리스크

- cellzone 대각선은 개별 셀 대각선과 겹칠 수 있다. Hancom 2024 샘플 기준으로 그리기 순서를 조심해서 정해야 한다.
- HWP5 bit 8..12와 HWPX `Crooked`/`isCounter`의 실제 시각 의미는 스펙 문구만으로 부족할 수 있다. 샘플과 Hancom 2024 출력 비교를 우선한다.
- UI에서 대각선/중심선을 새 BorderFill로 만들 때 기존 배경/테두리 속성을 잃지 않아야 한다.
