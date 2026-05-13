# Task #864 Stage F caption 다음 문단 겹침 정정

**작성일**: 2026-05-13
**브랜치**: `local/task864`

## 배경

Stage E 정정 (caption 위치) 완료 후, 페이지 4 의 "Visual Block을 이용한 대소문자 변경" caption 이 image 아래 정상 표시되나, **본문 "먼저 원하는 구간을..." 와 겹쳐 그려지는** 결함 추가 발견.

## 본질

`src/renderer/layout.rs` 의 inline picture 처리에서:

1. `result_y` (다음 paragraph 시작 y) 가 line 2966-2969 / 2973-2976 에서 `pic_y + line_advance.max(pic_h)` 로 결정됨 (image 만 고려).
2. 그 후 caption 이 그려짐 (line 2979+) 하지만 **`result_y` 는 caption 높이를 추가하지 않음**.
3. 다음 paragraph 가 `result_y` (= image 바로 아래) 부터 시작 → caption 과 겹침.

## 정정

caption 렌더 후 `result_y` 를 caption 의 bottom (`cap_y + caption_h`) 까지 진행:

```rust
self.layout_caption(...);
// [Task #864 Stage F] caption 이 차지한 영역까지 result_y 진행.
if matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    if cap_bottom > result_y {
        result_y = cap_bottom;
    }
}
```

`Top` direction 은 `offset_inline_image_y` 로 image 자체를 caption 높이만큼 밀어내므로 별도 처리 불필요. `Left/Right` 은 caption 이 image 옆에 위치 → image 높이 안에서 처리.

## 검증

- `cargo build --release`: ✓
- `cargo test --release --lib`: 1230 passed (회귀 0)
- hwp3-sample14 page 4 시각: caption "Visual Block을 이용한 대소문자 변경" 가 image 아래, 본문 "먼저 원하는 구간을..." 가 caption 아래 정상 배치 ✓
- hwp3-sample14 page 2, 3 회귀 0 (page 2 는 WMF 내부 caption, page 3 는 Stage E 정정 유지)

## Stage F 결론

inline picture caption 렌더 후 `result_y` 를 caption bottom 까지 진행하여 다음 paragraph 와의 겹침 결함 정정.

📋 **Stage F 완료. 종합 보고서 업데이트 + 커밋 진행합니다.**
