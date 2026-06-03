# Stage 8 분석 — Task #1139

## 배경

작업지시자가 `3-09월_교육_통합_2022.hwp` 9쪽 시각 정합을 다시 확인했다.

Stage 7에서는 9쪽 오른쪽 단의 `[다른 풀이]`가 미주 내부 TAC Shape 그룹 글상자임을 확인하고 렌더 누락을 보정했다. 이번 단계의 범위는 그 다음 흐름 차이다.

## 한컴오피스 기준

- 9쪽 왼쪽 단의 `문5)` 시작 위치가 rhwp-studio보다 더 아래쪽이다.
- 9쪽 오른쪽 단은 `문7)`의 `[다른 풀이]` 이후 풀이까지 표시된다.
- `문8)`은 9쪽이 아니라 10쪽에서 시작한다.
- `[다른 풀이]` 개체를 우클릭하면 `개체 속성(P)...` 메뉴로 개체 속성에 들어갈 수 있어야 한다.

## 현재 rhwp 결과

현재 브랜치 `local/task_m100_1139`의 release dump 기준:

```text
=== 페이지 9 ===
단 0
  FullParagraph[미주]  pi=491  "문5)   ③"
  ...
  FullParagraph[미주]  pi=497  "㉠, ㉡에서 , 이므로"
단 1
  FullParagraph[미주]  pi=498  "(빈)"
  ...
  FullParagraph[미주]  pi=511  "문7)   ⑤"
  ...
  FullParagraph[미주]  pi=522  vpos=124748..94604  "(빈)"
  FullParagraph[미주]  pi=523  "문8)   ①"
  FullParagraph[미주]  pi=524  ...
  ...
  FullParagraph[미주]  pi=528  ...

=== 페이지 10 ===
단 0
  FullParagraph[미주]  pi=529
  ...
```

즉 현재 rhwp는 `문8)`의 첫 미주 문단 `pi=523`부터 `pi=528`까지를 9쪽 하단에 넣고, 10쪽은 `pi=529`부터 시작한다.

## 잘못된 이전 가정

Stage 6에서는 페이지 수 23쪽을 맞추는 과정에서 `pi=523`이 9쪽에 남아야 한다고 판단했다. 작업지시자의 한컴오피스 기준 화면으로 볼 때 이 가정은 틀렸다.

따라서 기존 회귀 테스트의 다음 조건은 Stage 8에서 정정되어야 한다.

```rust
page9.contains("FullParagraph[미주]  pi=523")
```

정답 조건은 다음에 가깝다.

- 9쪽에는 `pi=522`까지 표시된다.
- 9쪽에는 `pi=523`이 없어야 한다.
- 10쪽에는 `pi=523` 또는 `문8)`이 시작해야 한다.
- 전체 페이지 수는 계속 23쪽이어야 한다.

## 원인 후보

첫 번째 비정상 지점은 Stage 6에서도 확인했던 `pi=522`다.

```text
FullParagraph[미주]  pi=522  vpos=124748..94604
```

이 문단은 같은 미주 문단 내부에서 `LINE_SEG.vertical_pos`가 큰 값에서 작은 값으로 되감기는 패턴이다. 현재 Stage 6 보정은 미주 문단의 bottom을 마지막 line segment가 아니라 최대 line bottom 기준으로 잡아 24쪽 증가 문제는 줄였지만, `pi=522` 뒤의 다음 미주 `pi=523`을 같은 쪽에 넣을 만큼 흐름을 과소 소비하고 있다.

즉 문제는 `[다른 풀이]` Shape 표시 여부가 아니라, 되감기는 `LINE_SEG.vertical_pos`를 가진 미주 문단의 logical height와 다음 미주 시작 판단이 한컴오피스보다 작게 계산되는 쪽에 있다.

왼쪽 단 `문5)` 시작 위치도 같은 미주 흐름 누적 문제의 앞쪽 증상일 가능성이 있다. `문5)`는 가상 미주 문단 `pi=491`이며, 현재 9쪽 왼쪽 단에서 한컴보다 높은 위치에 시작한다.

추가로 `[다른 풀이]`는 Stage 7에서 렌더 항목으로는 등록했지만, 개체 속성 진입이 되지 않는다면 hit-test 또는 page control layout API에는 미주 내부 가상 문단 Shape가 아직 개체로 노출되지 않았을 가능성이 있다. 이 경우 표시와 선택/속성 진입은 서로 다른 경로이므로, 렌더 누락 보정과 별도로 미주 Shape의 상호작용 메타데이터를 확인해야 한다.

WASM API로 확인한 결과, page 9 control layout에는 `[다른 풀이]` 그룹이 다음처럼 노출된다.

```json
{
  "type": "group",
  "x": 402.5,
  "y": 681.9,
  "w": 68.0,
  "h": 19.7,
  "secIdx": 0,
  "paraIdx": 518,
  "controlIdx": 0
}
```

하지만 `getShapeProperties(0, 518, 0)` 호출은 실패한다.

```text
렌더링 오류: 문단 인덱스 518 범위 초과
```

`paraIdx=518`은 실제 `section.paragraphs`의 본문 문단 인덱스가 아니라, typeset/rendering에서 만든 미주 가상 문단 인덱스다. 현재 `getShapeProperties`/`setShapeProperties`는 실제 본문 `section.paragraphs[parent_para_idx].controls[control_idx]`만 조회하므로, 미주 내부 Shape의 속성 경로를 해석하지 못한다.

## 수정 계획

소스 수정 승인 후 다음 순서로 진행한다.

1. 기존 Stage 6 테스트의 잘못된 `pi=523` 9쪽 포함 조건을 한컴 기준으로 정정한다.
2. 9쪽/10쪽 dump 기반 회귀 테스트를 추가한다.
   - 9쪽에는 `pi=522`가 있고 `pi=523`은 없어야 한다.
   - 10쪽에는 `pi=523`이 있어야 한다.
   - 페이지 수는 23쪽을 유지해야 한다.
3. `src/renderer/typeset.rs`의 미주 가상 문단 누적에서 `LINE_SEG.vertical_pos`가 되감기는 문단의 logical advance를 재검토한다.
4. `pi=522`처럼 되감기는 문단 뒤에 다음 미주가 과도하게 붙지 않도록 fit/advance 기준을 보정한다.
5. Stage 7의 Shape 렌더링 보정은 유지한다.
6. `[다른 풀이]` Shape가 page control layout 또는 hit-test 결과에 포함되는지 확인하고, 누락 시 미주 내부 `PageItem::Shape`의 선택/속성 진입 메타데이터를 보정한다.
7. 미주 가상 문단 `paraIdx`가 속성 API로 전달될 때 원본 경로를 찾을 수 있도록 source mapping을 추가하거나, 미주 내부 Shape 전용 속성 조회 경로를 추가한다.

## 검증 계획

- `cargo fmt --check`
- `cargo test --test issue_1139_inline_picture_duplicate`
- `cargo test --test issue_1082_endnote_multicolumn_drift`
- `cargo build --release`
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 8`
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 9`
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`
- page 9 render tree/control layout에서 `[다른 풀이]` Shape의 `para_index`/`control_index` 노출 확인
- `getShapeProperties` 또는 대체 속성 API로 `[다른 풀이]` 개체 속성 조회 성공 확인
- `wasm-pack build --target web --out-dir pkg`

UI/렌더링 정합 변경이므로 자동 검증 후 작업지시자의 한컴오피스 대비 시각 확인을 기다린다.

## 구현 결과

작업지시자의 승인 후 다음을 수정했다.

- `src/renderer/typeset.rs`
  - 되감기는 `LINE_SEG.vertical_pos`를 가진 미주 문단 뒤에서 다음 미주 묶음이 남은 하단 여백에 과도하게 붙지 않도록, 다단 미주 흐름의 near-bottom keep 판단을 추가했다.
  - 미주 문단의 다음 시작 기준 `vpos_offset`을 마지막 줄이 아니라 해당 문단에서 가장 낮은 line bottom 기준으로 갱신했다.
  - 9쪽은 `pi=522`까지 표시하고, `문8)` 첫 문단인 `pi=523`은 10쪽에서 시작하도록 회귀 조건을 정정했다.
- `src/document_core/commands/object_ops.rs`
  - Shape 속성 조회/수정 API가 미주 가상 문단 인덱스를 받으면 실제 본문 문단의 `Control::Endnote` 내부 문단으로 역해석하도록 보정했다.
  - `[다른 풀이]`의 page control layout이 넘기는 `secIdx=0, paraIdx=518, controlIdx=0`으로 `getShapeProperties`가 성공한다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - 페이지 수 23쪽, 9쪽 `pi=522` 포함, 9쪽 `pi=523` 제외, 10쪽 `pi=523` 포함 조건으로 Stage 6의 잘못된 가정을 정정했다.
  - 미주 내부 Shape 속성 API 회귀 테스트를 추가했다.
  - 작업지시자가 제공한 한컴오피스 미주 설정을 확인하는 테스트를 추가했다.

## 한컴오피스 미주 설정 참고

작업지시자가 제공한 한컴오피스 미주 모양 설정은 다음과 같다.

- 번호 모양: `1,2,3`
- 앞 장식 문자: `문`
- 뒤 장식 문자: `)`
- 구분선 넣기: 켜짐
- 구분선 위: `0.0mm`
- 미주 사이: `7.0mm`
- 구분선 아래: `2.0mm`
- 번호 매기기: 앞 구역에 이어서
- 미주 내용 번호 속성: 보통
- 미주 위치: 문서의 끝

HWP5 `FOOTNOTE_SHAPE` raw 확인 결과, 이 샘플의 두 번째 `FOOTNOTE_SHAPE`에는 `prefix=문`, `suffix=U+FF09`, 구분선 길이 약 `50.0mm`, `note_spacing` 약 `2.0mm`, `raw_unknown` 약 `7.0mm`가 들어 있다. `raw_unknown` 7mm를 typeset에 별도로 더하는 실험은 전체 페이지가 25쪽으로 늘어 실패했다. 이 문서의 미주 내부 `LINE_SEG` 흐름에는 해당 간격이 이미 반영되어 있으므로, Stage 8 최종 수정에서는 이를 이중 계산하지 않는다.

## 검증 결과

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate`: 5 passed
- `cargo test --test issue_1082_endnote_multicolumn_drift`: 4 passed
- `cargo build --release`: 성공
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`: 23페이지
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 8`: 9쪽 `pi=522` 포함, `pi=523` 없음
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 9`: 10쪽 `pi=523 "문8)   ①"` 시작 확인
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 8 -o output/diag_1139_stage8_page9`: 성공
- `wasm-pack build --target web --out-dir pkg`: 성공
- Node/WASM 확인: `pageCount()` 23, page 9 control layout의 `[다른 풀이]` 그룹 `paraIdx=518`, `getShapeProperties(0, 518, 0)` 결과 `width=5102`, `height=1474`
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored

자동 검증은 완료했다. UI/렌더링 정합 변경이므로 한컴오피스 대비 최종 시각 판정은 작업지시자 확인을 기다린다.
