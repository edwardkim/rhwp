# Stage 1 — Task M100 #2228: clip 계보 보존 설계

- 이슈: [#2228](https://github.com/edwardkim/rhwp/issues/2228)
- 상태: 완료

## 확인한 사실

- `20200830.hwp` 2쪽 로고 표 셀 [0,2]에는 같은 binary 그림을 참조하는 Picture control 3개가 있다.
- `PageLayerTree`는 해당 셀의 `clipRect(tableCell)`를 유지한다.
- Canvas2D의 정적 그림 DOM layer 수집기는 image bbox/crop만 평탄화해 clip 문맥을 잃는다.
- 그 결과 셀 밖 control 두 개가 DOM frame으로 생성되어 붉은 `창립교회` 그림이 반복 표시된다.

## 이번 단계 범위

- DOM flow image 수집에 누적 clip 교집합을 전달한다.
- DOM wrapper에 같은 clip을 적용한다.
- 구조 단위 테스트와 Studio build/WASM 브라우저 확인으로 회귀를 검증한다.

## 제외 범위

- `#2226`의 표 행높이 계산 및 Picture control 파싱은 수정하지 않는다.
- 글꼴 탐지·별칭·CanvasKit 글꼴 경로는 이 문제의 원인이 아니므로 변경하지 않는다.

## 구현 결과

- `flow-image-clip.ts`에서 PageLayerTree를 순회하며 부모 `clipRect`의 교집합을 각 flow image에 보존한다.
- 그림이 clip 바깥이면 DOM 정적 그림 레이어에 추가하지 않고, 일부만 보이면 clip wrapper 안에서 기존 그림 좌표·crop·회전을 유지해 표시한다.
- 따라서 같은 binary 그림을 참조하더라도 표 셀 clip 바깥의 control은 화면에 반복 표시되지 않는다.

## 검증 결과

- `node --test tests/flow-image-clip.test.ts tests/render-backend.test.ts`: 40개 통과.
- `npm test`: 206개 통과.
- `npx tsc --noEmit`, `npm run build`, `wasm-pack build --target web --out-dir pkg`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2226_cell_flow_pictures_overlap -- --nocapture`: 1개 통과.
- `rhwp-chrome`에서 `npm run build`를 로컬 수행해 extension bundle 생성 통과. 배포·게시하지 않았다.
- Headless Chrome Canvas2D에서 `samples/issue2217/20200830.hwp` 2쪽을 실제 로드했다. flow image layer는 정상 그림 3개만 생성됐고, clip 바깥의 두 control은 생성되지 않았다. 콘솔 error/warning은 없었으며, 한 개의 붉은 `창립교회`만 표시됨을 확인했다.
