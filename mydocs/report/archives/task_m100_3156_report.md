# task_m100_3156 처리결과: 곡선 도형 point count(i32) 음수 부호확장 무한루프

## 결함 요약

- 위치: `src/parser/control/shape.rs` `parse_curve_shape_data`
- 클래스: HWP5 바이너리 파서의 미검증 카운트 → 부호확장으로 인한 무한에 가까운 루프(DoS)
- 자매 결함: #3012(`parse_polygon_shape_data`, 다각형). #3012는 다각형만 수정하여 곡선 함수는 취약하게 남아 있었음.

## 근본 원인

곡선 도형 데이터의 점 개수 `count`(hwplib 스펙상 INT32, 부호 있음)를 파일에서 읽은 직후 검증 없이 `as usize`로 캐스팅:

```rust
let cnt = r.read_i32().unwrap_or(0) as usize;
```

조작된 파일이 `count = -1`(0xFFFFFFFF)을 지정하면 64비트에서 `usize::MAX` 근처의 거대한 값으로 부호확장된다. 이후 `for _ in 0..cnt` 및 `for _ in 0..(cnt - 1)` 루프가 입력 소진 후에도 `read_i32().unwrap_or(0)`으로 계속 0을 반환하며 수십억 회 반복 → 파싱 스레드가 사실상 멈추는 DoS.

## 재현 바이트

곡선 도형 데이터:
```
FF FF FF FF   // count = -1 (INT32)
```

## 수정

캐스팅 전에 음수를 검사하여 음수면 count를 0(빈 곡선)으로 처리. #3012와 동일 패턴.

```rust
let cnt_raw = r.read_i32().unwrap_or(0);
let cnt = if cnt_raw < 0 { 0 } else { cnt_raw as usize };
```

## 검증 (red → green)

- red: `curve_point_count_negative_is_treated_as_zero` 테스트가 수정 전에는 60초 이상 종료되지 않음(타임아웃, 무한루프 확인).
- green: 수정 후 동일 테스트 통과. `cargo test --lib shape::` 42개 전부 통과(회귀 없음), `task195_tests` 5개 통과.

빌드/테스트: `RUSTFLAGS="-C linker=rust-lld" cargo test --lib`
