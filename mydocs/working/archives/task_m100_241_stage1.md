# Task m100 #241 Stage 1: 그림 세로 기준 `문단` 바인딩 정정

## 1. 목적

`samples/hwpx/issue_241.hwpx`의 도장 그림 속성 대화창에서 세로 위치 기준 `문단`이 바인딩되지 않는 문제를 수정한다.

## 2. 원인

WASM/Rust 계층의 그림 속성 JSON contract는 다음 값을 사용한다.

```text
vertRelTo = "Paper" | "Page" | "Para"
horzRelTo = "Paper" | "Page" | "Column" | "Para"
```

그러나 rhwp-studio 그림 속성 대화창은 세로 기준의 `문단` option 값을 `"Paragraph"`로 정의하고 있었다.

```ts
this.vertRelSelect = this.selectEl([
  ['Paper', '종이'], ['Page', '쪽'], ['Paragraph', '문단'],
]);
```

결과:

- `populateFromProps()`에서 `this.vertRelSelect.value = this.props.vertRelTo` 실행 시 `"Para"`가 option에 없어 선택되지 않는다.
- 사용자가 설정을 적용해도 `"Paragraph"`가 전달되면 Rust setter가 `VertRelTo::Para`로 매핑하지 못한다.

## 3. 수정

`picture-props-dialog.ts`의 세로 기준 option value를 `"Para"`로 정정한다.

```ts
['Paper', '종이'], ['Page', '쪽'], ['Para', '문단'],
```

## 4. 판정표

| file | 속성 대화창 열기 | 세로 기준 `문단` 바인딩 | 저장/적용 | 렌더링 영향 | 비고 |
|---|---|---|---|---|---|
| `samples/hwpx/issue_241.hwpx` |  |  |  |  | target |
| `samples/hwpx/hancom-hwp/issue_241.hwp` |  |  |  |  | guard |

## 5. 구현 결과

수정 파일:

- `rhwp-studio/src/ui/picture-props-dialog.ts`

수정 내용:

```diff
- ['Paper', '종이'], ['Page', '쪽'], ['Paragraph', '문단'],
+ ['Paper', '종이'], ['Page', '쪽'], ['Para', '문단'],
```

## 6. 정적 검증

```text
./rhwp-studio/node_modules/.bin/tsc --noEmit -p rhwp-studio/tsconfig.json
=> success
```

## 7. 샘플 속성 확인

CLI dump 기준으로 target/guard 모두 도장 그림의 세로 기준은 이미 `문단`으로 파싱된다.

```text
target samples/hwpx/issue_241.hwpx
  위치: 가로=단 오프셋=103.3mm(29292) 정렬=Left,
        세로=문단 오프셋=2.7mm(754) 정렬=Top

guard samples/hwpx/hancom-hwp/issue_241.hwp
  위치: 가로=단 오프셋=103.3mm(29292) 정렬=Left,
        세로=문단 오프셋=2.7mm(754) 정렬=Top
```

따라서 이번 단계의 핵심은 파서/IR 문제가 아니라 rhwp-studio UI select option value mismatch로 확정한다.

## 8. 메인테이너 판정 요청

rhwp-studio에서 `samples/hwpx/issue_241.hwpx`를 열고 도장 이미지를 선택한 뒤,
개체 속성 대화창의 세로 기준이 `문단`으로 선택되는지 확인한다.

## 9. PDF 기준 도장 위치 비교

바인딩 성공 후 남은 배치 의심 축은 `samples/hwpx/issue_241.pdf`의 도장 위치와 현재 rhwp 산출물을
좌표로 비교했다.

PDF를 SVG로 변환한 결과, 도장 이미지는 다음 transform으로 배치된다.

```text
samples/hwpx/issue_241.pdf
  transform="matrix(0.135664, 0, 0, 0.129695, 377.797, 664.912)"

pdf/hwpx/issue_241-2022.pdf
  transform="matrix(0.143922, 0, 0, 0.143843, 377.797, 664.912)"
```

두 PDF 모두 최종 도장 bbox는 동일하다. 72dpi PDF 좌표를 rhwp의 96dpi 페이지 좌표로 환산하면 다음과 같다.

| source | x | y | width | height |
|---|---:|---:|---:|---:|
| Hancom PDF reference | 503.729 | 886.549 | 88.272 | 84.388 |
| rhwp `export-svg` | 503.947 | 887.400 | 88.320 | 84.533 |
| delta | +0.217 | +0.851 | +0.048 | +0.145 |

따라서 도장 이미지 자체 bbox는 PDF 기준과 1px 미만의 반올림 오차 수준으로 일치한다.
다만 후속 디버그에서 이미지 자체 좌표와 별개로, 이미지가 정의된 `s0:pi=9` 문단의 flow 높이가
확보되지 않아 `s0:pi=10` 문단이 같은 y에서 시작하는 문제가 확인되었다.

## 10. 회귀 테스트

PDF 기준 좌표를 `getPageOverlayImages()` 경로까지 고정하기 위해 회귀 테스트를 추가했다.

```text
tests/issue_241.rs
```

검증 항목:

- `samples/hwpx/issue_241.hwpx`의 도장 그림이 `front` overlay 1개로 산출되는지 확인
- 도장 bbox가 Hancom PDF 기준 좌표와 1px 이내로 일치하는지 확인

## 11. 디버그 오버레이 라벨 충돌 수정

`export-svg --debug-overlay --show-grid=3mm` 출력에서 `s0:pi=9`와 `s0:pi=10` 라벨이 같은 좌표에 배치되어
`s0:pi=9` 라벨이 보이지 않는 문제가 있었다.

실제 문단 경계는 둘 다 존재하지만, 디버그 도구가 원인 추적에 필요한 라벨을 가리는 상태이므로
SVG 디버그 오버레이 라벨에 충돌 회피 배치를 적용했다.

생성 파일:

```text
output/poc/task241_debug_grid_label_fix/hwpx/issue_241.svg
output/poc/task241_debug_grid_label_fix/hwp_guard/issue_241.svg
```

확인 결과:

```text
s0:pi=8  label y=854.0
s0:pi=9  label y=875.3
s0:pi=10 label y=887.3
```

따라서 도장 그림이 포함된 `s0:pi=9` 문단 라벨을 격자/디버그 레이아웃에서 직접 확인할 수 있다.

## 12. 이미지 bbox 라벨 추가

한컴 편집기 판정 화면에서는 `s0:pi=9` 위치에 `[그림]` 조판부호가 표시되고,
도장 이미지는 해당 문단 기준으로 오른쪽에 배치된다.

이를 디버그 SVG에서도 직접 확인할 수 있도록 이미지 bbox 라벨을 추가했다.

생성 파일:

```text
output/poc/task241_debug_grid_image_label/hwpx/issue_241.svg
output/poc/task241_debug_grid_image_label/hwp_guard/issue_241.svg
```

확인 결과:

```text
s0:pi=9 y=877.3
s0:pi=10 y=877.3
s0:pi=9 ci=0 image y=887.4
```

즉 도장 이미지는 `s0:pi=9` 문단의 `ci=0` 이미지 컨트롤이며,
native SVG/overlay bbox 기준에서는 HWPX target과 HWP guard가 동일 좌표로 출력된다.

## 13. 문단 flow height 미확보 수정

한컴 편집기에서는 `s0:pi=9`에 `[그림]` 조판부호가 별도 문단으로 표시되고,
그 문단의 line advance 뒤에 `s0:pi=10` 날짜 문단이 배치된다.

반면 수정 전 rhwp SVG에서는 다음처럼 두 문단의 시작 y가 같았다.

```text
s0:pi=9  y=877.3
s0:pi=10 y=877.3
```

원인은 `PageItem::FullParagraph(pi=9)`가 line advance를 반영한 뒤,
뒤이어 렌더되는 `PageItem::Shape(pi=9, ci=0)`가 Para-relative `InFrontOfText` 그림을 그리면서
반환 y를 문단 시작 y로 되돌린 데 있었다.

정정:

```text
Para-relative InFrontOfText/BehindText 그림은 좌표 계산에는 host paragraph y를 사용한다.
하지만 본문 flow cursor는 이미 진행된 y를 유지해야 하므로 Shape item에서 되감지 않는다.
```

수정 후 디버그 SVG:

```text
output/poc/task241_debug_grid_image_label/hwpx/issue_241.svg

s0:pi=9  y=877.3
s0:pi=10 y=898.7
s0:pi=9 ci=0 image y=887.4
```

즉 `s0:pi=9`가 약 21.3px, 곧 `line_height + line_spacing = 1600 HU`에 해당하는 문단 높이를 확보한다.

추가 회귀 테스트:

```text
tests/issue_241.rs
  issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height
```

검증 항목:

- 도장 overlay y는 Hancom PDF 기준과 유지
- `s0:pi=10` 날짜 문단 y가 도장 y보다 충분히 아래로 배치되어 `s0:pi=9` flow advance가 유지되는지 확인

## 14. 최종 판정

검증:

```text
cargo fmt
cargo check
cargo test --test issue_241
./rhwp-studio/node_modules/.bin/tsc --noEmit -p rhwp-studio/tsconfig.json
git diff --check
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
cargo check: success
issue_241: 2 passed
tsc: success
diff check: success
WASM build: success
```

작업지시자 판정:

```text
SVG 시각 판정: 통과
rhwp-studio 시각 판정: 통과
```
