# Task 1282 Stage 1 — 기준값과 재현 범위 확정

## 목적

`ta-pic-001-r` 샘플에서 회전된 표 안 그림의 현재 저장 계약과 조작 후 실패 지점을 분리한다.

이번 단계는 코드 수정 자체보다 다음 기준을 확정하는 데 둔다.

- 셀 내부 그림의 경로와 현재 크기 계약
- `setCellPicturePropertiesByPath` 적용 후 셀/표 높이가 따라오는지 여부
- Rust 회귀 테스트로 먼저 고정할 최소 실패 조건

## 선행 기준

PR #1279에서 이미 고정한 그림 계약:

- 대상 샘플: `samples/ta-pic-001-r.hwp`, `samples/hwpx/ta-pic-001-r.hwpx`
- 대상 셀 내부 그림 경로: `section=0`, `parentPara=0`, `cellPath=[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]`, `innerControlIdx=0`
- 회전 각도: `34`
- 회전 후 bbox: `common.width=18425`, `common.height=18160`
- 회전 전 표시 크기: `current_width=13668`, `current_height=12686`
- rotate-image storage bit: `flip & 0x0008_0000`
- HWP 저장 시 rendering matrix 필요

위 값은 `tests/issue_1279_picture_rotation_save.rs`가 이미 검증한다.

## 확인할 실패 조건

Issue #1282의 첫 red test는 다음 조건을 목표로 한다.

1. `samples/ta-pic-001-r.hwp`를 로드한다.
2. 같은 셀 내부 그림에 `set_cell_picture_properties_by_path_native`로 큰 `height` 또는 회전 변경을 적용한다.
3. 그림의 `common.height`가 증가했는데도 소유 `cell.height`와 `table.common.height`가 충분히 증가하지 않으면 실패로 본다.
4. 셀 높이는 shrink 조작에서는 자동 감소시키지 않는다. 이번 stage의 자동 조정 방향은 증가만이다.

## Red test 결과

신규 테스트:

```text
cargo test --test issue_1282_rotated_cell_picture_resize -- --nocapture
```

초기 실패:

```text
owner cell must grow to contain resized rotated picture:
cell.height=17476, required=30282,
pic.vertOffset=0, pic.height=30000, pad=(141, 141)
```

판정:

- 그림 bbox height는 `30000`으로 증가했다.
- 소유 cell height는 기존 `17476`에 남아 있었다.
- 필요 높이는 `picture.vertOffset + picture.common.height + padding.top + padding.bottom = 30282`다.
- 따라서 `set_cell_picture_properties_by_path_native` 이후 소유 cell/table 높이 증가가 필요하다.

## 구현 후보

`set_cell_picture_properties_by_path_native`는 현재 picture만 수정한 뒤 section recompose/paginate로 넘어간다.
따라서 picture 변경 직후, path의 마지막 cell을 소유한 outer table에서 필요 높이를 계산해 다음 값을 보정하는 후보를 둔다.

- `cell.height`
- `table.common.height`
- `table.raw_ctrl_data`의 common height

`Table::update_ctrl_dimensions()`는 이미 셀 높이 변경 후 `raw_ctrl_data`와 `common.height`를 같이 갱신하므로 이 경로를 재사용하는 쪽이 안전하다.

## 다음 작업

- 신규 Rust 테스트 `tests/issue_1282_rotated_cell_picture_resize.rs` 작성
- 현재 실패값을 확인한 뒤 Stage 2에서 최소 보정 구현
