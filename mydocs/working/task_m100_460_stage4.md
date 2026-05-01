# Task #460 Stage 4 완료 보고서: 후속 수정 (border_fill_id + body clip)

## 개요

Stage 3 완료 후 PR 업스트림 전 발견된 두 가지 추가 버그를 수정하였다.

## 수정 내용

### 수정 1: HWP3 표 셀 border_fill_id 오프-바이-원 버그 (parser)

**파일**: `src/parser/hwp3/mod.rs`

**원인**: `doc_border_fills.push()` 후 `.len() - 1`을 사용하여 0-기반 인덱스를 할당하였으나,
렌더러는 1-기반 인덱스(`idx = border_fill_id - 1`)를 기대함.
결과적으로 각 셀이 이전 셀의 border_fill을 사용하는 오프셋 버그 발생.

**수정**:
```rust
// 수정 전 (버그):
doc_border_fills.push(border_fill);
cell.border_fill_id = (doc_border_fills.len() - 1) as u16;

// 수정 후 (정상):
doc_border_fills.push(border_fill);
cell.border_fill_id = doc_border_fills.len() as u16;
```

### 수정 2: body clip 하단 무제한 확장 방지 (renderer)

**파일**: `src/renderer/layout.rs`

**원인**: `build_page`의 `expand_clip` 함수가 body 영역 바깥으로 넘치는 부동 개체(그림)를
모두 포함하도록 clip을 확장함. 대형 그림(533.9×474.4px)이 있는 페이지에서 body clip이
꼬리말 영역까지 확장되어 꼬리말이 가려지는 현상 발생.

- 페이지 3 (para=41): overflow=128.2px → body clip이 꼬리말 영역까지 확장
- 페이지 9 (para=76): overflow=427.1px → body clip이 페이지 하단까지 확장

**수정**: `expand_clip` 루프 후 body_bottom + 10px를 초과하는 하방 확장을 제한.
표 외곽 테두리 등 소폭 확장(10px)은 허용하여 표 테두리 잘림 방지.

```rust
let body_bottom = body_bbox.y + body_bbox.height;
let max_bottom = body_bottom + 10.0;
if clip.y + clip.height > max_bottom {
    clip.height = max_bottom - clip.y;
}
```

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1068 passed, 0 failed |
| `cargo build --release` | 성공 (경고 없음) |
| hwp3-sample.hwp SVG 내보내기 | 22페이지 정상 완료 |
| 페이지 3 body clip | 886.85px (body+10px 제한 정상 적용) |
| 페이지 9 body clip | 886.85px (427.1px 오버플로우 제한 정상 적용) |
| re_sample 회귀 테스트 | 13 passed, 0 failed |
| 전체 lib 회귀 | 1068 passed, 0 failed |

### 잔존 LAYOUT_OVERFLOW_DRAW 경고

`pi=18446744073709551615 overflow=36.0px` — 꼬리말 문단 LINE_SEG lh=6950 HU (92.67px)가
꼬리말 영역 56.7px를 초과하는 기존 경고. 시각적 렌더링에는 영향 없음 (꼬리말 내용 정상 표시).
이번 Stage 범위 외로 향후 별도 처리 가능.

## 수정 파일

| 파일 | 변경 내용 |
|------|---------|
| `src/parser/hwp3/mod.rs` | border_fill_id 1-기반 인덱스 수정 (1줄) |
| `src/renderer/layout.rs` | body clip 하단 10px 제한 추가 (6줄) |
