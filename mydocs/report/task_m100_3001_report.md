# Task m100-3001: WMF DIB 색상표 파싱 unbounded Vec::with_capacity 수정

## 배경

`src/emf/parser/records/drawing.rs`의 `parse_points16`에서 EMF 레코드의
검증되지 않은 `count: u32`를 그대로 `Vec::with_capacity`에 넘기던 문제가
#2992/#2998로 최근 수정되었다. 이번 태스크는 같은 클래스의 문제가 다른
레코드 파서(특히 EMF의 POLYPOLYLINE/POLYPOLYGON/EXTTEXTOUTW 글리프
배열, 비트맵 팔레트 항목 등)에도 남아 있는지 점검하기 위해 시작되었다.

## 조사 범위

`grep -rn "with_capacity" src/emf/` 및 `src/` 전체에 대해 각 호출에서
capacity 인자가 파일에서 온 검증되지 않은 카운트(u16/u32)를 `.min(...)`
같은 상한 없이 직접 쓰는지 확인했다.

- `src/emf/` 내부: `parse_points16`(이미 #2992로 수정됨), `text.rs`의
  `n_chars`(이미 `start + byte_len > payload.len()` 가드로 상한이
  걸려 있어 안전), `player.rs`의 `file_size`(실제 메모리상 슬라이스
  길이의 합이므로 안전) 등은 문제 없음.
- EMF 외 광범위한 `with_capacity` 호출도 대부분 이미 다른 `Vec`/문자열의
  `.len()`이나 상수 기반이라 안전.
- `src/wmf/` 파서 쪽에서 EMF와 동일한 패턴의 미검증 케이스를 발견:
  `src/wmf/parser/objects/structure/device_independent_bitmap.rs`의
  `Colors::parse_from_color_usage`가 `colors_length`를 그대로
  `Vec::with_capacity`에 사용. 호출부에서 이 값은
  `BitmapInfoHeader::color_used()` → BITMAPINFOHEADER의 `biClrUsed`
  필드(검증되지 않은 `u32`)에서 온다.

## 수정 내용

`Colors::parse_from_color_usage` 진입 시 `colors_length`를 256으로
클램프했다. 색상표가 존재하는 비트 심도(`BI_BITCOUNT_1/2/3`, 1/4/8bpp)
에서 사양상 색상표 최대 항목 수는 `2^8=256`이므로, 정상 파일에서는
`biClrUsed`가 이미 256 이하라 동작이 바뀌지 않는다.

```rust
let colors_length = colors_length.min(256);
```

파일: `src/wmf/parser/objects/structure/device_independent_bitmap.rs`
(수정 6줄 + 테스트 22줄).

## 검증 (red → green)

- **red (수정 전 근거)**: `colors_length`가 `u32::MAX`까지 갈 수 있고,
  RGBQuad 기준 4바이트/항목이므로 `Vec::with_capacity`가 약 16GB를
  즉시 예약 시도한다. 실제 파일 크기와 무관하게 발생하는 문제라 실제
  거대 할당을 로컬에서 실행해 재현하지는 않고(환경 OOM 위험), 코드
  경로 분석과 `doc_info.rs`/`emf::parse_points16`의 기존 동일 클래스
  수정 패턴으로 근거를 확인했다.
- **green**: `parse_from_color_usage_bounds_huge_colors_length` 테스트를
  추가해 `colors_length = u32::MAX as usize`, 버퍼는 4바이트만 준
  상태로 `Colors::parse_from_color_usage`를 직접 호출했다. 수정 후
  `colors_length`가 256으로 클램프되어 `Vec::with_capacity`가 안전한
  값만 예약하고, 이후 루프에서 데이터 부족으로 `Err`을 정상 반환하는
  것을 확인했다(`cargo test --lib parse_from_color_usage_bounds_huge_colors_length`
  → `test result: ok. 1 passed`).

```
cargo check --lib  → 통과 (경고 없음)
cargo test --lib parse_from_color_usage_bounds_huge_colors_length → 1 passed
rustfmt --edition 2021 (대상 파일만) → 적용
```

## 관련 이슈/PR

- 등록 이슈: edwardkim/rhwp#3000
- 참고(동일 클래스, 이미 수정됨): #2992, #2998, #2751, #2722
