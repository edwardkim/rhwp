# Stage 8 완료 보고서 - Task M100-1197

- 이슈: #1197
- 제목: HWPX 용지 기준 BehindText 그림/표 z-order 보존
- 브랜치: `local/task1197`
- 작성일: 2026-06-02
- 상태: 완료

## 1. 배경

Stage 7 이후 작업지시자가 원본 샘플을 다시 확인한 결과,
`01` 전면 표기만 보이고 중앙 배경 그림과 BehindText 내용이 계속 보이지 않았다.

원본 HWPX의 `PageLayerTree`를 확인한 결과 `01` 페이지에는
`pageBackground`, `behindText`, `flow`, `inFrontOfText` plane 이 모두 생성되어 있었다.
따라서 Rust layer metadata 또는 WASM filtered render 계약이 아니라,
`rhwp-studio` DOM canvas 합성 단계에서 하위 layer 가 가려지는 문제로 범위를 좁혔다.

## 2. 원인

`rhwp-studio/src/styles/editor.css`에는 다음 공통 규칙이 있다.

```css
#scroll-content canvas {
  background: var(--color-surface);
}
```

Stage 7에서 추가한 `background`/`behind`/`front` filtered canvas 도 모두 이 규칙을 상속했다.
특히 `front` canvas 는 z-index 3으로 flow canvas 위에 놓이기 때문에,
자체 PaintOp 는 `01`뿐이어도 CSS 배경이 흰색으로 칠해져
아래의 `background`/`behind`/`flow` layer 전체를 덮을 수 있었다.

작업지시자가 본 "흰 페이지 위에 01만 보이는" 화면은 이 조건과 일치한다.

## 3. 변경 내용

- `createFilteredCanvasLayer()`가 생성하는 모든 overlay canvas 에
  `style.background = 'transparent'`를 명시했다.
- `rhwp-studio` source-level 테스트에 overlay canvas 배경 투명화 회귀 검사를 추가했다.

## 4. PDF/HWPX 페이지 매핑 확인

작업지시자가 제공한 정답 PDF와 로컬 HWPX를 비교했다.

- 정답 PDF: 46쪽
- rhwp HWPX pagination: 47쪽
- PDF 2쪽: `MEMO`
- rhwp 3쪽: `MEMO`
- PDF 3쪽: `01 / 1주차`
- rhwp 4쪽: `01 / 1주차` layer 포함 페이지

rhwp 2쪽과 3쪽이 정답지에서 하나로 합쳐져 보이는 현상은
현재 #1197의 z-order/layer 합성 문제와 직접 같은 원인은 아니다.
앞쪽 표가 rhwp에서 추가 페이지로 분리되는 별도 pagination/partial table 문제로 판단한다.
다만 페이지 번호가 한 쪽씩 밀리므로, #1197 시각검증에서는 PDF 3쪽과 rhwp 4쪽을 대응시켜 봐야 한다.

## 5. 검증

통과한 명령:

```sh
npm test
npm run build
cargo fmt --all --check
git diff --check
```

## 6. 재검증 요청

작업지시자는 `rhwp-studio`에서 브라우저 hard reload 후 원본 샘플 파일을 다시 로드한다.

확인 기준:

- `01` 페이지에서 중앙 배경 그림이 보인다.
- `01` 전면 표기는 배경 위에 보인다.
- BehindText 표/텍스트가 CSS 배경에 의해 통째로 가려지지 않는다.
- PDF와 비교할 때는 정답 PDF 3쪽과 rhwp 4쪽을 먼저 대응시킨다.
