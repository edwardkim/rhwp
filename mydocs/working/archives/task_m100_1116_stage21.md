# Task #1116 Stage 21 보고서 — PR #1120 CI VPOS spacing_before 가드 복구

## 작업 시각

- 2026-05-26 KST

## 사용자 지시

- PR #1120 GitHub Actions `Build & Test` 실패 원인 확인.
- #1116 수정 방향이 맞다면 CI 실패를 수정.

## 실패 내용

대상 job:

- `https://github.com/edwardkim/rhwp/actions/runs/26406618699/job/77731349901?pr=1120`

실패 단계:

- `Run tests`
- `cargo test --lib`

실패 테스트:

```text
renderer::height_cursor::tests::page_path_sb_prededuct
```

실패 값:

```text
got=126.66666666666667
```

기존 테스트는 일반 문서의 `page_path` VPOS 보정에서 `spacing_before=10px`를 사전 차감해 `116.6667px`가 나와야 한다고 검증한다.

## 원인

#1116 Stage 14 이후 HWP3-origin sample16 p3 본문 정합을 위해 `vpos_corrected_end_y()`에서 `spacing_before` 사전 차감을 제거했다.

그 판단은 HWP3-origin 흐름에는 맞다. HWP3-origin 변환본은 문단 `vpos`와 `spacing_before` 분리가 이미 이뤄져 있어, VPOS 보정에서 다시 `spacing_before`를 빼면 sample16 p3 본문이 한컴 3mm 격자보다 위로 붙는다.

다만 기존 수정은 공용 함수 전체에 적용되어 일반 문서의 #643/#1027 동작까지 바꿨다. 따라서 기존 `page_path_sb_prededuct` 실패는 타당한 회귀 신호다.

## 수정 내용

### `src/renderer/layout.rs`

- `vpos_corrected_end_y()`에 `skip_spacing_before_prededuct` 인자를 추가했다.
- 기본 경로는 기존대로 `raw_end_y - curr_sb`를 사용한다.
- HWP3-origin 흐름에서만 `raw_end_y`를 그대로 사용한다.

### `src/renderer/height_cursor.rs`

- `HeightCursor`에 `skip_spacing_before_prededuct` 플래그를 추가했다.
- 기존 `page_path_sb_prededuct` 테스트는 그대로 유지해 일반 문서의 사전 차감을 보호한다.
- `hwp3_origin_page_path_keeps_spacing_before_in_vpos` 테스트를 추가해 #1116 전용 예외를 검증한다.

### `src/renderer/typeset.rs`

- `TypesetState`에 동일 플래그를 추가했다.
- typeset 측 `HeightCursor` 생성 시 플래그를 전달해 페이지네이션과 렌더러가 같은 정책을 쓰도록 맞췄다.

### `src/document_core/queries/rendering.rs`

- 기존 `hwp3_origin_flow_spacing_before` 판정값을 `TypesetEngine::typeset_section_with_variant()`에도 전달했다.

## 검증

```bash
cargo test page_path_sb_prededuct --lib -- --nocapture
cargo test hwp3_origin_page_path_keeps_spacing_before_in_vpos --lib -- --nocapture
cargo test --lib
cargo test --test issue_1116 -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo fmt --all -- --check
git diff --check
cargo build --bin rhwp
```

결과:

- `cargo test --lib`: 1387 passed, 0 failed, 6 ignored.
- `cargo test --test issue_1116 -- --nocapture`: 13 passed.
- `cargo test --test issue_1105 -- --nocapture`: 14 passed.

## 결론

#1116의 HWP3-origin 방향은 유지하되, 예외 범위를 HWP3-origin 흐름으로 좁혔다. 일반 문서의 `spacing_before` 사전 차감 가드와 sample16 p3 3mm 격자 정합 가드를 모두 통과한다.
