---
kind: working
status: active
canonical: mydocs/working/task_m100_4055_stage1.md
last_verified: 2026-08-05
---

# #4055 Stage 1 — 레거시 `Contents` 구조 기반 로케이터 (S1)

- **Issue**: [#4055](https://github.com/edwardkim/rhwp/issues/4055)
- **결론**: **S1 = 가능.** 코퍼스 28종 × 2포맷 + 대조군 = **57/57 일치, 실패 0.**

## 1. 왜 새 로케이터가 필요했나

현행 `src/ole_chart/parser.rs:385-390` 의 `is_plausible_grid_value` 는 값을 이렇게 거른다.

```rust
value.is_finite() && value >= 1.0 && value <= 1_000_000.0
    && (value.fract().abs() < 1e-9 || (1.0 - value.fract()).abs() < 1e-9)  // 정수만
```

**정수·1 이상·100만 이하**만 값으로 인정한다. 그런데 코퍼스 `묶은세로막대형` 의 정답은
`4.3/2.5/3.5/4.5`, `2.4/4.4/1.8/2.8`, `2/2/3/5` 다. 이 필터를 통과하는 건 정수 4개(`2,2,3,5`)
뿐이고, 개수 불일치(`values.len() != expected_count`)로 **파싱 전체가 실패**한다.

렌더에서는 OOXML 경로가 먼저 이기므로(`renderer/layout/shape_layout.rs:1962-1988`) 지금껏
드러나지 않았다. 하지만 편집에는 두 가지 이유로 못 쓴다.

1. **값이 아니라 바이트 오프셋이 필요하다.** 현행 파서는 값만 돌려주고 위치를 버린다.
2. **값 휴리스틱은 편집과 함께 무너진다.** 사용자가 값을 `0`·음수·소수로 바꾸는 순간
   그 값은 더 이상 "plausible" 하지 않아 다음 재파싱이 깨진다.

## 2. 실측한 셀 구조

`VtDataGrid` 구간의 셀은 26바이트이고, f64 값 **바로 뒤**에 언제나 같은 트레일러가 붙는다.

```text
@326  pre[-12:] = 00 56 74 44 6f 75 62 6c 65 00 01 00   ("\0VtDouble\0" 01 00)
      f64 = 328.0
      post      = ff ff 06 00 00 00                      ← 트레일러

@352  pre[-12:] = 04 00 00 00 07 00 00 00 07 00 00 00
      f64 = 50.0
      post      = ff ff 06 00 00 00
```

즉 **트레일러 `FF FF 06 00 00 00` 를 찾아 그 직전 8바이트를 읽으면 값**이고, 그 위치가 곧
in-place 패치 대상 오프셋이다. 값의 크기·부호·정수 여부를 전혀 보지 않는다.

`samples/143E433F503322BD33.hwp` 의 그리드 (4행 × 3열, 셀 stride 26, 행 pitch 117):

```text
row0  @326=328.0   @352=50.0   @378=11.0
row1  @443=812.0   @469=70.0   @495=15.0
row2  @560=1702.0  @586=189.0  @612=201.0
row3  @677=1477.0  @703=191.0  @729=289.0
```

행 pitch 는 문서마다 다르다(117 vs 모던 샘플 141) — 열 수가 다르고 행 끝 여백도 65 vs 63 으로
흔들린다. **전역 stride 를 가정하면 안 되고, 트레일러 앵커로 셀마다 잡아야 한다.**

## 3. 결과

`tests/issue_4055_b1_chart_edit_probe.rs` 3건 전부 통과.

| 테스트 | 무엇을 고정하나 |
|---|---|
| `legacy_grid_locator_matches_ooxml_ground_truth_across_corpus` | 28종 × 2포맷 = **56건** 전건에서 값 개수·순서가 OOXML 정답지(`c:val`/`c:yVal`)와 일치. 각 오프셋을 되읽어 그 값이 실제로 거기 있는지도 확인 |
| `legacy_grid_orientation_is_not_fixed` | 그리드 순서가 문서마다 다르다 — 모던 코퍼스 **계열-major**, 대조군 **카테고리-major** |
| `locator_must_be_bounded_to_the_data_grid_window` | 구간 제한 없이 훑으면 그리드 밖 `VtDouble`(축 눈금 등)까지 잡힌다 (제한 12 / 무제한 14) |

값 개수는 차트 종류를 따라간다 — 막대·라인 12, 주식형 16, 분산형 6, 원형 4, 특이케이스 1.
전건 정답지와 일치했다.

## 4. B1 본구현에 넘기는 제약

1. **순서를 가정하면 안 된다.** 코퍼스 56건은 계열-major, 대조군은 카테고리-major 다.
   `(계열, 카테고리)` → 셀 매핑을 하려면 순서를 **판정**해야 한다. 참고로 현행
   `parse_legacy_hwp_chart_contents` 는 `values[category_idx * series_count + series_idx]` 로
   카테고리-major 를 **고정 가정**하고 있어, 모던 문서에서는 계열이 뒤섞인다.
2. **로케이터를 `VtDataGrid` 구간에 묶어야 한다.** 안 그러면 축 눈금 같은 무관한
   `VtDouble` 을 덮어쓴다.
3. **길이 불변 8바이트 덮어쓰기**라 주변 구조를 건드리지 않는다 — 이 점은 유리하다.
4. 현행 `is_plausible_grid_value` 는 편집 후 재파싱을 깨뜨리므로, 레거시 스트림을 진실
   원천으로 쓰려면 **파서도 함께 구조 기반으로 바꿔야 한다**(별도 결함으로 보고 대상).

## 5. 검증

```
cargo test --profile release-test --test issue_4055_b1_chart_edit_probe
  running 3 tests ... test result: ok. 3 passed; 0 failed
```

프로덕션 코드 변경 0 — `src/` 를 건드리지 않았다.
