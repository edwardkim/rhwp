# Task #530 구현 계획서 — TAC Top caption 표 본문 y 오프셋 적용

## 전제

Stage 1 에서 #530 의 원인을 다음과 같이 확정했다.

- 문제 표는 `samples/basic/treatise sample.hwp` page 5 의 `pi=60 ci=0` TAC 표이다.
- 셀 내부 두 문단의 line segment 는 `vpos=0`, `vpos=1080` 으로 정상 분리되어 있다.
- 실제 SVG 에서는 Top caption 두 줄이 머리행 두 줄과 같은 y 영역에 렌더링된다.
- 원인은 `layout_table` 의 `inline_x_override.is_some()` 경로가 Top caption offset 을 표 본문 y 에 적용하지 않는 것이다.

## 구현 원칙

1. 정정 범위는 `src/renderer/layout/table_layout.rs` 로 제한한다.
2. `inline_x_override` 의 x 좌표 의미는 유지한다.
3. inline/TAC 경로에서도 Top caption 이 있으면 표 본문 y 만 `caption_height + caption_spacing` 만큼 아래로 이동한다.
4. caption 자체의 y 는 기존처럼 `y_start` 로 유지한다.
5. Bottom/Left/Right caption 과 caption 없는 표에는 영향을 주지 않는다.
6. 반환 y 계산은 기존 `table_height + caption_extra` 정책과 정합하도록 유지한다.

## 구현 상세

### 변경 파일

- `src/renderer/layout/table_layout.rs`
- `tests/issue_530.rs` 신규

### `table_layout.rs` 정정 방향

현재 코드:

```rust
// inline_x_override가 있으면 외부에서 이미 위치를 계산했으므로 y_start 그대로 사용
let table_y = if inline_x_override.is_some() {
    y_start
} else {
    self.compute_table_y_position(...)
};
```

정정 방향:

```rust
let inline_top_caption_offset = if inline_x_override.is_some() && depth == 0 {
    table.caption.as_ref()
        .filter(|caption| matches!(caption.direction, CaptionDirection::Top))
        .map(|_| caption_height + caption_spacing)
        .unwrap_or(0.0)
} else {
    0.0
};

let table_y = if inline_x_override.is_some() {
    y_start + inline_top_caption_offset
} else {
    self.compute_table_y_position(...)
};
```

주의:

- `caption_height + caption_spacing` 은 현재 `compute_table_y_position()` 의 `treat_as_char=true` Top caption 분기와 동일한 계산이다.
- `v_offset` 은 여기서 추가하지 않는다. `inline_x_override` 경로의 y 는 paragraph layout 이 이미 inline 흐름 좌표로 계산한 값이므로, 이번 정정은 누락된 Top caption body offset 만 추가한다.
- non-inline Top caption 표는 기존 `compute_table_y_position()` 경로를 계속 사용한다.

## 회귀 테스트 설계

### 신규 테스트

파일: `tests/issue_530.rs`

테스트명:

```rust
issue_530_tac_top_caption_does_not_overlap_header_row
```

절차:

1. `samples/basic/treatise sample.hwp` 를 `HwpDocument::from_bytes` 로 로드한다.
2. page index 4 를 `render_page_svg_native(4)` 로 SVG 렌더링한다.
3. 문제 표의 첫 머리행 cell clip rect 를 찾는다.
   - x: `431.97333333333336`
   - width: `88.33333333333333`
   - height: `30.16`
4. 같은 x 에서 Top caption 둘째 줄 시작 글자 `T` 의 baseline y 를 찾는다.
5. 다음 조건을 검증한다.

```text
header_cell_top_y > caption_second_line_baseline_y
```

현재 회귀 상태:

```text
header_cell_top_y = 525.49
caption_second_line_baseline_y = 551.32
```

따라서 현재 코드는 실패해야 한다.

정정 후 기대:

```text
header_cell_top_y > 551.32
```

즉, 표 머리행이 Top caption 아래에서 시작해야 한다.

### 보조 검증

테스트 안에서 다음도 같이 확인한다.

- SVG 에 `"cell-clip"` 이 존재한다.
- caption 시작 글자 `T` 를 찾지 못하면 명확한 panic 메시지를 낸다.
- 머리행 top y 와 caption baseline y 를 `println!` 으로 남겨 실패 시 좌표 판단이 가능하게 한다.

## 단계 분리 (3 stages)

### Stage 3 — Red 테스트 + 정정 구현

- `tests/issue_530.rs` 추가
- 수정 전 신규 테스트가 실패하는지 확인
- `src/renderer/layout/table_layout.rs` 에 inline Top caption offset 적용
- 신규 테스트 통과 확인
- 대상 SVG 재생성 후 머리행 y 가 caption 아래로 이동했는지 확인

**산출물**: `mydocs/working/task_m100_530_stage3.md`

### Stage 4 — 회귀 검증

실행 게이트:

```bash
cargo test --test issue_530
cargo test --test issue_501
cargo test --test issue_418
cargo test --test svg_snapshot
cargo test --lib
cargo clippy --lib -- -D warnings
```

검증 기준:

- #530 신규 테스트 통과
- #501 cell padding 방어 회귀 없음
- #418 / svg_snapshot 회귀 없음
- lib 테스트 회귀 없음
- clippy warning 0

**산출물**: `mydocs/working/task_m100_530_stage4.md`

### Stage 5 — 시각 검증 + 최종 보고

- 정정 전/후 `output/debug/treatise sample_005.svg` 좌표 비교
- 작업지시자 시각 판정 요청
- 최종 보고서 작성: `mydocs/report/task_m100_530_report.md`
- 오늘 할일 문서 완료 상태 갱신
- 이슈 #530 close 승인 요청

**산출물**: `mydocs/report/task_m100_530_report.md` + `mydocs/orders/20260502.md` 갱신

## 예상 변경 영향

| 영역 | 영향 |
|------|------|
| TAC + Top caption 표 | 표 본문 y 가 caption 아래로 이동 |
| TAC + Bottom caption 표 | 변경 없음 |
| TAC + Left/Right caption 표 | 변경 없음 |
| non-TAC Top caption 표 | 기존 `compute_table_y_position()` 경로 유지 |
| partial table caption | 별도 `table_partial.rs` 경로 유지 |
| 페이지네이션 반환 y | 기존 `caption_extra` 기반 반환 유지 |

## 위험과 대응

| 위험 | 대응 |
|------|------|
| 표 본문 y 만 내려가고 반환 y 가 어긋남 | 반환 계산은 변경하지 않는다. 기존 `y_start + table_height + caption_extra` 와 본문 이동량이 같은지 Stage 3 에서 좌표 확인 |
| inline position 이 이미 caption 을 포함한 경우 중복 이동 | Stage 1 SVG 증거상 현재 inline position 은 표 본문 기준 y 이므로 caption 미포함. 신규 테스트로 회귀 방지 |
| SVG 좌표 기반 테스트가 지나치게 fixture-specific | 본 결함은 특정 TAC Top caption fixture 의 회귀이므로 issue test 로 한정한다. 광범위 회귀는 `svg_snapshot` 과 기존 issue tests 로 보완 |
| Bottom/Left/Right caption 회귀 | `CaptionDirection::Top` 조건만 적용 |

## 승인 게이트

본 구현 계획서 승인 후 Stage 3 을 진행한다.
