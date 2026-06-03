# Stage 7 분석 — Task #1139

## 배경

작업지시자가 `3-09월_교육_통합_2022.hwp` 9쪽에서 한컴오피스와 표시 내용이 다르다고 보고했다.

이번 단계에서는 PR #1137의 쪽 외곽선/격자 문제는 제외한다. 확인 대상은 미주 본문 흐름과 `[다른 풀이]` 표시다.

## 관찰

한컴오피스 9쪽 오른쪽 단의 `문7)` 풀이 중간에는 `[다른 풀이]` 표식이 보인다. 작업지시자가 제공한 개체 속성 화면에 따르면 이 표식은 일반 문단 텍스트가 아니라 개체 속성을 가진 도형/글상자 항목이다.

rhwp-studio 쪽에서는 같은 위치에 개체가 표시되지 않고, 이후 `문8)`이 9쪽 하단에 이어진다.

## 원인

진단 결과 `[다른 풀이]`는 본문 텍스트가 아니라 `문7)` 미주 내부의 treat-as-char `Control::Shape`다.

```text
body pi=55 ci=0 endnote num=7 paras=12
  ep=7 text="" cc=9 controls=1 lines=1
    ctrl[0] Shape
      shape kind=묶음 tac=true wrap=InFrontOfText rel=(Para,Para)
        children=2
          shape kind=사각형 textbox text="수학교실"
          shape kind=곡선 textbox text="다른 풀이"
```

현재 `src/renderer/typeset.rs`의 미주 가상 문단 삽입 루프는 각 미주 문단을 `PageItem::FullParagraph`로만 추가한다. 본문 문단 처리 경로는 `Control::Shape`/`Picture`/`Equation` 컨트롤을 별도 `PageItem::Shape`로도 추가하지만, 미주 문단 경로에는 이 처리가 없다.

따라서 미주 내부 TAC Shape는 문단의 개체 치환 문자 자리만 렌더되고, 실제 그룹 도형 및 글상자 내용인 `[다른 풀이]`가 렌더 트리에 들어가지 않는다.

## 수정 계획

소스 수정 승인 후 다음 범위로 진행한다.

1. 미주 가상 문단을 `FullParagraph`로 push한 직후, 해당 미주 문단의 `Control::Shape`를 `PageItem::Shape`로 함께 등록한다.
2. 우선 이번 결함의 직접 원인인 `Control::Shape`에 한정해 blast radius를 줄인다. 필요 시 `Picture`는 별도 단계에서 검토한다.
3. page 9 render tree에 `다른 풀이` 텍스트박스가 포함되는 회귀 테스트를 추가한다.
4. 기존 page count 23쪽 회귀 테스트를 유지한다.

## 검증 계획

- `cargo fmt --check`
- `cargo test --test issue_1139_inline_picture_duplicate`
- `cargo test --test issue_1082_endnote_multicolumn_drift`
- `cargo build --release`
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`
- `wasm-pack build --target web --out-dir pkg`

UI/렌더링 정합 변경이므로 자동 검증 후 작업지시자의 한컴오피스 대비 시각 확인을 기다린다.

## 수정 결과

`src/renderer/typeset.rs`의 미주 가상 문단 삽입 경로에서 `FullParagraph`를 추가한 직후, 해당 미주 문단의 `Control::Shape`를 `PageItem::Shape`로 함께 등록하도록 수정했다.

이번 결함의 직접 원인인 미주 내부 TAC Shape 렌더 누락만 보정하기 위해 `Control::Shape`에 한정했다. 문단 높이 누적 방식은 Stage 6 보정을 유지하고, 이번 단계에서는 도형 렌더 항목 등록만 추가했다.

`tests/issue_1139_inline_picture_duplicate.rs`에는 9쪽 렌더 트리에 `다른 풀이` 텍스트가 포함되는지 확인하는 회귀 테스트를 추가했다.

## 검증 결과

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate`: 3 passed
- `cargo test --test issue_1082_endnote_multicolumn_drift`: 4 passed
- `cargo build --release`: 성공
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`: 23페이지
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 8 -o output/diag_1139_stage7_page9`: 성공
- Stage 7 page 9 SVG: `다른 풀이` 글상자 glyph 포함 확인
- `wasm-pack build --target web --out-dir pkg`: 성공
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored

## 판정 대기

자동 검증 기준으로는 9쪽 `문7)` 미주 내부 `[다른 풀이]` Shape 텍스트박스가 렌더 트리에 포함된다.

UI/렌더링 정합 작업이므로 한컴오피스 대비 실제 위치와 흐름은 작업지시자 시각 확인을 기다린다. 특히 작업지시자가 함께 지적한 왼쪽 단 `문5)` 세로 위치는 이번 단계에서 별도 위치 보정으로 확정하지 않았다.
