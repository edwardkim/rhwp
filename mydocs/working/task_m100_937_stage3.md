# Task #937 Stage 3 완료보고서 — 복학원서 회귀 검증

## 작업 범위

`samples/복학원서.hwp` 실제 SVG 산출물과 기존 `issue_677_bokhakwonseo_page1` golden 회귀를 검증했다.

## 확인 결과

최초 snapshot 실행에서 golden 불일치가 발생했다. diff를 확인한 결과, 서명란 `U+F012B`가 `(인)`으로 바뀌는 의도한 변화 외에 하단 접수증 블록이 20px 우측 이동하는 부작용이 있었다.

원인은 `U+F081C` HWP TAC filler가 `display_text` 측정 경로로 들어가면서 기존 0폭 규칙을 우회한 점이었다. `effective_text_for_metrics()`에서 `U+F081C`가 포함된 run은 원문을 반환하도록 보정하여 `text_measurement`의 0폭 처리 규칙을 유지했다.

작업지시자 시각 검증 피드백에서 하단 왼쪽 주석 앞 깨진 글리프가 추가로 확인되었다. 첨부 기준 PDF(`pdf/복학원서-2022.pdf`) 렌더를 확인한 결과 해당 위치는 `(인)`이 아니라 `※` 주석 시작부이며, 현재 SVG에는 정상 `※` 옆에 `U+F081C` filler 텍스트 노드가 중복 출력되고 있었다. 따라서 `U+F081C`는 `(인)`으로 치환하지 않고 실제 렌더 출력에서 숨기도록 보정했다.

## 변경 내용

- `src/renderer/composer.rs`
  - `effective_text_for_metrics()`에 `U+F081C` filler 예외를 추가했다.
  - `expand_pua_render_text()`에서 `U+F081C` filler를 출력하지 않도록 했다.
- `src/renderer/composer/tests.rs`
  - `test_677_effective_text_for_metrics_preserves_f081c_filler`를 추가했다.
- `tests/issue_937.rs`
  - `U+F081C` filler가 실제 렌더 텍스트로 출력되지 않는 회귀 테스트를 추가했다.
  - 복학원서 SVG 산출물에 원본 `U+F081C`가 직접 출력되지 않음을 검증했다.
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg`
  - 서명란 `U+F012B` 출력만 `(인)` 텍스트 노드로 갱신했다.
  - 하단 왼쪽 주석 앞에 중복 출력되던 `U+F081C` filler 텍스트 노드 2개를 제거했다.

## Golden diff 요약

의도한 변경만 남았다.

- 기존: `󰄫(Signature)`
- 변경: `(인)(Signature)`
- 하단 접수증 블록 좌표 변화 없음
- `U+F081C` filler 0폭 규칙 유지
- 하단 주석 시작부 `※`는 유지하고, 깨진 filler 글리프만 제거

## 산출물

```text
output/svg/task937/복학원서.svg
```

확인 사항:

- 서명란 셀 `cell-clip-116`에서 `(`, `인`, `)` 텍스트 노드 출력
- 원본 `U+F012B` 문자는 SVG 산출물에 직접 출력되지 않음
- 원본 `U+F081C` filler 문자는 SVG 산출물에 직접 출력되지 않음
- 하단 주석 `※` 2개와 빨간 `㊞` 도장은 유지됨

## Studio 시각 검증 준비

WASM 패키지를 Docker로 재빌드하고 `rhwp-studio` 개발 서버를 재시작했다.

```bash
docker-compose --env-file .env.docker run --rm wasm
npm run dev -- --host 127.0.0.1 --port 7700
```

확인 URL:

```text
http://127.0.0.1:7700/
```

Headless Chrome으로 `rhwp-studio`에서 `복학원서.hwp`를 로드한 뒤 `renderPageSvg(0)` 결과를 확인했다.

- `(인)(Signature)` 포함
- `U+F012B` 원문 출력 없음
- `U+F081C` 원문 출력 없음
- 깨진 filler 글리프(`󰠜`) 출력 없음
- 하단 주석 `※` 2개 유지

## 검증

```bash
cargo test --test svg_snapshot issue_677_bokhakwonseo_page1
cargo test --test issue_937
cargo test --test issue_826
cargo test --lib effective_text_for_metrics
cargo check --features native-skia --lib
cargo run --bin rhwp -- export-svg samples/복학원서.hwp -o output/svg/task937
docker-compose --env-file .env.docker run --rm wasm
```

결과:

- `issue_677_bokhakwonseo_page1` — 통과
- `issue_937` — 4개 통과
- `issue_826` — 4개 통과
- `effective_text_for_metrics` — 4개 통과
- `native-skia` feature check — 통과
- SVG 산출물 생성 — 통과
- WASM 패키지 재생성 — 통과

`cargo test --lib effective_text_for_metrics`에서는 기존 warning 6건이 출력되었으며, 이번 작업의 신규 warning은 확인되지 않았다.

## 결론

Stage 3 목표와 작업지시자 시각 검증 피드백 반영을 완료했다. 다음 Stage 4에서는 최종 보고서 작성, 오늘할일 상태 갱신, 최종 테스트 범위 재확인 후 승인 요청 상태로 정리한다.

## 승인 요청

Stage 4 최종 정리를 진행해도 되는지 승인 요청한다.
