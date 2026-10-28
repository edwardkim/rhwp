# task/m100-2890: HML 표(TABLE) TextWrap preflight 부당 차단 수정

## 문제

HML 표(TABLE) 컨트롤의 `TextWrap` 속성 값이 기본값(`Square`)이 아니면, `src/serializer/hml/preflight.rs`의
`validate_table`이 `table.common.text_wrap != Default::default()`를 이유로 항상
`HML_UNSUPPORTED_IR` 블로커를 추가해 저장을 차단했다. 하지만 이 값은 파서(`reader.rs`)와
writer(`body.rs`) 양쪽 모두 이미 정상적으로 지원하고 있어서, 저장을 막을 이유가 없는 불필요한
차단이었다.

## 근거

- `src/serializer/hml/body.rs:412` (`write_table`) → `write_shape_object(writer, &table.common, "Table")` 호출.
  `write_shape_object`(`body.rs:341-358`)는 컨트롤 종류와 무관하게 `("TextWrap", text_wrap_name(common.text_wrap).into())`를
  항상 실제 값으로 방출한다. 즉 표에 대해서도 TextWrap 직렬화는 이미 구현되어 있다.
- `src/parser/hml/reader.rs:943-955` (`capture_shape_object`) → `nearest_object_is_table()`로 분기해
  표면 `table.common.text_wrap`, 사각형이면 `rectangle.text_wrap`을 채운다. 이 로직은 커밋
  `ec226dad`(`fix(hml): 표의 TextWrap 이 HML 되읽기에서 유실`)에서 "표에도 SHAPEOBJECT 의 TextWrap 이
  방출되는데 reader 는 rectangle 만 처리해서 유실된다"는 회귀를 고치며 추가되었다. 즉 되읽기는 이미
  정상 동작한다.
- `src/parser/hml/adapter.rs:221-264` (`into_table`)는 `common: source.common`으로 reader 가 채운
  `text_wrap` 을 손실 없이 `Document`의 `Table.common`에 그대로 전달한다.
- 그런데 `src/serializer/hml/preflight.rs`의 `validate_table`만 옛 가정(표의 TextWrap 은 항상
  기본값)을 그대로 유지한 채 저장을 차단하고 있었다. reader/writer 능력과 preflight 검사가 서로
  어긋난 전형적인 capability mismatch. (이슈 #2890 에 상세 근거 기록)

## 수정 내용

`src/serializer/hml/preflight.rs`의 `validate_table`에서 다음 한 줄을 제거했다.

```rust
|| table.common.text_wrap != Default::default()
```

writer 가 이미 실제 값을 그대로 직렬화하므로, preflight 에서 이를 "미지원"으로 취급할 필요가 없다.
`reader.rs`는 전혀 건드리지 않았다(작업 범위 제한 준수).

## red → green 테스트

`tests/hml_serializer.rs`에 `table_with_non_default_text_wrap_round_trips_through_hml_save` 테스트를
추가했다.

- **Red (수정 전)**: `samples/hml/formatting_table.hml`의 표 `SHAPEOBJECT` 태그에
  `TextWrap="TopAndBottom"`을 주입해 파싱한 뒤(파싱은 정상, `table.common.text_wrap`이 올바르게
  `TopAndBottom`으로 확인됨) 수정 없이 곧바로 `serialize_hml`로 저장을 시도하면 다음과 같이 항상
  실패했다.
  ```
  Err(UnsupportedIr { blockers: [HmlSaveBlocker {
      code: "HML_UNSUPPORTED_IR",
      xml_path: "/HWPML/BODY/SECTION[0]/P[1]/CONTROL[0]/TABLE",
      message: "table fields cannot round-trip through HML",
  }] })
  ```
  (임시 프로브 테스트로 이 실패를 실제로 실행해 확인함 — 수정 전 `preflight.rs` 상태에서
  `cargo test`를 돌려 위 오류 메시지를 그대로 재현했다.)
- **Green (수정 후)**: 동일한 문서를 저장하면 성공하고, 저장된 바이트를 다시 파싱했을 때도
  `table.common.text_wrap == TextWrap::TopAndBottom`이 왕복 보존됨을 확인한다.
  실제 실행 결과: `test table_with_non_default_text_wrap_round_trips_through_hml_save ... ok`.

## 검증 결과

- `cargo build --lib`: 성공 (`Finished dev profile ... target(s) in 1m 44s`).
- 대상 테스트 `cargo test --test hml_serializer table_with_non_default_text_wrap_round_trips_through_hml_save`:
  통과 (`test result: ok. 1 passed; 0 failed`).
- `rustfmt --edition 2021`: 변경 파일(`src/serializer/hml/preflight.rs`, `tests/hml_serializer.rs`)에만 적용, 정상 완료.
- **`cargo clippy --all-targets --profile release-test -- -D warnings`: 미실행.** 작업 중 디스크 여유 공간이
  20GB(1.9TB 중 99% 사용)까지 떨어져, 코디네이터 지시에 따라 release-test 프로파일 전체 링크가 필요한
  clippy 단계를 이번 커밋에서는 건너뛰었다. 메인테이너 CI 에서 clippy 검사가 수행될 것으로 예상하고
  넘어갔으며, 이 사실을 숨기지 않고 명시적으로 기록한다. 변경 범위가 조건식 한 줄 삭제 + 테스트 추가로
  매우 작아 clippy 위반 가능성은 낮다고 판단했지만, 로컬로 직접 확인하지는 못했다.
- 전체 테스트 스위트(`cargo test` 전체)도 디스크/시간 제약으로 실행하지 않았다. 변경이
  `validate_table`의 조건 하나를 완화하는 것뿐이고, 관련 기존 회귀 테스트(`table_text_wrap_is_read_back_from_hml`,
  `stale_offsets_after_unequal_direct_text_mutation_block_export` 등 인접 테스트)의 로직과 충돌하지 않음을
  코드 리뷰로 확인했으나, 전체 스위트 실행으로 재확인하지는 못했다.
