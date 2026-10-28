# Task m100-2840: 수식 lock(개체 잠금) 속성 왕복 유실 해소

## 이슈

https://github.com/edwardkim/rhwp/issues/2840

## 근본 원인

`src/parser/hwpx/section.rs`의 `parse_object_element_attrs`(모든 그리기 개체 공통 속성
파서)는 `id`, `zOrder`, `textWrap`, `textFlow`, `instid`, `groupLevel`, `ratio`,
`numberingType`, `isReverseHV`만 처리하고 `lock` 속성은 매치 대상이 없어 `_ => {}`로
버려졌다. `CommonObjAttr`(`src/model/shape.rs`)에도 잠금 상태를 담을 필드가 없어,
`src/serializer/hwpx/section.rs`의 `render_equation`은 항상 `lock="0"`을 문자열
리터럴로 방출했다. 결과적으로 원본 HWPX 문서의 수식 개체가 `lock="1"`(개체 보호)이어도
rhwp를 거쳐 재저장하면 항상 잠금 해제 상태로 바뀐다.

## 변경 사항

- `src/model/shape.rs`: `CommonObjAttr`에 `locked: bool` 필드 추가.
- `src/parser/hwpx/section.rs`: `parse_object_element_attrs`에서 `b"lock"` 속성을
  읽어 `common.locked`에 반영.
- `src/serializer/hwpx/section.rs`: `render_equation`의 하드코딩 `lock="0"`을
  `common.locked` 기반 방출로 교체. legacy 공용 도형 경로(`hp:{tag}` 포맷 문자열)도 동일
  교체.
- `src/serializer/hwpx/shape.rs`: `write_rect`/선(line·connectLine)/`hp:container`/
  `hp:ole`의 `("lock", "0")` 하드코딩을 `common.locked` 방출로 교체.
- `src/serializer/hwpx/picture.rs`, `src/serializer/hwpx/table.rs`: 동일 교체.
- `src/document_core/converters/common_obj_attr_writer.rs`: 신규 필드 추가에 따른
  테스트 헬퍼 `make_sample()` 초기화 보정(`locked: false`).

처음에는 `<hp:equation>` 경로로 범위를 한정했으나, 파서(`parse_object_element_attrs`)가
모든 개체 공통으로 `lock`을 읽기 시작하면서 직렬화기가 여전히 `"0"` 하드코딩인 개체
(rect 등)에서 IR 왕복 발산(`common.locked : 1건`)이 새로 생겨
`ir_field_sweep_does_not_regress`가 CI에서 실패했다(샘플 `143E433F503322BD33.hwpx`의
`lock="1"` rect). 따라서 파서와 대칭이 되도록 HWPX 개체 직렬화기 전체에 `lock` 방출을
배선했다.

## 검증

- `cargo build --lib`: 성공.
- `cargo test --lib equation_lock_reflects_ir`: red(수정 전 컴파일 불가/구현 전
  `lock="0"` 고정) → green(수정 후 `lock="1"` 방출 확인) 전환.
- `cargo test --lib equation`: 166개 전부 통과(기존 수식 관련 테스트 회귀 없음).
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음.
- `rustfmt --edition 2021`: 변경 파일 대상 실행, 포맷 diff 없음(CRLF 경고만 발생,
  실질 변경 없음).

## 완료 기준

수식 `lock` 속성이 IR을 거쳐 왕복 시 보존됨을 최소 단위 테스트로 확인.
