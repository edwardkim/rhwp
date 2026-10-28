# #7333 Stage 7 — 전체 sweep 잔여 차이 분리

## 분석

Stage 6의 Native Visual Sweep은 `samples/issue7333/aaaaaa.hwp`와
`pdf/issue7333/aaaaaa-2020.pdf`의 50/50쪽을 완료했고 자동 구조 이상은 0건이었다.
그러나 pixel/ink 지표의 최저 쪽은 2·3·1·43·41·40·35·10·38·12쪽이다. 이 지표는
글리프 raster·삽입 이미지의 encoding 차이도 포함하므로, 수치만으로 저장 frame의 회귀라고
판정하지 않는다.

이 회차는 자동 지표가 낮은 쪽에서 (1) PDF와 rhwp의 공통 좌표계 차이, (2) 도형·표의 저장
frame 차이, (3) 이미지·글리프 raster 차이를 분리한다. 각 후보는 render tree와 PDF overlay를
함께 보며 실제 기하 차이일 때만 코드를 수정한다. Studio 탭의 사용자 문서는 건드리지 않으며,
page-local selection의 실제 UI 검증은 별도 독립 세션 또는 비사용 상태에서만 수행한다.

## 코드 수정

100% 두 쪽 보기에서 `TableObjectRenderer.renderMultiPage()`는 테두리는
`getPageLeftResolved()`의 실제 페이지 시작 x를 사용했지만, 조절점은
`(contentWidth - pageWidth) / 2`를 다시 계산했다. 따라서 왼쪽 쪽에서 선택한
도형의 테두리와 8개 조절점이 서로 다른 화면 x에 그려질 수 있었다.

`objectSelectionViewportBox()`로 page-local bbox를 scroll-content 좌표로 변환하고,
테두리와 조절점이 모두 같은 `pageLeft`와 `pageOffset`을 사용하도록 통일했다.

## 검증

- `cd rhwp-studio && npx tsc --noEmit`: 통과
- `cd rhwp-studio && node --test tests/picture-hit-policy.test.ts tests/object-selection-page.test.ts`:
  6개 통과
- 저장소 루트에서
  `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`:
  통과
- `pkg/rhwp.js`와 `rhwp-studio/public/rhwp.js`, `pkg/rhwp_bg.wasm`과
  `rhwp-studio/public/rhwp_bg.wasm`의 SHA-256 일치 확인

사용자가 열어 둔 Studio 문서는 자동 조작하지 않았다. 수정된 Vite 소스를 로드한 뒤 100% 두 쪽
보기에서 왼쪽 쪽 도형을 선택하면 테두리와 조절점이 같은 쪽에 표시돼야 한다.
