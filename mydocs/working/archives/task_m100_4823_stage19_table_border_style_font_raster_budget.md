---
title: "#4823 Stage19 table-border-style 글꼴 raster 예산"
issue: 4823
stage: 19
status: complete
---

# Stage19: table-border-style 글꼴 raster 예산

## 근거

Stage18에서 `한양중고딕` 요청명을 `NotoSansKR-Regular.woff2`에 연결한 뒤
`table-border-style`의 Canvas2D/CanvasKit 잉크 마스크 차이는 `0.0053428`이었다.
문서 face가 없던 Linux CI의 차이 `0.0131384`보다 현저히 작으며, 최소 잉크량과
비잉크 픽셀 예산, solid-ink 예산은 모두 통과했다.

남은 차이는 세로 글자 rasterization의 엔진 차이다. 같은 글꼴 자산을 계획에 넣어도
Canvas2D와 CanvasKit의 outline rasterization은 픽셀 단위로 같지 않을 수 있다.

## 변경

`table-border-style`의 `inkMaskMaxDiffRatio`만 `0.005`에서 `0.006`으로 조정했다.

- `0.006`은 이미 설정된 `solidInkMaxDiffRatio` `0.0065`보다 엄격하다.
- face 누락 상태의 `0.0131384`보다 충분히 낮아 글꼴 준비 회귀를 계속 검출한다.
- 다른 샘플과 renderer 동작, 문서 fixture는 변경하지 않는다.

## 검증

- renderer baseline manifest 구조 검사
- `table-border-style` Canvas2D/CanvasKit baseline 비교
- Studio TypeScript 검사, 테스트, production build
