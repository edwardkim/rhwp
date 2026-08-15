---
title: "#4823 Stage20 세로 글자 baseline 계약 예산"
issue: 4823
stage: 20
status: complete
---

# Stage20: 세로 글자 baseline 계약 예산

## 배경

Stage19는 `table-border-style`의 `inkMaskMaxDiffRatio`를 `0.006`으로 보정했다.
그러나 renderer contract는 같은 fixture 예산을 `0.005`로 고정 검증하고 있어
Canvas visual diff가 baseline 실행 전에 실패했다.

## 변경

계약의 기대값을 manifest와 같은 `0.006`으로 갱신했다. 설명은 이 값이
`한양중고딕`의 준비 뒤에 남는 Canvas2D/CanvasKit 글꼴 rasterization 차이에 대한
교정값임을 명시한다.

## 유지되는 경계

- `nonInkMaxDiffPixels: 4`
- `minimumInkPixels: 50000`
- 세로 글자 diagnostic axis와 feature-count 기대값
- face 누락 시 관측된 `0.0131384`보다 낮은 `0.006` 잉크 마스크 한계

## 검증

- `npm run e2e:renderer-contract`
- renderer baseline manifest 구조 검사
- `table-border-style` Canvas2D/CanvasKit baseline 비교

Rust, WASM, renderer 구현과 문서 fixture는 변경하지 않는다.
