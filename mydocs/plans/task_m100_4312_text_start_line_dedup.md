# 수행계획 — task_m100_4312_text_start_line_dedup

- **이슈**: [#4312](https://github.com/edwardkim/rhwp/issues/4312)
- **브랜치**: `task_m100_4312_text_start_line_dedup`
- **기준**: `upstream/devel` `f62f7503f`
- **작성 시각**: 2026-08-09 KST

## 1. 목표

문단의 "실제 텍스트가 있으면 leading 컨트롤-전용 줄을 건너뛰고 텍스트 줄부터 그린다" 판정
(`has_real_text` + `.position()` 패턴)이 `layout.rs:6560`과 `typeset.rs:13519`(scratch 측정
경로, #4277 지목 지점)에 독립적으로 재구현되어 있어 sep20/20 pi=936 회귀(측정 127.7px vs
렌더 101.3px)를 냈다. "첫 텍스트 줄 인덱스" 판정을 `ComposedParagraph`만 입력받는 순수 함수
하나로 추출해 두 지점이 공유하도록 한다.

## 2. 변경 경계

- `src/renderer/composer.rs`: `pub(crate) fn first_text_line(composed: &ComposedParagraph) ->
  Option<usize>` 신설 — 기존 두 곳의 동일 클로저를 그대로 옮긴 것으로 새 로직 없음.
- `src/renderer/layout.rs:6568`, `src/renderer/typeset.rs:13530-13539`: 각자 재구현한 클로저를
  `first_text_line` 호출로 교체. 주변 게이트(`has_real_text`, `is_wrap_host`, end_line 계산)는
  사이트별로 그대로 유지 — 이번 슬라이스는 "첫 텍스트 줄 찾기" 클로저 자체만 공유한다.
- 범위 밖: `layout.rs:8458`, `9723` 등 `text_end_line`도 함께 계산하고 공백 문자 처리가 다른
  변형은 술어가 미묘하게 달라(#4312 이슈 본문) 이번 PR에서 건드리지 않는다.
- `#4277`(scratch RenderNode 재사용 제거) 본체는 이 PR의 범위가 아니다 — 이 PR은 그 작업의
  선행 정리다.

## 검증 게이트

- `cargo test --profile release-test --lib lineseg_compare`
- `cargo test --profile release-test --test issue_1082_endnote_multicolumn_drift --test
  issue_1375_endnote_rewind_column_overflow` (sep20/sep2020 fixture 포함)
- `cargo test --profile release-test --tests` 전체
- `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings`
- Native Skia 3종 / `wasm-pack build --target web --out-dir pkg`

원격 push, PR 생성, 이슈 comment·close는 별도 승인 전 수행하지 않는다.
