# Task M100 #6672 구현 계획

## 판정 경계

full render의 frame-aware 줄 나눔은
`line_breaking::layout_paragraph_in_frame_impl`이 `SpaceMetric`을 고른 뒤
`tokenize_paragraph_with_regenerated_space_metric`을 호출하는 경로가 소유한다.
기존 `tokenize_paragraph` wrapper와 `split_composed_line_by_width` 계열은 이 경로의
fallback이 아니며 제품 호출자가 없다.

삭제는 다음 세 조건의 교집합으로만 승인한다.

1. `cargo rustc ... -- -W dead-code`가 native, wasm32, native-skia에서 모두 같은
   callable을 죽은 항목으로 판정한다.
2. non-test source 호출자가 0건이다.
3. LLDB가 실제 full render에서 현재 production owner에 정지하고 stack을 확보한다.

## 파일별 구현

### `scripts/debug/renderer.py`

`rhwp-render-flow` LLDB command 하나를 등록한다. command는 source marker로 production
tokenization 지점을 찾고 one-shot breakpoint를 설치한다. callback은 기본 저장 metric
호출은 계속하고, production이 `HalfCell`을 선택한 decision-bearing divergence에서만
실제 frame, 함수명과 줄 나눔 단위를 출력하고 정지한다.

### `src/renderer/composer.rs`

호출자 없는 `inject_footnote_markers`와
`missing_lineseg_legacy_bullet_requires_regenerated_space_metric`을 삭제한다.
단위 테스트가 직접 호출하는 `split_composed_line_by_width`와 전용 폭 helper는
`#[cfg(test)]`로 제품 graph에서 제외한다.

### `src/renderer/composer/line_breaking.rs`

production의 더 구체적인 tokenization 함수로 이미 수렴한 무인자 정책 wrapper
`tokenize_paragraph`와 legacy 꼬리 문자 판정 `is_line_end_forbidden`은 단위 테스트
전용이므로 `#[cfg(test)]`로 제한한다.

### `src/renderer/composer/tests.rs`

기존 line-breaking 회귀 검사를 유지한다. 테스트 전용 helper가 production build에
들어오지 않는지는 세 target의 liveness 진단으로 검증한다.

### renderer 전체 callable sweep

세 제품 구성에서 공통으로 죽은 함수·메서드를 module owner별로 묶는다.

- equation parser/tokenizer/SVG helper
- float, height, page number, pagination, endnote flow helper
- layout, table layout, text measurement, style helper
- exact-font kerning probe
- render-tree와 dormant shaping transaction accessor

기존 unit/integration test가 호출하는 계약은 `#[cfg(test)]`에서 그대로 보존한다.
한 target에서라도 제품 호출되는 SVG/font/native-skia callable은 이동하지 않는다.
함수가 아닌 field/type/constant 정리는 별도 작업으로 남긴다.

### 작업 기록

`mydocs/working/task_m100_6672_stage1.md`에 debugger와 liveness 증거를,
`mydocs/report/task_m100_6672_report.md`에 계획 대비 결과와 검증을 기록한다.

## 검증

- fresh LLDB import와 `rhwp-render-flow` replay
- 세 제품 구성의 renderer callable `-W dead-code` 교집합 0건 재검사
- composer·line-breaking focused unit tests
- AGENTS/CONTRIBUTING의 Rust lint 묶음
- release-test 전체 nextest
- WASM package build와 renderer 관련 Studio test/build
- `git diff --check`, 문서 링크 검사, Gestell
