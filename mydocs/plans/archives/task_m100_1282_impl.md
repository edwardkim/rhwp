# Task 1282 구현 계획서

## 구현 방향

이번 작업은 회전 그림 저장 계약을 새로 바꾸는 작업이 아니다.
PR #1279에서 고정한 `common bbox` / `curSz` / rendering matrix 계약을 유지하면서, 셀 내부 편집 조작 후 표 모델이 따라오도록 보강한다.

## 후보 설계

### 1. 셀 내부 picture 변경 후 소유 셀 높이 보정

대상:

- `src/document_core/commands/object_ops.rs`
- `src/model/table.rs`

구상:

1. `set_cell_picture_properties_by_path_native`에서 cell path를 해석한 뒤, 변경 대상 picture와 소유 cell 정보를 함께 얻는다.
2. `apply_picture_props_inner` 적용 후 picture의 회전 외곽 높이를 기준으로 필요 cell height를 계산한다.
3. 필요 높이가 현재 cell.height보다 큰 경우에만 cell.height를 증가시킨다.
4. table 전체에는 기존 `Table::update_ctrl_dimensions()`를 호출해 `raw_ctrl_data`와 `common.height`를 같이 갱신한다.

주의:

- shrink 조작에서는 셀 높이를 자동으로 줄이지 않는다. 한컴식 자동 증가 후 사용자가 명시적으로 줄이는 흐름과 충돌할 수 있기 때문이다.
- 병합 셀(`row_span > 1`)은 첫 단계에서는 보수적으로 다룬다. 필요하면 span 전체 높이 합과 비교해 후속 단계로 분리한다.
- 셀 padding과 paragraph line height를 무시하지 않는다. 필요 높이는 `picture bbox + cell padding`을 최소 기준으로 삼는다.

### 2. 회전 그림 resize 계산 정리

대상:

- `rhwp-studio/src/engine/input-handler-picture.ts`
- `rhwp-studio/src/engine/command.ts`

구상:

1. 드래그 시작 상태에 저장 속성의 원본 `common.width/height`, `current_width/current_height`, offset을 명확히 보존한다.
2. `calcResizedBboxRotated`는 화면 page bbox 계산만 담당하게 유지한다.
3. `updatePictureResizeDrag`/`finishPictureResizeDrag`는 저장 속성 적용 시 `common bbox`를 바꾸되, 회전 그림의 실제 표시 크기 스케일은 Rust setter 계약에 맡긴다.
4. 예비 테두리와 실제 이미지가 다른 기준점으로 움직이지 않도록 offset delta 계산을 Stage 1 실측값으로 검증한다.

주의:

- 현재 PR #1279에서 고친 `cellPath` 보존과 nested offset delta 계산은 유지한다.
- 본문 그림, 글상자 내부 그림, shape/line/group 조작 경로를 회귀시키지 않는다.

## 테스트 계획

### Rust

신규 테스트 후보:

- `tests/issue_1282_rotated_cell_picture_resize.rs`

검증 항목:

- `samples/ta-pic-001-r.hwp` 로드
- `set_cell_picture_properties_by_path_native`로 회전된 셀 내부 그림 height/rotation 변경
- picture `common.width/height`, `current_width/current_height`, rendering matrix 계약 보존
- 소유 cell.height와 table.common.height가 필요 높이 이상으로 증가
- HWP export 후 reparse에서도 동일 계약 유지

기존 회귀:

- `cargo test --test issue_1279_picture_rotation_save`
- 신규 `cargo test --test issue_1282_rotated_cell_picture_resize`

### Studio

E2E 후보:

- `rhwp-studio/e2e/table-picture-ops-1282.test.mjs`

검증 항목:

- `ta-pic-001-r.hwp` 로드
- 셀 내부 회전 picture 선택
- resize drag 후 속성 조회
- 셀/표 bbox가 그림 bbox를 포함하는지 확인
- undo/redo 후 bbox와 속성 회복 확인

### 시각 검토

- headless screenshot 또는 사용 중인 Vite dev server에서 수동 시각 판정
- 한컴편집기 기준: 회전된 표 안 그림 조작 후 셀 높이 자동 증가와 중심 안정성

## 구현 순서

1. Stage 1 문서 작성: 현재 샘플 구조와 재현 baseline 기록
2. Rust test red 작성: 셀 높이 자동 증가 실패를 먼저 고정
3. Rust 모델 보강: picture 변경 후 소유 cell/table height 갱신
4. Studio E2E baseline 작성: 리사이즈 중심/크기 불안정 재현
5. Studio 드래그 계산 보정
6. WASM build 및 Studio 검증
7. 단계 보고서와 최종 보고서 작성

## 승인 대기

본 구현 계획은 코드 수정 전 검토용이다. 작업지시자 승인 후 Stage 1 문서와 red test부터 진행한다.
