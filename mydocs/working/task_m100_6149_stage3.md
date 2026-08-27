# Task M100 #6149 — Stage 3 재검증 결과

- **이슈**: [#6149](https://github.com/edwardkim/rhwp/issues/6149)
- **단계**: 배치 공통 페이지 분리와 저배율 CSS 크기 정합
- **WIP 실측일**: 2026-08-27 KST
- **절차 상태**: Stage 2 승인 후 재검증 통과, Stage 3 결과 승인 완료

> 최초 기록은 Stage 2 결과 승인 뒤 진행된 완료 보고서가 아니었다. 기존 WIP 이력을 보존한 상태에서
> Stage 2 승인 뒤 자동 계약과 실제 브라우저 동작을 다시 검증했으며, 아래 결과의 작업지시자 승인
> 전에는 통합 검증으로 넘어가지 않는다.

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

## 최초 WIP 검증 기록

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

## Stage 2 승인 후 자동 재검증

- **검증 기준**: `5fdd96c4c` (`docs(test): #6149 Stage 2 재검증 승인`)
- **실행일**: 2026-08-27 KST
- **소스 변경**: 없음

```text
$ node --test \
    rhwp-studio/tests/ruler-scale.test.ts \
    rhwp-studio/tests/page-gap.test.ts \
    rhwp-studio/tests/render-backend.test.ts \
    rhwp-studio/tests/virtual-scroll-page-arrangement.test.ts \
    rhwp-studio/tests/virtual-scroll-horizontal-pan.test.ts \
    rhwp-studio/tests/virtual-scroll-grid-page.test.ts \
    rhwp-studio/tests/page-scroll-step.test.ts
tests 99, pass 99, fail 0, skipped 0
duration_ms 181.043167
```

10%의 6px 하한, 100%의 10px 기준과 고배율 연속 증가, Rust·Studio render scale의 0.25 하한,
자동·한 쪽·두 쪽·맞쪽·여러 쪽·가로 이동의 공통 gap, 스크롤·PageUp/PageDown 회귀가 모두
통과했다.

## 실제 브라우저 재검증

- **환경**: macOS Codex in-app browser, 1280×720, `exam_kor.hwp` 20쪽
- **URL**: `http://127.0.0.1:7720/?url=%2Fsamples%2Fexam_kor.hwp&filename=exam_kor.hwp`

| 조건 | 실측 | 판정 |
| --- | --- | --- |
| 10% 자동 | 10열×2행, 가로 `5.8516px`, 세로 `5.9375px` | 통과 |
| 10% 한 쪽 | 세로 `5.9375~5.9453px` | 통과 |
| 10% 두 쪽 | 가로 `5.8516px`, 세로 `5.9375~5.9453px` | 통과 |
| 10% 맞쪽 | 가로 `5.8516px`, 세로 `5.9375~5.9453px` | 통과 |
| 10% 여러 쪽 3×2 | 가로 `5.8516px`, 세로 `5.9375~5.9453px`; 배치 설정 뒤 상태바에서 10%로 고정 | 통과 |
| 10% 가로 이동 | 모든 쪽 Y 정렬 동일, 가로 `5.8516px` | 통과 |
| 100% 한 쪽 | 세로 `9.8984px` | 통과 |

10% 페이지 루트 canvas는 `112.398×158.797px`, 물리 bitmap은 `281×397px`, render zoom은
`0.1`이었다. 즉 renderer의 0.25 물리 하한을 유지하면서 CSS 표시 크기는 논리 10% 슬롯과
일치했고 다음 페이지를 덮지 않았다. 100%에서는 페이지 루트가 `1123×1587.5px`이고 세로 gap은
기존 10px과 서브픽셀 오차 안에서 일치했다.

현재 viewport의 canvas 63개 중 페이지 루트 20개만 `document-page-canvas` class와 외곽선·그림자를
가졌다. 정적 overlay 43개에는 이 class가 0건이어서 중복 경계를 만들지 않았다.

## 작업지시자 승인

위 재검증 결과와 다음 게이트를 보고한 뒤 작업지시자가 다음과 같이 승인했다.

> 진행해줘.

이 승인은 Stage 3 결과를 확정하고 통합 검증 범위를 제시하라는 승인이다. 통합 검증 실행과
push·PR 승인은 포함하지 않는다.

## 다음 단계

승인에 따라 통합 회귀·시각 검증 범위를 제시한다. 해당 범위의 별도 승인 전에는 검증을 실행하지
않으며, push·PR 작업은 그 이후에도 다시 승인받는다.
