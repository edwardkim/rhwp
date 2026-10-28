---
kind: working-note
status: active
issue: 2635
stage: 2
---

# Task #2635 Stage 2: RawSvg 조기 재렌더 구현

## 기준 측정

`쪼개진원형.hwp`를 headless Chrome에서 열어 유채색 픽셀을 측정했다.

| 경과 시간 | 유채색 픽셀 비율 |
| --- | ---: |
| 0ms | 0% |
| 50ms | 0% |
| 100ms | 0% |
| 200ms | 0% |
| 400ms | 0% |
| 800ms | 0% |
| 1500ms | 1.3101% |

문서 로드와 최초 canvas 배치는 약 88ms에 끝났으므로 차트 자체가 아니라 재렌더 예약이 지연의 원인이다.

## 변경 계획

1. `scheduleReRender`에 RawSvg 개수를 별도 전달한다.
2. RawSvg가 있을 때만 `0/32/96/240ms`의 제한된 조기 재렌더를 예약한다.
3. 조기 재렌더는 현재 job의 완료·취소 상태를 따르며, decode prefetch 성공 또는 1500ms fallback이 끝나면 취소한다.
4. 기존 raster 이미지의 decode 완료 기반 단일 재렌더 및 1500ms 안전망은 변경하지 않는다.
5. 순수 SVG 차트가 400ms 안에 화면에 나타나는 headless E2E를 추가한다.

## 구현 및 검증 결과

- `page-renderer.ts`는 순수 RawSvg가 있는 페이지에만 `0/32/96/240ms` 조기 재렌더를 예약한다.
  일반 raster 이미지의 prefetch 및 `1500ms` 안전망은 유지했다.
- `npm test`: 456 passed, 0 failed.
- `npm run build`: 성공했다. 기존 chunk-size 경고 외 오류는 없었다.
- `issue-2635-rawsvg-first-paint.test.mjs --mode=headless`: 성공했다.
  `400ms` 안에 유채색 픽셀 비율 `1.664%`를 확인했고, 생성된 화면에서 차트 제목·범례·4개 조각을 직접 확인했다.
