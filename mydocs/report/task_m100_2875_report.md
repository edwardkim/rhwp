# task_m100_2875 처리 결과 보고

## 이슈

#2875 — `hp:pic lock`(개체 잠금) 속성이 파싱되지 않고 직렬화 시 항상 0으로 하드코딩됨

## 원인

`<hp:pic>`(그림 개체)의 `lock` 속성은 파서(`src/parser/hwpx/section.rs` `parse_picture()`)의 속성
매치 리스트에 없어 조용히 버려졌고, 직렬화기(`src/serializer/hwpx/picture.rs` `write_picture()`)는
`("lock", "0")`을 항상 하드코딩 방출했다. `Picture` IR(`src/model/image.rs`)에도 값을 보관할 필드가
없었다. `<hp:tbl>`(#2855/#2867), `hp:equation`(#2840/#2850)에서 이미 확인·수정된 것과 동일한 패턴이
`<hp:pic>`에서만 남아 있었다.

## 조사 범위 — 겸사 확인한 형제 속성

- `<hp:pic reverse>`(좌우 반전): #2861/#2869 로 이미 수정됨(별도 이슈).
- `groupLevel`/`numberingType`: 이슈 #2745(PR #2746)에서 이미 다룸 — 본 작업에서 손대지 않음.
- `flip`(좌우/상하 반전)·`rotationInfo`(회전각): `hp:flip`/`hp:rotationInfo` 자식 요소로 이미 완전히
  파싱·직렬화되어 왕복 보존됨을 코드로 확인(`parse_shape_flip`/`parse_shape_rotation_info`,
  `write_flip`/`write_rotation_info`) — 갭 없음.

## 수정

1. `src/model/image.rs`: `Picture` 구조체에 `pub lock: bool` 필드 추가.
2. `src/parser/hwpx/section.rs`: `parse_picture()`에 `b"lock" => lock = attr_str(&attr) == "1"` 매치
   추가, 파싱 종료 시 `pic.lock = lock` 반영.
3. `src/serializer/hwpx/picture.rs`: `write_picture()`에서 `("lock", "0")` 하드코딩을
   `("lock", bool01(pic.lock))`로 교체.
4. `src/wasm_api/tests.rs`: `Picture` 구조체 리터럴 2곳에 신규 필드 `lock: false` 추가(컴파일 정합성).

## 테스트 (red → green)

`src/serializer/hwpx/picture.rs::tests::issue2875_pic_lock_is_preserved_on_serialize` 신규 추가:
`pic.lock = true` 설정 후 직렬화 결과에 `lock="1"`이 포함되는지 단언. 수정 전에는 하드코딩 `"0"`
때문에 실패(red), 수정 후 통과(green)를 코드 리뷰로 확인.

## 검증

- `cargo build --lib`: 성공
- `cargo test --lib issue2875_pic_lock_is_preserved_on_serialize`: 1 passed
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음
- `rustfmt --edition 2021`(변경 파일만): 적용, 기능적 diff 없음

## 관련

- 이슈: https://github.com/edwardkim/rhwp/issues/2875
- 전례: #2861/#2869(hp:pic reverse), #2855/#2867(hp:tbl lock), #2840/#2850(hp:equation lock)
