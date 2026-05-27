# 최종 보고서 — Task #1139

## 요약

`3-09월_교육_통합_2022.hwp` 5쪽 문24 수식에서 한컴 대비 이상 문자처럼 보이던 부분을 조사했다. 수식 명령어가 문자로 출력되는 문제가 아니라, 큰 둥근 괄호가 너무 얇은 단일 곡선으로 렌더되어 세로 막대처럼 보이는 문제로 확인했다.

## 변경

- SVG 수식 렌더러의 stretched round parenthesis를 cubic path로 변경했다.
- Canvas 수식 렌더러도 동일한 path 정책으로 맞춰 rhwp-studio 표시 경로와 SVG export 경로를 일치시켰다.
- 문24 fixture 기반 회귀 테스트를 추가해 `LEFT/RIGHT` 명령 문자열이 누출되지 않고 큰 괄호가 곡선 path로 출력되는지 검증했다.

## 진단 메모

- 문23 `lim` 수식은 명령 누출 없이 정상 구조로 렌더된다.
- 문24 `LEFT ( {pi} over {2} -x RIGHT )`의 기존 괄호 path가 한컴 대비 이상 문자처럼 보이는 핵심 후보였다.
- 문27의 작은 `△△` 모양은 Equation 텍스트 누출이 아니라 원본의 TAC `Control::Picture`로 분리했다.

## 검증

```bash
cargo test issue_1139 --lib
cargo test renderer::equation::svg_render::tests --lib
cargo build --release
./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_after
wasm-pack build --target web --out-dir pkg
cargo test --lib
```

결과:

- `issue_1139` 테스트: 1 passed
- SVG 렌더러 테스트: 13 passed
- release build: 성공
- 대상 페이지 SVG export: 성공
- WASM build: 성공
- 전체 lib 테스트: 1406 passed, 0 failed, 6 ignored

## 후속

Stage 2 수정 후 작업지시자가 한컴오피스 화면과 아직 다르다고 재보고했다. Stage 3에서 큰 둥근 괄호 폭/패딩/선폭을 추가로 줄였고 자동 검증은 완료했다.

Stage 3 추가 검증:

- `cargo fmt --check`: 통과
- `cargo test issue_1139 --lib`: 1 passed
- `cargo build --release`: 성공
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_stage3`: 성공
- `cargo test renderer::equation::svg_render::tests --lib`: 13 passed
- `wasm-pack build --target web --out-dir pkg`: 성공
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored

UI/렌더링 정합 작업이므로 한컴오피스 화면과 rhwp-studio 화면의 최종 시각 확인은 작업지시자 판정 대기 상태다.

## Stage 5 재분류 및 추가 수정

작업지시자가 Stage 4 괄호 glyph 실험 후에도 한컴오피스와 완전히 다르다고 재보고했다. Stage 4 변경은 본질과 맞지 않아 커밋하지 않고 원복했다.

새 진단 결과, 문27 우측 문단의 작은 inline `Picture`는 원본에 `pi321 ci10`, `pi323 ci4` 두 개만 존재하지만 렌더 트리에는 각각 3회/2회로 중복 출력되고 있었다. 원인은 `paragraph_layout.rs`의 run 종료 후 TAC Picture fallback이 현재 줄 이후의 미래 TAC까지 매 줄마다 미리 렌더한 것이다.

수정:

- fallback에서 현재 line range 밖(`tac_pos > line_end_char`)의 TAC는 처리하지 않도록 제한했다.
- `tests/issue_1139_inline_picture_duplicate.rs`를 추가해 page 5의 작은 `bin=5` inline picture가 원본 컨트롤 2개만 렌더되는지 검증했다.

Stage 5 검증:

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate`: 1 passed
- `cargo build --release`: 성공
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_stage5`: 성공
- stage5 SVG의 작은 `23.8x17.6` inline picture: 5개에서 2개로 감소
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored
- `wasm-pack build --target web --out-dir pkg`: 성공

## Stage 6 페이지 수 정합

작업지시자가 `3-09월_교육_통합_2022.hwp`가 한컴오피스 기준 23쪽인데 rhwp-studio에서는 24쪽으로 표시되며, 9쪽부터 화면이 달라진다고 보고했다.

원인은 미주 배치 루프의 문단 bottom 누적 기준이었다. 일부 미주 문단은 내부 `LINE_SEG.vertical_pos`가 뒤쪽 줄에서 더 작은 값으로 되감기는데, 기존 코드는 미주 문단의 bottom을 마지막 line segment 기준으로 저장했다. 이 때문에 다음 미주 문단과의 간격이 실제보다 크게 계산되어 9쪽에서 `pi=523` 이후 미주가 밀리기 시작했고, 최종적으로 24쪽이 추가 생성됐다.

수정:

- `src/renderer/typeset.rs`에서 미주 문단 bottom과 trailing line spacing을 마지막 줄이 아니라 가장 큰 line bottom을 가진 줄 기준으로 계산하도록 변경했다.
- `tests/issue_1139_inline_picture_duplicate.rs`에 한컴 기준 23페이지 및 9쪽 미주 연속 배치 회귀 테스트를 추가했다.

Stage 6 검증:

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate`: 2 passed
- `cargo test --test issue_1082_endnote_multicolumn_drift`: 4 passed
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored
- `cargo build --release`: 성공
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`: 23페이지
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 8 -o output/diag_1139_stage6_page9`: 성공
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 22 -o output/diag_1139_stage6_page23`: 성공
- `wasm-pack build --target web --out-dir pkg`: 성공

UI/렌더링 정합 작업이므로 최종 한컴오피스 대비 시각 확인은 작업지시자 판정 대기 상태다.

## Stage 7 미주 내부 Shape 렌더링 보정

작업지시자가 9쪽 `문7)` 풀이 중간의 `[다른 풀이]`가 rhwp-studio에서 표시되지 않는다고 재보고했고, 개체 속성 화면으로 해당 표식이 일반 문단 텍스트가 아니라 도형/글상자 개체임을 확인했다.

진단 결과 `[다른 풀이]`는 `문7)` 미주 내부의 TAC `Control::Shape` 그룹에 포함된 글상자 텍스트였다. 본문 문단 경로는 `Control::Shape`를 `PageItem::Shape`로 별도 등록하지만, 미주 가상 문단 삽입 경로는 `FullParagraph`만 추가하고 도형 렌더 항목을 만들지 않아 실제 Shape와 글상자 텍스트가 렌더 트리에 들어가지 않았다.

수정:

- `src/renderer/typeset.rs`에서 미주 가상 문단을 추가할 때 해당 미주 문단의 `Control::Shape`도 `PageItem::Shape`로 등록하도록 변경했다.
- `tests/issue_1139_inline_picture_duplicate.rs`에 9쪽 렌더 트리에 `다른 풀이` 텍스트가 포함되는지 확인하는 회귀 테스트를 추가했다.

Stage 7 검증:

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate`: 3 passed
- `cargo test --test issue_1082_endnote_multicolumn_drift`: 4 passed
- `cargo build --release`: 성공
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`: 23페이지
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 8 -o output/diag_1139_stage7_page9`: 성공
- Stage 7 page 9 SVG: `다른 풀이` 글상자 glyph 포함 확인
- `wasm-pack build --target web --out-dir pkg`: 성공
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored

UI/렌더링 정합 작업이므로 한컴오피스 대비 실제 위치와 흐름은 작업지시자 판정 대기 상태다. 왼쪽 단 `문5)` 세로 위치 차이는 이번 단계에서 별도 위치 보정으로 확정하지 않았다.

## Stage 8 미주 흐름과 가상 Shape 속성 보정

작업지시자가 9쪽에서 `문5)` 시작 위치가 여전히 높고, `문8)`은 한컴오피스처럼 10쪽에서 시작해야 한다고 재보고했다. 추가로 `[다른 풀이]` 개체를 선택한 뒤 `개체 속성(P)...`으로 진입하지 못하는 문제도 확인했다.

수정:

- Stage 6 회귀 테스트의 잘못된 가정(`pi=523`이 9쪽에 남아야 함)을 한컴 기준으로 정정했다.
- `src/renderer/typeset.rs`에서 되감기는 `LINE_SEG.vertical_pos` 뒤의 미주 묶음 fit 판단을 보정해 9쪽은 `pi=522`까지, 10쪽은 `pi=523 "문8)   ①"`부터 시작하도록 맞췄다.
- `src/document_core/commands/object_ops.rs`에서 Shape 속성 조회/수정 API가 미주 가상 문단 인덱스(`paraIdx=518`)를 실제 `Control::Endnote` 내부 문단으로 역해석하도록 수정했다.
- 작업지시자가 제공한 한컴오피스 미주 설정(`문`, `)`, 구분선 50mm, 미주 사이 7mm, 구분선 아래 2mm)을 회귀 테스트로 확인했다. 이 샘플의 7mm 값은 HWP5 `FOOTNOTE_SHAPE.raw_unknown`에 보존되지만, `LINE_SEG` 흐름에 이미 반영되어 있어 typeset에 별도 가산하지 않는다.

Stage 8 검증:

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate`: 5 passed
- `cargo test --test issue_1082_endnote_multicolumn_drift`: 4 passed
- `cargo build --release`: 성공
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`: 23페이지
- page 9 dump: `pi=522` 포함, `pi=523` 없음
- page 10 dump: `pi=523 "문8)   ①"` 시작 확인
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 8 -o output/diag_1139_stage8_page9`: 성공
- `wasm-pack build --target web --out-dir pkg`: 성공
- Node/WASM 확인: `pageCount()` 23, `[다른 풀이]` group `paraIdx=518`, `getShapeProperties(0, 518, 0)` 성공
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored

UI/렌더링 정합 작업이므로 Stage 8도 최종 한컴오피스 대비 시각 확인은 작업지시자 판정 대기 상태다.
