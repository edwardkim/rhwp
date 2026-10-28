# task_m100_3012: 다각형 도형 point count(i32) 음수 부호확장 수정

## 이슈
#3012 fix(parser): 다각형 도형 point count(i32) 음수 값이 부호확장되어 무한에 가까운 루프를 유발하는 문제

## 원인

`src/parser/control/shape.rs`의 `parse_polygon_shape_data`에서 hwplib 스펙상 부호 있는
INT32인 다각형 점 개수(count)를 검증 없이 바로 `as usize`로 캐스팅했다. 조작된 값(예: `-1`)이
들어오면 부호 확장으로 `usize::MAX` 근처의 거대한 값이 되고, 이후 `for _ in 0..cnt` 루프가
입력이 소진된 뒤에도 `unwrap_or(0)` 기본값으로 계속 실행되어 사실상 종료되지 않는 루프(DoS)가
된다.

이는 #3004/#3008(`region.rs`의 `scan_count: i16`)과 동일한 버그 클래스(부호 있는 필드를 검증
없이 usize로 캐스팅)의 i32 사례다.

## 수정

캐스팅 전에 음수 여부를 검사해 음수면 count를 0(빈 다각형)으로 처리하도록 방어했다.

```rust
let cnt_raw = r.read_i32().unwrap_or(0);
let cnt = if cnt_raw < 0 { 0 } else { cnt_raw as usize };
```

## 테스트

`polygon_point_count_negative_is_treated_as_zero`: count에 `-1`(`0xFFFFFFFF`)을 넣은 최소
페이로드로 `parse_polygon_shape_data`를 호출해 `poly.points`가 비어 있는지 검증한다(수정 전에는
사실상 무한 루프에 빠져 테스트가 종료되지 않음).

## 검증

- `cargo check --lib`: 통과
- `cargo test --lib polygon_point_count_negative_is_treated_as_zero`: 통과 (exit 0)
- `rustfmt --edition 2021`: 적용, 변경 없음

## 범위

`src/parser/control/shape.rs` 1개 파일, 핵심 수정 2줄 + 테스트 15줄.
