# Task M100 #1139 Stage 10

## 목적

`3-09월_교육_통합_2022.hwp` 9쪽 미주 영역이 한컴오피스 정답지와 여전히 다르게 보이는 문제를 다시 보정한다.

작업지시자가 지정한 기준은 다음과 같다.

- `target/debug/rhwp export-svg --show-grid=3mm` 산출물을 한컴오피스 정답지와 직접 비교한다.
- 9쪽 오른쪽 미주 영역의 화면 구성, 특히 `[다른 풀이]` 이하 흐름과 수식/표시 위치를 맞춘다.
- rhwp-studio에서 `[다른 풀이]` 개체를 우클릭한 뒤 `개체 속성(P)...`을 선택해도 속성 대화상자가 열리지 않는 문제를 포함한다.

## 현재 판단

Stage 9에서는 미주 구분선, 가상 미주 문단, 그룹 개체 속성 경로를 보정했지만, 작업지시자의 실제 화면에서는 다음 문제가 남아 있다.

- `export-svg --show-grid=3mm` 기준 실제 9쪽 구성과 한컴오피스 9쪽 정답지의 미주 배치가 다르다.
- `[다른 풀이]` 개체 속성 메뉴는 표시되지만 클릭 후 속성창 진입이 실패한다.
- 자동 검증에서 통과한 `getShapeProperties` API 경로와 rhwp-studio 명령 실행 경로 사이에 아직 누락된 조건이 있을 수 있다.

## 분석 계획

1. 현재 브랜치와 작업트리를 보존한 상태에서 9쪽 SVG를 `--show-grid=3mm`로 재생성한다.
2. `dump-pages`, `dump`, SVG 내 좌표를 함께 비교해 한컴 정답지와 어긋나는 첫 문단/개체를 찾는다.
3. `[다른 풀이]` 선택 상태에서 rhwp-studio 명령이 어떤 `ObjectReference`를 넘기는지 확인하고, 그룹/미주 내부 개체의 속성 진입 경로를 보정한다.
4. 수정 후 페이지 수 23쪽, 9쪽/10쪽 문단 분기, SVG 내 `[다른 풀이]`, green separator, object properties 경로를 재검증한다.

## 검증 예정

- `target/debug/rhwp export-svg ... --show-grid=3mm`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `npm run build` (`rhwp-studio`)
- `npm test` (`rhwp-studio`)
- `wasm-pack build --target web --out-dir pkg`

## 수행 결과

### 개체 속성 진입

우클릭 컨텍스트 메뉴의 `개체 속성(P)...` 항목은 `format:object-properties`가 아니라 `insert:picture-props` 명령을 호출한다.
Stage 9에서는 전자 경로만 `group` 허용으로 바뀌어, 실제 우클릭 메뉴에서는 여전히 다음 조건에 막혔다.

```ts
if (!ref || ref.type === 'equation' || ref.type === 'group') return;
```

Stage 10에서는 `insert:picture-props`에서 `group` 차단을 제거하고, `PicturePropsDialog`가 `group` 타입에도 그림 탭이 아니라 도형/그룹 속성 탭을 쓰도록 맞췄다.

### 9쪽 시각 구성 분석

`target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage10_svg -p 8 --show-grid=3mm --debug-overlay`로 재현했다.

확인된 현재 차이:

- 현재 9쪽은 전체 23쪽을 유지한다.
- 9쪽 왼쪽 단은 `pi=491 문5)` 이후 `pi=497`까지 같은 단에 남으며, `pi=497`은 `LAYOUT_OVERFLOW_DRAW ... overflow=15.5px`를 낸다.
- 한컴오피스 정답지에 가까운 후보는 `문5)` 뒤 vpos 되감김 묶음(`pi=493` 이후)을 다음 단으로 넘기는 방향이었다.
- 그러나 이 후보를 그대로 적용하면 9쪽은 가까워지지만 마지막 미주가 새 페이지로 밀려 전체가 24쪽이 된다.
- 따라서 Stage 10에는 이 레이아웃 보정을 반영하지 않았다. 페이지 수 23쪽을 깨는 보정은 커밋하지 않는다.

남은 레이아웃 과제:

- 미주 내부 vpos 되감김을 단 바닥에서 처리하되, 후속 미주 전체 페이지 수가 23쪽을 유지되도록 후반부 되감김/빈 단 회수를 함께 보정해야 한다.
- 특히 후보 보정 시 24쪽에 생기는 `pi=1163` 이후 문30) 미주가 23쪽 마지막 단으로 회수되지 못하는 원인을 별도 Stage에서 추적해야 한다.

## 검증 결과

- `target/debug/rhwp export-svg ... -p 8 --show-grid=3mm --debug-overlay`: 재현 완료. 현재 기준 23쪽 유지, `pi=497` overflow 재현.
- `npm run build` (`rhwp-studio`): 통과.
- `npm test` (`rhwp-studio`): 통과, 38개 테스트 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과, 5개 테스트 통과.

`wasm-pack build --target web --out-dir pkg`는 Stage 10의 남은 반영분이 rhwp-studio TypeScript 경로라 아직 재실행하지 않았다.
