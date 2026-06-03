# Task #469 구현계획서

## 수정 대상

`src/renderer/layout.rs` L1740-1816 paragraph border 그룹 rendering 블록.

## 변경 내용

### Before (L1738-1741)

```rust
const DEFAULT_MIN_INSET: f64 = 2.0;
let top_pad = if stroke_width > 0.0 && !prev_touches { top_inset.max(DEFAULT_MIN_INSET) } else { top_inset };
let bot_pad = if stroke_width > 0.0 && !next_touches { bottom_inset.max(DEFAULT_MIN_INSET) } else { bottom_inset };
let rect_y = y_start - top_pad;
let rect_h = height + top_pad + bot_pad;
```

### After

```rust
const DEFAULT_MIN_INSET: f64 = 2.0;
let top_pad = if stroke_width > 0.0 && !prev_touches { top_inset.max(DEFAULT_MIN_INSET) } else { top_inset };
let bot_pad = if stroke_width > 0.0 && !next_touches { bottom_inset.max(DEFAULT_MIN_INSET) } else { bottom_inset };
// partial_start: 박스가 이전 컬럼/페이지에서 이어진 부분이므로 top_pad 만큼
// col_top 위로 확장하면 안 된다 (헤더선과 충돌). 마찬가지로 partial_end 는 col_bot 아래.
// y_start/y_end 는 이미 L1707 에서 col_top..col_bot 으로 클램프되어 있다.
let effective_top_pad = if is_partial_start { 0.0 } else { top_pad };
let effective_bot_pad = if is_partial_end { 0.0 } else { bot_pad };
let rect_y = y_start - effective_top_pad;
let rect_h = height + effective_top_pad + effective_bot_pad;
```

이로써:
- `partial_start=true` → `rect_y = y_start` (col_top 까지만, 위로 확장 X)
- `partial_end=true` → `rect_h` 가 bot_pad 만큼 더 늘어나지 않음 (col_bot 아래로 확장 X)

skip_top/skip_bottom 분기는 그대로 유지 (윗변/아랫변 가로선 미렌더링).

## 단계 구성

### Stage 1: 단위 테스트 추가 (red)

`src/renderer/layout/integration_tests.rs` 에 cross-column partial_start 박스 테스트 추가.

검증 항목:
- 우측 단의 partial_start 박스의 `rect_y >= col_top`
- 좌·우 세로선의 `y1` 이 col_top 이상

또는 더 가벼운 통합 테스트로 `samples/exam_kor.hwp` 페이지 2 SVG 를 생성해 우측 단 영역의 line element y1 이 196.55 미만이 아닌지 검증.

가장 자연스러운 검증은 새로운 단위 테스트보다 **exam_kor 의 골든 SVG 검증**. 그러나 exam_kor 전체 페이지가 골든에 없으므로 신규 작성 또는 좁은 단위 테스트 둘 중 하나.

→ **단위 테스트 (Mock 으로 partial_start range 한 건 강제 입력)** 채택.

### Stage 2: 코드 수정 + green

L1740 부근에 `effective_top_pad`/`effective_bot_pad` 도입.

### Stage 3: 회귀 검증

- `cargo test --release` 전체 통과
- `cargo test --test svg_snapshot` 골든 SVG 검증 (변경 발생 시 의도된 변화인지 검토 후 갱신)
- `exam_kor.hwp -p 1` 재출력하여 우측 단 (나) 박스 좌·우 세로선의 y1 좌표가 헤더선(196.55) 보다 낮은지 확인 (211.65 근방 기대)
- 좌측 단 (가) 박스가 영향 없는지 확인

## 산출물

- 수정 파일: `src/renderer/layout.rs`
- 신규 테스트: `src/renderer/layout/integration_tests.rs` 또는 별도 모듈
- 갱신 가능 파일: 영향 받는 골든 SVG (있을 경우)

## 검증 명령

```bash
cargo build --release
cargo test --release
cargo test --release --test svg_snapshot
./target/release/rhwp export-svg samples/exam_kor.hwp -p 1 -o /tmp/p1/
grep "y1=\"196" /tmp/p1/exam_kor_002.svg | wc -l   # 헤더선 1개만 (line 19) 기대
```
