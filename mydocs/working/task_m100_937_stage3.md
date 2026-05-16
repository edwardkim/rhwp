# Task #937 Stage 3 완료보고서 — 복학원서 회귀 검증

## 작업 범위

`samples/복학원서.hwp` 실제 SVG 산출물과 기존 `issue_677_bokhakwonseo_page1` golden 회귀를 검증했다.

## 확인 결과

최초 snapshot 실행에서 golden 불일치가 발생했다. diff를 확인한 결과, 서명란 `U+F012B`가 `(인)`으로 바뀌는 의도한 변화 외에 하단 접수증 블록이 20px 우측 이동하는 부작용이 있었다.

원인은 `U+F081C` HWP TAC filler가 `display_text` 측정 경로로 들어가면서 기존 0폭 규칙을 우회한 점이었다. `effective_text_for_metrics()`에서 `U+F081C`가 포함된 run은 원문을 반환하도록 보정하여 `text_measurement`의 0폭 처리 규칙을 유지했다.

## 변경 내용

- `src/renderer/composer.rs`
  - `effective_text_for_metrics()`에 `U+F081C` filler 예외를 추가했다.
- `src/renderer/composer/tests.rs`
  - `test_677_effective_text_for_metrics_preserves_f081c_filler`를 추가했다.
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg`
  - 서명란 `U+F012B` 출력만 `(인)` 텍스트 노드로 갱신했다.

## Golden diff 요약

의도한 변경만 남았다.

- 기존: `󰄫(Signature)`
- 변경: `(인)(Signature)`
- 하단 접수증 블록 좌표 변화 없음
- `U+F081C` filler 0폭 규칙 유지

## 산출물

```text
output/svg/task937/복학원서.svg
```

확인 사항:

- 서명란 셀 `cell-clip-116`에서 `(`, `인`, `)` 텍스트 노드 출력
- 원본 `U+F012B` 문자는 SVG 산출물에 직접 출력되지 않음

## 검증

```bash
cargo test --test svg_snapshot issue_677_bokhakwonseo_page1
cargo test --test issue_937
cargo test --test issue_826
cargo test --lib effective_text_for_metrics
cargo check --features native-skia --lib
cargo run --bin rhwp -- export-svg samples/복학원서.hwp -o output/svg/task937
```

결과:

- `issue_677_bokhakwonseo_page1` — 통과
- `issue_937` — 3개 통과
- `issue_826` — 4개 통과
- `effective_text_for_metrics` — 4개 통과
- `native-skia` feature check — 통과
- SVG 산출물 생성 — 통과

`cargo test --lib effective_text_for_metrics`에서는 기존 warning 6건이 출력되었으며, 이번 작업의 신규 warning은 확인되지 않았다.

## 결론

Stage 3 목표는 달성했다. 다음 Stage 4에서는 최종 보고서 작성, 오늘할일 상태 갱신, 최종 테스트 범위 재확인 후 승인 요청 상태로 정리한다.

## 승인 요청

Stage 4 최종 정리를 진행해도 되는지 승인 요청한다.
