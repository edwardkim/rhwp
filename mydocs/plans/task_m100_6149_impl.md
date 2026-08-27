# 구현 계획 — Task M100 #6149

- **이슈**: [#6149](https://github.com/edwardkim/rhwp/issues/6149)
- **브랜치**: `codex/issue-6149-low-zoom-ruler`
- **기준 commit**: `upstream/devel` `1b91c2025` (PR #6176 병합)
- **최초 작업 기준**: `upstream/devel` `9be8b0562`
- **문서 성격**: 사후 재구성한 파일 단위 설계, 정정 계획·Stage 1~3·최초 통합 결과 승인 완료,
  PR #6176 반영 재검증 통과·결과 승인 대기

> 이 문서는 실제 구현 전에 승인된 설계가 아니다. 현재 5개 WIP 커밋의 동작 계약과 변경 경계를
> 감사 가능한 형태로 재구성한 문서다. 작업지시자는 이 설계와 기존 WIP의 검증 후보 채택을 승인했지만,
> 각 Stage 결과는 재검증 보고 뒤 별도로 승인한다.

## 표시 계약

### 눈금 단계

- 1mm의 화면 폭은 `96 / 25.4 × zoom` CSS px다.
- 숫자 눈금은 화면상 최소 30px를 확보하고 10mm보다 작은 단위에는 숫자를 붙이지 않는다.
- 세부 눈금은 화면상 최소 3.5px를 확보한다.
- 두 단계는 모두 `1·2·5 × 10ⁿ mm`의 올림값을 사용한다.
- 중간 눈금은 숫자 눈금의 절반 지점에 표시한다.
- 부동소수점 modulo 대신 정수 step index를 사용해 0.2/0.5mm 단계도 안정적으로 그린다.

### 페이지 간격과 경계

- 기본 페이지 간격은 기존 값과 같은 100% 기준 10 CSS px다.
- 실제 gap은 `max(6 CSS px, 10 × zoom)`으로 계산한다.
- gap은 VirtualScroll의 쪽 좌표에만 반영하고 페이지 내부 좌표·용지 여백·렌더 배율에는 섞지 않는다.
- 페이지 루트 Canvas는 테마 border token의 1px 외곽선을 사용한다. 정적 overlay Canvas는 제외한다.

## 파일별 구현

### `rhwp-studio/src/view/ruler-scale.ts` (신규)

- `niceStepCeil`, `resolveRulerScale` 순수 함수를 제공한다.
- 가로·세로가 같은 `labelStepMm`, `tickStepMm`을 소비한다.

### `rhwp-studio/src/view/ruler.ts`

- 고정 1/5/10mm 분기를 순수 scale 결과 기반 반복으로 교체한다.
- 숫자는 선택한 label step에서만 표시하고 mm 값을 cm 문자열로 변환한다.
- 세로는 보이는 모든 페이지 대신 `rulerPageIndex()`가 선택한 한 쪽만 그린다.
- focus 쪽의 전체 용지 시작/끝 선을 그려 본문 띠가 용지 전체 길이로 오인되지 않게 한다.

### `rhwp-studio/src/view/page-gap.ts` (신규)

- 배율과 100% 기준 gap을 받아 화면 gap을 반환하는 순수 함수를 둔다.

### `rhwp-studio/src/view/virtual-scroll.ts`

- 생성자 인자는 100% 기준 gap으로 보관한다.
- `setPageDimensions()`마다 현재 zoom의 화면 gap을 계산하고 모든 배치 함수가 이를 사용한다.

### `rhwp-studio/src/view/canvas-pool.ts`, `rhwp-studio/src/styles/editor.css`

- 페이지 루트 Canvas에 `document-page-canvas` class를 부여한다.
- 해당 class에만 테마 외곽선과 얕은 그림자를 적용한다.
- CanvasKit 교체 canvas는 clone으로 class가 보존되는 기존 계약을 사용한다.

### `rhwp-studio/src/view/render-backend.ts`

- 실제 브라우저 검증에서 Rust `normalize_canvas_scale()`의 0.25 하한과 Studio DPR 계산이
  불일치하는 것을 확인했다.
- TypeScript render scale도 같은 0.25 하한을 사용한다. 물리 bitmap 배율은 바꾸지 않고,
  renderer가 올린 bitmap을 올바른 DPR로 나눠 10% CSS 크기가 VirtualScroll 슬롯과 일치하게 한다.

## 테스트

- `ruler-scale.test.ts`: 10/20/25/50/100/500% 숫자·세부 눈금 간격과 1·2·5 단계
- `page-gap.test.ts`: 10% 하한, 50/100/222/500% 연속 증가
- `virtual-scroll-page-arrangement.test.ts`: 모든 배치의 실제 좌표 차이가 동적 gap과 일치
- 기존 zoom anchor, PageUp/PageDown, page arrangement 전체 회귀
- `render-backend.test.ts`: Rust와 Studio의 저배율 render scale 하한 일치

## 사후 확인한 WIP 커밋 경계

1. `docs(test): #6149 저배율 표시 계약`
2. `fix(studio): 눈금자 밀도와 focus 쪽 경계 정합`
3. `fix(studio): 배율별 페이지 간격과 외곽선 정합`
4. `docs(test): #6149 통합 회귀와 시각 검증`

이 경계는 이미 생성된 로컬 이력을 설명할 뿐, 단계별 사전 승인을 받았다는 뜻이 아니다. 기존 WIP는
삭제하거나 재작성하지 않고 계획 승인 뒤 각 단계의 검증 후보로만 사용한다.
