# Task M100 #6149 — 3단계 WIP 실측 기록

- **이슈**: [#6149](https://github.com/edwardkim/rhwp/issues/6149)
- **단계**: 배치 공통 페이지 분리와 저배율 CSS 크기 정합
- **WIP 실측일**: 2026-08-27 KST
- **절차 상태**: 사후 재구성한 WIP 실측 기록, Stage 3 승인 전

> 이 기록은 Stage 2 결과 승인 뒤 진행된 완료 보고서가 아니다. 승인 게이트 없이 연속 구현한 WIP의
> 변경과 당시 관측을 보존하며, Stage 3 완료 승인은 앞 단계 승인 뒤 재검증 후 받는다.

## WIP 구현 내용

### 배율별 페이지 간격

- `VirtualScroll`이 100% 기준 10px를 보관하고 레이아웃 계산마다
  `max(6px, 10px × zoom)`을 적용한다.
- 단일 열·자동·한 쪽·두 쪽·맞쪽·여러 쪽·가로 이동이 같은 현재 gap을 사용한다.
- 페이지 루트 canvas에만 `document-page-canvas` class를 부여하고 테마 border token의 1px
  외곽선과 얕은 그림자를 적용했다. 정적 overlay canvas는 제외했다.

### 실제 브라우저에서 발견한 render scale 경계

- Rust Canvas2D 렌더러는 bitmap scale을 최소 0.25로 올리지만 Studio는 요청값 0.1~0.2로 DPR을
  계산하고 있었다.
- 그 결과 10%에서 bitmap CSS 폭이 VirtualScroll 슬롯보다 25% 넓어 페이지가 서로 겹쳤다.
- `clampRenderScale()`도 Rust와 같은 0.25 하한을 사용하게 해 물리 bitmap은 그대로 두고 CSS 폭과
  overlay 좌표만 논리 zoom에 맞췄다.

## 변경 파일

- `rhwp-studio/src/view/virtual-scroll.ts`
- `rhwp-studio/src/view/canvas-pool.ts`
- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/src/view/render-backend.ts`
- `rhwp-studio/src/styles/editor.css`
- 관련 page gap·배치·render backend 테스트

## 검증 결과

`exam_kor.hwp` 20쪽, 1280×720 실제 브라우저에서 다음 값을 측정했다.

| 배율·배치 | 가로 gap | 세로 gap | 판정 |
| --- | ---: | ---: | --- |
| 10% 자동 | 5.85px | 5.94px | 6px 하한과 서브픽셀 오차 내 일치 |
| 10% 두 쪽 | 5.85px | 5.95px | 통과 |
| 10% 여러 쪽 3×2 | 5.81px | 6.00px | 통과 |
| 10% 가로 이동 | 5.85px | 같은 행 | 통과 |
| 100% 단일 열 | — | 9.90px | 기존 10px 유지 |

밝은·어두운 테마에서 저배율 페이지 경계와 gap이 배경에서 구분됐고, 10% canvas 폭은
112.40px로 다음 페이지 슬롯과 겹치지 않았다.
