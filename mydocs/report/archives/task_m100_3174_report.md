---
kind: report
status: final
---

# 처리결과: WMF top-down 비트맵 height() 언더플로

Issue: #3174

## 요약

`BitmapInfoHeader::height()`가 Info/V4/V5 헤더에서 부호 있는 32비트 `height`를
부호 확인 없이 `as usize`로 캐스팅해, top-down DIB(음수 Height, MS-WMF 2.2.2.9)를
만나면 값이 `usize::MAX` 근방으로 언더플로했다. 같은 파일의 `size()`는 이미
`height.unsigned_abs()`로 절댓값을 취하고 있어 두 메서드가 서로 다른 규칙을 쓰고
있었다.

## 근본 원인

`src/wmf/parser/objects/structure/bitmap_info_header/mod.rs`의 `height()`:

```rust
Self::Info(BitmapInfoHeaderInfo { height, .. })
| Self::V4(BitmapInfoHeaderV4 { height, .. })
| Self::V5(BitmapInfoHeaderV5 { height, .. }) => *height as usize,
```

`height: i32`가 음수일 때 `as usize` 캐스팅은 2의 보수 재해석으로 거대한 값을
만든다.

## 영향 경로

- `src/wmf/converter/bitmap.rs::expand_color_palette()` — 행 순회 상한으로
  이 값을 그대로 사용, top-down DIB 입력 시 즉시 패닉하거나 슬라이스 계산이
  깨진다.
- `src/wmf/converter/svg/util.rs` — DIBPatternPT 브러시의 SVG `height` 속성에
  같은 값을 그대로 써 렌더링이 깨진다.

## 재현 (red)

`bitmap_info_header::mod::tests::height_of_top_down_dib_is_absolute_value`
추가 후 `cargo test --lib bitmap_info_header` 실행:

```
left: 18446744073709551606
right: 10
```

## 수정 (green)

`height()`도 `size()`처럼 Info/V4/V5 변형에서 `height.unsigned_abs()`를 쓰도록
통일. 최소 변경, 로직 재구성 없음.

## 검증

```
cargo test --lib bitmap_info_header  -> 1 passed
cargo test --lib bitmap              -> 8 passed (회귀 없음, 관련 변환기 테스트 포함)
cargo test --lib wmf                 -> 2 passed (회귀 없음)
```

RUSTFLAGS="-C linker=rust-lld" 사용 (dbghelp 링커 오류 회피).

## 변경 파일

- `src/wmf/parser/objects/structure/bitmap_info_header/mod.rs`
