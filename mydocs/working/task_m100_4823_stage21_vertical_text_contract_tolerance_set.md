---
title: "#4823 Stage21 세로 글자 계약 허용 예산 집합"
issue: 4823
stage: 21
status: complete
---

# Stage21: 세로 글자 계약 허용 예산 집합

## 배경

Stage20 계약은 `table-border-style`의 잉크 마스크 예산을 현재 manifest 값인
`0.006` 하나로 고정했다. 이 fixture는 기존 엄격값 `0.005`와 글꼴 face 준비 뒤의
근거 기반 값 `0.006`을 모두 유효한 보정값으로 다뤄야 한다.

## 변경

renderer contract가 아래 두 값만 허용하도록 변경했다.

- `0.005`: 기존의 엄격한 세로 글자 raster 예산
- `0.006`: `한양중고딕` face 준비 후 관측한 플랫폼 간 raster 차이 예산

그 밖의 값은 계약 위반이다. `nonInkMaxDiffPixels`, `minimumInkPixels`, 세로 글자
diagnostic axis와 feature-count 검증은 그대로 유지된다.

## 검증

- `npm run e2e:renderer-contract`
- renderer baseline manifest 구조 검사
- `table-border-style` Canvas2D/CanvasKit baseline 비교

Rust, WASM, renderer 구현과 문서 fixture는 변경하지 않는다.
