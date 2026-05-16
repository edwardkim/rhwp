# Task #931 Stage 6 완료 보고서

## 1. 단계 목적

Stage 5 수정이 기존 단일 canvas 렌더 결과와 비교해 불필요한 회귀를 만들지 않았는지 확인하고, 실제 한컴 기준 PDF에서 추가로 확인된 차이를 #931 범위에 포함할지 판단했다.

## 2. 회귀 검증 방법

같은 문서(`samples/복학원서.hwp`)를 같은 화면 조건에서 두 방식으로 캡처했다.

1. 현재 rhwp-studio 경로
   - `page background DOM layer`
   - `BehindText DOM overlay`
   - `transparent flow canvas`
2. 기존 기준 경로
   - overlay DOM layer 제거
   - `renderPageToCanvasFiltered(0, canvas, zoom, 'all')`로 단일 canvas 렌더

캡처 후 전체 픽셀 차이와 로고/워터마크 이미지 bbox 외부 차이를 분리했다.

검증 산출물:

- `/private/tmp/rhwp-regression-931/capture-summary.json`
- `/private/tmp/rhwp-regression-931/pixel-diff-report.json`
- `/private/tmp/rhwp-regression-931/layered_100.png`
- `/private/tmp/rhwp-regression-931/layered_25.png`
- `/private/tmp/rhwp-regression-931/all_canvas_100.png`
- `/private/tmp/rhwp-regression-931/all_canvas_25.png`
- `/private/tmp/rhwp-regression-931/diff_100.png`
- `/private/tmp/rhwp-regression-931/diff_25.png`

## 3. 회귀 검증 결과

| 줌 | 전체 차이 픽셀(`>2`) | 전체 비율 | 이미지 bbox 외부 차이(`>2`) | bbox 외부 최대 차이 | 판정 |
|----|----------------------|-----------|------------------------------|---------------------|------|
| 100% | 21,307 | 1.33169% | 1,764 | 34 | 이미지 합성/안티앨리어싱 차이로 한정 |
| 25% | 2,615 | 0.16344% | 7 | 16 | 사실상 이미지 bbox 내부 차이로 한정 |

추가 임계값 확인:

- 100%에서 이미지 bbox 외부 차이는 임계값 `>40` 기준 0px였다.
- 25%에서 이미지 bbox 외부 차이는 임계값 `>20` 기준 0px였다.

결론:

- 기존 `all` canvas 기준과 완전한 byte/pixel identity는 아니다.
- 차이는 DOM `<img>`/CSS compositor 합성과 canvas 내부 이미지 합성 방식의 차이이며, 의미 있는 차이는 로고/워터마크 이미지 영역에 한정된다.
- 텍스트, 표, 페이지 배경, 배치 좌표의 회귀는 확인되지 않았다.

## 4. 최소 수정 검증

Stage 5 기준 제품 코드 변경 파일은 3개로 제한됐다.

| 파일 | 변경 이유 | 범위 판단 |
|------|-----------|-----------|
| `src/renderer/web_canvas.rs` | `FlowOnly + BehindText`일 때 flow canvas가 흰 페이지 배경을 그려 DOM BehindText를 가리는 문제 해결 | WASM canvas renderer의 page background 처리만 조건부 변경 |
| `rhwp-studio/src/view/page-renderer.ts` | page background, BehindText, flow canvas, InFrontOfText의 z-order와 overlay 정리 정책 구현 | rhwp-studio view layer 합성 범위로 제한 |
| `rhwp-studio/src/view/canvas-view.ts` | canvas release/re-render 시 sibling overlay layer 정리 | canvas pool 생명주기와 overlay 생명주기 동기화만 추가 |

변경하지 않은 영역:

- HWP parser
- model/IR
- layout engine
- SVG renderer
- native renderer
- 폰트 fallback/shape parser

결론: #931의 원인인 `BehindText DOM overlay`의 배율/가시성 문제를 해결하는 데 필요한 최소 경로만 수정했다.

## 5. 추가 PDF 기준 차이 분석

작업지시자가 제공한 `pdf/복학원서-2022.pdf`는 실제 한컴 기준 렌더로 확인했다. 이 기준과 현재 rhwp-studio 결과 사이에 다음 차이가 있다.

1. 서명란의 `(인)` 기호가 현재 rhwp-studio에서는 기대와 다르게 렌더링된다.
2. 중앙 워터마크 이미지가 투명 배경이 아니라 사각 배경 영역을 포함한 이미지처럼 보인다.

분리 판단:

- 워터마크 사각 배경은 현재 DOM layer 수정 이전의 `all` canvas 기준 렌더에서도 동일하게 보인다. 따라서 #931 Stage 5의 신규 회귀가 아니다.
- `(인)` 기호 문제는 BehindText overlay 줌/가시성과 독립적인 텍스트/특수기호/폰트 매핑 계열 문제로 보인다.

따라서 두 항목은 #931에 포함하지 않고 후속 이슈로 분리한다.

후속 이슈:

- `#937`: 복학원서 서명란 `(인)` 기호 렌더링 불일치
- `#938`: 복학원서 워터마크 투명 배경이 사각 영역으로 렌더링됨

## 6. 실행 검증

```bash
cargo test --test issue_516
```

결과: `8 passed`

```bash
cd rhwp-studio
npm run build
```

결과: 통과

```bash
git diff --check
```

결과: 통과

## 7. 결론

Stage 5 수정은 기존 단일 canvas 렌더 대비 텍스트/표/페이지 배치 회귀를 만들지 않았다. 차이는 BehindText 이미지 합성 방식의 차이로 제한된다.

PDF 기준으로 추가 확인된 `(인)` 기호 렌더링 불일치와 워터마크 사각 배경 문제는 #931의 줌/가시성 결함과 원인이 다르므로 후속 이슈로 등록한다.
