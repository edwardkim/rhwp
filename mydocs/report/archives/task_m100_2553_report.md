# task_m100_2553 처리결과 보고서 — 높이 측정 경로를 조판/렌더와 정합

- **이슈**: [#2553](https://github.com/edwardkim/rhwp/issues/2553)
- **브랜치**: `task/m100-2553-height-measurer-body-recompose` (base `devel` @ `3c54abfd`)
- **범위**: `src/renderer/height_measurer.rs` 의 recompose 분기 20줄
- **분류**: 결함 수정 (페이지 분할 입력 오차)

## 1. 문제

`HeightMeasurer` 가 **본문** 문단을 측정하면서 형제 두 경로와 다르게 동작했다. 측정 결과는
페이지네이션 입력이므로, 측정과 실제 조판이 서로 다른 줄 수·높이를 갖게 된다.

### (a) cell recompose 오용

`height_measurer.rs:527` 이 `recompose_for_cell_width` 를 호출한다. 형제는 모두 body 래퍼다:

- `typeset.rs:11162` — 주석: `[#2279] 본문 NO_LS 는 글자모양 재분할 포함 래퍼 사용 —
  paragraph_layout(렌더)와 동일 (측정/렌더 줄수·pitch 정합)`
- `paragraph_layout.rs:1907`

body 는 cell 의 strict superset 이다(`composer.rs`):

```rust
pub fn recompose_for_body_width(...) {
    restyle_fallback_runs_by_char_shapes(composed, para);   // 측정 경로가 빠뜨린 부분
    recompose_for_cell_width(composed, para, column_inner_width_px, styles);
}
```

`restyle_fallback_runs_by_char_shapes` 는 `compose_lines` NO_LS 폴백이 만든 단일
`default_style_id` run 을 실제 글자모양으로 재분할하는 유일한 지점이다. 빠지면 혼합 글자모양
문단(15pt 도입부 + 14pt 본문 등)을 전부 도입부 크기로 측정해 폭이 과대해지고, 줄별 `max_fs`
가 일괄 15pt 라 `Percent` 줄간격 기준까지 부풀어 누적 높이 오차가 생긴다.

### (b) 도달 불가 분기

`height_measurer.rs:521` 이 `para.line_segs.is_empty()` 를 **match guard** 에 올려, 저장
line_segs 가 있는 문단은 곧장 `_ => None` 으로 떨어졌다. 형제는 같은 술어를 arm 본문에 두고
`else if` 로 stale 재래핑을 잇는다(`typeset.rs:11158-11182`,
`paragraph_layout.rs:1913-1930`). `recompose_stored_lines_if_overflowing_body` 의 호출부는
그 두 곳뿐이었고 측정 경로에는 대응물이 없었다 — 즉 기능이 구조적으로 도달 불가였다.

마스킹 문단에서 조판/렌더는 stale 분할을 버리고 재래핑하는데 측정은 저장 분할을 유지하므로
`line_heights` 의 줄 수 자체가 달라져 `PartialParagraph` 시작/끝 줄 인덱스가 어긋난다.

## 2. 변경

두 결함이 같은 블록에서 각각 갈라진 것이라, `typeset.rs:11153-11185` 구조를 통째로 이식했다.

1. guard 의 `para.line_segs.is_empty()` 를 arm 본문으로 이동
2. NO_LS 경로를 `recompose_for_body_width` 로 교체
3. `else if masked_stored_lines_stale(...)` → `recompose_stored_lines_if_overflowing_body`
   분기 추가

## 3. 검증

페이지 분할 입력을 바꾸는 변경이라 회귀를 넓게 확인했다.

| 스위트 | 결과 |
|---|---|
| `cargo test --lib renderer` | **862 passed / 0 failed** |
| `cargo test --lib document_core` | **254 passed / 0 failed** |
| `cargo test --lib` (전체) | **2377 passed / 0 failed / 7 ignored** |

측정 경로를 조판 경로와 동일하게 만드는 변경이므로, 두 경로가 이미 일치하던 문서에서는 동작이
바뀌지 않는다(전체 무회귀가 이를 뒷받침한다). 어긋나 있던 문단에서만 측정이 조판을 따라간다.

### 미실행 항목 (투명 고지)

- **전용 red→green 단위 테스트 미추가**. 이 결함은 "측정과 조판이 서로 다른 값을 낸다" 는
  경로 간 불일치라, 단언하려면 `measure_section` 결과와 조판 결과를 같은 문단에 대해 비교하는
  하네스가 필요하다. 현재 `height_measurer` 에는 그런 교차 비교 하네스가 없다. 대신 근거는
  ① 형제 두 경로의 코드와 `[#2279]` 주석이 "측정/렌더 정합" 을 이 래퍼의 목적으로 명시,
  ② body 가 cell 의 strict superset 이라는 정의, ③ 전체 2377건 무회귀다.
  하네스를 세워 교차 비교 테스트까지 넣기를 원하시면 별도로 진행하겠다.
- **PR CI 전체 검증**(`cargo clippy -- -D warnings`, visual sweep): 저장소 규약상 작업지시자
  별도 승인 사항이라 실행하지 않았다. 렌더 출력 경로를 바꾸는 변경이므로 visual sweep 판단이
  필요하면 지시 바란다.
