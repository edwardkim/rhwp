# Task M100 #2228 결과 보고서

- 이슈: [#2228](https://github.com/edwardkim/rhwp/issues/2228)
- 샘플: `samples/issue2217/20200830.hwp`
- 기준: `pdf/issue2217/20200830-2020.pdf` 2쪽

## 문제와 원인

기본 Canvas2D 경로에서 2쪽 우측 하단 표 셀의 붉은 `창립교회` 그림이 셀 밖에도 반복 표시됐다.

문서의 Picture control이나 글꼴 문제가 아니라, 정적 flow 그림을 DOM layer로 분리하는 과정에서
`PageLayerTree`의 상위 `clipRect(tableCell)` 문맥을 잃은 것이 원인이다. 같은 binary 그림을 참조하는
TopAndBottom control 두 개가 셀 밖에서도 독립 DOM frame으로 생성됐다.

## 수정

- `flow-image-clip.ts`에서 flow image를 수집할 때 상위 clipRect의 누적 교집합을 함께 보존한다.
- clip 바깥 그림은 DOM 정적 그림 레이어에서 제외한다.
- 일부만 보이는 그림은 clip wrapper 안에 배치해 원래 bbox, crop, 회전, 좌우 반전 값을 유지한다.
- table-cell clip 계보와 clip 바깥 그림 제외를 단위 테스트로 고정했다.

## 프론트엔드 검증

- `rhwp-studio`: `npm test` 206개 통과.
- `rhwp-studio`: `npx tsc --noEmit` 통과.
- `rhwp-studio`: `npm run build` 통과.
- `rhwp-chrome`: `npm run build` 통과. 로컬 bundle만 생성했고 배포·게시하지 않았다.
- Headless Chrome Canvas2D에서 샘플을 실제 로드했다. 2쪽 flow image layer에는 정상 그림 3개만 생성됐고,
  표 셀 밖 TopAndBottom control 두 개는 생성되지 않았다. 붉은 `창립교회`는 한 번만 표시됐으며 콘솔
  error/warning은 없었다.

이번 변경은 TypeScript 프론트엔드 렌더링 경로만 수정한다. 작업지시 범위에 따라 Cargo 전체 테스트와 clippy는
이번 PR 전 검증 대상에서 제외했다.
