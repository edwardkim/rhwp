# Task M100 #1633 Stage 5 작업 문서

## 목표

`셀 테두리/배경 - 하나의 셀처럼 적용`으로 생성된 cellzone 대각선이 `다른 이름으로 저장`한 HWP에서도 한컴에서 동일하게 표시되도록 HWP TABLE 레코드 직렬화를 보정한다.

## 관찰

- 기대 파일 `samples/대각선샘플2.hwp`는 10열 8줄 표에 `zone[0] row=0..7 col=0..9 bf=4`가 존재한다.
- 오류 파일 `samples/대각선샘플2-오류.hwp`는 동일하게 10열 8줄이지만 TABLE 레코드에 zone이 없다.
- `hwp5-inventory-diff` 기준 TABLE record size가 기대 파일 48, 오류 파일 38로 10바이트 차이가 난다.
- 현재 `serialize_table_record`는 `table.zones`를 쓰지 않고 `raw_table_record_extra`만 복원한다.
- HWP TABLE zone 하나는 `nZones` 2바이트 + `TableZone` 10바이트 중 zone 엔트리 10바이트이며, 오류 파일은 `nZones=0`만 있는 상태다.

## 작업 범위

1. HWP serializer의 TABLE 레코드에 `table.zones`를 `nZones + TableZone[]` 형식으로 직렬화한다.
2. 기존 raw tail은 zones 뒤에 유지한다.
3. `대각선샘플2.hwpx` 또는 신규 cellzone 생성 문서를 HWP로 export한 뒤 재파싱해 zone이 보존되는 회귀 테스트를 추가한다.

## 검증 계획

- 관련 Rust 테스트 추가.
- `cargo test --test issue_1623_cellzone_diagonal`
- `cargo fmt --check`
- 필요 시 `wasm-pack build --target web --out-dir pkg`

## 구현 결과

- `src/serializer/control.rs`의 HWP TABLE 레코드 직렬화에서 `table.zones`를 기록하도록 수정했다.
  - `border_fill_id` 뒤에 `UINT16 nZones`를 쓴다.
  - 각 zone은 `start_row`, `start_col`, `end_row`, `end_col`, `border_fill_id` 순서로 10바이트를 쓴다.
  - 기존 `raw_table_record_extra`는 zones 뒤에 이어서 보존한다.
- `issue_1633_as_one_cellzone_survives_hwp_export` 회귀 테스트를 추가했다.
  - 10열 8줄 표 전체에 cellzone 대각선을 적용한다.
  - HWP로 export 후 재파싱해 `zone row=0..7 col=0..9`와 대각선 BorderFill 속성이 유지되는지 확인한다.

## 검증 결과

- 비교 확인:
  - `samples/대각선샘플2.hwp`: `zone[0] row=0..7 col=0..9 bf=4`.
  - `samples/대각선샘플2-오류.hwp`: 동일 10열 8줄이나 zone 없음.
  - `hwp5-inventory-diff`: TABLE record size 기대 48, 오류 38.
- `cargo fmt --check` 통과.
- `cargo test --test issue_1623_cellzone_diagonal` 통과: 6개.
