# PR #2647: page-renderer 순수 rawSvg 차트 1.5초 지연 회귀 수정

## 이슈
- **Issue**: #2635 — 순수 RawSvg 차트가 첫 화면에서 1.5초 뒤에 표시됨

## 분석

f32a99856에서 다단계 재렌더(200ms/600ms/1500ms)가 단일 1500ms fallback으로
변경되었다. 이때 `scheduleReRender()`의 `prefetchLayerImages()`는
base64 이미지만 디코드 대상으로 찾고, 순수 SVG에는 data URL이 없어
`tasks.length === 0` → `return false`가 반환된다.

호출부(`scheduleReRender` 라인 873-876):
```typescript
this.prefetchLayerImages(pageIdx)
  .then((decoded) => {
    if (decoded) finish();  // false면 finish() 호출 안 됨
  })
```

순수 SVG 차트는 비동기 디코드가 필요 없으므로 즉시 렌더 가능함에도
1500ms fallback까지 finish()가 호출되지 않는다.

## 변경

`page-renderer.ts:988`: prefetch할 이미지가 전혀 없으면 `false` 대신 `true` 반환.
이는 `tasks.length === 0` = "모든 imageCount 항목이 순수 rawSvg(차트/OLE)"
임을 의미하며, 비동기 디코드가 필요 없으므로 즉시 완료로 간주한다.

## 검증

- 순수 SVG 차트(`samples/chart/원형/쪼개진원형.hwp`): prefetch가 없다 → 즉시 finish()
- raster 이미지가 있는 페이지: tasks.length > 0 → 기존 decode 완료 후 finish()
- 이미지/차트가 없는 페이지: imageCount = 0 → scheduleReRender 미진입 (변동 없음)
- 회귀: 기존 raster 이미지 decode 경로는 tasks.length > 0이므로 동일

## 결과
- **PR**: https://github.com/edwardkim/rhwp/pull/2647
- **Closes**: #2635
