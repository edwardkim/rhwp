# Task #937 Stage 2 완료보고서 — `U+F012B` 표시 문자열 치환

## 작업 범위

복학원서 서명란 원문 PUA `U+F012B`를 보존하면서 렌더링과 폭 측정에서는 `(인)`으로 표시되도록 수정했다.

## 구현 내용

1. `src/renderer/composer.rs`
   - `U+F012B -> "(인)"` 표시 문자열 매핑을 추가했다.
   - 기존 Hanyang-PUA 옛한글 변환 경로를 `convert_pua_display_text()`로 확장했다.
   - 일반 렌더러가 공유하는 `expand_pua_render_text()`를 추가했다.
   - `pua_to_display_text()`가 글자겹침 숫자와 `U+F012B` 표시 문자열을 함께 처리하도록 확장했다.

2. 렌더러 출력 경로
   - SVG, Web Canvas, HTML, 명령형 Canvas, native-skia 텍스트 출력 경로에서 `expand_pua_render_text()`를 사용하도록 정리했다.
   - 원본 텍스트의 PUA는 유지하되 최종 출력 문자열에는 `U+F012B`가 그대로 새지 않도록 했다.

3. 레이아웃 측정
   - `paragraph_layout`의 composed run 폭 측정에서 `effective_text_for_metrics()`를 사용하도록 보강했다.
   - `U+F012B` 1글자 원문 인덱싱은 유지하면서 표시 폭은 `(인)` 기준으로 계산한다.

4. 테스트
   - `tests/issue_937.rs`에 SVG 텍스트 노드 기반 검증을 추가했다.
   - 실제 `samples/복학원서.hwp` 1페이지 SVG 텍스트가 `(인)(Signature)` 순서로 렌더링되고, `U+F012B`가 SVG에 직접 출력되지 않는지 확인한다.

## 변경 파일

- `src/renderer/composer.rs`
- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`
- `src/renderer/html.rs`
- `src/renderer/canvas.rs`
- `src/renderer/skia/text_replay.rs`
- `tests/issue_937.rs`

## 검증

```bash
cargo test --test issue_937
cargo test --test issue_826
cargo test --lib test_555_effective_text_for_metrics
cargo check --features native-skia --lib
```

결과:

- `issue_937` — 3개 통과
- `issue_826` — 4개 통과
- `test_555_effective_text_for_metrics` — 3개 통과
- `native-skia` feature check — 통과

참고: `native-skia` 확인은 최초 실행 시 optional dependency 다운로드가 필요해 승인된 네트워크 실행으로 의존성을 받은 뒤 통과했다.

## 결론

Stage 2 목표는 달성했다. 다음 Stage 3에서는 실제 출력 산출물과 기존 회귀 범위를 추가 확인하고, 최종 보고서와 오늘 할일 상태 갱신을 진행한다.

## 승인 요청

Stage 3 검증 및 최종 정리를 진행해도 되는지 승인 요청한다.
