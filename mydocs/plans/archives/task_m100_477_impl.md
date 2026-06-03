# Task #477 구현 계획서 — 호출부 컨테이너 정합 정정

## 정정 요약

`layout_picture` 호출부 3개소에서 **컨테이너 영역**을 그림 폭 자체 (`pic_w`) 가 아닌 **셀/도형 안쪽 영역** (`inner_area`) 으로 전달.

## 정확한 호출부 (3개소)

shape_layout.rs:1533 (비-TAC) 은 이미 `inner_area.width` 사용 중 → 정정 불필요.

| # | 파일 | 라인 | 정황 |
|---|------|------|------|
| 1 | `src/renderer/layout/table_layout.rs` | 1533-1539 | TAC 표 셀 안 단독 이미지 (TAC 줄별 배치) |
| 2 | `src/renderer/layout/table_partial.rs` | 709-715 | TAC 분할 표 셀 안 단독 이미지 |
| 3 | `src/renderer/layout/shape_layout.rs` | 1517-1523 | TAC 도형 컨테이너 안 인라인 그림 |

## 정정 패턴

### 정정 전 (결함)

```rust
let pic_area = LayoutRect {
    x: inline_x,
    y: tac_img_y,
    width: pic_w,    // ← 그림 폭 자체 → layout_picture 내부 클램프 무효화
    height: pic_h,
};
self.layout_picture(tree, &mut cell_node, pic, &pic_area, ...);
```

### 정정 후 (의도)

```rust
let pic_area = LayoutRect {
    x: inline_x,
    y: tac_img_y,
    width: inner_area.width,    // ← 셀 안쪽 폭 → 클램프 작동
    height: inner_area.height,  // ← 셀 안쪽 높이
};
self.layout_picture(tree, &mut cell_node, pic, &pic_area, ...);
```

`layout_picture` 내부 (`picture_footnote.rs:39-48`) 의 비율 유지 축소 로직이 작동:

```rust
if container.width > 0.0 && pic_width > container.width {
    let scale = container.width / pic_width;
    pic_width = container.width;
    pic_height *= scale;
}
```

→ 셀 폭 (164 mm) < 그림 폭 (165 mm) → 비율 유지 축소 (164 mm × 34.97 mm).

## 컨테이너 후보

호출부별 셀 안쪽 영역:

| 호출부 | 컨테이너 변수 | 정의 위치 |
|--------|--------------|-----------|
| `table_layout.rs:1539` | `inner_area` | 같은 함수 내 1327 줄 (`layout_table_cells`) |
| `table_partial.rs:715` | `inner_area` | 같은 함수 내 (라인 685 등에서 사용) |
| `shape_layout.rs:1523` | (도형 컨테이너의 안쪽 영역) | TAC 컨테이너 내부 — 정밀 점검 필요 |

`shape_layout.rs:1523` 은 도형 컨테이너의 변수명 확인 후 결정.

## 정정 영향

**의도 변화**: 셀 폭 초과 그림이 셀 폭으로 자동 클램프 (비율 유지).

**비-결함 영향 (회귀 없음)**:
- 셀 폭 미만 그림: `pic_width > container.width` false → 클램프 미작동 (기존 동작 그대로)
- 비-TAC 그림: 이미 `inner_area` 컨테이너 사용 중 (정정 무관)

## inline_x 정합 점검

호출부에서 `inline_x += pic_w` 하는 부분이 있음 — `pic_w` 는 클램프 전 값. 클램프 후 폭으로 갱신해야 인라인 흐름 정합.

```rust
// 정정 후
let clamped_w = pic_w.min(inner_area.width);
let clamped_h = if pic_w > 0.0 {
    pic_h * (clamped_w / pic_w)
} else {
    pic_h
};
let pic_area = LayoutRect {
    x: inline_x, y: tac_img_y,
    width: clamped_w,
    height: clamped_h,
};
self.layout_picture(...);
inline_x += clamped_w;  // ← 클램프 폭으로 갱신
```

호출부에서 미리 클램프 → `inline_x` 정합 + `layout_picture` 내부 클램프와 중복 없음 (이미 셀 폭과 같으므로 무효).

## Stage 별 작업

### Stage 1: 베이스라인 측정

```bash
# 결함 케이스 SVG 추출 (debug-overlay)
mkdir -p output/svg/task477-baseline
cargo run --release --quiet --bin rhwp -- export-svg samples/k-water-rfp.hwp -p 15 \
  --debug-overlay -o output/svg/task477-baseline/

# pi=186 그림 정황 dump
cargo run --release --quiet --bin rhwp -- dump samples/k-water-rfp.hwp \
  --section 1 --para 186 > /tmp/baseline_pi186.txt
```

### Stage 2: 정정 적용

3개소 정정 패턴 적용:

1. `table_layout.rs:1532-1541` — 클램프 변수 + pic_area 갱신 + `inline_x += clamped_w`
2. `table_partial.rs:707-717` — 동일 패턴
3. `shape_layout.rs:1515-1524` — 동일 패턴 (컨테이너 변수 정합)

### Stage 3: 회귀 검증

```bash
cargo test --lib
cargo test --test svg_snapshot
cargo test --test issue_418
cargo clippy --lib -- -D warnings

# 정정 결과 SVG (debug-overlay)
mkdir -p output/svg/task477-test
cargo run --release --quiet --bin rhwp -- export-svg samples/k-water-rfp.hwp -p 15 \
  --debug-overlay -o output/svg/task477-test/

# 광범위 회귀 점검 (10 샘플 SVG byte 비교)
```

### Stage 4: 시각 검증 (작업지시자)

- k-water-rfp 16쪽 — pi=186 그림이 셀 안에 클램프되어 표시 확인
- 다른 표 안 그림 샘플 회귀 점검 (aift, exam_kor 등)

### Stage 5: 최종 결과보고서 + 오늘할일 갱신

## 검증 게이트

- `cargo test --lib`: 회귀 0건 (1077 passed 유지)
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_418`: 1/1
- `cargo clippy --lib -- -D warnings`: 0건
- 단위 테스트 추가 (선택 — `layout_picture` 의 비율 유지 축소 동작 단위 테스트가 이미 있는지 점검)

## 위험 정황 + 회피

- **inline_x 정합**: 클램프 후 폭으로 `inline_x` 갱신 필수 (안 하면 다음 인라인 컨트롤 위치 어긋남)
- **shape_layout.rs:1523 컨테이너 정합**: 도형 컨테이너의 안쪽 영역 변수명 확인 후 정정
- **회귀 위험**: 셀 폭 미만 그림은 영향 없음 (`pic_w.min(inner_area.width) == pic_w`)
