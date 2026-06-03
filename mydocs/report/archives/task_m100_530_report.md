# Task #530 최종 보고서 — treatise sample.hwp TAC 표 Top caption 머리행 겹침 정정

## 대상

- **Issue**: [#530](https://github.com/edwardkim/rhwp/issues/530)
- **브랜치**: `local/task530`
- **대표 샘플**: `samples/basic/treatise sample.hwp`
- **문제 위치**: page=5, global_idx=4, section=0, column=1, `Table pi=60 ci=0`
- **표 속성**: 3행×3열, `wrap=TopAndBottom`, `treat_as_char=true`

## 본질

이슈의 시각 증상은 머리행 셀 내부 두 문단의 line advance 붕괴가 아니라, **Top caption 이 TAC 표 본문 위에 겹쳐 렌더링되는 문제**였다.

Stage 1 진단에서 확인한 정정 전 좌표:

```text
머리행 top: y=525.49
caption 2줄 baseline: y=551.32
```

caption 둘째 줄이 머리행 영역 내부에 들어와 텍스트가 겹쳐 보였다.

## 원인

`src/renderer/layout/table_layout.rs` 의 `layout_table` 에서 `inline_x_override.is_some()` 인 경우 표 본문 y 를 `y_start` 로 그대로 사용했다.

일반 표 경로의 `compute_table_y_position()` 은 Top caption 일 때 표 본문 y 에 `caption_height + caption_spacing` 을 적용하지만, TAC inline 경로는 이 처리를 우회했다.

결과적으로 TAC Top caption 표에서는 다음 배치가 발생했다.

| 요소 | 기존 y 기준 |
|------|-------------|
| Top caption | `y_start` |
| 표 본문 | `y_start` |

## 구현

`inline_x_override` 경로에서도 Top caption 인 경우 표 본문 y 에 `caption_height + caption_spacing` 을 적용했다.

수정 파일:

- `src/renderer/layout/table_layout.rs`
- `tests/issue_530.rs`

정책:

- caption 자체는 기존처럼 `y_start` 에 렌더링한다.
- 표 본문만 caption 아래로 이동한다.
- Bottom/Left/Right caption 과 caption 없는 표는 변경하지 않는다.
- non-TAC 표는 기존 `compute_table_y_position()` 경로를 유지한다.

## 시각 검증 자료

사용자 직접 PR 본문 이미지 삽입용으로 전/후 비교 자료를 생성했다.

```text
output/debug/task530_compare/index.html
output/debug/task530_compare/before/treatise sample_005.svg
output/debug/task530_compare/after/treatise sample_005.svg
```

정정 후 좌표:

```text
머리행 top: y=560.83
caption 2줄 baseline: y=551.32
```

비교:

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| 머리행 top | 525.49 | 560.83 |
| caption 2줄 baseline | 551.32 | 551.32 |
| 판정 | 겹침 | 분리 |

## 검증

Stage 4 검증 결과:

```bash
cargo test --test issue_530 --test issue_501 --test issue_418 --test svg_snapshot
cargo test --lib
cargo clippy --lib -- -D warnings
```

결과:

- `issue_530`: 1 passed
- `issue_501`: 1 passed
- `issue_418`: 1 passed
- `svg_snapshot`: 6 passed
- `cargo test --lib`: 1110 passed, 0 failed, 1 ignored
- `cargo clippy --lib -- -D warnings`: 통과

`cargo test --lib` 중 기존 테스트 코드 warning 4건이 출력되었으나 이번 변경 파일과 무관하며 테스트는 통과했다.

## 커밋

```text
ac298d4 Task #530 Stage 3: TAC Top caption 표 본문 y 오프셋 정정
104b4f7 Task #530 Stage 4: 회귀 검증
```

## 결론

Task #530 정정은 구현, 신규 회귀 테스트, 기존 회귀 검증, 시각 검증 자료 준비까지 완료되었다.

이슈 close 는 작업지시자 승인 후 수행한다.
